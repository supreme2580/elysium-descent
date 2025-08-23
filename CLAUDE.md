# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Elysium Descent is a roguelike game built with Rust/Bevy 0.16.0. The game focuses on local gameplay mechanics and offline experiences.

## Common Development Commands

### Docker Setup (Recommended)
```bash


```

### Manual Development Setup
```bash
# Run game client
cd client
cargo run
```

### Testing
```bash
# Test client
cd client && cargo test
```

### Building
```bash
# Build client
cd client && cargo build
```

## Architecture Overview

### Client (Rust/Bevy 0.16.0)
- **ECS Architecture**: Uses Bevy's Entity Component System
- **Systems**: Located in `client/src/systems/` - handle game logic, input
- **Screens**: State management in `client/src/screens/` - menu, gameplay, settings
- **Resources**: Asset and configuration management




### Key Components
- **Character Controller**: Player movement and input handling

## Critical Technical Notes

### Bevy 0.16.0 Breaking Changes
- `Query::single()` returns `Result` - always handle with `expect()` or proper error handling
- Required Components replace Bundles - components automatically inject dependencies
- Built-in entity relationships - use parent-child system for hierarchies
- Observer system for reactive programming - prefer over direct system dependencies



### Performance Considerations
- GPU instancing for similar objects
- Texture atlasing for UI elements
- Spatial indexing for efficient collision detection
- LOD system for 3D models at distance

## Development Guidelines

### Code Organization
- Client logic in `client/src/` with clear separation of systems, screens, and resources
- Assets organized by type in `client/assets/`



### Asset Management
- 3D models in glTF format
- Audio files in OGG format for cross-platform compatibility
- Texture atlasing for optimized rendering
- Font consistency using Rajdhani family

## Project Structure Context

- `client/AI_DOCS/`: Contains detailed Bevy 0.16 migration documentation
- `docs/src/gdd/`: Game Design Document
