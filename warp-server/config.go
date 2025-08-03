package main

import (
	"os"
	"strconv"
)

// Config holds the application configuration
type Config struct {
	Port         string
	Environment  string
	LogLevel     string
	DatabasePath string
}

// LoadConfig loads configuration from environment variables
func LoadConfig() *Config {
	return &Config{
		Port:         getEnv("PORT", "8080"),
		Environment:  getEnv("ENV", "development"),
		LogLevel:     getEnv("LOG_LEVEL", "info"),
		DatabasePath: getEnv("DATABASE_PATH", "/Users/KhulnaSoft/Library/Application Support/dev.warp.Warp-Preview/warp.sqlite"),
	}
}

// getEnv gets an environment variable with a fallback default value
func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

// getEnvAsInt gets an environment variable as integer with a fallback default value
func getEnvAsInt(key string, defaultValue int) int {
	if value := os.Getenv(key); value != "" {
		if intVal, err := strconv.Atoi(value); err == nil {
			return intVal
		}
	}
	return defaultValue
}
