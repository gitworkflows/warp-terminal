package merkle_tree

import (
	"testing"
)

func TestParseRowKey(t *testing.T) {
	tests := []struct {
		name    string
		rowKey  string
		wantErr bool
	}{
		{
			name:    "Valid_rowKey_with_node_type",
			rowKey:  "hash123:node_type:file",
			wantErr: false,
		},
		{
			name:    "Valid_rowKey_with_node_type#01",
			rowKey:  "hash456#node_type#directory",
			wantErr: false,
		},
		{
			name:    "Valid_rowKey_with_n2l_node_type",
			rowKey:  "hash789/n2l_node_type/leaf",
			wantErr: false,
		},
		{
			name:    "Valid_rowKey_with_rmeta_node_type",
			rowKey:  "hash_abc_rmeta_node_type_metadata",
			wantErr: false,
		},
		{
			name:    "Invalid_rowKey_with_insufficient_segments",
			rowKey:  "",
			wantErr: true,
		},
		{
			name:    "Empty_rowKey",
			rowKey:  "",
			wantErr: true,
		},
		{
			name:    "RowKey_with_excessive_delimiters",
			rowKey:  "hash:::node_type:::file:::extra",
			wantErr: false,
		},
		{
			name:    "RowKey_with_empty_hash_segment",
			rowKey:  ":node_type:file",
			wantErr: false, // Will be caught by validation, not parsing
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			rowKey, err := ParseRowKey(tt.rowKey)
			if (err != nil) != tt.wantErr {
				t.Errorf("ParseRowKey() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			
			if !tt.wantErr && rowKey == nil {
				t.Errorf("ParseRowKey() returned nil without error")
				return
			}
			
			if !tt.wantErr {
				// Additional validation for successful parses
				if len(rowKey.Segments) == 0 {
					t.Errorf("ParseRowKey() returned empty segments")
				}
				
				// Test that we can convert back to string
				reconstructed := rowKey.String()
				if reconstructed == "" {
					t.Errorf("RowKey.String() returned empty string")
				}
			}
		})
	}
}
