# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Elysium Descent is a roguelike game with:
- **Client**: Rust/Bevy 0.16.0 game engine with ECS architecture

- **Architecture**: Local roguelike game with offline gameplay

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




### Local Development Environment

#### Manual Setup:
```bash
# Run client
cd client
cargo run
```

## Architecture & Code Structure

### Client Architecture (Bevy ECS)
- **Systems** (`src/systems/`): Game logic components
  - `character_controller.rs`: Player movement and controls
  
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




### Asset Organization
- `assets/audio/`: Sound effects and music
- `assets/models/`: 3D models (player, environment, collectibles)
- `assets/images/`: UI textures and backgrounds
- `assets/fonts/`: Rajdhani font family

### Current Development
- Active branch: `feat/yarn-spinner`
- Main branch for PRs: `main`
- Recent focus: Character movement, input handling