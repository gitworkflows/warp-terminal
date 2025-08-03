class TelemetryClient {
  constructor(wsUrl, options = {}) {
    this.wsUrl = wsUrl;
    this.options = {
      autoReconnect: true,
      reconnectInterval: 5000,
      maxReconnectAttempts: 10,
      ...options
    };
    
    this.ws = null;
    this.reconnectAttempts = 0;
    this.isConnecting = false;
    this.sessionId = null;
    this.eventListeners = new Map();
    
    // Bind methods
    this.connect = this.connect.bind(this);
    this.onMessage = this.onMessage.bind(this);
    this.onClose = this.onClose.bind(this);
    this.onError = this.onError.bind(this);
  }

  // Connect to WebSocket server
  connect() {
    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return;
    }

    this.isConnecting = true;
    console.log('🔌 Connecting to telemetry server:', this.wsUrl);

    try {
      this.ws = new WebSocket(this.wsUrl);
      this.ws.onopen = this.onOpen.bind(this);
      this.ws.onmessage = this.onMessage;
      this.ws.onclose = this.onClose;
      this.ws.onerror = this.onError;
    } catch (error) {
      console.error('❌ Failed to create WebSocket connection:', error);
      this.isConnecting = false;
      this.scheduleReconnect();
    }
  }

  // Handle connection open
  onOpen(event) {
    console.log('✅ Connected to telemetry server');
    this.isConnecting = false;
    this.reconnectAttempts = 0;
    
    this.emit('connected', { 
      timestamp: new Date().toISOString() 
    });
  }

  // Handle incoming messages
  onMessage(event) {
    try {
      const data = JSON.parse(event.data);
      
      // Store session ID from connection message
      if (data.type === 'connection' && data.sessionId) {
        this.sessionId = data.sessionId;
        console.log('📝 Session ID assigned:', this.sessionId);
      }

      // Emit event to listeners
      this.emit(data.type, data);
      this.emit('message', data);
      
      // Log based on message type
      switch (data.type) {
        case 'telemetry_event_intercepted':
          console.log('📊 Event intercepted:', data.eventType, data.sessionId);
          break;
        case 'session_flushed':
          console.log('🚀 Session flushed:', data.sessionId, data.eventCount, 'events');
          break;
        case 'flush_failed':
          console.warn('⚠️ Flush failed:', data.sessionId, data.issue.message);
          break;
        case 'validation_warning':
          console.warn('⚠️ Validation warning:', data.eventType, data.errors);
          break;
      }
    } catch (error) {
      console.error('❌ Failed to parse WebSocket message:', error);
    }
  }

  // Handle connection close
  onClose(event) {
    console.log('🔌 WebSocket connection closed:', event.code, event.reason);
    this.isConnecting = false;
    
    this.emit('disconnected', {
      code: event.code,
      reason: event.reason,
      timestamp: new Date().toISOString()
    });

    if (this.options.autoReconnect && !this.isDestroyed) {
      this.scheduleReconnect();
    }
  }

  // Handle connection error
  onError(event) {
    console.error('❌ WebSocket error:', event);
    this.emit('error', {
      error: event,
      timestamp: new Date().toISOString()
    });
  }

  // Schedule reconnection attempt
  scheduleReconnect() {
    if (this.reconnectAttempts >= this.options.maxReconnectAttempts) {
      console.error('❌ Max reconnection attempts reached');
      this.emit('reconnect_failed', {
        attempts: this.reconnectAttempts,
        timestamp: new Date().toISOString()
      });
      return;
    }

    this.reconnectAttempts++;
    const delay = this.options.reconnectInterval * this.reconnectAttempts;
    
    console.log(`🔄 Scheduling reconnection attempt ${this.reconnectAttempts} in ${delay}ms`);
    
    setTimeout(() => {
      if (!this.isDestroyed) {
        this.connect();
      }
    }, delay);
  }

  // Send message to server
  send(data) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
      return true;
    } else {
      console.warn('⚠️ WebSocket not connected, message not sent:', data);
      return false;
    }
  }

  // Subscribe to session events
  subscribeToSession(sessionId) {
    return this.send({
      type: 'subscribe_session',
      sessionId
    });
  }

  // Get session data
  getSessionData(sessionId) {
    return this.send({
      type: 'get_session_data',
      sessionId
    });
  }

  // Clear session
  clearSession(sessionId) {
    return this.send({
      type: 'clear_session',
      sessionId
    });
  }

  // Add event listener
  on(eventType, listener) {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, new Set());
    }
    this.eventListeners.get(eventType).add(listener);
  }

  // Remove event listener
  off(eventType, listener) {
    if (this.eventListeners.has(eventType)) {
      this.eventListeners.get(eventType).delete(listener);
    }
  }

  // Emit event to listeners
  emit(eventType, data) {
    if (this.eventListeners.has(eventType)) {
      this.eventListeners.get(eventType).forEach(listener => {
        try {
          listener(data);
        } catch (error) {
          console.error('❌ Error in event listener:', error);
        }
      });
    }
  }

  // Get connection status
  getStatus() {
    if (!this.ws) return 'disconnected';
    
    switch (this.ws.readyState) {
      case WebSocket.CONNECTING: return 'connecting';
      case WebSocket.OPEN: return 'connected';
      case WebSocket.CLOSING: return 'closing';
      case WebSocket.CLOSED: return 'disconnected';
      default: return 'unknown';
    }
  }

  // Close connection
  disconnect() {
    this.isDestroyed = true;
    
    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }
    
    this.eventListeners.clear();
    console.log('🔌 Telemetry client disconnected');
  }
}

// Helper function to create and configure telemetry client
function createTelemetryClient(wsUrl, options = {}) {
  const client = new TelemetryClient(wsUrl, options);
  
  // Auto-connect by default
  if (options.autoConnect !== false) {
    client.connect();
  }
  
  return client;
}

// Export for both browser and Node.js environments
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { TelemetryClient, createTelemetryClient };
} else if (typeof window !== 'undefined') {
  window.TelemetryClient = TelemetryClient;
  window.createTelemetryClient = createTelemetryClient;
}
