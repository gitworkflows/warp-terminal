const { v4: uuidv4 } = require('uuid');

class TelemetryMiddleware {
  constructor(sessionManager) {
    this.sessionManager = sessionManager;
    this.interceptedEvents = new Map();
    this.config = {
      logLevel: 'info',
      enableInterception: true,
      enableValidation: true,
      enableEnrichment: true
    };
  }

  // Process single event
  processEvent(req, res, eventType, broadcastFn) {
    const sessionId = this.extractSessionId(req);
    const eventData = this.enrichEventData(req.body, eventType, req);

    if (this.config.enableInterception) {
      // Intercept and store the event
      const interceptedEvent = {
        id: uuidv4(),
        sessionId,
        type: eventType,
        originalData: req.body,
        enrichedData: eventData,
        headers: this.sanitizeHeaders(req.headers),
        timestamp: new Date(),
        source: 'middleware'
      };

      this.interceptedEvents.set(interceptedEvent.id, interceptedEvent);
      
      // Add to session
      this.sessionManager.addEvent(sessionId, {
        ...eventData,
        type: eventType,
        interceptedId: interceptedEvent.id
      });

      // Broadcast to WebSocket clients
      broadcastFn({
        type: 'telemetry_event_intercepted',
        sessionId,
        eventType,
        eventId: interceptedEvent.id,
        data: eventData,
        timestamp: new Date().toISOString()
      });

      this.logEvent('intercepted', eventType, sessionId, eventData);
    }

    // Validate event data
    if (this.config.enableValidation) {
      const validationResult = this.validateEvent(eventData, eventType);
      if (!validationResult.valid) {
        console.warn(`⚠️ Validation failed for ${eventType} event:`, validationResult.errors);
        
        broadcastFn({
          type: 'validation_warning',
          sessionId,
          eventType,
          errors: validationResult.errors,
          timestamp: new Date().toISOString()
        });
      }
    }

    return eventData;
  }

  // Process batch events
  processBatchEvents(req, res, broadcastFn) {
    const sessionId = this.extractSessionId(req);
    const batchData = req.body;

    if (!batchData.batch || !Array.isArray(batchData.batch)) {
      console.error('Invalid batch format');
      return;
    }

    const processedEvents = [];
    const validationErrors = [];

    batchData.batch.forEach((event, index) => {
      const eventType = event.type || 'unknown';
      const enrichedEvent = this.enrichEventData(event, eventType, req);

      if (this.config.enableInterception) {
        const interceptedEvent = {
          id: uuidv4(),
          sessionId,
          type: eventType,
          originalData: event,
          enrichedData: enrichedEvent,
          headers: this.sanitizeHeaders(req.headers),
          timestamp: new Date(),
          source: 'batch_middleware',
          batchIndex: index
        };

        this.interceptedEvents.set(interceptedEvent.id, interceptedEvent);
        processedEvents.push({
          ...enrichedEvent,
          type: eventType,
          interceptedId: interceptedEvent.id
        });
      }

      // Validate each event
      if (this.config.enableValidation) {
        const validationResult = this.validateEvent(enrichedEvent, eventType);
        if (!validationResult.valid) {
          validationErrors.push({
            index,
            eventType,
            errors: validationResult.errors
          });
        }
      }
    });

    // Add batch to session
    this.sessionManager.addBatchEvents(sessionId, processedEvents);

    // Broadcast batch processing
    broadcastFn({
      type: 'telemetry_batch_intercepted',
      sessionId,
      eventCount: processedEvents.length,
      validationErrors,
      timestamp: new Date().toISOString()
    });

    if (validationErrors.length > 0) {
      console.warn(`⚠️ Batch validation issues:`, validationErrors);
    }

    this.logEvent('batch_intercepted', 'batch', sessionId, { 
      eventCount: processedEvents.length,
      validationErrors: validationErrors.length
    });

    return processedEvents;
  }

  // Extract session ID from request
  extractSessionId(req) {
    // Try various sources for session ID
    return req.headers['x-session-id'] ||
           req.body.anonymousId ||
           req.body.userId ||
           req.headers['x-anonymous-id'] ||
           req.query.sessionId ||
           uuidv4(); // Fallback to new UUID
  }

  // Enrich event data with additional context
  enrichEventData(eventData, eventType, req) {
    if (!this.config.enableEnrichment) {
      return eventData;
    }

    const enriched = {
      ...eventData,
      // Add middleware metadata
      _middleware: {
        processedAt: new Date().toISOString(),
        userAgent: req.headers['user-agent'],
        ip: req.ip || req.connection.remoteAddress,
        referer: req.headers.referer,
        origin: req.headers.origin,
        contentType: req.headers['content-type'],
        eventType
      },
      // Ensure required fields exist
      timestamp: eventData.timestamp || new Date().toISOString(),
      messageId: eventData.messageId || uuidv4()
    };

    // Add context if missing
    if (!enriched.context) {
      enriched.context = {};
    }

    // Enrich context with request data
    enriched.context.middleware = {
      intercepted: true,
      processingTime: new Date().toISOString(),
      requestId: uuidv4()
    };

    // Add library info if missing
    if (!enriched.context.library) {
      enriched.context.library = {
        name: 'warp-telemetry-middleware',
        version: '1.0.0'
      };
    }

    return enriched;
  }

