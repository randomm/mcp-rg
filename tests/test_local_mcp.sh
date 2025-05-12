#!/bin/bash
# Test the MCP server locally with a proper JSON-RPC sequence

# Create a temporary file for the response
TEMP_FILE=$(mktemp)

# Cleanup function
cleanup() {
  rm -f "$TEMP_FILE"
}
trap cleanup EXIT

echo "Testing local MCP server..."
echo "Sending initialize, tools/list, and a search request..."

# Test with proper initialization sequence
cat << 'JSON' | FILES_ROOT=$(pwd) LOG_LEVEL=debug ./target/release/mcp-rg > "$TEMP_FILE"
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search","arguments":{"pattern":"fn main","path":"src","fixed_strings":false,"line_numbers":true}}}
JSON

# Display the response
echo "Server Response:"
cat "$TEMP_FILE"

# Check if the response contains valid JSON with search results
if grep -q "matches" "$TEMP_FILE" && grep -q "stats" "$TEMP_FILE"; then
  echo ""
  echo "✅ SUCCESS: Local MCP server test passed!"
  echo "The MCP server correctly processed JSON-RPC requests via STDIN"
  echo "and responded via STDOUT."
else
  echo ""
  echo "❌ ERROR: Local MCP server test failed!"
  echo "The server did not correctly process the input or return the expected output."
fi