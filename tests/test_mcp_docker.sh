#!/bin/bash
# Test the MCP server running in Docker with STDIO transport

# Create a temporary file for the response
TEMP_FILE=$(mktemp)

# Cleanup function
cleanup() {
  rm -f "$TEMP_FILE"
}
trap cleanup EXIT

echo "Testing MCP server in Docker with STDIO transport..."
echo "Sending initialize, tools/list, and a search request..."

# Use the Docker container with STDIO transport (-i flag is critical)
cat << 'JSON' | docker run -i --rm \
  -v "$(pwd):/app/files" \
  -e FILES_ROOT=/app/files \
  -e LOG_LEVEL=debug \
  ripgrep-mcp > "$TEMP_FILE"
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search","arguments":{"pattern":"fn main","path":"src","fixed_strings":false,"line_numbers":true}}}
JSON

# Display the response
echo "Server Response from Docker container:"
cat "$TEMP_FILE"

# Check if the response contains valid JSON with search results
if grep -q "matches" "$TEMP_FILE" && grep -q "stats" "$TEMP_FILE"; then
  echo ""
  echo "✅ SUCCESS: Docker STDIO transport test passed!"
  echo "The MCP server correctly processed JSON-RPC requests via STDIN"
  echo "and responded via STDOUT while running in a Docker container."
  echo ""
  echo "This confirms that Docker containers can successfully use STDIO transport"
  echo "for MCP communication when run with the -i flag."
else
  echo ""
  echo "❌ ERROR: Docker STDIO transport test failed!"
  echo "The server did not correctly process the input or return the expected output."
fi