  // Validate event data
  validateEvent(eventData, eventType) {
    const errors = [];

    // Common validations
    if (!eventData.messageId) {
      errors.push('Missing messageId');
    }

    if (!eventData.timestamp) {
      errors.push('Missing timestamp');
    }

    // Event-specific validations
    switch (eventType) {
      case 'track':
        if (!eventData.event) {
          errors.push('Track events must have an event name');
        }
        break;

      case 'identify':
        if (!eventData.userId && !eventData.anonymousId) {
          errors.push('Identify events must have userId or anonymousId');
        }
        break;

      case 'page':
      case 'screen':
        if (!eventData.name && !eventData.properties?.name) {
          errors.push(`${eventType} events should have a name`);
        }
        break;

      case 'group':
        if (!eventData.groupId) {
          errors.push('Group events must have a groupId');
        }
        break;

      case 'alias':
        if (!eventData.userId || !eventData.previousId) {
          errors.push('Alias events must have userId and previousId');
        }
        break;
    }

    // Check for PII in properties (basic check)
    if (eventData.properties) {
      const piiFields = ['email', 'phone', 'ssn', 'credit_card'];
      const foundPII = [];

      const checkForPII = (obj, path = '') => {
        Object.keys(obj).forEach(key => {
          const fullPath = path ? `${path}.${key}` : key;
          const value = obj[key];

          if (piiFields.some(pii => key.toLowerCase().includes(pii))) {
            foundPII.push(fullPath);
          }

          if (typeof value === 'object' && value !== null) {
            checkForPII(value, fullPath);
          }
        });
      };

      checkForPII(eventData.properties);

      if (foundPII.length > 0) {
        errors.push(`Potential PII detected in fields: ${foundPII.join(', ')}`);
      }
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }

  // Sanitize headers for logging
  sanitizeHeaders(headers) {
    const sanitized = { ...headers };

    // Remove sensitive headers
    delete sanitized.authorization;
    delete sanitized.cookie;
    delete sanitized['x-auth-token'];
    delete sanitized['x-api-key'];

    return sanitized;
  }

  // Log events
  logEvent(action, eventType, sessionId, data) {
    if (this.config.logLevel === 'debug') {
      console.log(`🔍 [${action.toUpperCase()}] ${eventType} - Session: ${sessionId}`, {
        timestamp: new Date().toISOString(),
        data: JSON.stringify(data, null, 2)
      });
    } else if (this.config.logLevel === 'info') {
      console.log(`📊 [${action.toUpperCase()}] ${eventType} - Session: ${sessionId}`);
    }
  }

  // Get intercepted events
  getInterceptedEvents(sessionId = null, limit = 100) {
    let events = Array.from(this.interceptedEvents.values());

    if (sessionId) {
      events = events.filter(event => event.sessionId === sessionId);
    }

    return events
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, limit);
  }

  // Clear intercepted events
  clearInterceptedEvents(sessionId = null) {
    if (sessionId) {
      // Clear events for specific session
      const toDelete = [];
      this.interceptedEvents.forEach((event, id) => {
        if (event.sessionId === sessionId) {
          toDelete.push(id);
        }
      });
      toDelete.forEach(id => this.interceptedEvents.delete(id));
    } else {
      // Clear all events
      this.interceptedEvents.clear();
    }
  }

  // Get statistics
  getStats() {
    const events = Array.from(this.interceptedEvents.values());
    const now = new Date();
    const oneHourAgo = new Date(now - 60 * 60 * 1000);

    const recentEvents = events.filter(event => event.timestamp >= oneHourAgo);

    const eventTypeStats = {};
    recentEvents.forEach(event => {
      eventTypeStats[event.type] = (eventTypeStats[event.type] || 0) + 1;
    });

    return {
      totalIntercepted: events.length,
      recentIntercepted: recentEvents.length,
      eventTypeBreakdown: eventTypeStats,
      activeSessions: [...new Set(recentEvents.map(e => e.sessionId))].length,
      oldestEvent: events.length > 0 ? Math.min(...events.map(e => e.timestamp)) : null,
      newestEvent: events.length > 0 ? Math.max(...events.map(e => e.timestamp)) : null
    };
  }

  // Update configuration
  updateConfig(newConfig) {
    this.config = { ...this.config, ...newConfig };
    console.log('📝 Telemetry middleware config updated:', this.config);
  }
}

module.exports = TelemetryMiddleware;
