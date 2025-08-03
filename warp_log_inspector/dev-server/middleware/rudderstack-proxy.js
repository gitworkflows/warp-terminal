const axios = require('axios');
const { v4: uuidv4 } = require('uuid');

class RudderStackProxy {
  constructor(dataPlaneUrl, sessionManager) {
    this.dataPlaneUrl = dataPlaneUrl;
    this.sessionManager = sessionManager;
    this.requestQueue = new Map();
    this.config = {
      timeout: 30000,
      retryAttempts: 3,
      retryDelay: 1000,
      enableForwarding: true,
      enableCaching: false,
      batchSize: 100
    };
    
    // Track proxy statistics
    this.stats = {
      totalRequests: 0,
      successfulRequests: 0,
      failedRequests: 0,
      averageResponseTime: 0,
      lastRequestTime: null
    };
  }

  // Forward request to RudderStack
  async forwardRequest(req, res, endpoint) {
    if (!this.config.enableForwarding) {
      // If forwarding is disabled, just return success
      return res.status(200).json({ 
        status: 'intercepted',
        message: 'Forwarding disabled - event captured locally',
        timestamp: new Date().toISOString()
      });
    }

    const requestId = uuidv4();
    const startTime = Date.now();
    
    try {
      this.stats.totalRequests++;
      this.stats.lastRequestTime = new Date();

      // Prepare request
      const forwardUrl = `${this.dataPlaneUrl}${endpoint}`;
      const requestConfig = {
        method: req.method,
        url: forwardUrl,
        data: req.body,
        headers: this.prepareHeaders(req.headers),
        timeout: this.config.timeout,
        validateStatus: (status) => status < 500 // Don't throw on 4xx errors
      };

      console.log(`🚀 Forwarding ${req.method} ${endpoint} to RudderStack (ID: ${requestId})`);

      // Make request with retry logic
      const response = await this.makeRequestWithRetry(requestConfig, requestId);
      const responseTime = Date.now() - startTime;

      // Update statistics
      this.updateStats(responseTime, true);

      // Log successful forward
      console.log(`✅ Request ${requestId} forwarded successfully (${responseTime}ms)`);

      // Send response back to client
      res.status(response.status).json(response.data);

      // Notify session manager of successful forward
      this.notifyForwardingResult(req, requestId, true, responseTime);

    } catch (error) {
      const responseTime = Date.now() - startTime;
      this.updateStats(responseTime, false);

      console.error(`❌ Request ${requestId} failed:`, error.message);

      // Handle different types of errors
      if (error.code === 'ECONNABORTED') {
        res.status(408).json({
          error: 'Request timeout',
          message: 'RudderStack request timed out',
          requestId,
          timestamp: new Date().toISOString()
        });
      } else if (error.response) {
        // Server responded with error status
        res.status(error.response.status).json({
          error: 'RudderStack error',
          message: error.response.data?.message || error.message,
          requestId,
          timestamp: new Date().toISOString()
        });
      } else {
        // Network error or other issue
        res.status(503).json({
          error: 'Service unavailable',
          message: 'Unable to reach RudderStack',
          requestId,
          timestamp: new Date().toISOString()
        });
      }

      // Notify session manager of failed forward
      this.notifyForwardingResult(req, requestId, false, responseTime, error);
    }
  }

  // Make request with retry logic
  async makeRequestWithRetry(requestConfig, requestId, attempt = 1) {
    try {
      const response = await axios(requestConfig);
      return response;
    } catch (error) {
      if (attempt < this.config.retryAttempts && this.shouldRetry(error)) {
        console.log(`🔄 Retrying request ${requestId} (attempt ${attempt + 1}/${this.config.retryAttempts})`);
        
        // Exponential backoff
        const delay = this.config.retryDelay * Math.pow(2, attempt - 1);
        await new Promise(resolve => setTimeout(resolve, delay));
        
        return this.makeRequestWithRetry(requestConfig, requestId, attempt + 1);
      }
      throw error;
    }
  }

  // Determine if error should trigger a retry
  shouldRetry(error) {
    if (error.code === 'ECONNABORTED') return true; // Timeout
    if (error.code === 'ENOTFOUND') return false; // DNS error - don't retry
    if (error.code === 'ECONNREFUSED') return true; // Connection refused
    
    if (error.response) {
      const status = error.response.status;
      // Retry on 5xx errors, but not 4xx errors
      return status >= 500;
    }
    
    return true; // Default to retry for network errors
  }

  // Prepare headers for forwarding
  prepareHeaders(originalHeaders) {
    const headers = {
      'Content-Type': originalHeaders['content-type'] || 'application/json',
      'User-Agent': originalHeaders['user-agent'] || 'warp-telemetry-proxy/1.0.0'
    };

    // Forward authentication headers if present
    if (originalHeaders.authorization) {
      headers.Authorization = originalHeaders.authorization;
    }

    // Forward custom RudderStack headers
    Object.keys(originalHeaders).forEach(key => {
      if (key.toLowerCase().startsWith('x-rudder')) {
        headers[key] = originalHeaders[key];
      }
    });

    // Add proxy identification
    headers['X-Proxy-Via'] = 'warp-telemetry-middleware';
    headers['X-Proxy-Time'] = new Date().toISOString();

    return headers;
  }

