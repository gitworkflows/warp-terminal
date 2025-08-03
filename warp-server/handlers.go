package main

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"time"

	_ "github.com/mattn/go-sqlite3"
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

// DatabaseStatsHandler provides database statistics
func DatabaseStatsHandler(w http.ResponseWriter, r *http.Request) {
	config := LoadConfig()
	dbPath := config.DatabasePath
	
	db, err := sql.Open("sqlite3", dbPath)
	if err != nil {
		response := APIResponse{
			Success: false,
			Error:   fmt.Sprintf("Failed to open database: %v", err),
		}
		writeJSONResponse(w, http.StatusInternalServerError, response)
		return
	}
	defer db.Close()

	// Get table counts
	tables := []string{
		"blocks", "commands", "ai_queries", "ai_blocks", "terminal_panes",
		"notebooks", "workflows", "tabs", "windows", "users",
	}

	tableCounts := make(map[string]int)
	for _, table := range tables {
		var count int
		err := db.QueryRow(fmt.Sprintf("SELECT COUNT(*) FROM %s", table)).Scan(&count)
		if err != nil {
			tableCounts[table] = -1 // Mark as error
		} else {
			tableCounts[table] = count
		}
	}

	// Get database size (if possible)
	var pageCount, pageSize int
	db.QueryRow("PRAGMA page_count").Scan(&pageCount)
	db.QueryRow("PRAGMA page_size").Scan(&pageSize)
	dbSize := pageCount * pageSize

	response := APIResponse{
		Success: true,
		Data: map[string]interface{}{
			"database_path": dbPath,
			"database_size_bytes": dbSize,
			"table_counts": tableCounts,
			"timestamp": time.Now().Format(time.RFC3339),
		},
	}

	writeJSONResponse(w, http.StatusOK, response)
}

// RecentCommandsHandler returns recent commands from the database
func RecentCommandsHandler(w http.ResponseWriter, r *http.Request) {
	config := LoadConfig()
	dbPath := config.DatabasePath

	limitStr := r.URL.Query().Get("limit")
	limit := 10 // default
	if limitStr != "" {
		if l, err := strconv.Atoi(limitStr); err == nil && l > 0 && l <= 100 {
			limit = l
		}
	}

	db, err := sql.Open("sqlite3", dbPath)
	if err != nil {
		response := APIResponse{
			Success: false,
			Error:   fmt.Sprintf("Failed to open database: %v", err),
		}
		writeJSONResponse(w, http.StatusInternalServerError, response)
		return
	}
	defer db.Close()

	query := `
		SELECT command, exit_code, start_ts, completed_ts, pwd, shell, username, hostname 
		FROM commands 
		ORDER BY start_ts DESC 
		LIMIT ?
	`

	rows, err := db.Query(query, limit)
	if err != nil {
		response := APIResponse{
			Success: false,
			Error:   fmt.Sprintf("Failed to query commands: %v", err),
		}
		writeJSONResponse(w, http.StatusInternalServerError, response)
		return
	}
	defer rows.Close()

	type Command struct {
		Command     string     `json:"command"`
		ExitCode    *int       `json:"exit_code"`
		StartTs     *time.Time `json:"start_ts"`
		CompletedTs *time.Time `json:"completed_ts"`
		Pwd         *string    `json:"pwd"`
		Shell       *string    `json:"shell"`
		Username    *string    `json:"username"`
		Hostname    *string    `json:"hostname"`
	}

	var commands []Command
	for rows.Next() {
		var cmd Command
		err := rows.Scan(&cmd.Command, &cmd.ExitCode, &cmd.StartTs, &cmd.CompletedTs, &cmd.Pwd, &cmd.Shell, &cmd.Username, &cmd.Hostname)
		if err != nil {
			continue
		}
		commands = append(commands, cmd)
	}

	response := APIResponse{
		Success: true,
		Data: map[string]interface{}{
			"commands": commands,
			"count":    len(commands),
			"limit":    limit,
		},
	}

	writeJSONResponse(w, http.StatusOK, response)
}

// RecentAIQueriesHandler returns recent AI queries
func RecentAIQueriesHandler(w http.ResponseWriter, r *http.Request) {
	config := LoadConfig()
	dbPath := config.DatabasePath

	limitStr := r.URL.Query().Get("limit")
	limit := 10
	if limitStr != "" {
		if l, err := strconv.Atoi(limitStr); err == nil && l > 0 && l <= 50 {
			limit = l
		}
	}

	db, err := sql.Open("sqlite3", dbPath)
	if err != nil {
		response := APIResponse{
			Success: false,
			Error:   fmt.Sprintf("Failed to open database: %v", err),
		}
		writeJSONResponse(w, http.StatusInternalServerError, response)
		return
	}
	defer db.Close()

	query := `
		SELECT exchange_id, conversation_id, start_ts, input, working_directory, output_status, model_id
		FROM ai_queries 
		ORDER BY start_ts DESC 
		LIMIT ?
	`

	rows, err := db.Query(query, limit)
	if err != nil {
		response := APIResponse{
			Success: false,
			Error:   fmt.Sprintf("Failed to query AI queries: %v", err),
		}
		writeJSONResponse(w, http.StatusInternalServerError, response)
		return
	}
	defer rows.Close()

	type AIQuery struct {
		ExchangeId       string     `json:"exchange_id"`
		ConversationId   string     `json:"conversation_id"`
		StartTs          *time.Time `json:"start_ts"`
		Input            string     `json:"input"`
		WorkingDirectory *string    `json:"working_directory"`
		OutputStatus     string     `json:"output_status"`
		ModelId          string     `json:"model_id"`
	}

	var queries []AIQuery
	for rows.Next() {
		var query AIQuery
		err := rows.Scan(&query.ExchangeId, &query.ConversationId, &query.StartTs, &query.Input, &query.WorkingDirectory, &query.OutputStatus, &query.ModelId)
		if err != nil {
			continue
		}
		queries = append(queries, query)
	}

	response := APIResponse{
		Success: true,
		Data: map[string]interface{}{
			"ai_queries": queries,
			"count":      len(queries),
			"limit":      limit,
		},
	}

	writeJSONResponse(w, http.StatusOK, response)
}

// writeJSONResponse is a helper function to write JSON responses
func writeJSONResponse(w http.ResponseWriter, statusCode int, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(statusCode)

	if err := json.NewEncoder(w).Encode(data); err != nil {
		fmt.Printf("Error encoding JSON response: %v\n", err)
	}
}
