package bigtable

import (
	"context"
	"fmt"
	"log"
)

// Client represents a BigTable client
type Client struct {
	ProjectID string
	Instance  string
	TableName string
}

// NewClient creates a new BigTable client
func NewClient(projectID, instance, tableName string) *Client {
	return &Client{
		ProjectID: projectID,
		Instance:  instance,
		TableName: tableName,
	}
}

// Row represents a BigTable row
type Row struct {
	Key    string
	Values map[string][]byte
}

// ReadRow reads a single row from BigTable
func (c *Client) ReadRow(ctx context.Context, rowKey string) (*Row, error) {
	if rowKey == "" {
		return nil, fmt.Errorf("row key cannot be empty")
	}

	// Mock implementation for testing
	log.Printf("Reading row with key: %s from table: %s", rowKey, c.TableName)
	
	// Return a mock row
	return &Row{
		Key: rowKey,
		Values: map[string][]byte{
			"cf1:data": []byte("mock_data"),
		},
	}, nil
}

// WriteRow writes a single row to BigTable
func (c *Client) WriteRow(ctx context.Context, row *Row) error {
	if row == nil || row.Key == "" {
		return fmt.Errorf("invalid row or empty key")
	}

	// Mock implementation for testing
	log.Printf("Writing row with key: %s to table: %s", row.Key, c.TableName)
	
	return nil
}

// DeleteRow deletes a single row from BigTable
func (c *Client) DeleteRow(ctx context.Context, rowKey string) error {
	if rowKey == "" {
		return fmt.Errorf("row key cannot be empty")
	}

	// Mock implementation for testing
	log.Printf("Deleting row with key: %s from table: %s", rowKey, c.TableName)
	
	return nil
}

// Close closes the BigTable client connection
func (c *Client) Close() error {
	log.Printf("Closing BigTable client for project: %s, instance: %s", c.ProjectID, c.Instance)
	return nil
}
