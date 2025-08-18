// Example: Creating "The Beginning" level using admin_create_level
// This demonstrates how to use the contract to create a level with the exact data structure from requirements

use elysium_descent::models::{PlayerType, BeastType, ObjectiveType};

// Example level data for "The Beginning" level
// This matches the JSON structure provided in the requirements

fn create_the_beginning_level_example() {
    // 1. Coins Data
    // Format: [spawn_count, x1, y1, z1, x2, y2, z2, ...]
    let coins_data = array![
        15,                    // spawn_count
        10.0, 1.0, 5.0,       // coin 1: x, y, z
        -8.0, 1.0, 12.0,      // coin 2: x, y, z
        15.0, 1.0, -3.0,      // coin 3: x, y, z
        -12.0, 1.0, -8.0,     // coin 4: x, y, z
        20.0, 1.0, 15.0,      // coin 5: x, y, z
        -18.0, 1.0, 20.0,     // coin 6: x, y, z
        25.0, 1.0, -10.0,     // coin 7: x, y, z
        -25.0, 1.0, -15.0,    // coin 8: x, y, z
        30.0, 1.0, 25.0,      // coin 9: x, y, z
        -30.0, 1.0, 30.0,     // coin 10: x, y, z
        35.0, 1.0, -20.0,     // coin 11: x, y, z
        -35.0, 1.0, -25.0,    // coin 12: x, y, z
        40.0, 1.0, 35.0,      // coin 13: x, y, z
        -40.0, 1.0, 40.0,     // coin 14: x, y, z
        45.0, 1.0, -30.0      // coin 15: x, y, z
    ];

    // 2. Beasts Data
    // Format: [beast_id1, beast_type1, x1, y1, z1, health1, damage1, speed1, ...]
    let beasts_data = array![
        "monster_1",           // beast_id
        0,                     // beast_type (0 = Monster)
        0.0, 1.0, 0.0,        // x, y, z
        100,                   // health
        25,                    // damage
        3.0                    // speed
    ];

    // 3. Objectives Data
    // Format: [objective_id1, title1, desc1, type1, target1, required1, reward1, ...]
    let objectives_data = array![
        "collect_coins",       // objective_id
        "Collect Ancient Coins", // title
        "Collect 5 coins to unlock the path forward", // description
        0,                     // objective_type (0 = collect)
        "coins",               // target
        5,                     // required_count
        "unlock_level_2",      // reward
        
        "find_ancient_book",   // objective_id
        "Find the Ancient Book", // title
        "Locate the ancient book to gain knowledge", // description
        1,                     // objective_type (1 = reach_location)
        "ancient_book",        // target
        0,                     // required_count (not used for location objectives)
        "ancient_knowledge",   // reward
        
        "defeat_monster",      // objective_id
        "Defeat the Guardian", // title
        "Defeat the monster to prove your worth", // description
        2,                     // objective_type (2 = defeat)
        "monster_1",           // target
        0,                     // required_count (not used for defeat objectives)
        "guardian_essence"     // reward
    ];

    // 4. Environment Data
    // Format: [scale, x, y, z, rotation]
    let environment_data = array![
        7.5,                   // dungeon_scale
        0.0, -1.5, 0.0,       // dungeon_x, dungeon_y, dungeon_z
        -1.5708                // dungeon_rotation
    ];

    // 5. Call the admin_create_level function
    // Note: This would be called by an admin account
    // let level_id = contract.admin_create_level(
    //     "The Beginning",       // level_name
    //     PlayerType::Man,       // player_type
    //     2,                     // next_level
    //     coins_data,            // coins_data
    //     beasts_data,           // beasts_data
    //     objectives_data,       // objectives_data
    //     environment_data       // environment_data
    // );
}

