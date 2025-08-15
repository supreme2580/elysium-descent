#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::level_manager::{LevelManager, LevelData, CoinSpawnData, Position3D, BeastData, LevelObjectiveData, EnvironmentData};

    #[test]
    fn test_level_manager_creation() {
        let mut level_manager = LevelManager::default();
        assert_eq!(level_manager.current_level, None);
        assert_eq!(level_manager.levels.len(), 0);
        assert_eq!(level_manager.level_completed, false);
    }

    #[test]
    fn test_level_data_structures() {
        let position = Position3D { x: 10.0, y: 1.0, z: 5.0 };
        assert_eq!(position.x, 10.0);
        assert_eq!(position.y, 1.0);
        assert_eq!(position.z, 5.0);

        let coin_data = CoinSpawnData {
            spawn_count: 15,
            spawn_positions: vec![position],
        };
        assert_eq!(coin_data.spawn_count, 15);
        assert_eq!(coin_data.spawn_positions.len(), 1);

        let beast = BeastData {
            id: "monster_1".to_string(),
            beast_type: "Monster".to_string(),
            spawn_position: position,
            health: 100,
            damage: 25,
            speed: 3.0,
        };
        assert_eq!(beast.health, 100);
        assert_eq!(beast.damage, 25);

        let objective = LevelObjectiveData {
            id: "collect_coins".to_string(),
            title: "Collect Ancient Coins".to_string(),
            description: "Collect 5 coins to unlock the path forward".to_string(),
            objective_type: "collect".to_string(),
            target: "coins".to_string(),
            required_count: Some(5),
            position: None,
            completion_radius: None,
            reward: "unlock_level_2".to_string(),
        };
        assert_eq!(objective.required_count, Some(5));
        assert_eq!(objective.target, "coins");

        let environment = EnvironmentData {
            dungeon_scale: 7.5,
            dungeon_position: position,
            dungeon_rotation: -1.5708,
        };
        assert_eq!(environment.dungeon_scale, 7.5);

        let level_data = LevelData {
            level_id: 1,
            level_name: "The Beginning".to_string(),
            player_type: "Man".to_string(),
            coins: coin_data,
            beasts: vec![beast],
            objectives: vec![objective],
            environment,
            next_level: Some(2),
        };
        assert_eq!(level_data.level_id, 1);
        assert_eq!(level_data.next_level, Some(2));
    }

    #[test]
    fn test_level_manager_operations() {
        let mut level_manager = LevelManager::default();
        
        // Test loading a level
        let position = Position3D { x: 0.0, y: 0.0, z: 0.0 };
        let coin_data = CoinSpawnData {
            spawn_count: 5,
            spawn_positions: vec![position],
        };
        let beast = BeastData {
            id: "test_monster".to_string(),
            beast_type: "Test".to_string(),
            spawn_position: position,
            health: 50,
            damage: 10,
            speed: 2.0,
        };
        let objective = LevelObjectiveData {
            id: "test_obj".to_string(),
            title: "Test Objective".to_string(),
            description: "Test Description".to_string(),
            objective_type: "collect".to_string(),
            target: "coins".to_string(),
            required_count: Some(3),
            position: None,
            completion_radius: None,
            reward: "test_reward".to_string(),
        };
        let environment = EnvironmentData {
            dungeon_scale: 1.0,
            dungeon_position: position,
            dungeon_rotation: 0.0,
        };
        
        let test_level = LevelData {
            level_id: 999,
            level_name: "Test Level".to_string(),
            player_type: "Test".to_string(),
            coins: coin_data,
            beasts: vec![beast],
            objectives: vec![objective],
            environment,
            next_level: None,
        };
        
        level_manager.levels.insert(999, test_level);
        
        // Test loading the level
        let loaded_level = level_manager.load_level(999);
        assert!(loaded_level.is_some());
        assert_eq!(level_manager.current_level, Some(999));
        assert_eq!(level_manager.level_completed, false);
        
        // Test getting current level
        let current_level = level_manager.get_current_level();
        assert!(current_level.is_some());
        assert_eq!(current_level.unwrap().level_id, 999);
        
        // Test marking level as completed
        level_manager.mark_level_completed();
        assert_eq!(level_manager.is_level_completed(), true);
    }
}
