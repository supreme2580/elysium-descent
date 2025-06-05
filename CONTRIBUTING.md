# Contributing to Elysium Descent

Welcome to Elysium Descent! We're excited that you're interested in contributing to our Bevy-based roguelike game with blockchain integration. This guide will help you get started with contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [How to Contribute](#how-to-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Features](#suggesting-features)
  - [Code Contributions](#code-contributions)
- [Development Guidelines](#development-guidelines)
  - [Rust Guidelines](#rust-guidelines)
  - [Cairo/Starknet Guidelines](#cairo-starknet-guidelines)
  - [Game Design Principles](#game-design-principles)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Community](#community)

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct:

- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive criticism
- Respect differing viewpoints and experiences
- Show empathy towards other community members

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

**For Game Client (Bevy/Rust):**
- Rust 1.87.0 or later
- Cargo (comes with Rust)

**For Smart Contracts (Cairo/Starknet):**
- Scarb 2.10.1
- Katana (local Starknet devnet)
- Sozo (Dojo CLI)
- Torii (Dojo indexer)

**Optional but Recommended:**
- Docker and Docker Compose
- VS Code with Rust Analyzer and Cairo extensions

### Development Setup

1. **Clone the Repository**
   ```bash
   git clone https://github.com/realmofra/elysium_descent.git
   cd elysium_descent
   ```

2. **Set up the Game Client**
   ```bash
   cd client
   cargo build
   ```

3. **Set up the Smart Contracts**
   ```bash
   cd contracts
   scarb build
   ```

4. **Run the Development Environment**

   Using Docker (Recommended):
   ```bash
   docker compose up
   ```

   Or manually:
   ```bash
   # Terminal 1: Run local blockchain
   katana --dev --dev.no-fee

   # Terminal 2: Deploy contracts
   sozo build
   sozo migrate

   # Terminal 3: Run indexer (replace <WORLD_ADDRESS> with actual address)
   torii --world <WORLD_ADDRESS> --http.cors_origins "*"

   # Terminal 4: Run game client
   cd client
   cargo run
   ```

## Project Structure

```
elysium_descent/
├── client/                 # Bevy game client
│   ├── src/               # Rust source code
│   └── Cargo.toml         # Rust dependencies
├── contracts/             # Cairo smart contracts
│   ├── src/              # Cairo source code
│   └── Scarb.toml        # Cairo dependencies
├── crates/               # Shared Rust crates
│   └── bevy_dojo/        # Bevy-Dojo integration
├── packages/             # Shared packages
└── README.md            # Project overview
```

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates.

**To report a bug:**
1. Use the issue template for bug reports
2. Include a clear title and description
3. Provide steps to reproduce the issue
4. Include expected vs actual behavior
5. Add system information (OS, Rust version, etc.)
6. If possible, include screenshots or error logs

### Suggesting Features

We welcome feature suggestions! Please:
1. Check if the feature has already been suggested
2. Use the feature request template
3. Clearly describe the feature and its benefits
4. Explain how it fits with the game's vision
5. Consider blockchain integration aspects if relevant

### Code Contributions

1. **Find an Issue**
   - Look for issues labeled `good first issue` or `help wanted`
   - Comment on the issue to claim it
   - Wait for maintainer approval before starting

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

3. **Make Your Changes**
   - Write clean, documented code
   - Follow the coding guidelines
   - Add tests for new functionality
   - Update documentation as needed

## Development Guidelines

### Rust Guidelines

1. **Code Style**
   - Run `cargo fmt` before committing
   - Run `cargo clippy` and address warnings
   - Follow Rust naming conventions

2. **Performance Considerations**
   - Profile before optimizing
   - Prefer iterators over manual loops
   - Use `&str` instead of `String` when possible
   - Leverage Bevy's ECS for efficient data access

## Testing

### Client Testing
```bash
cd client
cargo test
```

### Contract Testing
```bash
cd contracts
sozo test
```

### Integration Testing
- Test client-contract interaction
- Check event emissions and indexing

## Documentation

- Document all public APIs
- Include examples in doc comments
- Update README files when adding features
- Add inline comments for complex logic
- Keep architectural decisions documented

## Pull Request Process

1. **Before Submitting**
   - Ensure all tests pass
   - Run formatters and linters
   - Update documentation

2. **PR Description**
   - Reference related issues
   - Describe changes clearly
   - Include testing steps

3. **Review Process**
   - Address reviewer feedback promptly
   - Keep discussions focused and professional
   - Be open to suggestions
   - Request re-review after making changes

4. **Merge Requirements**
   - All CI checks must pass
   - At least one maintainer approval
   - No unresolved conversations
   - Up-to-date with main branch

## Community

### Communication Channels

- **Discord**: [Join our server](#) for real-time discussions
- **GitHub Discussions**: For longer-form conversations
- **Twitter/X**: Follow [@elysium_descent](#) for updates

### Getting Help

- Ask questions in Discord #help channel
- Check documentation and existing issues
- Be specific about your problem
- Share relevant code snippets

### Contribution Recognition

We value all contributions! Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Given special Discord roles
- Reward valuable contributions through OnlyDust

## Advanced Topics

### Working with Bevy ECS

- Understand entity-component-system architecture
- Learn about systems, queries, and resources
- Study Bevy's scheduling and stages
- Practice with smaller examples first

### Blockchain Development

- Understand Cairo language basics
- Learn about Starknet architecture
- Study Dojo framework patterns
- Test with local devnet before mainnet

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

---

Thank you for contributing to Elysium Descent! Your efforts help make this game better for everyone. Happy coding! 🎮🚀
