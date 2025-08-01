package fragment_embedding

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/warpdotdev/warp-server/model/types"
	graphql_types "github.com/warpdotdev/warp-server/model/types/v2"
)

func TestGenerateEmbeddings_ContextCancellation(t *testing.T) {
	tests := []struct {
		name             string
		fragments        []types.Fragment
		setupContext     func() (context.Context, context.CancelFunc)
		expectError      bool
		expectNilResults bool
	}{
		{
			name: "context_cancelled_before_processing",
			fragments: []types.Fragment{
				{
					Hash:        "fragment1",
					ContentHash: "hash1",
					Content:     "test content 1",
					Path:        "src/test1.go",
				},
				{
					Hash:        "fragment2", 
					ContentHash: "hash2",
					Content:     "test content 2",
					Path:        "src/test2.go",
				},
			},
			setupContext: func() (context.Context, context.CancelFunc) {
				ctx, cancel := context.WithCancel(context.Background())
				cancel() // Cancel immediately
				return ctx, cancel
			},
			expectError:      true,
			expectNilResults: false, // Should not have nil results due to our fix
		},
		{
			name: "context_cancelled_during_processing",
			fragments: []types.Fragment{
				{
					Hash:        "fragment1",
					ContentHash: "hash1", 
					Content:     "test content 1",
					Path:        "src/test1.go",
				},
			},
			setupContext: func() (context.Context, context.CancelFunc) {
				ctx, cancel := context.WithTimeout(context.Background(), 1*time.Millisecond)
				return ctx, cancel
			},
			expectError:      false, // May or may not error depending on timing
			expectNilResults: false, // Should not have nil results due to our fix
		},
		{
			name: "normal_processing_without_cancellation",
			fragments: []types.Fragment{
				{
					Hash:        "fragment1",
					ContentHash: "hash1",
					Content:     "test content 1", 
					Path:        "src/test1.go",
				},
			},
			setupContext: func() (context.Context, context.CancelFunc) {
				return context.WithCancel(context.Background())
			},
			expectError:      false,
			expectNilResults: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx, cancel := tt.setupContext()
			defer cancel()

			// Add a small delay for timeout scenarios
			if tt.name == "context_cancelled_during_processing" {
				time.Sleep(2 * time.Millisecond)
			}

			results, err := GenerateEmbeddings(ctx, tt.fragments)

			// Check error expectation
			if tt.expectError {
				assert.Error(t, err)
			}

			// Most importantly: verify no nil results (the bug we're fixing)
			assert.NotNil(t, results)
			assert.Equal(t, len(tt.fragments), len(results))
			
			for i, result := range results {
				assert.NotNil(t, result, "Result at index %d should not be nil", i)
				assert.Equal(t, tt.fragments[i].ContentHash, result.Hash)
				// Success should be false if context was cancelled
				if ctx.Err() != nil {
					assert.False(t, result.Success)
				}
			}
		})
	}
}

func TestProcessResults_DefensiveNilCheck(t *testing.T) {
	tests := []struct {
		name           string
		results        []*graphql_types.GenerateCodeEmbeddingResult
		expectedHashes []string
	}{
		{
			name: "all_valid_results",
			results: []*graphql_types.GenerateCodeEmbeddingResult{
				{Hash: "hash1", Success: true},
				{Hash: "hash2", Success: true},
			},
			expectedHashes: []string{"hash1", "hash2"},
		},
		{
			name: "mixed_success_and_failure",
			results: []*graphql_types.GenerateCodeEmbeddingResult{
				{Hash: "hash1", Success: true},
				{Hash: "hash2", Success: false},
				{Hash: "hash3", Success: true},
			},
			expectedHashes: []string{"hash1", "hash3"},
		},
		{
			name: "nil_results_in_slice",
			results: []*graphql_types.GenerateCodeEmbeddingResult{
				{Hash: "hash1", Success: true},
				nil, // This should be handled gracefully
				{Hash: "hash3", Success: true},
			},
			expectedHashes: []string{"hash1", "hash3"},
		},
		{
			name:           "all_nil_results",
			results:        []*graphql_types.GenerateCodeEmbeddingResult{nil, nil, nil},
			expectedHashes: []string{},
		},
		{
			name:           "empty_results",
			results:        []*graphql_types.GenerateCodeEmbeddingResult{},
			expectedHashes: []string{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// This should not panic even with nil entries
			hashes := ProcessResults(tt.results)
			assert.Equal(t, tt.expectedHashes, hashes)
		})
	}
}

func TestAppendResults_DefensiveNilCheck(t *testing.T) {
	initialHashes := []string{"existing1", "existing2"}
	
	results := []*graphql_types.GenerateCodeEmbeddingResult{
		{Hash: "hash1", Success: true},
		nil, // This should be handled gracefully
		{Hash: "hash2", Success: false},
		{Hash: "hash3", Success: true},
	}

	finalHashes := AppendResults(initialHashes, results)
	
	expected := []string{"existing1", "existing2", "hash1", "hash3"}
	assert.Equal(t, expected, finalHashes)
}
