`
# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.
``

Repository: Elysium Descent

Overview
- Monorepo with two primary parts:
  - client: Rust/Bevy 0.16.0 game client (offline/local roguelike)
  - contracts: Dojo (Cairo/StarkNet) smart contracts for on-chain/world logic
- Key docs: README.md (root), CLAUDE.md (root and client), client/AI_DOCS/Bevy.md, contracts/README.md

Common commands
Client (Rust/Bevy 0.16)
- Build (debug):
  - cd client && cargo build
- Build (release):
  - cd client && cargo build --release
- Run the game:
  - cd client && cargo run
- Test all:
  - cd client && cargo test
- Run a single test by filter (substring match on test name or module path):
  - cd client && cargo test <filter>
  - Examples: cargo test systems::collectibles, cargo test inventory_adds_item
- Lint (clippy):
  - cd client && cargo clippy --all-targets --all-features -- -D warnings
- Format:
  - cd client && cargo fmt

Contracts (Dojo/Cairo/StarkNet)
- Local stack (per contracts/README.md):
  - Terminal A (local devnet): katana --dev --dev.no-fee
  - Terminal B (project root or contracts/):
    - sozo build
    - sozo inspect
    - sozo migrate
    - torii --world <WORLD_ADDRESS> --http.cors_origins "*"
- Notes:
  - Scarb manifest at contracts/Scarb.toml pins Cairo = 2.10.1 and Dojo v1.6.0-alpha.2
  - Additional environment profiles/configs in contracts/*.toml (katana/torii/dojo)

High-level architecture
Client (Bevy ECS)
- Entry point: client/src/main.rs
  - Builds App with DefaultPlugins (WindowPlugin configured to 1920x1080), UiLunex (UI), bevy_kira_audio (AudioPlugin), custom asset/audio plugins, a StarkNet plugin (resources::starknet::plugin), screen/state plugin (screens::plugin), keybinding plugin, and a simple modal UI plugin. Registers a PickupItemEvent.
  - Startup systems: setup_camera (Camera2d on RenderLayers 0/1, UiSourceCamera), setup_global_lighting (warm AmbientLight).
- State & flow:
  - Screens manage game states and transitions: client/src/screens/ (main_menu, loading/pregame_loading, gameplay, fight, settings, mod.rs exposes plugin that wires states and screen systems).
- Gameplay domain:
  - Systems (client/src/systems/): character_controller (player input/movement), collectibles (+ config), boundary, enemy_ai, objectives, interactions (book_interaction). Systems operate over components/resources and emit/consume events (e.g., PickupItemEvent) in typical ECS fashion.
  - Game module (client/src/game/): domain components and shared resources for gameplay.
- Rendering & cameras:
  - client/src/rendering/: camera orchestration under rendering/cameras (player_camera, staging via mod.rs). RenderLayers used to separate UI and world.
- UI:
  - client/src/ui/: dialog, inventory, modal, widgets, shared styles. Uses bevy_lunex for UI layout/widgets.
- Resources:
  - client/src/resources/: assets preloading/handles; audio (plugins GameAudioPlugin, SfxPlugin); StarkNet integration at resources/starknet.rs (plugin hooked in main.rs) for any on-chain/world interactions.
- Constants:
  - client/src/constants/: movement, collectibles, boundary, etc., centralizing gameplay tuning values.

Contracts (Dojo)
- contracts/src:
  - models.cairo: world data models
  - systems/actions.cairo: system logic (actions over models)
  - lib.cairo: crate entry
  - tests/: cairo tests (e.g., test_world.cairo)
- Tooling:
  - Managed via Scarb and Dojo (sozo, katana, torii). Target world contract specified in Scarb.toml (build-external-contracts).

Bevy 0.16 critical notes (from CLAUDE.md and client/AI_DOCS/Bevy.md)
- Query::single() returns Result; avoid panics—handle Ok/Err explicitly in systems.
- Required Components supersede many bundle patterns; prefer #[require(...)] on components.
- Built-in entity relationships (ChildOf/Children) replace manual parent/child wiring.
- Observer system enables reactive ECS; favor observers over direct system ordering where appropriate.
- Color system uses explicit color spaces (Srgba, LinearRgba); update constants/constructors accordingly.

What to consult first
- For API differences and migration patterns: client/AI_DOCS/Bevy.md (comprehensive 0.13→0.16 deltas and examples).
- For gameplay/state wiring: screens/mod.rs and systems/mod.rs to see how the App’s schedule is composed.
- For assets/audio wiring: resources/assets.rs and resources/audio/*.
- For on-chain integration touchpoints: resources/starknet.rs and contracts/ Scarb/dojos configs.

Notes for agents
- Prefer running cargo commands from the client directory for Rust tasks, and sozo/katana/torii from the contracts directory (or repo root if your shell environment is equivalent).
- When running targeted tests, rely on cargo’s filter matching your test function or module path; avoid globbing shells that may expand unexpectedly—quote patterns if needed.
- If you need to interact with Bevy 0.16-only APIs, check client/AI_DOCS/Bevy.md before modifying system signatures or component patterns.

