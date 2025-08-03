const { v4: uuidv4 } = require('uuid');

class SessionManager {
  constructor() {
    this.sessions = new Map();
    this.subscribers = new Map(); // WebSocket subscribers for each session
    this.flushTimeouts = new Map();
    this.broadcastEvents = []; // Store broadcast events for HTTP polling
    this.config = {
      maxBatchSize: 100,
      flushInterval: 30000, // 30 seconds
      maxSessionTime: 1800000, // 30 minutes
      retryAttempts: 3,
      retryDelay: 1000,
      maxBroadcastEvents: 1000 // Maximum broadcast events to store
    };
  }

  // Create or get existing session
  getOrCreateSession(sessionId) {
    if (!sessionId) {
      sessionId = uuidv4();
    }

    if (!this.sessions.has(sessionId)) {
      const session = {
        id: sessionId,
        createdAt: new Date(),
        lastActivity: new Date(),
        events: [],
        flushAttempts: 0,
        status: 'active',
        metadata: {
          userAgent: null,
          source: null,
          flushIssues: [],
          networkCondition: 'good'
        },
        stats: {
          totalEvents: 0,
          flushedEvents: 0,
          pendingEvents: 0,
          failedFlushes: 0,
          successfulFlushes: 0
        }
      };

      this.sessions.set(sessionId, session);
      this.scheduleFlush(sessionId);
      
      console.log(`📝 Created new session: ${sessionId}`);
    }

    return this.sessions.get(sessionId);
  }

  // Add event to session
  addEvent(sessionId, eventData) {
    const session = this.getOrCreateSession(sessionId);
    
    const event = {
      id: uuidv4(),
      timestamp: new Date(),
      type: eventData.type,
      data: eventData,
      status: 'pending',
      retryCount: 0
    };

    session.events.push(event);
    session.lastActivity = new Date();
    session.stats.totalEvents++;
    session.stats.pendingEvents++;

    // Check if we need immediate flush due to batch size
    if (session.events.filter(e => e.status === 'pending').length >= this.config.maxBatchSize) {
      this.flushSession(sessionId, false);
    }

    // Notify subscribers
    this.notifySubscribers(sessionId, {
      type: 'event_added',
      sessionId,
      event,
      stats: session.stats
    });

    return event;
  }

  // Add multiple events (batch)
  addBatchEvents(sessionId, eventsData) {
    const session = this.getOrCreateSession(sessionId);
    const addedEvents = [];

    eventsData.forEach(eventData => {
      const event = {
        id: uuidv4(),
        timestamp: new Date(),
        type: eventData.type,
        data: eventData,
        status: 'pending',
        retryCount: 0
      };

      session.events.push(event);
      addedEvents.push(event);
      session.stats.totalEvents++;
      session.stats.pendingEvents++;
    });

    session.lastActivity = new Date();

    // Check batch size limit
    if (session.events.filter(e => e.status === 'pending').length >= this.config.maxBatchSize) {
      this.flushSession(sessionId, false);
    }

    // Notify subscribers
    this.notifySubscribers(sessionId, {
      type: 'batch_events_added',
      sessionId,
      events: addedEvents,
      stats: session.stats
    });

    return addedEvents;
  }

  // Flush session events
  flushSession(sessionId, force = false) {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new Error(`Session ${sessionId} not found`);
    }

    const pendingEvents = session.events.filter(e => e.status === 'pending');
    
    if (pendingEvents.length === 0 && !force) {
      return { eventCount: 0, status: 'no_events' };
    }

    console.log(`🚀 Flushing session ${sessionId}: ${pendingEvents.length} events`);

