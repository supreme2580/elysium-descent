use elysium_descent::models::{
    GAME_COUNTER_ID, Game, GameCounter, GameStatus, ItemType, LevelItems, PlayerInventory,
    PlayerStats, WorldItem,
};
use starknet::{ContractAddress, get_block_timestamp};

// define the interface
#[starknet::interface]
pub trait IActions<T> {
    fn create_game(ref self: T) -> u32;
    fn start_level(ref self: T, game_id: u32, level: u32);
    fn pickup_item(ref self: T) -> bool;
    fn get_player_stats(self: @T, player: ContractAddress) -> PlayerStats;
    fn get_player_inventory(self: @T, player: ContractAddress) -> PlayerInventory;
    fn get_level_items(self: @T, game_id: u32, level: u32) -> LevelItems;
}
//fn _pickup_item(ref self: T, game_id: u32, item_id: u32) -> bool;

// dojo decorator
#[dojo::contract]
pub mod actions {
    use core::poseidon::poseidon_hash_span;
    use dojo::event::EventStorage;
    use dojo::model::ModelStorage;
    use starknet::{ContractAddress, get_caller_address};
    use super::{
        GAME_COUNTER_ID, Game, GameCounter, GameStatus, IActions, ItemType, LevelItems,
        PlayerInventory, PlayerStats, WorldItem, get_block_timestamp,
    };

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct GameCreated {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub created_at: u64,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct LevelStarted {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub level: u32,
        pub items_spawned: u32,
    }

    #[derive(Copy, Drop, Serde)]
    #[dojo::event]
    pub struct ItemPickedUp {
        #[key]
        pub player: ContractAddress,
        pub game_id: u32,
        pub item_id: u32,
        pub item_type: ItemType,
        pub level: u32,
    }

    #[abi(embed_v0)]
    impl ActionsImpl of IActions<ContractState> {
        fn create_game(ref self: ContractState) -> u32 {
            let mut world = self.world_default();
            let player = get_caller_address();

            // Get or initialize the game counter
            let mut counter: GameCounter = world.read_model(GAME_COUNTER_ID);

            // Always ensure the counter_id is set correctly (for new instances)
            if counter.next_game_id == 0 {
                counter.counter_id = GAME_COUNTER_ID; // ✅ CRITICAL: Set the key field!
                counter.next_game_id = 1; // Initialize if first game
            }

            let game_id = counter.next_game_id;
            counter.next_game_id += 1;

            // Save updated counter
            world.write_model(@counter);

            // Create new game
            let game = Game {
                game_id,
                player,
                status: GameStatus::InProgress,
                current_level: 0, // Start at level 0, first level starts with start_level()
                created_at: get_block_timestamp(),
                score: 0,
            };
            world.write_model(@game);

            // Initialize player stats
            let player_stats = PlayerStats {
                player, health: 100, max_health: 100, level: 1, experience: 0, items_collected: 0,
            };
            world.write_model(@player_stats);

            // Initialize player inventory
            let inventory = PlayerInventory {
                player,
                health_potions: 0,
                survival_kits: 0,
                books: 0,
                capacity: 50 // Default inventory capacity
            };
            world.write_model(@inventory);

            // Emit game created event
            world.emit_event(@GameCreated { player, game_id, created_at: get_block_timestamp() });

            game_id
        }

        fn start_level(ref self: ContractState, game_id: u32, level: u32) {
            let mut world = self.world_default();
            let player = get_caller_address();

            // Verify game exists and player owns it
            let mut game: Game = world.read_model(game_id);
            assert(game.player == player, 'Not your game');
            assert(game.status == GameStatus::InProgress, 'Game not in progress');

            // Update game level
            game.current_level = level;
            world.write_model(@game);

            // Generate items for this level based on level number
            let health_potions_count = self.calculate_level_health_potions(level);
            let survival_kits_count = self.calculate_level_survival_kits(level);
            let books_count = self.calculate_level_books(level);

            // Store level items metadata
            let level_items = LevelItems {
                game_id,
                level,
                total_health_potions: health_potions_count,
                total_survival_kits: survival_kits_count,
                total_books: books_count,
                collected_health_potions: 0,
                collected_survival_kits: 0,
                collected_books: 0,
            };
            world.write_model(@level_items);

            // Generate actual item instances in the world
            let mut item_counter = 0_u32;

            // Generate health potions
            let mut i = 0_u32;
            loop {
                if i >= health_potions_count {
                    break;
                }
                let item_id = self.generate_item_id(game_id, level, item_counter);
                let (x, y) = self.generate_item_position(game_id, level, item_counter);

                let world_item = WorldItem {
                    game_id,
                    item_id,
                    item_type: ItemType::HealthPotion,
                    x_position: x,
                    y_position: y,
                    is_collected: false,
                    level,
                };
                world.write_model(@world_item);

                item_counter += 1;
                i += 1;
            };

            // Generate survival kits
            let mut i = 0_u32;
            loop {
                if i >= survival_kits_count {
                    break;
                }
                let item_id = self.generate_item_id(game_id, level, item_counter);
                let (x, y) = self.generate_item_position(game_id, level, item_counter);

                let world_item = WorldItem {
                    game_id,
                    item_id,
                    item_type: ItemType::SurvivalKit,
                    x_position: x,
                    y_position: y,
                    is_collected: false,
                    level,
                };
                world.write_model(@world_item);

                item_counter += 1;
                i += 1;
            };

            // Generate books
            let mut i = 0_u32;
            loop {
                if i >= books_count {
                    break;
                }
                let item_id = self.generate_item_id(game_id, level, item_counter);
                let (x, y) = self.generate_item_position(game_id, level, item_counter);

                let world_item = WorldItem {
                    game_id,
                    item_id,
                    item_type: ItemType::Book,
                    x_position: x,
                    y_position: y,
                    is_collected: false,
                    level,
                };
                world.write_model(@world_item);

                item_counter += 1;
                i += 1;
            };

            let total_items = health_potions_count + survival_kits_count + books_count;
            world.emit_event(@LevelStarted { player, game_id, level, items_spawned: total_items });
        }

        fn pickup_item(ref self: ContractState) -> bool {
            let mut world = self.world_default();
            let player = get_caller_address();

            // Verify game exists and player owns it
            //let game: Game = world.read_model(game_id);
            //assert(game.player == player, 'Not your game');
            //assert(game.status == GameStatus::InProgress, 'Game not in progress');

            //// Get the item
            //let mut world_item: WorldItem = world.read_model((game_id, item_id));
            //assert(!world_item.is_collected, 'Item already collected');
            //assert(world_item.level == game.current_level, 'Item not in current level');

            //// Mark item as collected
            //world_item.is_collected = true;
            //world.write_model(@world_item);

            //// Update player inventory
            //let mut inventory: PlayerInventory = world.read_model(player);
            //match world_item.item_type {
            //    ItemType::HealthPotion => { inventory.health_potions += 1; },
            //    ItemType::SurvivalKit => { inventory.survival_kits += 1; },
            //    ItemType::Book => { inventory.books += 1; },
            //};
            //world.write_model(@inventory);

            //// Update level items collected count
            //let mut level_items: LevelItems = world.read_model((game_id, world_item.level));
            //match world_item.item_type {
            //    ItemType::HealthPotion => { level_items.collected_health_potions += 1; },
            //    ItemType::SurvivalKit => { level_items.collected_survival_kits += 1; },
            //    ItemType::Book => { level_items.collected_books += 1; },
            //};
            //world.write_model(@level_items);

            //// Update player stats
            //let mut player_stats: PlayerStats = world.read_model(player);
            //player_stats.items_collected += 1;
            //player_stats.experience += 10; // Give experience for collecting items

            //// Simple leveling: every 100 exp = level up
            //let new_level = (player_stats.experience / 100) + 1;
            //if new_level > player_stats.level {
            //    player_stats.level = new_level;
            //    player_stats.max_health += 10; // Increase max health on level up
            //    player_stats.health = player_stats.max_health; // Full heal on level up
            //}

            //world.write_model(@player_stats);

            //// Emit pickup event
            //world
            //    .emit_event(
            //        @ItemPickedUp {
            //            player,
            //            game_id,
            //            item_id,
            //            item_type: world_item.item_type,
            //            level: world_item.level,
            //        },
            //    );

            true
        }

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
    }

    #[generate_trait]
    impl InternalImpl of InternalTrait {
        /// Use the default namespace "elysium_descent". This function is handy since the ByteArray
        /// can't be const.
        fn world_default(self: @ContractState) -> dojo::world::WorldStorage {
            self.world(@"elysium_001")
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
