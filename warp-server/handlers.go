package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

// APIResponse represents a standard API response
type APIResponse struct {
	Success bool        `json:"success"`
	Message string      `json:"message,omitempty"`
	Data    interface{} `json:"data,omitempty"`
	Error   string      `json:"error,omitempty"`
}

// HomeHandler handles the root endpoint
func HomeHandler(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path != "/" {
		http.NotFound(w, r)
		return
	}

	response := APIResponse{
		Success: true,
		Message: "Welcome to Warp Server",
		Data: map[string]string{
			"version":   "1.0.0",
			"timestamp": time.Now().Format(time.RFC3339),
		},
	}

	writeJSONResponse(w, http.StatusOK, response)
}

// HealthHandler handles health check requests
func HealthHandler(w http.ResponseWriter, r *http.Request) {
	response := APIResponse{
		Success: true,
		Data: map[string]string{
			"status":    "healthy",
			"timestamp": time.Now().Format(time.RFC3339),
		},
	}

	writeJSONResponse(w, http.StatusOK, response)
}

// StatusHandler provides detailed server status
func StatusHandler(w http.ResponseWriter, r *http.Request) {
	response := APIResponse{
		Success: true,
		Data: map[string]interface{}{
			"server":     "warp-server",
			"status":     "running",
			"version":    "1.0.0",
			"timestamp":  time.Now().Format(time.RFC3339),
			"uptime":     "unknown", // Could be enhanced with actual uptime tracking
			"go_version": "1.24.4",
		},
	}

	writeJSONResponse(w, http.StatusOK, response)
}

// NotFoundHandler handles 404 errors
func NotFoundHandler(w http.ResponseWriter, r *http.Request) {
	response := APIResponse{
		Success: false,
		Error:   "endpoint not found",
	}

	writeJSONResponse(w, http.StatusNotFound, response)
}

// writeJSONResponse is a helper function to write JSON responses
func writeJSONResponse(w http.ResponseWriter, statusCode int, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(statusCode)

	if err := json.NewEncoder(w).Encode(data); err != nil {
		fmt.Printf("Error encoding JSON response: %v\n", err)
	}
}
