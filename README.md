# Ripgrep MCP Server

A minimal and powerful MCP (Model Context Protocol) server that wraps ripgrep for efficient code search. This server allows AI models like Anthropic's Claude to search code repositories quickly and accurately using ripgrep's high-performance search capabilities.

## Overview

The Ripgrep MCP server provides AI models with a powerful tool to search through code repositories. It supports:

- Fast, efficient code search using ripgrep's optimized algorithms
- Regular expression and literal string search
- Path filtering and type-specific searches
- Clear, consistent JSON output that AI models can easily interpret
- Security features like path traversal prevention
- Both local binary and Docker container deployment options

## Installation

### Prerequisites

- Rust 1.86 or later
- ripgrep (`rg`) installed and in PATH
- Docker (optional, for containerized deployment)

### 1. Building from source (recommended for local use)

```bash
# Clone the repository
git clone https://github.com/yourusername/mcp-rg.git
cd mcp-rg

# Build for release
cargo build --release

# The binary will be available at target/release/mcp-rg
```

### 2. Building Docker image (recommended for cross-platform use)

```bash
# Clone the repository
git clone https://github.com/yourusername/mcp-rg.git
cd mcp-rg

# Build Docker image
docker build -t ripgrep-mcp .
```

## MCP Server Setup in Different Environments

### 1. Claude Desktop Setup

Claude Desktop allows you to use MCP servers through simple configuration. Follow these steps:

1. Open Claude menu on your computer and select "Settings…"
2. Click on "Developer" in the left-hand bar of the Settings pane
3. Click on "Edit Config"
4. Add the ripgrep MCP configuration to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "ripgrep": {
      "command": "/path/to/mcp-rg/target/release/mcp-rg",
      "env": {
        "FILES_ROOT": "/path/to/your/code/directory",
        "LOG_LEVEL": "error"
      }
    }
  }
}
```

5. Replace `/path/to/mcp-rg/target/release/mcp-rg` with the actual absolute path to the compiled binary
6. Set `FILES_ROOT` to the directory you want to search (must be an absolute path)
7. Restart Claude Desktop
8. A hammer icon should appear in your input box - click it to see and use the ripgrep search tool

#### Using Docker with Claude Desktop

You can also run the MCP server in a Docker container:

```json
{
  "mcpServers": {
    "ripgrep-docker": {
      "command": "docker",
      "args": [
        "run", 
        "-i", 
        "--rm", 
        "-v", 
        "/path/to/your/code:/app/files", 
        "-e", 
        "FILES_ROOT=/app/files", 
        "-e", 
        "LOG_LEVEL=error", 
        "ripgrep-mcp"
      ]
    }
  }
}
```

**Important**: The `-i` flag is crucial for STDIO transport to work correctly with Docker.

### 2. Claude Code CLI Setup

Claude Code CLI provides a more direct way to add and manage MCP servers:

```bash
# Add the ripgrep MCP server to your local environment
claude mcp add ripgrep -e FILES_ROOT=/path/to/your/code/directory -- /path/to/mcp-rg/target/release/mcp-rg

# Or use the add-json command for more configuration options
claude mcp add-json ripgrep '{
  "type": "stdio",
  "command": "/path/to/mcp-rg/target/release/mcp-rg",
  "env": {
    "FILES_ROOT": "/path/to/your/code/directory",
    "LOG_LEVEL": "error"
  }
}'
```

To add it to a project for the entire team, use:

```bash
# Add to project scope (creates/updates .mcp.json)
claude mcp add ripgrep -s project -e FILES_ROOT=/path/to/your/code/directory -- /path/to/mcp-rg/target/release/mcp-rg
```

#### Using Docker with Claude Code

You can also run the MCP server in a Docker container with Claude Code:

```bash
# Add the ripgrep MCP server in a Docker container
claude mcp add ripgrep-docker -- docker run -i --rm -v /path/to/your/code:/app/files -e FILES_ROOT=/app/files ripgrep-mcp
```

### 3. Cursor Editor Setup

To use the ripgrep MCP server with Cursor editor:

1. Navigate to Settings, then Cursor Settings
2. Select MCP on the left
3. Click "Add new global MCP server" at the top right
4. Enter the following configuration:

```json
{
  "ripgrep": {
    "command": "/path/to/mcp-rg/target/release/mcp-rg",
    "env": {
      "FILES_ROOT": "/path/to/your/code/directory",
      "LOG_LEVEL": "error"
    }
  }
}
```

5. Save the configuration and restart Cursor

#### Using Docker with Cursor

You can also run the MCP server in a Docker container:

```json
{
  "ripgrep-docker": {
    "command": "docker",
    "args": [
      "run", 
      "-i", 
      "--rm", 
      "-v", 
      "/path/to/your/code:/app/files", 
      "-e", 
      "FILES_ROOT=/app/files", 
      "ripgrep-mcp"
    ]
  }
}
```

### 4. Docker Deployment

For containerized deployment:

```bash
# Build Docker image
docker build -t ripgrep-mcp .

# Run with STDIO transport (for direct model connections)
# The -i flag is crucial for STDIO transport to work
docker run -i --rm -v /path/to/your/code:/app/files -e FILES_ROOT=/app/files ripgrep-mcp

