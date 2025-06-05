#[derive(Clone, Drop, Serde)]
#[dojo::model]
struct Survivor {
    #[key]
    entity_id: felt252,
    owner: felt252,
    health: u32,
    max_health: u32,
    exhaustion_level: u32,
    max_exhaustion: u32,
    is_alive: bool,
}

#[derive(Clone, Drop, Serde)]
#[dojo::model]
struct SurvivorStats {
    #[key]
    entity_id: felt252,
    agility: u32,
    gathering_efficiency: u32,
}
