package ai

// ContentHashJbool represents the existence status of a content hash
type ContentHashJbool struct {
	Hash   string `json:"hash"`
	Exists bool   `json:"exists"`
}

// Embedding represents a vector embedding
type Embedding struct {
	Hash      string    `json:"hash"`
	Vector    []float64 `json:"vector"`
	Model     string    `json:"model"`
	Dimension int       `json:"dimension"`
}

// EmbeddingResult contains the result of embedding generation
type EmbeddingResult struct {
	Embeddings []Embedding `json:"embeddings"`
	Success    bool        `json:"success"`
	Error      string      `json:"error,omitempty"`
}
