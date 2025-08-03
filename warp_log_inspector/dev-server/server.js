const express = require('express');
const http = require('http');
const cors = require('cors');
const bodyParser = require('body-parser');
const { v4: uuidv4 } = require('uuid');
const TelemetryMiddleware = require('./middleware/telemetry-middleware');
const SessionManager = require('./middleware/session-manager');
const RudderStackProxy = require('./middleware/rudderstack-proxy');

const app = express();
const server = http.createServer(app);

// Configuration
const PORT = process.env.PORT || 3001;
const RUDDERSTACK_DATA_PLANE_URL = process.env.RUDDERSTACK_DATA_PLANE_URL || 'https://hosted.rudderlabs.com';
const ENABLE_WEBSOCKET = process.env.ENABLE_WEBSOCKET === 'true' || false;

// Middleware
app.use(cors({
  origin: ['http://localhost:3000', 'http://localhost:8080', 'http://127.0.0.1:8080'],
  credentials: true
}));
app.use(bodyParser.json({ limit: '10mb' }));
app.use(bodyParser.urlencoded({ extended: true }));

// Initialize components
const sessionManager = new SessionManager();
const telemetryMiddleware = new TelemetryMiddleware(sessionManager);
const rudderStackProxy = new RudderStackProxy(RUDDERSTACK_DATA_PLANE_URL, sessionManager);

// HTTP-only broadcast function (logs to console instead of WebSocket)
function broadcast(data) {
  console.log('📡 Telemetry Event:', JSON.stringify(data, null, 2));
  // Store the event for HTTP polling if needed
  sessionManager.addBroadcastEvent(data);
}

// HTTP endpoint to get recent broadcast events
app.get('/api/events/recent', (req, res) => {
  const limit = parseInt(req.query.limit) || 50;
  const sessionId = req.query.sessionId;
  const events = sessionManager.getRecentBroadcastEvents(sessionId, limit);
  
  res.json({
    events,
    timestamp: new Date().toISOString(),
    total: events.length
  });
});

// Routes

// Health check
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    mode: 'HTTP-only (WebSocket disabled)',
    timestamp: new Date().toISOString(),
    active_sessions: sessionManager.getActiveSessions().length,
    rudderstack_url: RUDDERSTACK_DATA_PLANE_URL
  });
});

// Get all active sessions
app.get('/api/sessions', (req, res) => {
  res.json({
    sessions: sessionManager.getActiveSessions(),
    timestamp: new Date().toISOString()
  });
});

// Get specific session data
app.get('/api/sessions/:sessionId', (req, res) => {
  const sessionData = sessionManager.getSessionData(req.params.sessionId);
  if (!sessionData) {
    return res.status(404).json({
      error: 'Session not found',
      sessionId: req.params.sessionId
    });
  }
  res.json(sessionData);
});

// RudderStack telemetry proxy endpoints
app.post('/v1/track', (req, res) => {
  telemetryMiddleware.processEvent(req, res, 'track', broadcast);
  rudderStackProxy.forwardRequest(req, res, '/v1/track');
});

app.post('/v1/identify', (req, res) => {
  telemetryMiddleware.processEvent(req, res, 'identify', broadcast);
  rudderStackProxy.forwardRequest(req, res, '/v1/identify');
});

app.post('/v1/page', (req, res) => {
  telemetryMiddleware.processEvent(req, res, 'page', broadcast);
  rudderStackProxy.forwardRequest(req, res, '/v1/page');
});

app.post('/v1/screen', (req, res) => {
  telemetryMiddleware.processEvent(req, res, 'screen', broadcast);
  rudderStackProxy.forwardRequest(req, res, '/v1/screen');
});

app.post('/v1/group', (req, res) => {
  telemetryMiddleware.processEvent(req, res, 'group', broadcast);
  rudderStackProxy.forwardRequest(req, res, '/v1/group');
});

app.post('/v1/alias', (req, res) => {
  telemetryMiddleware.processEvent(req, res, 'alias', broadcast);
  rudderStackProxy.forwardRequest(req, res, '/v1/alias');
});

// Batch endpoint for multiple events
app.post('/v1/batch', (req, res) => {
  telemetryMiddleware.processBatchEvents(req, res, broadcast);
  rudderStackProxy.forwardRequest(req, res, '/v1/batch');
});

// Debug endpoints
app.post('/api/debug/flush-session', (req, res) => {
  const { sessionId, force } = req.body;
  
  try {
    const result = sessionManager.flushSession(sessionId, force);
    
    broadcast({
      type: 'session_flushed',
      sessionId,
      result,
      timestamp: new Date().toISOString()
    });
    
    res.json({
      success: true,
      sessionId,
      flushed_events: result.eventCount,
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    res.status(500).json({
      error: 'Failed to flush session',
      sessionId,
      message: error.message
    });
  }
});

app.post('/api/debug/simulate-flush-issue', (req, res) => {
  const { sessionId, issueType } = req.body;
  
  const issueTypes = {
    'timeout': () => sessionManager.simulateTimeout(sessionId),
    'network_error': () => sessionManager.simulateNetworkError(sessionId),
    'batch_limit': () => sessionManager.simulateBatchLimit(sessionId),
    'memory_pressure': () => sessionManager.simulateMemoryPressure(sessionId)
  };
  
  if (!issueTypes[issueType]) {
    return res.status(400).json({
      error: 'Invalid issue type',
      available_types: Object.keys(issueTypes)
    });
  }
  
  try {
    const result = issueTypes[issueType]();
    
    broadcast({
      type: 'flush_issue_simulated',
      sessionId,
      issueType,
      result,
      timestamp: new Date().toISOString()
    });
    
    res.json({
      success: true,
      sessionId,
      issueType,
      result,
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    res.status(500).json({
      error: 'Failed to simulate issue',
      message: error.message
    });
  }
});

// Error handling
app.use((error, req, res, next) => {
  console.error('Server error:', error);
  res.status(500).json({
    error: 'Internal server error',
    message: error.message,
    timestamp: new Date().toISOString()
  });
});

// Start server
server.listen(PORT, () => {
  console.log(`🚀 Telemetry development server running on port ${PORT}`);
  console.log(`📡 Mode: HTTP-only (WebSocket disabled)`);
  console.log(`📊 Dashboard: http://localhost:${PORT}/health`);
  console.log(`🔍 RudderStack Proxy: http://localhost:${PORT}/v1/*`);
  console.log(`📈 Events API: http://localhost:${PORT}/api/events/recent`);
});

// Graceful shutdown
process.on('SIGINT', () => {
  console.log('\n🛑 Shutting down gracefully...');
  
  // Close HTTP server
  server.close(() => {
    console.log('✅ Server closed');
    process.exit(0);
  });
});

module.exports = { app, server };
