package types

import "time"

// RepoMetadata contains metadata about a repository
type RepoMetadata struct {
	RepoName  string `json:"repo_name"`
	Branch    string `json:"branch"`
	CommitSHA string `json:"commit_sha"`
	RepoID    string `json:"repo_id,omitempty"`
	Owner     string `json:"owner,omitempty"`
}

// Fragment represents a code fragment for embedding
type Fragment struct {
	Hash        string            `json:"hash"`
	ContentHash string            `json:"content_hash"`
	Content     string            `json:"content"`
	Path        string            `json:"path"`
	StartLine   int               `json:"start_line,omitempty"`
	EndLine     int               `json:"end_line,omitempty"`
	Language    string            `json:"language,omitempty"`
	Metadata    map[string]string `json:"metadata,omitempty"`
	CreatedAt   time.Time         `json:"created_at,omitempty"`
}

// GraphQLTypes represents GraphQL type definitions
type GraphQLTypes struct {
	Types []string `json:"types"`
}
