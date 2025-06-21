# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Elysium Descent is a Fully On-Chain Game (FOCG) combining a Rust/Bevy 0.16.0 client with Cairo smart contracts on Starknet using Dojo v1.5.0 framework. The game is a roguelike where core game logic runs on the blockchain.

## Common Development Commands

### Docker Setup (Recommended)
```bash
cd contracts
docker compose up  # Starts Katana, Sozo, and Torii services
```

### Manual Development Setup
```bash
# Terminal 1: Start local blockchain
katana --dev --dev.no-fee

# Terminal 2: Build and deploy contracts
cd contracts
sozo build
sozo migrate

# Terminal 3: Start indexer (replace with actual world address)
torii --world <WORLD_ADDRESS> --http.cors_origins "*"

# Terminal 4: Run game client
cd client
cargo run
```

### Testing
```bash
# Test client
cd client && cargo test

# Test contracts
cd contracts && sozo test
```

### Building
```bash
# Build client
cd client && cargo build

# Build contracts
cd contracts && sozo build
```

## Architecture Overview

### Client (Rust/Bevy 0.16.0)
- **ECS Architecture**: Uses Bevy's Entity Component System
- **Systems**: Located in `client/src/systems/` - handle game logic, Dojo integration, input
- **Screens**: State management in `client/src/screens/` - menu, gameplay, settings
- **Resources**: Asset and configuration management
- **Dojo Integration**: Custom plugin connecting Bevy to Starknet contracts

### Smart Contracts (Cairo/Dojo)
- **Models**: Game state data structures in `contracts/src/models/`
- **Systems**: Game logic and player actions in `contracts/src/systems/`
- **World**: Central registry managing all models and systems
- **Events**: State change notifications for indexing

### Key Components
- **Character Controller**: Player movement and input handling

## Critical Technical Notes

### Bevy 0.16.0 Breaking Changes
- `Query::single()` returns `Result` - always handle with `expect()` or proper error handling
- Required Components replace Bundles - components automatically inject dependencies
- Built-in entity relationships - use parent-child system for hierarchies
- Observer system for reactive programming - prefer over direct system dependencies

### Dojo Integration
- Game state synchronization between client and blockchain
- Use `dojo_bevy_plugin` for Starknet connectivity
- World address configuration required for Torii indexer
- Contract deployment needed before client connection

### Performance Considerations
- GPU instancing for similar objects
- Texture atlasing for UI elements
- Spatial indexing for efficient collision detection
- LOD system for 3D models at distance

## Development Guidelines

### Code Organization
- Client logic in `client/src/` with clear separation of systems, screens, and resources
- Contract logic in `contracts/src/` following Dojo model/system patterns
- Assets organized by type in `client/assets/`

### Blockchain Development
- Always test contracts with `sozo test` before deployment
- Use Katana devnet for local development
- Torii indexer required for client-blockchain communication
- World address changes require client configuration updates

### Asset Management
- 3D models in glTF format
- Audio files in OGG format for cross-platform compatibility
- Texture atlasing for optimized rendering
- Font consistency using Rajdhani family

## Project Structure Context

- `client/AI_DOCS/`: Contains detailed Bevy 0.16 migration documentation
- `docs/src/gdd/`: Game Design Document
- Individual CLAUDE.md files exist in `client/` and `contracts/` subdirectories for specific guidance