// Example: Creating a more complex level with multiple beasts and objectives
fn create_complex_level_example() {
    // Coins data for a larger level
    let coins_data = array![
        25,                    // spawn_count
        10.0, 1.0, 5.0,       // coin 1
        -8.0, 1.0, 12.0,      // coin 2
        15.0, 1.0, -3.0,      // coin 3
        // ... more coins would be added here
    ];

    // Multiple beasts with different types
    let beasts_data = array![
        "goblin_1",            // beast_id
        2,                     // beast_type (2 = Goblin)
        10.0, 1.0, 15.0,      // x, y, z
        80,                    // health
        20,                    // damage
        4.0,                   // speed
        
        "orc_1",               // beast_id
        3,                     // beast_type (3 = Orc)
        -15.0, 1.0, -20.0,    // x, y, z
        150,                   // health
        35,                    // damage
        2.5,                   // speed
        
        "dragon_1",            // beast_id
        1,                     // beast_type (1 = Dragon)
        50.0, 5.0, 30.0,      // x, y, z
        300,                   // health
        60,                    // damage
        1.5                    // speed
    ];

    // Multiple objectives with different types
    let objectives_data = array![
        "collect_coins",       // objective_id
        "Collect Ancient Coins", // title
        "Collect 10 coins to unlock the path forward", // description
        0,                     // objective_type (0 = collect)
        "coins",               // target
        10,                    // required_count
        "unlock_level_3",      // reward
        
        "defeat_goblins",      // objective_id
        "Defeat Goblins",      // title
        "Defeat 3 goblins to weaken the enemy forces", // description
        2,                     // objective_type (2 = defeat)
        "goblin",              // target
        3,                     // required_count
        "goblin_essence",      // reward
        
        "reach_dragon_lair",   // objective_id
        "Reach Dragon Lair",   // title
        "Find the entrance to the dragon's lair", // description
        1,                     // objective_type (1 = reach_location)
        "dragon_lair",         // target
        0,                     // required_count
        "dragon_map"           // reward
    ];

    let environment_data = array![
        10.0,                  // dungeon_scale
        0.0, -2.0, 0.0,       // dungeon_x, dungeon_y, dungeon_z
        -2.0                   // dungeon_rotation
    ];

    // Call admin_create_level with complex data
    // let level_id = contract.admin_create_level(
    //     "The Goblin Stronghold", // level_name
    //     PlayerType::Beast,       // player_type
    //     4,                       // next_level
    //     coins_data,              // coins_data
    //     beasts_data,             // beasts_data
    //     objectives_data,         // objectives_data
    //     environment_data         // environment_data
    // );
}

// Example: Modifying an existing level
fn modify_level_example() {
    // Get existing level data and modify it
    let modified_coins_data = array![
        20,                    // increased spawn_count
        10.0, 1.0, 5.0,       // coin 1
        -8.0, 1.0, 12.0,      // coin 2
        // ... modified coin positions
    ];

    let modified_beasts_data = array![
        "monster_1",           // beast_id
        0,                     // beast_type (0 = Monster)
        0.0, 1.0, 0.0,        // x, y, z
        120,                   // increased health
        30,                    // increased damage
        3.5                    // increased speed
    ];

    let modified_objectives_data = array![
        "collect_coins",       // objective_id
        "Collect Ancient Coins", // title
        "Collect 8 coins to unlock the path forward", // description
        0,                     // objective_type (0 = collect)
        "coins",               // target
        8,                     // modified required_count
        "unlock_level_2"       // reward
    ];

    let modified_environment_data = array![
        8.0,                   // modified dungeon_scale
        0.0, -1.5, 0.0,       // dungeon_x, dungeon_y, dungeon_z
        -1.5708                // dungeon_rotation
    ];

    // Call admin_modify_level
    // contract.admin_modify_level(
    //     1,                    // level_id
    //     "The Beginning v2",   // updated level_name
    //     PlayerType::Man,      // player_type
    //     2,                    // next_level
    //     modified_coins_data,  // coins_data
    //     modified_beasts_data, // beasts_data
    //     modified_objectives_data, // objectives_data
    //     modified_environment_data  // environment_data
    // );
}

// Helper function to convert string to felt252 (for beast_id and objective_id)
fn string_to_felt252(input: Array<u8>) -> felt252 {
    // This is a simplified conversion - in practice you'd use proper string handling
    let mut result = 0;
    let mut i = 0;
    loop {
        if i >= input.len() {
            break;
        }
        result = result * 256 + input.at(i).into();
        i += 1;
    };
    result
}

// Helper function to create beast data from individual components
fn create_beast_data(
    beast_id: felt252,
    beast_type: BeastType,
    x: felt252,
    y: felt252,
    z: felt252,
    health: u32,
    damage: u32,
    speed: felt252
) -> Array<felt252> {
    array![
        beast_id,
        beast_type.into(),
        x,
        y,
        z,
        health.into(),
        damage.into(),
        speed
    ]
}

// Helper function to create objective data from individual components
fn create_objective_data(
    objective_id: felt252,
    title: felt252,
    description: felt252,
    objective_type: ObjectiveType,
    target: felt252,
    required_count: u32,
    reward: felt252
) -> Array<felt252> {
    array![
        objective_id,
        title,
        description,
        objective_type.into(),
        target,
        required_count.into(),
        reward
    ]
}
