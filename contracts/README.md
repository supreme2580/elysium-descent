# Elysium Descent Smart Contract

A comprehensive Starknet smart contract for managing player profiles, games, and levels in the Elysium Descent game.

## Overview

This smart contract provides a complete backend system for:
- **Player Profile Management**: Create and manage player profiles with different character types
- **Game Management**: Create games, track progress, and manage game states
- **Level Management**: Admin-controlled level creation and modification with detailed configuration
- **Gameplay Tracking**: Monitor player progress, collectibles, and achievements

## Contract Features

### Player Types
- **Man**: Human male character
- **Woman**: Human female character  
- **Beast**: Beast-like character
- **Spirit**: Ethereal character

### Level Configuration
Each level can be configured with:
- **Coins**: Spawn count and 3D positions
- **Beasts**: Enemy types with health, damage, and speed stats
- **Objectives**: Multiple objective types (collect, reach location, defeat, survive, explore)
- **Environment**: Dungeon scale, position, and rotation

### Admin Functions
- Create and modify levels
- Activate/deactivate levels
- Manage admin permissions
- Full level data control

## Usage Examples

### 1. Creating a Player Profile

```typescript
// Create a player profile
await contract.create_player_profile(
    "PlayerName",           // username
    PlayerType.Man          // player_type
);
```

### 2. Creating a Game

```typescript
// Create a new game
const gameId = await contract.create_game();
```

### 3. Starting a Level

```typescript
// Start a specific level
await contract.start_level(gameId, 1);
```

### 4. Admin Creating a Level

```typescript
// Example level data matching the JSON structure from requirements
const coinsData = [
    15,                    // spawn_count
    10.0, 1.0, 5.0,       // coin 1: x, y, z
    -8.0, 1.0, 12.0,      // coin 2: x, y, z
    15.0, 1.0, -3.0,      // coin 3: x, y, z
    // ... more coin positions
];

const beastsData = [
    "monster_1",           // beast_id
    0,                     // beast_type (0 = Monster)
    0.0, 1.0, 0.0,        // x, y, z
    100,                   // health
    25,                    // damage
    3.0                    // speed
];

const objectivesData = [
    "collect_coins",       // objective_id
    "Collect Ancient Coins", // title
    "Collect 5 coins to unlock the path forward", // description
    0,                     // objective_type (0 = collect)
    "coins",               // target
    5,                     // required_count
    "unlock_level_2"       // reward
];

const environmentData = [
    7.5,                   // dungeon_scale
    0.0, -1.5, 0.0,       // dungeon_x, dungeon_y, dungeon_z
    -1.5708                // dungeon_rotation
];

// Create the level
const levelId = await contract.admin_create_level(
    "The Beginning",       // level_name
    PlayerType.Man,        // player_type
    2,                     // next_level
    coinsData,             // coins_data
    beastsData,            // beasts_data
    objectivesData,        // objectives_data
    environmentData        // environment_data
);
```

### 5. Gameplay Actions

```typescript
// Collect a coin
await contract.collect_coin(gameId, 1, 0);

// Defeat a beast
await contract.defeat_beast(gameId, 1, "monster_1");

// Complete an objective
await contract.complete_objective(gameId, 1, "collect_coins");

// Complete the level
await contract.complete_level(gameId, 1);
```

## Data Structures

### Level Data Format

The contract expects level data in specific array formats:

#### Coins Data
```
[spawn_count, x1, y1, z1, x2, y2, z2, ...]
```

#### Beasts Data
```
[beast_id1, beast_type1, x1, y1, z1, health1, damage1, speed1, ...]
```

#### Objectives Data
```
[objective_id1, title1, desc1, type1, target1, required1, reward1, ...]
```

#### Environment Data
```
[scale, x, y, z, rotation]
```

### Beast Types
- 0: Monster
- 1: Dragon
- 2: Goblin
- 3: Orc
- 4: Demon
- 5: Undead
- 6: Elemental

### Objective Types
- 0: Collect
- 1: Reach Location
- 2: Defeat
- 3: Survive
- 4: Explore

## Events

The contract emits comprehensive events for tracking:

- `PlayerProfileCreated`: When a new player profile is created
- `PlayerProfileUpdated`: When a player profile is updated
- `GameCreated`: When a new game is started
- `LevelStarted`: When a level begins
- `LevelCompleted`: When a level is finished
- `LevelCreated`: When an admin creates a level
- `LevelModified`: When an admin modifies a level
- `CoinCollected`: When a player collects a coin
- `BeastDefeated`: When a player defeats a beast
- `ObjectiveCompleted`: When a player completes an objective

## Admin Management

### Adding Admins
```typescript
await contract.add_admin(
    adminAddress,          // admin_address
    "owner",               // role
    0xFFFFFFFF            // permissions (bit flags)
);
```

### Level Management
```typescript
// Deactivate a level
await contract.admin_deactivate_level(levelId);

// Activate a level
await contract.admin_activate_level(levelId);

// Modify an existing level
await contract.admin_modify_level(
    levelId,
    "Updated Level Name",
    PlayerType.Man,
    3,
    newCoinsData,
    newBeastsData,
    newObjectivesData,
    newEnvironmentData
);
```

## Security Features

- **Admin-only functions**: Level creation and modification require admin privileges
- **Player ownership**: Players can only interact with their own games
- **Validation**: Comprehensive input validation and assertions
- **Event logging**: All major actions are logged as events for transparency

## Integration with Game Client

The contract is designed to work seamlessly with the Bevy game engine through:

1. **Event listening**: Game client listens to contract events for real-time updates
2. **State synchronization**: Game state is synchronized with blockchain data
3. **Progressive disclosure**: Level data is revealed as players progress
4. **Achievement tracking**: All player accomplishments are recorded on-chain

## Development and Testing

### Building
```bash
cd contracts
scarb build
```

### Testing
```bash
scarb test
```

### Deployment
```bash
# Deploy to testnet
scarb deploy --target slot

# Deploy to mainnet
scarb deploy --target release
```

## Future Enhancements

- **NFT Integration**: Player profiles and achievements as NFTs
- **Multiplayer Support**: Shared game states and collaborative objectives
- **Dynamic Difficulty**: AI-driven level adjustment based on player performance
- **Cross-chain Integration**: Support for multiple blockchain networks
- **Modding Support**: Community-created level sharing and validation

## License

This project is licensed under both MIT and Apache 2.0 licenses. See LICENSE-MIT and LICENSE-APACHE for details.
