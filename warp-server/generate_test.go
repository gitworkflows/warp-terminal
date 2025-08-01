package main

import (
	"context"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/warpdotdev/warp-server/logic/ai/embeddings/generator/caching_generator"
	"github.com/warpdotdev/warp-server/model/types"
	"github.com/warpdotdev/warp-server/model/types/ai"
	graphql_types "github.com/warpdotdev/warp-server/model/types/v2"
)

// MockStore implements the Store interface for testing
type MockStore struct {
	mock.Mock
}

func (m *MockStore) CheckFragmentEmbeddingsExist(ctx context.Context, embeddingConfig graphql_types.EmbeddingConfig, repoMetadata types.RepoMetadata, hashes []string, userUID string) ([]ai.ContentHashJbool, error) {
	args := m.Called(ctx, embeddingConfig, repoMetadata, hashes, userUID)
	return args.Get(0).([]ai.ContentHashJbool), args.Error(1)
}

func (m *MockStore) StoreFragmentEmbeddings(ctx context.Context, embeddingConfig graphql_types.EmbeddingConfig, repoMetadata types.RepoMetadata, graphql_types types.GraphQLTypes) error {
	args := m.Called(ctx, embeddingConfig, repoMetadata, graphql_types)
	return args.Error(0)
}

func TestGenerateEmbeddings(t *testing.T) {
	ctx := context.Background()
	
	tests := []struct {
		name           string
		embeddingConfig graphql_types.EmbeddingConfig
		repoMetadata   types.RepoMetadata
		fragments      []types.Fragment
		userUID        string
		setupMock      func(*MockStore)
		expectError    bool
		expectedCount  int
	}{
		{
			name: "successful_generation_with_new_fragments",
			embeddingConfig: graphql_types.EmbeddingConfig{
				Model:     "text-embedding-ada-002",
				BatchSize: 100,
			},
			repoMetadata: types.RepoMetadata{
				RepoName: "test-repo",
				Branch:   "main",
				CommitSHA: "abc123",
			},
			fragments: []types.Fragment{
				{
					Hash:    "fragment1",
					Content: "test content 1",
					Path:    "src/test1.go",
				},
				{
					Hash:    "fragment2", 
					Content: "test content 2",
					Path:    "src/test2.go",
				},
			},
			userUID: "user123",
			setupMock: func(mockStore *MockStore) {
				// Mock that no embeddings exist yet
				mockStore.On("CheckFragmentEmbeddingsExist", mock.Anything, mock.Anything, mock.Anything, mock.MatchedBy(func(hashes []string) bool {
					return len(hashes) == 2 && hashes[0] == "fragment1" && hashes[1] == "fragment2"
				}), "user123").Return([]ai.ContentHashJbool{
					{Hash: "fragment1", Exists: false},
					{Hash: "fragment2", Exists: false},
				}, nil)
				
				// Mock successful storage
				mockStore.On("StoreFragmentEmbeddings", mock.Anything, mock.Anything, mock.Anything, mock.Anything).Return(nil)
			},
			expectError:   false,
			expectedCount: 2,
		},
		{
			name: "partial_generation_with_existing_fragments",
			embeddingConfig: graphql_types.EmbeddingConfig{
				Model:     "text-embedding-ada-002",
				BatchSize: 100,
			},
			repoMetadata: types.RepoMetadata{
				RepoName: "test-repo",
				Branch:   "main", 
				CommitSHA: "abc123",
			},
			fragments: []types.Fragment{
				{
					Hash:    "fragment1",
					Content: "test content 1",
					Path:    "src/test1.go",
				},
				{
					Hash:    "fragment2",
					Content: "test content 2", 
					Path:    "src/test2.go",
				},
			},
			userUID: "user123",
			setupMock: func(mockStore *MockStore) {
				// Mock that one embedding already exists
				mockStore.On("CheckFragmentEmbeddingsExist", mock.Anything, mock.Anything, mock.Anything, mock.MatchedBy(func(hashes []string) bool {
					return len(hashes) == 2
				}), "user123").Return([]ai.ContentHashJbool{
					{Hash: "fragment1", Exists: true},
					{Hash: "fragment2", Exists: false},
				}, nil)
				
				// Mock successful storage for only the new fragment
				mockStore.On("StoreFragmentEmbeddings", mock.Anything, mock.Anything, mock.Anything, mock.Anything).Return(nil)
			},
			expectError:   false,
			expectedCount: 1, // Only one new embedding generated
		},
		{
			name: "error_checking_existing_embeddings",
			embeddingConfig: graphql_types.EmbeddingConfig{
				Model:     "text-embedding-ada-002",
				BatchSize: 100,
			},
			repoMetadata: types.RepoMetadata{
				RepoName: "test-repo",
				Branch:   "main",
				CommitSHA: "abc123",
			},
			fragments: []types.Fragment{
				{
					Hash:    "fragment1",
					Content: "test content 1",
					Path:    "src/test1.go",
				},
			},
			userUID: "user123",
			setupMock: func(mockStore *MockStore) {
				mockStore.On("CheckFragmentEmbeddingsExist", mock.Anything, mock.Anything, mock.Anything, mock.Anything, "user123").Return([]ai.ContentHashJbool{}, errors.New("database error"))
			},
			expectError:   true,
			expectedCount: 0,
		},
		{
			name: "error_storing_embeddings",
			embeddingConfig: graphql_types.EmbeddingConfig{
				Model:     "text-embedding-ada-002",
				BatchSize: 100,
			},
			repoMetadata: types.RepoMetadata{
				RepoName: "test-repo",
				Branch:   "main",
				CommitSHA: "abc123",
			},
			fragments: []types.Fragment{
				{
					Hash:    "fragment1", 
					Content: "test content 1",
					Path:    "src/test1.go",
				},
			},
			userUID: "user123",
			setupMock: func(mockStore *MockStore) {
				mockStore.On("CheckFragmentEmbeddingsExist", mock.Anything, mock.Anything, mock.Anything, mock.Anything, "user123").Return([]ai.ContentHashJbool{
					{Hash: "fragment1", Exists: false},
				}, nil)
				
				mockStore.On("StoreFragmentEmbeddings", mock.Anything, mock.Anything, mock.Anything, mock.Anything).Return(errors.New("storage error"))
			},
			expectError:   true,
			expectedCount: 0,
		},
		{
			name: "empty_fragments_list",
			embeddingConfig: graphql_types.EmbeddingConfig{
				Model:     "text-embedding-ada-002",
				BatchSize: 100,
			},
			repoMetadata: types.RepoMetadata{
				RepoName: "test-repo",
				Branch:   "main",
				CommitSHA: "abc123",
			},
			fragments:     []types.Fragment{},
			userUID:       "user123",
			setupMock:     func(mockStore *MockStore) {
				// No mock setup needed for empty fragments
			},
			expectError:   false,
			expectedCount: 0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockStore := &MockStore{}
			tt.setupMock(mockStore)

			// Create the generator with the mock store
			generator := caching_generator.New(mockStore)

			// Call GenerateEmbeddings
			result, err := generator.GenerateEmbeddings(ctx, tt.embeddingConfig, tt.repoMetadata, tt.fragments, tt.userUID)

			// Assert error expectation
			if tt.expectError {
				assert.Error(t, err)
				assert.Nil(t, result)
			} else {
				assert.NoError(t, err)
				assert.NotNil(t, result)
				assert.Equal(t, tt.expectedCount, len(result.Embeddings))
			}

			// Verify all mock expectations were met
			mockStore.AssertExpectations(t)
		})
	}
}
