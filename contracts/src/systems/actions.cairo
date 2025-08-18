use elysium_descent::models::{
    Admin, BeastType, Game, GameCounter, GameProgress, GameStatus, ItemType, Level, LevelBeast,
    LevelCoins, LevelCounter, LevelEnvironment, LevelItems, LevelObjective, ObjectiveType,
    PlayerInventory, PlayerProfile, PlayerStats, PlayerType, SessionProgress, FightSession, FightBeastState, WorldItem, GAME_COUNTER_ID, LEVEL_COUNTER_ID,
};
use starknet::{ContractAddress, get_block_timestamp};

// define the interface
#[starknet::interface]
pub trait IActions<T> {
    // Player Profile Management
    fn create_player_profile(ref self: T, username: felt252, player_type: PlayerType);
    fn update_player_profile(ref self: T, username: felt252, player_type: PlayerType);
    fn get_player_profile(self: @T, player: ContractAddress) -> PlayerProfile;
    
    // Game Management
    fn create_game(ref self: T) -> u32;
    fn start_level(ref self: T, game_id: u32, level: u32);
    fn complete_level(ref self: T, game_id: u32, level: u32);
    fn get_game(self: @T, game_id: u32) -> Game;
    fn get_game_progress(self: @T, game_id: u32, level: u32) -> GameProgress;
    
    // Level Management (Admin Only)
    fn admin_create_level(
        ref self: T,
        level_name: felt252,
        player_type: PlayerType,
        next_level: u32,
        coins_data: Array<felt252>,
        beasts_data: Array<felt252>,
        objectives_data: Array<felt252>,
        environment_data: Array<felt252>
    ) -> u32;
    
    fn admin_modify_level(
        ref self: T,
        level_id: u32,
        level_name: felt252,
        player_type: PlayerType,
        next_level: u32,
        coins_data: Array<felt252>,
        beasts_data: Array<felt252>,
        objectives_data: Array<felt252>,
        environment_data: Array<felt252>
    );
    
    fn admin_deactivate_level(ref self: T, level_id: u32);
    fn admin_activate_level(ref self: T, level_id: u32);
    fn get_level(self: @T, level_id: u32) -> Level;
    fn get_level_coins(self: @T, level_id: u32) -> LevelCoins;
    fn get_level_beasts(self: @T, level_id: u32) -> Array<LevelBeast>;
    fn get_level_objectives(self: @T, level_id: u32) -> Array<LevelObjective>;
    fn get_level_environment(self: @T, level_id: u32) -> LevelEnvironment;
    
    // Gameplay Actions
    fn pickup_item(ref self: T, game_id: u32, item_id: u32) -> bool;
    fn collect_coin(ref self: T, game_id: u32, level: u32, coin_index: u32);
    fn defeat_beast(ref self: T, game_id: u32, level: u32, beast_id: felt252);
    fn complete_objective(ref self: T, game_id: u32, level: u32, objective_id: felt252);
    
    // Session-based Actions
    fn pickup_coin_session(ref self: T, game_id: u32, level: u32, coin_index: u32) -> bool;
    fn get_session_progress(self: @T, game_id: u32, level: u32) -> SessionProgress;
    fn reset_session(ref self: T, game_id: u32, level: u32);
    fn finalize_session(ref self: T, game_id: u32, level: u32, success: bool);

    // Turn-based combat
    fn fight_start(ref self: T, game_id: u32, level: u32);
    fn fight_player_attack(ref self: T, game_id: u32, level: u32, target_beast_id: felt252);
    fn fight_enemy_turn(ref self: T, game_id: u32, level: u32);
    fn fight_flee(ref self: T, game_id: u32, level: u32);
    
    // Player Stats & Inventory
    fn get_player_stats(self: @T, player: ContractAddress) -> PlayerStats;
    fn get_player_inventory(self: @T, player: ContractAddress) -> PlayerInventory;
    fn get_level_items(self: @T, game_id: u32, level: u32) -> LevelItems;
    
    // Admin Management
    fn add_admin(ref self: T, admin_address: ContractAddress, role: felt252, permissions: u32);
    fn remove_admin(ref self: T, admin_address: ContractAddress);
    fn is_admin(self: @T, address: ContractAddress) -> bool;
    
    // Helper Functions
    fn calculate_player_level(self: @T, total_experience: u32) -> (u32, u32);
    fn calculate_level_experience(self: @T, game_progress: GameProgress, level: u32) -> u32;
    fn serialize_coins_data(self: @T, level_coins: LevelCoins) -> Array<felt252>;
    fn serialize_beasts_data(self: @T, level_beasts: LevelBeast) -> Array<felt252>;
    fn serialize_objectives_data(self: @T, level_objectives: LevelObjective) -> Array<felt252>;
    fn serialize_environment_data(self: @T, level_environment: LevelEnvironment) -> Array<felt252>;
    // combat helper signatures (kept in trait to avoid impl warnings)
    // Note: helper logic is inlined in methods for MVP, keep signatures for future use
    // fn get_enemy_attack_power(self: @T, beast: LevelBeast) -> u32;
    // fn get_player_attack_power(self: @T, stats: PlayerStats) -> u32;
}

// dojo decorator
#[dojo::contract]
pub mod actions {
    use core::poseidon::poseidon_hash_span;
    use dojo::event::EventStorage;
    use dojo::model::ModelStorage;
    use starknet::{ContractAddress, get_caller_address};
    use super::{
        Admin, BeastType, Game, GameCounter, GameProgress, GameStatus, IActions, ItemType, Level,
        LevelBeast, LevelCoins, LevelCounter, LevelEnvironment, LevelItems, LevelObjective,
        ObjectiveType, PlayerInventory, PlayerProfile, PlayerStats, PlayerType, SessionProgress, FightSession, FightBeastState, WorldItem,
        GAME_COUNTER_ID, LEVEL_COUNTER_ID, get_block_timestamp,
    };

