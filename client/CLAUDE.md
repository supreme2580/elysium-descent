# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Elysium Descent is a blockchain-integrated roguelike game with:
- **Client**: Rust/Bevy 0.16.0 game engine with ECS architecture
- **Blockchain**: Cairo/Starknet smart contracts using Dojo framework
- **Architecture**: Fully On-Chain Game (FOCG) where game logic runs on Starknet

## Development Commands

### Client (Rust/Bevy)
```bash
cd client
cargo build              # Development build
cargo build --release    # Release build  
cargo run               # Run the game
cargo test              # Run tests
cargo fmt               # Format code
cargo clippy            # Lint code
```

### Contracts (Cairo/Starknet)
```bash
cd contracts
scarb build             # Build Cairo contracts
sozo build              # Build using Dojo CLI
sozo test               # Run contract tests
sozo migrate            # Deploy contracts
```

### Local Development Environment

#### Using Docker (Recommended):
```bash
cd contracts
docker compose up       # Starts Katana, Sozo, and Torii
```

#### Manual Setup (4 terminals):
```bash
# Terminal 1: Local blockchain
katana --dev --dev.no-fee

# Terminal 2: Deploy contracts
cd contracts
sozo build && sozo migrate

# Terminal 3: Indexer (use WORLD_ADDRESS from Terminal 2)
torii --world <WORLD_ADDRESS> --http.cors_origins "*"

# Terminal 4: Run client
cd client
cargo run
```

## Architecture & Code Structure

### Client Architecture (Bevy ECS)
- **Systems** (`src/systems/`): Game logic components
  - `character_controller.rs`: Player movement and controls
  - `dojo/`: Blockchain integration systems
  - `collectibles.rs`: Item collection mechanics
- **Screens** (`src/screens/`): Game states and UI screens
  - `gameplay.rs`: Main game loop
  - `main_menu.rs`: Title screen
  - `settings.rs`: Configuration screen
- **Resources** (`src/resources/`): Asset and state management
- **Constants** (`src/constants/`): Game configuration values

### Key Dependencies
- **bevy**: Core game engine with ECS
- **avian3d**: Physics simulation
- **bevy_kira_audio**: Audio system
- **bevy_lunex**: UI framework
- **dojo_bevy_plugin**: Blockchain integration
- **starknet**: Blockchain interaction

### Development Guidelines
1. **Rust Standards**: Use Rust 1.87.0+, follow standard naming conventions
2. **Code Quality**: Run `cargo fmt` and `cargo clippy` before commits
3. **ECS Pattern**: Leverage Bevy's ECS for efficient data access
4. **Performance**: Use iterators over manual loops, prefer `&str` over `String`
5. **Testing**: Write tests for new functionality, run with `cargo test`

## Bevy 0.16 Specific Guidelines

⚠️ **Critical**: This project uses Bevy 0.16.0 which introduced major breaking changes. See `./AI_DOCS/Bevy.md` for comprehensive migration guide.

### Key Bevy 0.16 Changes to Remember:

#### Error Handling Revolution
- `Query::single()` now returns `Result<T, QuerySingleError>` instead of panicking
- Use `query.single()?` in systems that return `Result` or `let Ok(item) = query.single() else { return; }`
- Configure global error handling: panic in dev, log in production

#### Required Components System
- Bundles largely replaced with Required Components
- `#[require(Transform, Visibility)]` automatically adds dependencies
- Cleaner entity spawning: `commands.spawn(MyComponent)` vs complex bundles

#### Entity Relationships
- `Parent` component renamed to `ChildOf(Entity)`
- `Children` component automatically maintained
- Built-in relationship system for hierarchies

#### GPU-Driven Rendering
- `#[bindless]` attribute for materials enables massive performance gains
- GPU occlusion culling available for large scenes
- Multi-draw indirect for batched rendering

#### Component Lifecycle
- Component hooks split: `on_add()`, `on_insert()`, `on_remove()`
- Observer system for reactive ECS programming
- `apply_deferred()` is now zero-sized type `ApplyDeferred`

### Common Migration Patterns:

```rust
// OLD (0.13-0.15): Panic-prone
let transform = query.single();

// NEW (0.16): Safe error handling
let Ok(transform) = query.single() else {
    warn!("Expected single entity");
    return;
};

// OLD: Manual bundles
commands.spawn(PlayerBundle { /* ... */ });

// NEW: Required components
#[derive(Component)]
#[require(Transform, Visibility, Health)]
struct Player;

commands.spawn(Player); // Transform, Visibility, Health auto-added

// OLD: Manual parent-child
commands.entity(parent).push_children(&[child]);

// NEW: Automatic relationships
commands.spawn((Child, ChildOf(parent_entity)));
```

For detailed migration examples, breaking changes, and performance optimizations, see:
**📖 [Complete Bevy 0.16 Guide](./AI_DOCS/Bevy.md)**

### Blockchain Integration
The game uses Dojo framework to connect Bevy client with Starknet:
- Connection constants in `src/constants/dojo_constants.rs`
- Setup system in `src/systems/dojo/setup.rs`
- Game creation in `src/systems/dojo/create_game.rs`

### Asset Organization
- `assets/audio/`: Sound effects and music
- `assets/models/`: 3D models (player, environment, collectibles)
- `assets/images/`: UI textures and backgrounds
- `assets/fonts/`: Rajdhani font family

### Current Development
- Active branch: `feat/yarn-spinner`
- Main branch for PRs: `main`
- Recent focus: Character movement, Dojo integration, input handling