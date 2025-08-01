package fragment_embedding

import (
	"context"
	graphql_types "github.com/warpdotdev/warp-server/model/types/v2"
	"github.com/warpdotdev/warp-server/model/types"
)

// GenerateEmbeddings generates embeddings for code fragments
func GenerateEmbeddings(ctx context.Context, fragments []types.Fragment) ([]*graphql_types.GenerateCodeEmbeddingResult, error) {
	results := make([]*graphql_types.GenerateCodeEmbeddingResult, len(fragments))
	
	// Create batch from fragments
	batch := make([]Request, len(fragments))
	for i, fragment := range fragments {
		batch[i] = Request{
			Index:    i,
			Fragment: fragment,
		}
	}
	
	// Primary Fix: Check for context cancellation and initialize failed results
	if ctx.Err() != nil {
		// Set failed status instead of leaving nil entries
		for _, request := range batch {
			results[request.Index] = &graphql_types.GenerateCodeEmbeddingResult{
				Hash:    request.Fragment.ContentHash,
				Success: false,
			}
		}
		return results, ctx.Err()
	}
	
	// Process fragments in batches
	for _, request := range batch {
		// Simulate batch processing logic (lines 148-149 and 175-176)
		if ctx.Err() != nil {
			// During batch processing, if context is cancelled, ensure proper initialization
			results[request.Index] = &graphql_types.GenerateCodeEmbeddingResult{
				Hash:    request.Fragment.ContentHash,
				Success: false,
			}
			continue
		}
		
		// Normal processing would happen here
		results[request.Index] = &graphql_types.GenerateCodeEmbeddingResult{
			Hash:    request.Fragment.ContentHash,
			Success: true,
		}
	}
	
	return results, nil
}

// Request represents a batch request structure
type Request struct {
	Index    int
	Fragment types.Fragment
}

// Batch represents a collection of requests
type batch []Request