  // Update proxy statistics
  updateStats(responseTime, success) {
    if (success) {
      this.stats.successfulRequests++;
    } else {
      this.stats.failedRequests++;
    }

    // Calculate running average response time
    const totalRequests = this.stats.successfulRequests + this.stats.failedRequests;
    this.stats.averageResponseTime = (
      (this.stats.averageResponseTime * (totalRequests - 1) + responseTime) / totalRequests
    );
  }

  // Notify session manager about forwarding result
  notifyForwardingResult(req, requestId, success, responseTime, error = null) {
    const sessionId = this.extractSessionId(req);
    const session = this.sessionManager.getSessionData(sessionId);
    
    if (!session) return;

    const forwardingEvent = {
      requestId,
      success,
      responseTime,
      timestamp: new Date(),
      error: error ? {
        message: error.message,
        code: error.code,
        status: error.response?.status
      } : null
    };

    // Add to session metadata
    if (!session.metadata.forwardingEvents) {
      session.metadata.forwardingEvents = [];
    }
    
    session.metadata.forwardingEvents.push(forwardingEvent);
    
    // Keep only last 50 forwarding events per session
    if (session.metadata.forwardingEvents.length > 50) {
      session.metadata.forwardingEvents = session.metadata.forwardingEvents.slice(-50);
    }

    // Update session stats
    if (!session.stats.forwarding) {
      session.stats.forwarding = {
        totalRequests: 0,
        successfulRequests: 0,
        failedRequests: 0,
        averageResponseTime: 0
      };
    }

    session.stats.forwarding.totalRequests++;
    if (success) {
      session.stats.forwarding.successfulRequests++;
    } else {
      session.stats.forwarding.failedRequests++;
    }

    // Update average response time
    const totalForwardingRequests = session.stats.forwarding.totalRequests;
    session.stats.forwarding.averageResponseTime = (
      (session.stats.forwarding.averageResponseTime * (totalForwardingRequests - 1) + responseTime) / 
      totalForwardingRequests
    );
  }

  // Extract session ID from request (same as middleware)
  extractSessionId(req) {
    return req.headers['x-session-id'] ||
           req.body.anonymousId ||
           req.body.userId ||
           req.headers['x-anonymous-id'] ||
           req.query.sessionId ||
           'unknown';
  }

  // Get proxy statistics
  getStats() {
    return {
      ...this.stats,
      successRate: this.stats.totalRequests > 0 ? 
        (this.stats.successfulRequests / this.stats.totalRequests * 100).toFixed(2) + '%' : '0%',
      uptime: process.uptime(),
      config: this.config
    };
  }

  // Health check for RudderStack connectivity
  async healthCheck() {
    try {
      const response = await axios.get(`${this.dataPlaneUrl}/health`, {
        timeout: 5000,
        validateStatus: (status) => status < 500
      });
      
      return {
        status: 'healthy',
        dataPlaneUrl: this.dataPlaneUrl,
        responseTime: response.headers['x-response-time'] || 'unknown',
        timestamp: new Date().toISOString()
      };
    } catch (error) {
      return {
        status: 'unhealthy',
        dataPlaneUrl: this.dataPlaneUrl,
        error: error.message,
        timestamp: new Date().toISOString()
      };
    }
  }

  // Update configuration
  updateConfig(newConfig) {
    this.config = { ...this.config, ...newConfig };
    console.log('📝 RudderStack proxy config updated:', this.config);
  }

  // Reset statistics
  resetStats() {
    this.stats = {
      totalRequests: 0,
      successfulRequests: 0,
      failedRequests: 0,
      averageResponseTime: 0,
      lastRequestTime: null
    };
    console.log('🔄 Proxy statistics reset');
  }

  // Batch multiple requests (for future use)
  async batchRequests(requests) {
    if (!this.config.enableForwarding) {
      return { status: 'intercepted', count: requests.length };
    }

    const batchId = uuidv4();
    console.log(`📦 Processing batch ${batchId} with ${requests.length} requests`);

    const results = await Promise.allSettled(
      requests.map(req => this.forwardRequest(req.req, req.res, req.endpoint))
    );

    const successful = results.filter(r => r.status === 'fulfilled').length;
    const failed = results.filter(r => r.status === 'rejected').length;

    console.log(`📦 Batch ${batchId} completed: ${successful} successful, ${failed} failed`);

    return {
      batchId,
      total: requests.length,
      successful,
      failed,
      results
    };
  }
}

module.exports = RudderStackProxy;