# Using docker-compose
docker-compose up
```

## Transport Protocols

The server implements the following MCP transport protocols:

- **STDIO (Standard Input/Output)**: The default transport for local integrations. All examples above use this transport method.
- **HTTP with SSE**: Not currently implemented, but could be added for remote connections in the future.

### STDIO Transport with Docker

Docker containers can successfully use STDIO transport for MCP communication. Key requirements:

1. The Docker container must be run with the `-i` flag or `stdin_open: true` in docker-compose.yml to keep STDIN open
2. All logging in the application should be directed to STDERR, not STDOUT, to avoid interfering with the JSON-RPC protocol
3. The container should be built for the same architecture as the host system, or using a multi-stage build
4. For Claude Desktop or other MCP clients to communicate with the Docker container, they must be configured to launch Docker with the proper arguments

Example docker-compose.yml configuration:
```yaml
services:
  ripgrep-mcp:
    build: .
    stdin_open: true  # Equivalent to -i flag, keeps STDIN open
    volumes:
      - ./:/app/files
    environment:
      - FILES_ROOT=/app/files
```

## API Documentation

The server implements the Model Context Protocol (MCP) and exposes a single tool:

### Tool: `search`

Searches files using ripgrep.

#### Parameters

- `pattern` (string, required): Search pattern
- `path` (string, optional): Relative path within root directory
- `fixed_strings` (boolean, optional): Use fixed strings instead of regex
- `case_sensitive` (boolean, optional): Case-sensitive search
- `line_numbers` (boolean, optional): Include line numbers in output
- `context_lines` (number, optional): Number of context lines to show
- `file_types` (array of strings, optional): File types to include (e.g., "rust", "js")
- `max_depth` (number, optional): Maximum depth to search

#### Response

```json
{
  "matches": [
    "path/to/file.rs:10:    println!(\"Hello, world!\");"
  ],
  "stats": {
    "matched_lines": 1,
    "elapsed_ms": 5
  }
}
```

### Example MCP Client Usage

With an MCP client, you can send requests to the server using the following format:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "pattern": "fn main",
      "path": "src",
      "fixed_strings": false,
      "line_numbers": true
    }
  }
}
```

## Quick Testing

The repository includes test scripts to verify the functionality of the MCP server:

### Testing the local binary

```bash
# Make the test script executable
chmod +x tests/test_local_mcp.sh

# Run the test
./tests/test_local_mcp.sh
```

### Testing the Docker container

```bash
# Make the test script executable
chmod +x tests/test_mcp_docker.sh

# Build the Docker image if you haven't already
docker build -t ripgrep-mcp .

# Run the test
./tests/test_mcp_docker.sh
```

If the tests pass, you'll see a success message confirming that the MCP server is working correctly.

## Usage Examples

### Example 1: Basic Search with Claude

Once the ripgrep MCP server is connected to Claude, you can use it like this:

**Query:** "Search for 'async fn' in the src directory"

Claude will use the ripgrep search tool to find instances of "async fn" in your source code and display the results.

### Example 2: Finding Definitions

**Query:** "Find all function definitions related to configuration in the project"

Claude will search for functions related to configuration using ripgrep and analyze the results.

### Example 3: Advanced Search

**Query:** "Search for error handling patterns in the codebase and exclude test files"

Claude will craft an appropriate ripgrep search query to find error handling patterns while excluding test files.

### Example 4: Manual JSON-RPC Testing

For direct testing without Claude, you can send JSON-RPC requests to the server:

```bash
# For local binary
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
  FILES_ROOT=/path/to/your/code ./target/release/mcp-rg

# For Docker
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
  docker run -i --rm -v /path/to/your/code:/app/files ripgrep-mcp
```

## Environment Variables

- `FILES_ROOT`: Root directory to search (default: current directory)
- `LOG_LEVEL`: Logging level (trace, debug, info, warn, error) (default: info)

## Security Considerations

- The server implements path traversal prevention
- All inputs are validated
- The Docker container runs as a non-root user
- Only provide access to code repositories that you want the AI model to search

## Troubleshooting

If you encounter issues:

1. Check that ripgrep (`rg`) is installed and available in your PATH
2. Verify that the `FILES_ROOT` directory exists and is accessible
3. For permission issues, check that the user running the server has read access to the files
4. Set `LOG_LEVEL=debug` for more detailed logs

### Docker-Specific Troubleshooting

If you're running the server in Docker:

1. Make sure to include the `-i` flag when running the container (`docker run -i ...`)
2. Ensure the volume is correctly mounted (`-v /host/path:/app/files`)
3. Check that ripgrep is installed in the container (our Dockerfile takes care of this)
4. For filesystem permission issues, you may need to adjust the user in the Dockerfile
5. Always build the Docker image using the multi-stage build in our Dockerfile rather than using a pre-built binary, as architecture compatibility issues may arise (e.g., ARM64 vs x86_64)
6. If you encounter "too many open files" errors when mounting volumes, try restarting the Docker daemon

## Development

### Project Structure

- `src/main.rs`: Application entry point
- `src/config.rs`: Configuration management
- `src/error.rs`: Error handling
- `src/ripgrep.rs`: Ripgrep wrapper
- `src/mcp.rs`: MCP server implementation

### Running Tests

```bash
cargo test
```

## Contributing

Contributions are welcome! Here are ways you can contribute:

1. **Bug Fixes**: Help fix issues and improve the server's stability
2. **New Features**: Implement new search capabilities or MCP tools
3. **Documentation**: Improve README, code documentation, or examples
4. **Testing**: Add more tests or testing utilities

Please follow these steps to contribute:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/awesome-feature`)
3. Make your changes
4. Run tests to ensure everything works
5. Commit your changes (`git commit -m 'Add awesome feature'`)
6. Push to the branch (`git push origin feature/awesome-feature`)
7. Open a Pull Request

## License

[MIT](LICENSE)