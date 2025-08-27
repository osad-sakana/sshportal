# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building
```bash
cargo build                # Debug build
cargo build --release      # Release build for distribution
```

### Testing
```bash
cargo test                 # Run unit tests
cargo check               # Fast syntax/type checking
```

### Installation & Development
```bash
./install.sh              # Full installation (builds + installs to ~/.local/bin + zsh setup)
cargo run -- <command>    # Run during development without installing
```

### Binary Location
- Development: `target/debug/sshportal` or `target/release/sshportal`  
- Installed: `~/.local/bin/sshportal`

## Architecture Overview

### Core Module Structure
The project follows a clean modular architecture with separation of concerns:

- **`main.rs`** - Entry point, command parsing, and error handling
- **`config.rs`** - Configuration management (JSON serialization, file I/O)
- **`commands.rs`** - CLI argument definitions using clap
- **`host.rs`** - SSH host management functionality
- **`path.rs`** - Path alias management and SCP file transfer logic

### Configuration System
- Uses JSON configuration stored in `~/.config/sshportal/config.json`
- Auto-creates config directory and default config on first run
- Two main data structures: `hosts` (HashMap) and `paths` (HashMap)
- Path expansion support for tilde (`~`) resolution

### Key Data Structures
```rust
struct Host {
    connection: String,  // "user@hostname" format
    port: u16           // SSH port number
}

struct Path {
    path: String,       // Actual path
    is_remote: bool     // Local vs remote path flag
}

struct Config {
    hosts: HashMap<String, Host>,
    paths: HashMap<String, Path>
}
```

### Command Processing Flow
1. `main.rs` parses CLI args using clap
2. `handle_command()` in `commands.rs` routes to appropriate module functions
3. Module functions load config, perform operations, save config if modified

### File Transfer Logic (`path.rs`)
Complex parsing logic for handling various path specifications:
- Path aliases (local/remote)
- Direct paths
- Host:path combinations  
- Host alias resolution
- Direct SSH connection strings (user@host)
- Hostname/IP validation

### zsh Integration
- Plugin provides comprehensive tab completion for all commands
- Enhances standard `ssh` and `scp` commands with sshportal host/path completion
- Provides convenient aliases: `sp`, `spc`, `spl`, `spp`

## Critical Implementation Notes

### SCP Command Construction
The SCP functionality in `copy_files()` has complex logic for handling different scenarios:
- Local-to-remote, remote-to-local, remote-to-remote transfers
- Port specification handling (avoids duplicate -P flags)
- Path alias resolution vs direct path usage
- Both configured host aliases and direct SSH connection strings

### Error Handling Patterns
- Functions return `Result<(), Box<dyn std::error::Error>>`
- User-friendly colored error messages via `colored` crate
- Non-fatal warnings for duplicate entries
- Graceful handling of missing config files

### Configuration Management
- Lazy initialization - config created on first use
- Atomic operations - config loaded, modified, saved
- Pretty-printed JSON for human readability
- Directory structure auto-creation

## Privacy and Security Guidelines

**CRITICAL: When working with this repository, Claude Code must NEVER include any actual personal information, real hostnames, IP addresses, usernames, directory paths, or configuration data in commits, documentation, or examples.**

### Prohibited Information in Commits
- **NO** real usernames or personal names
- **NO** real hostnames or IP addresses (e.g., 192.168.x.x, specific domain names)
- **NO** actual directory paths from the local system (e.g., /Users/username/...)
- **NO** real SSH key paths or configuration details
- **NO** actual configuration file contents

### Required Practices
1. **Use Generic Examples Only**: Always use placeholder examples like:
   - Hostnames: `prod.example.com`, `staging.example.com`, `dev.example.com`
   - Users: `user`, `admin`, `developer`
   - IP addresses: `192.168.1.100`, `10.0.0.1` (RFC 1918 examples)
   - Paths: `~/projects`, `/var/www/html`, `/opt/api`

2. **Documentation Standards**: All README examples, code comments, and documentation must use generic, non-identifying placeholders.

3. **Git History Protection**: 
   - Never commit `.claude/` directory (already in .gitignore)
   - Remove any accidentally committed personal information using git history rewriting
   - Verify git history contains no sensitive data before pushing

4. **Configuration Examples**: When showing configuration examples, use fictional data that demonstrates functionality without revealing actual setup details.

### Compliance Verification
Before any commit or documentation update, verify that:
- [ ] No real personal information is included
- [ ] All examples use generic placeholders  
- [ ] No actual system paths or configurations are exposed
- [ ] Git history contains no sensitive information