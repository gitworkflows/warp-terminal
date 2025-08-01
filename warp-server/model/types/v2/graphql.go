package v2

// EmbeddingConfig contains configuration for embedding generation
type EmbeddingConfig struct {
	Model         string  `json:"model"`
	BatchSize     int     `json:"batch_size"`
	MaxTokens     int     `json:"max_tokens,omitempty"`
	Temperature   float64 `json:"temperature,omitempty"`
	Dimension     int     `json:"dimension,omitempty"`
	Normalize     bool    `json:"normalize,omitempty"`
	Provider      string  `json:"provider,omitempty"`
}

// GenerateCodeEmbeddingResult represents the result of code embedding generation
type GenerateCodeEmbeddingResult struct {
	Hash    string `json:"hash"`
	Success bool   `json:"success"`
}