    // Events
    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct PlayerProfileCreated {
        #[key]
        pub player: ContractAddress,
        pub username: felt252,
        pub player_type: PlayerType,
        pub created_at: u64,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct PlayerProfileUpdated {
        #[key]
        pub player: ContractAddress,
        pub username: felt252,
        pub player_type: PlayerType,
        pub updated_at: u64,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct GameCreated {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub created_at: u64,
    }

    #[derive(Drop, Serde)]
    #[dojo::event]
    pub struct LevelStarted {
        #[key]
        pub level_id: u32,
        pub level_name: felt252,
        pub player_type: felt252,
        pub coins_data: Array<felt252>,
        pub beasts_data: Array<felt252>,
        pub objectives_data: Array<felt252>,
        pub environment_data: Array<felt252>,
        pub next_level: u32,
        pub player: ContractAddress,
        pub game_id: u32,
        pub started_at: u64,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct LevelCompleted {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub level: u32,
        pub completed_at: u64,
        pub score: u32,
        pub experience_gained: u32,
        pub new_player_level: u32,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct LevelCreated {
        #[key]
        pub level_id: u32,
        pub level_name: felt252,
        pub player_type: PlayerType,
        pub created_by: ContractAddress,
        pub created_at: u64,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct LevelModified {
        #[key]
        pub level_id: u32,
        pub modified_by: ContractAddress,
        pub modified_at: u64,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct CoinCollected {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub level: u32,
        pub coin_index: u32,
        pub collected_at: u64,
        pub session_coins: u32,
        pub total_coins: u32,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct SessionReset {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub level: u32,
        pub reset_at: u64,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct SessionFinalized {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub level: u32,
        pub finalized_at: u64,
        pub success: bool,
        pub coins_collected: u32,
        pub beasts_defeated: u32,
        pub objectives_completed: u32,
        pub message: felt252,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct FightStarted {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub level: u32,
        pub started_at: u64,
        pub enemies: u32,
        pub player_hp: u32,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct PlayerAttacked {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub level: u32,
        pub turn: u32,
        pub target_beast_id: felt252,
        pub damage: u32,
        pub beast_hp_after: u32,
        pub beast_is_alive: bool,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct EnemyAttacked {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub level: u32,
        pub turn: u32,
        pub beast_id: felt252,
        pub damage: u32,
        pub player_hp_after: u32,
        pub player_is_alive: bool,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct FightEnded {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub level: u32,
        pub ended_at: u64,
        pub victory: bool,
        pub enemies_defeated: u32,
        pub player_hp: u32,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct BeastDefeated {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub level: u32,
        pub beast_id: felt252,
        pub defeated_at: u64,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct ObjectiveCompleted {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub level: u32,
        pub objective_id: felt252,
        pub completed_at: u64,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct PlayerLevelUp {
        #[key]
        pub player: ContractAddress,
        pub old_level: u32,
        pub new_level: u32,
        pub new_max_health: u32,
        pub experience_gained: u32,
        pub timestamp: u64,
    }

    #[abi(embed_v0)]
    impl ActionsImpl of IActions<ContractState> {
        // Player Profile Management
        fn create_player_profile(ref self: ContractState, username: felt252, player_type: PlayerType) {
            let mut world = self.world_default();
            let player = get_caller_address();
            let current_time = get_block_timestamp();

            // Check if player profile already exists
            let profile_exists = self.player_profile_exists(world, player);
            assert(!profile_exists, 'Player profile already exists');

            let profile = PlayerProfile {
                player,
                username,
                player_type,
                created_at: current_time,
                last_active: current_time,
                total_games_played: 0,
                total_score: 0,
                highest_level_reached: 1,
                is_active: true,
            };
            world.write_model(@profile);

            // Initialize player stats
            let player_stats = PlayerStats {
                player,
                health: 100,
                max_health: 100,
                level: 1,
                experience: 0,
                items_collected: 0,
                beasts_defeated: 0,
                objectives_completed: 0,
            };
            world.write_model(@player_stats);

            // Initialize player inventory
            let inventory = PlayerInventory {
                player,
                health_potions: 0,
                survival_kits: 0,
                books: 0,
                coins: 0,
                beast_essences: 0,
                ancient_knowledge: 0,
                capacity: 50,
            };
            world.write_model(@inventory);

            world.emit_event(@PlayerProfileCreated {
                player,
                username,
                player_type,
                created_at: current_time,
            });
        }

        fn update_player_profile(ref self: ContractState, username: felt252, player_type: PlayerType) {
            let mut world = self.world_default();
            let player = get_caller_address();
            let current_time = get_block_timestamp();

            let mut profile: PlayerProfile = world.read_model(player);
            profile.username = username;
            profile.player_type = player_type;
            profile.last_active = current_time;
            world.write_model(@profile);

            world.emit_event(@PlayerProfileUpdated {
                player,
                username,
                player_type,
                updated_at: current_time,
            });
        }

        fn get_player_profile(self: @ContractState, player: ContractAddress) -> PlayerProfile {
            let world = self.world_default();
            world.read_model(player)
        }

        // Game Management
        fn create_game(ref self: ContractState) -> u32 {
            let mut world = self.world_default();
            let player = get_caller_address();
            let current_time = get_block_timestamp();

            // Verify player profile exists
            let profile_exists = self.player_profile_exists(world, player);
            assert(profile_exists, 'Player profile does not exist');

            // Get player profile to determine player type
            let profile: PlayerProfile = world.read_model(player);

            // Get or initialize the game counter
            let mut counter: GameCounter = world.read_model(GAME_COUNTER_ID);
            if counter.next_game_id == 0 {
                counter.counter_id = GAME_COUNTER_ID;
                counter.next_game_id = 1;
            }

            let game_id = counter.next_game_id;
            counter.next_game_id += 1;
            world.write_model(@counter);

            // Create new game
            let game = Game {
                game_id,
                player,
                status: GameStatus::InProgress,
                current_level: 1,
                created_at: current_time,
                score: 0,
                player_type: profile.player_type,
            };
            world.write_model(@game);

            // Update player profile
            let mut player_profile: PlayerProfile = world.read_model(player);
            player_profile.total_games_played += 1;
            player_profile.last_active = current_time;
            world.write_model(@player_profile);

            world.emit_event(@GameCreated {
                player,
                game_id,
                created_at: current_time,
            });

            game_id
        }

        fn start_level(ref self: ContractState, game_id: u32, level: u32) {
            let mut world = self.world_default();
            let player = get_caller_address();
            let current_time = get_block_timestamp();

            // Verify game exists and player owns it
            let mut game: Game = world.read_model(game_id);
            assert(game.player == player, 'Not your game');
            assert(game.status == GameStatus::InProgress, 'Game not in progress');

            // Verify level exists and is active
            let level_data: Level = world.read_model(level);
            assert(level_data.is_active, 'Level is not active');
            assert(level_data.player_type == game.player_type, 'Level not compatible');

            // Verify player has reached this level (can't skip levels)
            assert(level <= game.current_level + 1, 'Cannot skip levels');

            // Update game level
            game.current_level = level;
            world.write_model(@game);

            // Initialize or update game progress for this level
            let game_progress = GameProgress {
                game_id,
                level,
                coins_collected: 0,
                beasts_defeated: 0,
                objectives_completed: 0,
                level_started_at: current_time,
                level_completed_at: 0,
                is_level_completed: false,
            };
            world.write_model(@game_progress);

            // Get level data for emission
            let level_coins: LevelCoins = world.read_model(level);
            let level_beasts: LevelBeast = world.read_model(level);
            let level_objectives: LevelObjective = world.read_model(level);
            let level_environment: LevelEnvironment = world.read_model(level);

            // Emit comprehensive level data for game client
            world.emit_event(@LevelStarted {
                level_id: level,
                level_name: level_data.level_name,
                player_type: level_data.player_type.into(),
                coins_data: self.serialize_coins_data(level_coins),
                beasts_data: self.serialize_beasts_data(level_beasts),
                objectives_data: self.serialize_objectives_data(level_objectives),
                environment_data: self.serialize_environment_data(level_environment),
                next_level: level_data.next_level,
                player,
                game_id,
                started_at: current_time,
            });
        }

        fn complete_level(ref self: ContractState, game_id: u32, level: u32) {
            let mut world = self.world_default();
            let player = get_caller_address();
            let current_time = get_block_timestamp();

            // Verify game exists and player owns it
            let mut game: Game = world.read_model(game_id);
            assert(game.player == player, 'Not your game');
            assert(game.status == GameStatus::InProgress, 'Game not in progress');

            // Verify this is the current level being played
            assert(game.current_level == level, 'Complete current level first');

            // Get game progress for this level
            let mut game_progress: GameProgress = world.read_model((game_id, level));
            assert(!game_progress.is_level_completed, 'Level already completed');

            // Verify level completion requirements
            let level_objectives: LevelObjective = world.read_model(level);
            let required_objectives = level_objectives.objective_count;
            assert(game_progress.objectives_completed >= required_objectives, 'Objectives not completed');

            // Mark level as completed
            game_progress.is_level_completed = true;
            game_progress.level_completed_at = current_time;
            world.write_model(@game_progress);

            // Calculate level score and experience
            let level_score = self.calculate_level_score(game_progress);
            let level_experience = self.calculate_level_experience(game_progress, level);

            // Update game score and level
            game.score += level_score;
            game.current_level = level + 1;
            world.write_model(@game);

            // Update player profile and stats
            let mut player_profile: PlayerProfile = world.read_model(player);
            let mut player_stats: PlayerStats = world.read_model(player);

            player_profile.total_score = player_profile.total_score + level_score.into();
            if level > player_profile.highest_level_reached {
                player_profile.highest_level_reached = level;
            }
            player_profile.last_active = current_time;

            // Level up logic
            let new_experience = player_stats.experience + level_experience;
            let (new_level, remaining_experience) = self.calculate_player_level(new_experience);
            
            if new_level > player_stats.level {
                // Level up!
                let old_level = player_stats.level;
                player_stats.level = new_level;
                player_stats.max_health += 10; // Increase max health
                player_stats.health = player_stats.max_health; // Restore health on level up
                
                world.emit_event(@PlayerLevelUp {
                    player,
                    old_level,
                    new_level,
                    new_max_health: player_stats.max_health,
                    experience_gained: level_experience,
                    timestamp: current_time,
                });
            }

            player_stats.experience = remaining_experience;
            world.write_model(@player_profile);
            world.write_model(@player_stats);

            world.emit_event(@LevelCompleted {
                player,
                game_id,
                level,
                completed_at: current_time,
                score: level_score,
                experience_gained: level_experience,
                new_player_level: player_stats.level,
            });
        }

        // Helper function to calculate player level from experience
        fn calculate_player_level(self: @ContractState, total_experience: u32) -> (u32, u32) {
            // Simple level calculation: every 100 exp = level up
            let level = (total_experience / 100) + 1;
            let remaining_exp = total_experience % 100;
            (level, remaining_exp)
        }

        // Helper function to calculate level experience
        fn calculate_level_experience(self: @ContractState, game_progress: GameProgress, level: u32) -> u32 {
            let base_experience = level * 50; // Base experience per level
            let coin_bonus = game_progress.coins_collected * 10; // 10 exp per coin
            let beast_bonus = game_progress.beasts_defeated * 25; // 25 exp per beast
            let objective_bonus = game_progress.objectives_completed * 100; // 100 exp per objective
            
            base_experience + coin_bonus + beast_bonus + objective_bonus
        }

        // Helper function to serialize coins data
        fn serialize_coins_data(self: @ContractState, level_coins: LevelCoins) -> Array<felt252> {
            let mut data = ArrayTrait::new();
            data.append(level_coins.spawn_count.into());
            
            // For now, return simplified data structure
            // In a full implementation, you'd iterate through coin positions
            data
        }

        // Helper function to serialize beasts data
        fn serialize_beasts_data(self: @ContractState, level_beasts: LevelBeast) -> Array<felt252> {
            let mut data = ArrayTrait::new();
            data.append(level_beasts.beast_id.into());
            data.append(level_beasts.beast_type.into());
            data.append(level_beasts.spawn_position_x.into());
            data.append(level_beasts.spawn_position_y.into());
            data.append(level_beasts.spawn_position_z.into());
            data.append(level_beasts.health.into());
            data.append(level_beasts.damage.into());
            data.append(level_beasts.speed.into());
            data
        }

        // Helper function to serialize objectives data
        fn serialize_objectives_data(self: @ContractState, level_objectives: LevelObjective) -> Array<felt252> {
            let mut data = ArrayTrait::new();
            data.append(level_objectives.objective_id.into());
            data.append(level_objectives.title.into());
            data.append(level_objectives.description.into());
            data.append(level_objectives.objective_type.into());
            data.append(level_objectives.target.into());
            data.append(level_objectives.required_count.into());
            data.append(level_objectives.reward.into());
            data
        }

        // Helper function to serialize environment data
        fn serialize_environment_data(self: @ContractState, level_environment: LevelEnvironment) -> Array<felt252> {
            let mut data = ArrayTrait::new();
            data.append(level_environment.dungeon_scale.into());
            data.append(level_environment.dungeon_position_x.into());
            data.append(level_environment.dungeon_position_y.into());
            data.append(level_environment.dungeon_position_z.into());
            data.append(level_environment.dungeon_rotation.into());
            data
        }

        fn get_game(self: @ContractState, game_id: u32) -> Game {
            let world = self.world_default();
            world.read_model(game_id)
        }

        fn get_game_progress(self: @ContractState, game_id: u32, level: u32) -> GameProgress {
            let world = self.world_default();
            world.read_model((game_id, level))
        }

        // Level Management (Admin Only)
        fn admin_create_level(
            ref self: ContractState,
            level_name: felt252,
            player_type: PlayerType,
            next_level: u32,
            coins_data: Array<felt252>,
            beasts_data: Array<felt252>,
            objectives_data: Array<felt252>,
            environment_data: Array<felt252>
        ) -> u32 {
            let caller = get_caller_address();
            assert(self.is_admin(caller), 'Only admins can create levels');

            let mut world = self.world_default();
            let current_time = get_block_timestamp();

            // Get or initialize level counter
            let mut level_counter: LevelCounter = world.read_model(LEVEL_COUNTER_ID);
            if level_counter.next_level_id == 0 {
                level_counter.counter_id = LEVEL_COUNTER_ID;
                level_counter.next_level_id = 1;
            }

            let level_id = level_counter.next_level_id;
            level_counter.next_level_id += 1;
            world.write_model(@level_counter);

            // Create level
            let level = Level {
                level_id,
                level_name,
                player_type,
                is_active: true,
                created_at: current_time,
                modified_at: current_time,
                created_by: caller,
                next_level,
            };
            world.write_model(@level);

            // Parse and create coins data
            self.create_level_coins(world, level_id, coins_data);

            // Parse and create beasts data
            self.create_level_beasts(world, level_id, beasts_data);

            // Parse and create objectives data
            self.create_level_objectives(world, level_id, objectives_data);

            // Parse and create environment data
            self.create_level_environment(world, level_id, environment_data);

            world.emit_event(@LevelCreated {
                level_id,
                level_name,
                player_type,
                created_by: caller,
                created_at: current_time,
            });

            level_id
        }

        fn admin_modify_level(
            ref self: ContractState,
            level_id: u32,
            level_name: felt252,
            player_type: PlayerType,
            next_level: u32,
            coins_data: Array<felt252>,
            beasts_data: Array<felt252>,
            objectives_data: Array<felt252>,
            environment_data: Array<felt252>
        ) {
            let caller = get_caller_address();
            assert(self.is_admin(caller), 'Only admins can modify levels');

            let mut world = self.world_default();
            let current_time = get_block_timestamp();

            // Verify level exists
            let mut level: Level = world.read_model(level_id);
            level.level_name = level_name;
            level.player_type = player_type;
            level.next_level = next_level;
            level.modified_at = current_time;
            world.write_model(@level);

            // Clear existing data and recreate
            self.clear_level_data(world, level_id);

            // Recreate with new data
            self.create_level_coins(world, level_id, coins_data);
            self.create_level_beasts(world, level_id, beasts_data);
            self.create_level_objectives(world, level_id, objectives_data);
            self.create_level_environment(world, level_id, environment_data);

            world.emit_event(@LevelModified {
                level_id,
                modified_by: caller,
                modified_at: current_time,
            });
        }

        fn admin_deactivate_level(ref self: ContractState, level_id: u32) {
            let caller = get_caller_address();
            assert(self.is_admin(caller), 'Only admins can deactivate');

            let mut world = self.world_default();
            let mut level: Level = world.read_model(level_id);
            level.is_active = false;
            world.write_model(@level);
        }

        fn admin_activate_level(ref self: ContractState, level_id: u32) {
            let caller = get_caller_address();
            assert(self.is_admin(caller), 'Only admins can activate levels');

            let mut world = self.world_default();
            let mut level: Level = world.read_model(level_id);
            level.is_active = true;
            world.write_model(@level);
        }

        fn get_level(self: @ContractState, level_id: u32) -> Level {
            let world = self.world_default();
            world.read_model(level_id)
        }

        fn get_level_coins(self: @ContractState, level_id: u32) -> LevelCoins {
            let world = self.world_default();
            world.read_model(level_id)
        }

        fn get_level_beasts(self: @ContractState, level_id: u32) -> Array<LevelBeast> {
            // This would need to be implemented with proper array handling
            // For now, returning empty array
            array![]
        }

        fn get_level_objectives(self: @ContractState, level_id: u32) -> Array<LevelObjective> {
            // This would need to be implemented with proper array handling
            // For now, returning empty array
            array![]
        }

        fn get_level_environment(self: @ContractState, level_id: u32) -> LevelEnvironment {
            let world = self.world_default();
            world.read_model(level_id)
        }

        // Gameplay Actions
        fn pickup_item(ref self: ContractState, game_id: u32, item_id: u32) -> bool {
            let mut world = self.world_default();
            let player = get_caller_address();

            // Verify game exists and player owns it
            let game: Game = world.read_model(game_id);
            assert(game.player == player, 'Not your game');
            assert(game.status == GameStatus::InProgress, 'Game not in progress');

            // Get the item
            let mut world_item: WorldItem = world.read_model((game_id, item_id));
            assert(!world_item.is_collected, 'Item already collected');
            assert(world_item.level == game.current_level, 'Item not in current level');

            // Mark item as collected
            world_item.is_collected = true;
            world.write_model(@world_item);

            // Update player inventory
            let mut inventory: PlayerInventory = world.read_model(player);
            match world_item.item_type {
                ItemType::HealthPotion => { inventory.health_potions += 1; },
                ItemType::SurvivalKit => { inventory.survival_kits += 1; },
                ItemType::Book => { inventory.books += 1; },
                ItemType::Coin => { inventory.coins += 1; },
                ItemType::BeastEssence => { inventory.beast_essences += 1; },
                ItemType::AncientKnowledge => { inventory.ancient_knowledge += 1; },
            };
            world.write_model(@inventory);

            // Update level items collected count
            let mut level_items: LevelItems = world.read_model((game_id, world_item.level));
            match world_item.item_type {
                ItemType::HealthPotion => { level_items.collected_health_potions += 1; },
                ItemType::SurvivalKit => { level_items.collected_survival_kits += 1; },
                ItemType::Book => { level_items.collected_books += 1; },
                _ => {},
            };
            world.write_model(@level_items);

            // Update player stats
            let mut player_stats: PlayerStats = world.read_model(player);
            player_stats.items_collected += 1;
            player_stats.experience += 10;

            // Simple leveling: every 100 exp = level up
            let new_level = (player_stats.experience / 100) + 1;
            if new_level > player_stats.level {
                player_stats.level = new_level;
                player_stats.max_health += 10;
                player_stats.health = player_stats.max_health;
            }

            world.write_model(@player_stats);

            true
        }

        fn collect_coin(ref self: ContractState, game_id: u32, level: u32, coin_index: u32) {
            let mut world = self.world_default();
            let player = get_caller_address();
            let current_time = get_block_timestamp();

            // Verify game exists and player owns it
            let game: Game = world.read_model(game_id);
            assert(game.player == player, 'Not your game');
            assert(game.status == GameStatus::InProgress, 'Game not in progress');

            // Update game progress
            let mut game_progress: GameProgress = world.read_model((game_id, level));
            game_progress.coins_collected += 1;
            world.write_model(@game_progress);

            // Update player inventory
            let mut inventory: PlayerInventory = world.read_model(player);
            inventory.coins += 1;
            world.write_model(@inventory);

            world.emit_event(@CoinCollected {
                player,
                game_id,
                level,
                coin_index,
                collected_at: current_time,
                session_coins: 0, // No session data for this event
                total_coins: game_progress.coins_collected,
            });
        }

        fn defeat_beast(ref self: ContractState, game_id: u32, level: u32, beast_id: felt252) {
            let mut world = self.world_default();
            let player = get_caller_address();
            let current_time = get_block_timestamp();

            // Verify game exists and player owns it
            let game: Game = world.read_model(game_id);
            assert(game.player == player, 'Not your game');
            assert(game.status == GameStatus::InProgress, 'Game not in progress');

            // Update game progress
            let mut game_progress: GameProgress = world.read_model((game_id, level));
            game_progress.beasts_defeated += 1;
            world.write_model(@game_progress);

            // Update player stats
            let mut player_stats: PlayerStats = world.read_model(player);
            player_stats.beasts_defeated += 1;
            player_stats.experience += 50;
            world.write_model(@player_stats);

            // Add beast essence to inventory
            let mut inventory: PlayerInventory = world.read_model(player);
            inventory.beast_essences += 1;
            world.write_model(@inventory);

            world.emit_event(@BeastDefeated {
                player,
                game_id,
                level,
                beast_id,
                defeated_at: current_time,
            });
        }

        fn complete_objective(ref self: ContractState, game_id: u32, level: u32, objective_id: felt252) {
            let mut world = self.world_default();
            let player = get_caller_address();
            let current_time = get_block_timestamp();

            // Verify game exists and player owns it
            let game: Game = world.read_model(game_id);
            assert(game.player == player, 'Not your game');
            assert(game.status == GameStatus::InProgress, 'Game not in progress');

            // Update game progress
            let mut game_progress: GameProgress = world.read_model((game_id, level));
            game_progress.objectives_completed += 1;
            world.write_model(@game_progress);

            // Update player stats
            let mut player_stats: PlayerStats = world.read_model(player);
            player_stats.objectives_completed += 1;
            player_stats.experience += 100;
            world.write_model(@player_stats);

            world.emit_event(@ObjectiveCompleted {
                player,
                game_id,
                level,
                objective_id,
                completed_at: current_time,
            });
        }

                // Session-based coin pickup function
        fn pickup_coin_session(ref self: ContractState, game_id: u32, level: u32, coin_index: u32) -> bool {
            let mut world = self.world_default();
            let player = get_caller_address();
            let current_time = get_block_timestamp();

            // Verify game exists and player owns it
            let game: Game = world.read_model(game_id);
            assert(game.player == player, 'Not your game');
            assert(game.status == GameStatus::InProgress, 'Game not in progress');
            assert(game.current_level == level, 'Not current level');

            // Get or create session progress for this level attempt
            let mut session_progress: SessionProgress = world.read_model((game_id, level));
            if !session_progress.is_session_active {
                // Start new session
                session_progress = SessionProgress {
                    game_id,
                    level,
                    session_started_at: current_time,
                    coins_collected: 0,
                    beasts_defeated: 0,
                    objectives_completed: 0,
                    health_potions_found: 0,
                    survival_kits_found: 0,
                    books_found: 0,
                    beast_essences_found: 0,
                    ancient_knowledge_found: 0,
                    is_session_active: true,
                };
            }

            // Update session progress - add coin
            session_progress.coins_collected += 1;
            world.write_model(@session_progress);

            // Update permanent game progress
            let mut game_progress: GameProgress = world.read_model((game_id, level));
            game_progress.coins_collected += 1;
            world.write_model(@game_progress);

            // Update permanent player inventory
            let mut inventory: PlayerInventory = world.read_model(player);
            inventory.coins += 1;
            world.write_model(@inventory);

            // Emit event for coin collection
            world.emit_event(@CoinCollected {
                player,
                game_id,
                level,
                coin_index,
                collected_at: current_time,
                session_coins: session_progress.coins_collected,
                total_coins: game_progress.coins_collected,
            });

            true
        }

        // Get current session progress
        fn get_session_progress(self: @ContractState, game_id: u32, level: u32) -> SessionProgress {
            let world = self.world_default();
            world.read_model((game_id, level))
        }

        // Reset session for a new level attempt
        fn reset_session(ref self: ContractState, game_id: u32, level: u32) {
            let mut world = self.world_default();
            let player = get_caller_address();
            let current_time = get_block_timestamp();

            // Verify game exists and player owns it
            let game: Game = world.read_model(game_id);
            assert(game.player == player, 'Not your game');
            assert(game.status == GameStatus::InProgress, 'Game not in progress');

            // Reset session progress for fresh attempt
            let session_progress = SessionProgress {
                game_id,
                level,
                session_started_at: current_time,
                coins_collected: 0,
                beasts_defeated: 0,
                objectives_completed: 0,
                health_potions_found: 0,
                survival_kits_found: 0,
                books_found: 0,
                beast_essences_found: 0,
                ancient_knowledge_found: 0,
                is_session_active: true,
            };
            world.write_model(@session_progress);

            // Emit session reset event
            world.emit_event(@SessionReset {
                player,
                game_id,
                level,
                reset_at: current_time,
            });
        }

        // Finalize session and decide what to keep
        fn finalize_session(ref self: ContractState, game_id: u32, level: u32, success: bool) {
            let mut world = self.world_default();
            let player = get_caller_address();
            let current_time = get_block_timestamp();

            // Verify game exists and player owns it
            let game: Game = world.read_model(game_id);
            assert(game.player == player, 'Not your game');
            assert(game.status == GameStatus::InProgress, 'Game not in progress');

            // Get session progress
            let mut session_progress: SessionProgress = world.read_model((game_id, level));

            if success {
                // Level completed successfully - keep all session progress
                // Update permanent game progress with session totals
                let mut game_progress: GameProgress = world.read_model((game_id, level));
                game_progress.coins_collected = session_progress.coins_collected;
                game_progress.beasts_defeated = session_progress.beasts_defeated;
                game_progress.objectives_completed = session_progress.objectives_completed;
                world.write_model(@game_progress);

                // Mark session as completed
                session_progress.is_session_active = false;
                world.write_model(@session_progress);

                world.emit_event(@SessionFinalized {
                    player,
                    game_id,
                    level,
                    finalized_at: current_time,
                    success: true,
                    coins_collected: session_progress.coins_collected,
                    beasts_defeated: session_progress.beasts_defeated,
                    objectives_completed: session_progress.objectives_completed,
                    message: 'Success',
                });
            } else {
                // Level failed - reset session but keep permanent inventory
                // Session progress is lost, but coins in inventory remain
                session_progress.is_session_active = false;
                world.write_model(@session_progress);

                world.emit_event(@SessionFinalized {
                    player,
                    game_id,
                    level,
                    finalized_at: current_time,
                    success: false,
                    coins_collected: session_progress.coins_collected,
                    beasts_defeated: session_progress.beasts_defeated,
                    objectives_completed: session_progress.objectives_completed,
                    message: 'Failed',
                });
            }
        }

        // Initialize a fight session using level beasts data
        fn fight_start(ref self: ContractState, game_id: u32, level: u32) {
            let mut world = self.world_default();
            let player = get_caller_address();
            let now = get_block_timestamp();

            // Validate game and level
            let game: Game = world.read_model(game_id);
            assert(game.player == player, 'Not your game');
            assert(game.status == GameStatus::InProgress, 'Game not in progress');
            assert(game.current_level == level, 'Not current level');

            // Load player stats
            let stats: PlayerStats = world.read_model(player);

            // Count enemies for this level
            // Note: If multiple beasts per level are stored as an Array, adapt retrieval accordingly.
            // Here we assume one record per beast keyed by (level, beast_id) is retrievable via a known set.
            // For MVP, we read one canonical beast and set enemies_remaining to 1 if exists.
            let maybe_beast: LevelBeast = world.read_model(level);
            let enemies_remaining = 1;

            // Initialize fight session
            let fight = FightSession {
                game_id,
                level,
                started_at: now,
                turn_number: 1,
                is_player_turn: true,
                player_hp_current: stats.health,
                enemies_remaining,
                is_active: true,
            };
            world.write_model(@fight);

            // Initialize beast state
            let beast_state = FightBeastState {
                game_id,
                level,
                beast_id: maybe_beast.beast_id,
                hp_current: maybe_beast.health,
                is_alive: true,
            };
            world.write_model(@beast_state);

            world.emit_event(@FightStarted {
                player,
                game_id,
                level,
                started_at: now,
                enemies: enemies_remaining,
                player_hp: stats.health,
            });
        }

        // Player performs an attack against a target beast
        fn fight_player_attack(ref self: ContractState, game_id: u32, level: u32, target_beast_id: felt252) {
            let mut world = self.world_default();
            let player = get_caller_address();

            let mut fight: FightSession = world.read_model((game_id, level));
            assert(fight.is_active, 'Fight not active');
            assert(fight.is_player_turn, 'Not player turn');

            let stats: PlayerStats = world.read_model(player);
            let mut beast_state: FightBeastState = world.read_model((game_id, level, target_beast_id));
            assert(beast_state.is_alive, 'Beast already dead');

            // Compute damage
            // Player damage formula (MVP): level * 5
            let damage = stats.level * 5;
            if damage >= beast_state.hp_current {
                beast_state.hp_current = 0;
                beast_state.is_alive = false;
                fight.enemies_remaining -= 1;
            } else {
                beast_state.hp_current -= damage;
            }
            world.write_model(@beast_state);

            // Advance turn
            fight.is_player_turn = false;
            fight.turn_number += 1;
            world.write_model(@fight);

            world.emit_event(@PlayerAttacked {
                player,
                game_id,
                level,
                turn: fight.turn_number,
                target_beast_id,
                damage,
                beast_hp_after: beast_state.hp_current,
                beast_is_alive: beast_state.is_alive,
            });
        }

        // Enemies perform their turn; single active enemy for MVP
        fn fight_enemy_turn(ref self: ContractState, game_id: u32, level: u32) {
            let mut world = self.world_default();
            let player = get_caller_address();

            let mut fight: FightSession = world.read_model((game_id, level));
            assert(fight.is_active, 'Fight not active');
            assert(!fight.is_player_turn, 'Not enemy turn');

            let mut beast_state: FightBeastState = world.read_model((game_id, level, 'monster_1'));
            if !beast_state.is_alive {
                // If no alive beast, end fight with victory
                fight.is_active = false;
                world.write_model(@fight);
                world.emit_event(@FightEnded {
                    player,
                    game_id,
                    level,
                    ended_at: get_block_timestamp(),
                    victory: true,
                    enemies_defeated: 1,
                    player_hp: fight.player_hp_current,
                });
                return ();
            }

            // Load level beast config for damage and compute enemy damage
            let level_beast: LevelBeast = world.read_model(level);
            let mut enemy_damage = level_beast.damage;
            enemy_damage = match level_beast.beast_type {
                BeastType::Monster => enemy_damage + 0,
                BeastType::Dragon => enemy_damage + 10,
                BeastType::Goblin => enemy_damage + 2,
                BeastType::Orc => enemy_damage + 5,
                BeastType::Demon => enemy_damage + 12,
                BeastType::Undead => enemy_damage + 6,
                BeastType::Elemental => enemy_damage + 8,
            };

            // Apply damage to player
            if enemy_damage >= fight.player_hp_current {
                fight.player_hp_current = 0;
            } else {
                fight.player_hp_current -= enemy_damage;
            }

            // Advance turn to player or end fight if player dead
            let player_alive = fight.player_hp_current > 0;
            if player_alive {
                fight.is_player_turn = true;
                fight.turn_number += 1;
                world.write_model(@fight);
            } else {
                fight.is_active = false;
                world.write_model(@fight);
            }

            world.emit_event(@EnemyAttacked {
                player,
                game_id,
                level,
                turn: fight.turn_number,
                beast_id: beast_state.beast_id,
                damage: enemy_damage,
                player_hp_after: fight.player_hp_current,
                player_is_alive: player_alive,
            });

            if !player_alive {
                world.emit_event(@FightEnded {
                    player,
                    game_id,
                    level,
                    ended_at: get_block_timestamp(),
                    victory: false,
                    enemies_defeated: 0,
                    player_hp: 0,
                });
            }
        }

        // Player flees; ends the fight as loss
        fn fight_flee(ref self: ContractState, game_id: u32, level: u32) {
            let mut world = self.world_default();
            let player = get_caller_address();

            let mut fight: FightSession = world.read_model((game_id, level));
            assert(fight.is_active, 'Fight not active');

            fight.is_active = false;
            world.write_model(@fight);

            world.emit_event(@FightEnded {
                player,
                game_id,
                level,
                ended_at: get_block_timestamp(),
                victory: false,
                enemies_defeated: 0,
                player_hp: fight.player_hp_current,
            });
        }

        // Player Stats & Inventory
        fn get_player_stats(self: @ContractState, player: ContractAddress) -> PlayerStats {
            let world = self.world_default();
            world.read_model(player)
        }

        fn get_player_inventory(self: @ContractState, player: ContractAddress) -> PlayerInventory {
            let world = self.world_default();
            world.read_model(player)
        }

        fn get_level_items(self: @ContractState, game_id: u32, level: u32) -> LevelItems {
            let world = self.world_default();
            world.read_model((game_id, level))
        }

        // Admin Management
        fn add_admin(ref self: ContractState, admin_address: ContractAddress, role: felt252, permissions: u32) {
            let caller = get_caller_address();
            assert(self.is_admin(caller), 'Only admins can add');

            let mut world = self.world_default();
            let current_time = get_block_timestamp();

            let admin = Admin {
                admin_address,
                role,
                permissions,
                added_at: current_time,
            };
            world.write_model(@admin);
        }

        fn remove_admin(ref self: ContractState, admin_address: ContractAddress) {
            let caller = get_caller_address();
            assert(self.is_admin(caller), 'Only admins can remove');

            // Note: In a real implementation, you'd need to handle model deletion
            // For now, this is a placeholder
        }

        fn is_admin(self: @ContractState, address: ContractAddress) -> bool {
            // Check if the address is an admin
            // This would need proper implementation based on your admin model
            // For now, returning false as placeholder
            false
        }
    }

    #[generate_trait]
    impl InternalImpl of InternalTrait {
        /// Use the default namespace "elysium_descent". This function is handy since the ByteArray
        /// can't be const.
        fn world_default(self: @ContractState) -> dojo::world::WorldStorage {
            self.world(@"elysium_001")
        }

        /// Check if player profile exists
        fn player_profile_exists(self: @ContractState, world: dojo::world::WorldStorage, player: ContractAddress) -> bool {
            // This would need proper implementation to check if model exists
            // For now, returning false as placeholder
            false
        }

        /// Calculate level score based on performance
        fn calculate_level_score(self: @ContractState, game_progress: GameProgress) -> u32 {
            let mut score = 0_u32;
            
            // Base score for completing level
            score += 100;
            
            // Bonus for coins collected
            score += game_progress.coins_collected * 10;
            
            // Bonus for beasts defeated
            score += game_progress.beasts_defeated * 25;
            
            // Bonus for objectives completed
            score += game_progress.objectives_completed * 50;
            
            score
        }

        /// Create level coins data
        fn create_level_coins(
            ref self: ContractState,
            mut world: dojo::world::WorldStorage,
            level_id: u32,
            coins_data: Array<felt252>
        ) {
            // Simplified implementation - just create basic level coins
            let level_coins = LevelCoins {
                level_id,
                spawn_count: 15,
                total_collected: 0,
            };
            world.write_model(@level_coins);
        }

        /// Create level beasts data
        fn create_level_beasts(
            ref self: ContractState,
            mut world: dojo::world::WorldStorage,
            level_id: u32,
            beasts_data: Array<felt252>
        ) {
            // Simplified implementation - just create basic beast
            let beast = LevelBeast {
                level_id,
                beast_id: 'monster_1',
                beast_type: BeastType::Monster,
                spawn_position_x: 0,
                spawn_position_y: 1,
                spawn_position_z: 0,
                health: 100,
                damage: 25,
                speed: 3,
                is_defeated: false,
            };
            world.write_model(@beast);
        }

        /// Create level objectives data
        fn create_level_objectives(
            ref self: ContractState,
            mut world: dojo::world::WorldStorage,
            level_id: u32,
            objectives_data: Array<felt252>
        ) {
            // Simplified implementation - just create basic objective
            let objective = LevelObjective {
                level_id,
                objective_id: 'collect_coins',
                title: 'Collect Ancient Coins',
                description: 'Collect 5 coins to unlock',
                objective_type: ObjectiveType::Collect,
                target: 'coins',
                required_count: 5,
                current_count: 0,
                reward: 'unlock_level_2',
                is_completed: false,
                objective_count: 1, // Added for new event
            };
            world.write_model(@objective);
        }

        /// Create level environment data
        fn create_level_environment(
            ref self: ContractState,
            mut world: dojo::world::WorldStorage,
            level_id: u32,
            environment_data: Array<felt252>
        ) {
            // Simplified implementation - just create basic environment
            let environment = LevelEnvironment {
                level_id,
                dungeon_scale: 7,
                dungeon_position_x: 0,
                dungeon_position_y: -1,
                dungeon_position_z: 0,
                dungeon_rotation: -1,
            };
            world.write_model(@environment);
        }

        /// Clear level data when modifying
        fn clear_level_data(
            ref self: ContractState,
            world: dojo::world::WorldStorage,
            level_id: u32
        ) {
            // Note: In a real implementation, you would need to implement proper deletion
            // of all related models. This is a placeholder for the concept.
            // You might need to track all created models and delete them systematically.
        }

        /// Parse beast type from felt252
        fn parse_beast_type(self: @ContractState, beast_type_raw: felt252) -> BeastType {
            let beast_type_u32: u32 = beast_type_raw.try_into().unwrap();
            match beast_type_u32 {
                0 => BeastType::Monster,
                1 => BeastType::Dragon,
                2 => BeastType::Goblin,
                3 => BeastType::Orc,
                4 => BeastType::Demon,
                5 => BeastType::Undead,
                6 => BeastType::Elemental,
                _ => BeastType::Monster, // Default fallback
            }
        }

        /// Parse objective type from felt252
        fn parse_objective_type(self: @ContractState, objective_type_raw: felt252) -> ObjectiveType {
            let objective_type_u32: u32 = objective_type_raw.try_into().unwrap();
            match objective_type_u32 {
                0 => ObjectiveType::Collect,
                1 => ObjectiveType::ReachLocation,
                2 => ObjectiveType::Defeat,
                3 => ObjectiveType::Survive,
                4 => ObjectiveType::Explore,
                _ => ObjectiveType::Collect, // Default fallback
            }
        }

        /// Calculate number of health potions for a level
        fn calculate_level_health_potions(self: @ContractState, level: u32) -> u32 {
            // Formula: base 3 potions + 1 per level, max 10
            let potions = 3 + level;
            if potions > 10 {
                10
            } else {
                potions
            }
        }

        /// Calculate number of survival kits for a level
        fn calculate_level_survival_kits(self: @ContractState, level: u32) -> u32 {
            // Formula: 1 survival kit every 2 levels, max 3
            let kits = (level + 1) / 2;
            if kits > 3 {
                3
            } else {
                kits
            }
        }

        /// Calculate number of books for a level
        fn calculate_level_books(self: @ContractState, level: u32) -> u32 {
            // Formula: 1 book every 3 levels, max 2
            let books = level / 3;
            if books > 2 {
                2
            } else {
                books
            }
        }

        /// Generate unique item ID using hash
        fn generate_item_id(
            self: @ContractState, game_id: u32, level: u32, item_counter: u32,
        ) -> u32 {
            let hash = poseidon_hash_span(
                array![game_id.into(), level.into(), item_counter.into()].span(),
            );
            hash.try_into().unwrap()
        }

        /// Generate deterministic but varied item positions
        fn generate_item_position(
            self: @ContractState, game_id: u32, level: u32, item_counter: u32,
        ) -> (u32, u32) {
            let seed_x = poseidon_hash_span(
                array![game_id.into(), level.into(), item_counter.into(), 'POSX'.into()].span(),
            );
            let seed_y = poseidon_hash_span(
                array![game_id.into(), level.into(), item_counter.into(), 'POSY'.into()].span(),
            );

            // Convert felt252 to u256 for modulo operations
            let x_u256: u256 = seed_x.into();
            let y_u256: u256 = seed_y.into();

            let x = ((x_u256 % 100) + 10).try_into().unwrap(); // X position between 10-109
            let y = ((y_u256 % 100) + 10).try_into().unwrap(); // Y position between 10-109
            (x, y)
        }
    }
}
