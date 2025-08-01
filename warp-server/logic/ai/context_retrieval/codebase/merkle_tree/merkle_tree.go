package merkle_tree

import (
	"errors"
	"strings"
)

// NodeType represents different types of nodes in the merkle tree
type NodeType int

const (
	Unknown NodeType = iota
	File
	Directory
	Root
	N2L    // Node to Leaf
	RMeta  // Root Metadata
)

// RowKey represents a parsed row key for the merkle tree
type RowKey struct {
	NodeType   NodeType
	Hash       string
	Segments   []string
	Delimiters []string
}

// ParseRowKey parses a row key string into a RowKey struct
func ParseRowKey(rowKey string) (*RowKey, error) {
	if rowKey == "" {
		return nil, errors.New("empty row key")
	}

	// Split by common delimiters
	delimiters := []string{":", "/", "#", "_"}
	segments := []string{rowKey}
	usedDelimiters := []string{}

	for _, delimiter := range delimiters {
		newSegments := []string{}
		newDelimiters := []string{}
		
		for i, segment := range segments {
			if i > 0 {
				newDelimiters = append(newDelimiters, usedDelimiters[i-1])
			}
			
			parts := strings.Split(segment, delimiter)
			if len(parts) > 1 {
				for j, part := range parts {
					if j > 0 {
						newDelimiters = append(newDelimiters, delimiter)
					}
					newSegments = append(newSegments, part)
				}
			} else {
				newSegments = append(newSegments, segment)
			}
		}
		
		segments = newSegments
		usedDelimiters = newDelimiters
	}

	// Validate minimum segments
	if len(segments) < 1 {
		return nil, errors.New("insufficient segments in row key")
	}

	// Determine node type and extract hash
	nodeType := Unknown
	hash := ""
	
	if len(segments) > 0 {
		switch {
		case strings.Contains(rowKey, "node_type"):
			nodeType = File
		case strings.Contains(rowKey, "n2l_node_type"):
			nodeType = N2L
		case strings.Contains(rowKey, "rmeta_node_type"):
			nodeType = RMeta
		case strings.Contains(rowKey, "directory"):
			nodeType = Directory
		case strings.Contains(rowKey, "root"):
			nodeType = Root
		}
		
		// Extract hash from segments (usually the first or second segment)
		for _, segment := range segments {
			if len(segment) > 0 && segment != "node_type" && segment != "n2l_node_type" && segment != "rmeta_node_type" {
				hash = segment
				break
			}
		}
	}

	return &RowKey{
		NodeType:   nodeType,
		Hash:       hash,
		Segments:   segments,
		Delimiters: usedDelimiters,
	}, nil
}

// String returns the string representation of the RowKey
func (rk *RowKey) String() string {
	result := ""
	for i, segment := range rk.Segments {
		if i > 0 && i-1 < len(rk.Delimiters) {
			result += rk.Delimiters[i-1]
		}
		result += segment
	}
	return result
}

// Validate checks if the RowKey is valid
func (rk *RowKey) Validate() error {
	if rk.Hash == "" {
		return errors.New("hash segment is empty")
	}
	
	if len(rk.Segments) == 0 {
		return errors.New("no segments found")
	}
	
	if rk.NodeType == Unknown {
		return errors.New("unknown node type")
	}
	
	return nil
}

// GetNodeTypeString returns the string representation of the node type
func (rk *RowKey) GetNodeTypeString() string {
	switch rk.NodeType {
	case File:
		return "file"
	case Directory:
		return "directory"
	case Root:
		return "root"
	case N2L:
		return "n2l"
	case RMeta:
		return "rmeta"
	default:
		return "unknown"
	}
}
