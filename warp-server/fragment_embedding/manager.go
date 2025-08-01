package fragment_embedding

import (
	graphql_types "github.com/warpdotdev/warp-server/model/types/v2"
)

// ProcessResults processes embedding results with defensive nil checks
func ProcessResults(results []*graphql_types.GenerateCodeEmbeddingResult) []string {
	successfulHashes := make([]string, 0)
	
	for _, result := range results {
		// Defensive check to avoid nil pointer dereference
		if result != nil && result.Success {
			successfulHashes = append(successfulHashes, result.Hash)
		}
	}
	
	return successfulHashes
}

// AppendResults safely appends embedding results to a slice
func AppendResults(successfulHashes []string, results []*graphql_types.GenerateCodeEmbeddingResult) []string {
	for _, result := range results {
		// Defensive check to avoid nil pointer dereference  
		if result != nil && result.Success {
			successfulHashes = append(successfulHashes, result.Hash)
		}
	}
	
	return successfulHashes
}
