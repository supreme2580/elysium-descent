use crate::systems::collectibles::{CollectibleConfig, CollectibleType, CollectibleRotation};
use bevy::math::Vec3;
use once_cell::sync::Lazy;

pub static COLLECTIBLES: Lazy<Vec<CollectibleConfig>> = Lazy::new(|| vec![
    CollectibleConfig {
        position: Vec3::new(10.0, 2.0, 60.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: true, speed: 2.0 }),
    },
    CollectibleConfig {
        position: Vec3::new(25.0, 2.0, 60.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: false, speed: 1.5 }),
    },
    CollectibleConfig {
        position: Vec3::new(40.0, 2.0, 60.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: true, speed: 2.5 }),
    },
    CollectibleConfig {
        position: Vec3::new(55.0, 2.0, 60.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: false, speed: 1.0 }),
    },
    CollectibleConfig {
        position: Vec3::new(58.5, 5.0, 50.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: true, speed: 1.8 }),
    },
    CollectibleConfig {
        position: Vec3::new(60.0, 8.0, 48.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: false, speed: 2.2 }),
    },
    CollectibleConfig {
        position: Vec3::new(60.0, 12.0, 42.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: true, speed: 1.3 }),
    },
    CollectibleConfig {
        position: Vec3::new(60.0, 12.0, 32.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: false, speed: 2.7 }),
    },
    CollectibleConfig {
        position: Vec3::new(60.0, 12.0, 22.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: true, speed: 2.1 }),
    },
    CollectibleConfig {
        position: Vec3::new(75.0, 12.0, 22.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: false, speed: 1.9 }),
    },
    CollectibleConfig {
        position: Vec3::new(90.0, 12.0, 22.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: true, speed: 2.4 }),
    },
    CollectibleConfig {
        position: Vec3::new(90.0, 12.0, 12.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: false, speed: 1.6 }),
    },
    CollectibleConfig {
        position: Vec3::new(90.0, 12.0, 2.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: true, speed: 2.3 }),
    },
    CollectibleConfig {
        position: Vec3::new(90.0, 12.0, -10.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: false, speed: 1.2 }),
    },
    CollectibleConfig {
        position: Vec3::new(90.0, 12.0, -22.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: true, speed: 2.6 }),
    },
    CollectibleConfig {
        position: Vec3::new(90.0, 12.0, -34.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: false, speed: 1.7 }),
    },
    CollectibleConfig {
        position: Vec3::new(90.0, 15.0, -40.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: true, speed: 2.8 }),
    },
    CollectibleConfig {
        position: Vec3::new(90.0, 18.0, -46.0),
        collectible_type: CollectibleType::Coin,
        scale: 0.7,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: false, speed: 1.4 }),
    },
    CollectibleConfig {
        position: Vec3::new(90.0, 22.0, -54.0),
        collectible_type: CollectibleType::Book,
        scale: 1.0,
        rotation: Some(CollectibleRotation { enabled: true, clockwise: true, speed: 2.0 }),
    },
]); 