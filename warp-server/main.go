package main

import (
	"log"
	"net/http"
)

func main() {
	// Load configuration
	config := LoadConfig()

	// Setup routes using the new handlers
	http.HandleFunc("/", HomeHandler)
	http.HandleFunc("/health", HealthHandler)
	http.HandleFunc("/api/status", StatusHandler)
	http.HandleFunc("/api/database/stats", DatabaseStatsHandler)
	http.HandleFunc("/api/commands/recent", RecentCommandsHandler)
	http.HandleFunc("/api/ai/queries/recent", RecentAIQueriesHandler)

	log.Printf("Warp Server starting on port %s (Environment: %s)", config.Port, config.Environment)
	log.Fatal(http.ListenAndServe(":"+config.Port, nil))
}
