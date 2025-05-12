# MCP Server Tests

This directory contains test scripts to verify the functionality of the Ripgrep MCP server.

## Test Scripts

- `test_local_mcp.sh`: Tests the locally built binary using STDIO transport
- `test_mcp_docker.sh`: Tests the Docker container using STDIO transport
- `test_mcp.json`: Example JSON-RPC search request

## Running Tests

Make sure the scripts are executable:

```bash
chmod +x test_local_mcp.sh test_mcp_docker.sh
```

### Testing the local build

```bash
./test_local_mcp.sh
```

### Testing the Docker container

Ensure you've built the Docker image first:

```bash
cd ..  # Go to the project root
docker build -t ripgrep-mcp .
./tests/test_mcp_docker.sh
```

## Expected Results

Both test scripts should show a successful response with search results. If the tests pass, you'll see a success message confirming that the MCP server is working correctly.

## Common Issues

- If the local test fails, make sure you've built the binary (`cargo build --release`)
- If the Docker test fails, make sure you've built the Docker image (`docker build -t ripgrep-mcp .`)
- Both tests assume you're running them from the project root directory