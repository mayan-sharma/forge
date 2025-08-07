# ⚡ Forge - Autonomous CLI Agent Tool

An intelligent command-line interface built in Rust that provides AI-powered coding assistance, file operations, and task automation with local LLM support via Ollama.

## 🚀 Features

- **AI Chat Interface** - Interactive conversation with local LLMs
- **Intelligent File Editing** - AI-assisted code modification and generation  
- **Advanced Search** - Fast text search across files and directories
- **Safe Command Execution** - AI-analyzed command execution with security checks
- **Interactive Shell** - Enhanced shell experience with safety features
- **Workflow Automation** - Multi-step task automation and management
- **Cross-Platform** - Works on Unix/Linux and Windows systems

## 🏗️ Architecture

Forge is built with a minimal dependency philosophy, implementing core functionality from scratch:

- **Custom HTTP Client** - Hand-built HTTP/1.1 client for Ollama API communication
- **Native JSON Parser** - Lightweight JSON handling without external libraries
- **Terminal Control** - Cross-platform terminal interface with ANSI styling
- **File System Operations** - Direct file I/O with custom search algorithms
- **Process Management** - Secure command execution with safety analysis

## 📦 Installation

### Prerequisites

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install Ollama**:
   ```bash
   # Linux/macOS
   curl -fsSL https://ollama.ai/install.sh | sh
   
   # Or visit https://ollama.ai for other installation methods
   ```

3. **Start Ollama and install a model**:
   ```bash
   ollama serve  # Start the Ollama service
   ollama pull llama3  # Install a language model
   ```

### Build Forge

```bash
git clone <repository-url>
cd forge
cargo build --release
```

The compiled binary will be available at `./target/release/forge`.

## 🔧 Usage

### Basic Commands

```bash
# Start interactive AI chat
forge chat

# Edit a file with AI assistance
forge edit src/main.rs "add error handling"

# Search for text in files
forge search "fn main" src/

# Execute commands safely
forge exec "ls -la"

# Start interactive shell
forge shell

# Manage workflows
forge workflow list
forge workflow demo

# Show system status
forge status

# Test connections
forge test-ollama
forge test-files
```

### Chat Interface

The chat interface provides streaming responses and conversation history:

```
💬 Forge Chat Interface
   AI-powered coding assistance with conversation history

💡 Commands: /help, /clear, /history, exit
   Press Ctrl+C to interrupt at any time

🔍 Checking available models...
✅ Found models: llama3, codellama, mistral
🧪 Using: llama3

forge> How do I optimize this Rust function?
```

### File Operations

Edit files with natural language instructions:

```bash
# Add specific functionality
forge edit src/lib.rs "add a function to calculate fibonacci numbers"

# Refactor existing code
forge edit main.rs "extract the error handling into a separate function"

# Fix issues
forge edit server.rs "fix the memory leak in the connection handler"
```

### Search Capabilities

Fast text search with pattern matching:

```bash
# Search in current directory
forge search "struct User"

# Search in specific path
forge search "async fn" src/handlers/

# Complex patterns
forge search "pub fn.*Error" src/
```

## ⚙️ Configuration

Forge uses minimal configuration stored in system directories. The tool automatically detects Ollama models and configures itself for optimal performance.

### Available Models

Forge works with any Ollama-compatible model:
- **Code-focused**: `codellama`, `deepseek-coder`, `starcoder`
- **General purpose**: `llama3`, `mistral`, `gemma`
- **Specialized**: `codellama:python`, `codellama:instruct`

## 🛡️ Safety Features

- **Command Analysis** - AI evaluates commands before execution
- **Permission Checks** - Validates file access before operations  
- **Risk Assessment** - Identifies potentially dangerous operations
- **User Confirmation** - Prompts for approval on risky commands
- **Local Processing** - All data stays on your machine

## 🏃‍♂️ Development

### Project Structure

```
src/
├── main.rs                 # Entry point and command routing
├── cli/                    # Command-line interface
│   └── commands/           # Individual command implementations
├── http/                   # Custom HTTP client for Ollama
├── fs/                     # File system operations
├── terminal/               # Terminal control and styling
├── forge_process/          # Process execution and workflows
└── config/                 # Configuration management
```

### Running Tests

```bash
# Test Ollama connection
./target/debug/forge test-ollama

# Test file operations
./target/debug/forge test-files

# Build and test
cargo build && cargo test
```

### Adding New Commands

1. Create a new file in `src/cli/commands/`
2. Implement the `run()` function
3. Add command handling in `src/main.rs`
4. Update help text

## 📋 Workflows

Forge includes built-in workflows for common development tasks:

- **rust-build-test** - Build and test Rust projects
- **git-commit-push** - Git workflow with commit and push
- **Custom workflows** - Define your own multi-step automations

```bash
# List available workflows
forge workflow list

# Run a workflow
forge workflow rust-build-test

# Demo workflow features
forge workflow demo
```

## 🔍 Troubleshooting

### Common Issues

**Ollama Connection Failed**
```bash
# Ensure Ollama is running
ollama serve

# Check available models  
ollama list

# Test connection
forge test-ollama
```

**File Permission Errors**
```bash
# Test file operations
forge test-files

# Check permissions on target files
ls -la <file-path>
```

**Command Execution Blocked**
- Safety checker may block dangerous commands
- Use explicit confirmation when prompted
- Check command against internal safety rules

## 🤝 Contributing

We welcome contributions! Please see our development guidelines:

1. Maintain minimal dependency philosophy
2. Follow Rust idioms and best practices
3. Add comprehensive error handling
4. Test with real Ollama installation
5. Ensure cross-platform compatibility

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **Ollama** - For providing excellent local LLM infrastructure
- **Rust Community** - For the amazing ecosystem and tooling
- **Contributors** - Everyone who helps improve Forge

---

**Built with ❤️ in Rust** - Demonstrating low-level programming concepts while providing practical CLI functionality.