    try {
      // Simulate flush operation
      const flushResult = this.simulateFlush(session, pendingEvents);
      
      if (flushResult.success) {
        // Mark events as flushed
        pendingEvents.forEach(event => {
          event.status = 'flushed';
          event.flushedAt = new Date();
        });

        session.stats.flushedEvents += pendingEvents.length;
        session.stats.pendingEvents -= pendingEvents.length;
        session.stats.successfulFlushes++;
        session.flushAttempts = 0; // Reset on success

        // Notify subscribers
        this.notifySubscribers(sessionId, {
          type: 'session_flushed',
          sessionId,
          eventCount: pendingEvents.length,
          stats: session.stats
        });

        // Schedule next flush
        this.scheduleFlush(sessionId);

        return { 
          eventCount: pendingEvents.length, 
          status: 'success',
          timestamp: new Date().toISOString()
        };
      } else {
        // Handle flush failure
        session.flushAttempts++;
        session.stats.failedFlushes++;
        
        const issue = {
          type: flushResult.errorType,
          message: flushResult.error,
          timestamp: new Date(),
          attempt: session.flushAttempts
        };

        session.metadata.flushIssues.push(issue);

        // Notify subscribers about the issue
        this.notifySubscribers(sessionId, {
          type: 'flush_failed',
          sessionId,
          issue,
          stats: session.stats
        });

        // Schedule retry if within limits
        if (session.flushAttempts < this.config.retryAttempts) {
          setTimeout(() => {
            this.flushSession(sessionId, false);
          }, this.config.retryDelay * session.flushAttempts);
        }

        throw new Error(`Flush failed: ${flushResult.error}`);
      }
    } catch (error) {
      console.error(`❌ Flush failed for session ${sessionId}:`, error);
      throw error;
    }
  }

  // Simulate flush operation with potential issues
  simulateFlush(session, events) {
    // Check for simulated network conditions
    if (session.metadata.networkCondition === 'poor') {
      if (Math.random() < 0.3) { // 30% chance of failure with poor network
        return {
          success: false,
          errorType: 'network_timeout',
          error: 'Network timeout - poor connection simulated'
        };
      }
    }

    // Simulate memory pressure issues
    if (events.length > 50 && Math.random() < 0.2) {
      return {
        success: false,
        errorType: 'memory_pressure',
        error: 'Memory pressure detected - batch too large'
      };
    }

    // Simulate server errors
    if (Math.random() < 0.05) { // 5% random server error
      return {
        success: false,
        errorType: 'server_error',
        error: 'RudderStack server returned 500 error'
      };
    }

    // Success case
    return {
      success: true,
      timestamp: new Date().toISOString()
    };
  }

  // Schedule automatic flush
  scheduleFlush(sessionId) {
    // Clear existing timeout
    if (this.flushTimeouts.has(sessionId)) {
      clearTimeout(this.flushTimeouts.get(sessionId));
    }

    // Schedule new flush
    const timeout = setTimeout(() => {
      try {
        this.flushSession(sessionId, false);
      } catch (error) {
        console.error(`Scheduled flush failed for ${sessionId}:`, error);
      }
    }, this.config.flushInterval);

    this.flushTimeouts.set(sessionId, timeout);
  }

  // Get session data
  getSessionData(sessionId) {
    return this.sessions.get(sessionId) || null;
  }

  // Get all active sessions
  getActiveSessions() {
    const now = new Date();
    return Array.from(this.sessions.values())
      .filter(session => {
        const age = now - session.lastActivity;
        return age < this.config.maxSessionTime;
      })
      .map(session => ({
        id: session.id,
        createdAt: session.createdAt,
        lastActivity: session.lastActivity,
        status: session.status,
        stats: session.stats,
        issueCount: session.metadata.flushIssues.length
      }));
  }

  // End session
  endSession(sessionId) {
    const session = this.sessions.get(sessionId);
    if (!session) return;

    // Flush remaining events
    try {
      this.flushSession(sessionId, true);
    } catch (error) {
      console.error(`Failed to flush session ${sessionId} on end:`, error);
    }

    // Clear timeout
    if (this.flushTimeouts.has(sessionId)) {
      clearTimeout(this.flushTimeouts.get(sessionId));
      this.flushTimeouts.delete(sessionId);
    }

    // Mark as ended
    session.status = 'ended';
    session.endedAt = new Date();

    console.log(`📝 Ended session: ${sessionId}`);
  }

  // Clear session data
  clearSession(sessionId) {
    if (this.flushTimeouts.has(sessionId)) {
      clearTimeout(this.flushTimeouts.get(sessionId));
      this.flushTimeouts.delete(sessionId);
    }
    
    this.sessions.delete(sessionId);
    this.subscribers.delete(sessionId);
    
    console.log(`🗑️ Cleared session: ${sessionId}`);
  }

  // Subscribe to session events
  subscribeToSession(sessionId, websocket) {
    if (!this.subscribers.has(sessionId)) {
      this.subscribers.set(sessionId, new Set());
    }
    this.subscribers.get(sessionId).add(websocket);
  }

  // Notify subscribers
  notifySubscribers(sessionId, data) {
    const subscribers = this.subscribers.get(sessionId);
    if (!subscribers) return;

    const message = JSON.stringify({
      ...data,
      timestamp: new Date().toISOString()
    });

    subscribers.forEach(ws => {
      if (ws.readyState === 1) { // WebSocket.OPEN
        ws.send(message);
      }
    });
  }

  // Debug methods for simulating issues
  simulateTimeout(sessionId) {
    const session = this.sessions.get(sessionId);
    if (!session) throw new Error('Session not found');
    
    session.metadata.networkCondition = 'timeout';
    return { simulated: 'timeout', sessionId };
  }

  simulateNetworkError(sessionId) {
    const session = this.sessions.get(sessionId);
    if (!session) throw new Error('Session not found');
    
    session.metadata.networkCondition = 'poor';
    return { simulated: 'network_error', sessionId };
  }

  simulateBatchLimit(sessionId) {
    const session = this.sessions.get(sessionId);
    if (!session) throw new Error('Session not found');
    
    // Add many dummy events to trigger batch limit
    for (let i = 0; i < this.config.maxBatchSize + 10; i++) {
      this.addEvent(sessionId, {
        type: 'track',
        event: 'dummy_event',
        properties: { index: i }
      });
    }
    
    return { simulated: 'batch_limit', sessionId, eventsAdded: this.config.maxBatchSize + 10 };
  }

  simulateMemoryPressure(sessionId) {
    const session = this.sessions.get(sessionId);
    if (!session) throw new Error('Session not found');
    
    // Add large dummy events
    for (let i = 0; i < 60; i++) {
      this.addEvent(sessionId, {
        type: 'track',
        event: 'large_event',
        properties: {
          largeData: 'x'.repeat(1000), // 1KB per event
          index: i
        }
      });
    }
    
    return { simulated: 'memory_pressure', sessionId, eventsAdded: 60 };
  }

  // Add broadcast event for HTTP polling
  addBroadcastEvent(eventData) {
    const broadcastEvent = {
      id: uuidv4(),
      timestamp: new Date(),
      data: eventData
    };
    
    this.broadcastEvents.push(broadcastEvent);
    
    // Keep only the latest events within limit
    if (this.broadcastEvents.length > this.config.maxBroadcastEvents) {
      this.broadcastEvents = this.broadcastEvents.slice(-this.config.maxBroadcastEvents);
    }
    
    return broadcastEvent;
  }

  // Get recent broadcast events for HTTP polling
  getRecentBroadcastEvents(sessionId = null, limit = 50) {
    let events = [...this.broadcastEvents];
    
    // Filter by session if specified
    if (sessionId) {
      events = events.filter(event => 
        event.data.sessionId === sessionId ||
        (event.data.data && event.data.data.sessionId === sessionId)
      );
    }
    
    // Return most recent events
    return events
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, limit);
  }

  // Cleanup old sessions and broadcast events
  cleanup() {
    const now = new Date();
    const toDelete = [];

    this.sessions.forEach((session, sessionId) => {
      const age = now - session.lastActivity;
      if (age > this.config.maxSessionTime) {
        toDelete.push(sessionId);
      }
    });

    toDelete.forEach(sessionId => {
      this.clearSession(sessionId);
    });

    // Clean up old broadcast events (older than 1 hour)
    const oneHourAgo = new Date(now - 3600000);
    this.broadcastEvents = this.broadcastEvents.filter(event => event.timestamp > oneHourAgo);

    console.log(`🧹 Cleaned up ${toDelete.length} old sessions and old broadcast events`);
  }
}

module.exports = SessionManager;
