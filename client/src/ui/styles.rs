use bevy::prelude::*;

// ===== COLOR PALETTE =====

pub trait ElysiumDescentColorPalette {
    // Primary Brand Colors
    const ELYSIUM_GOLD: Color;
    const ELYSIUM_GOLD_DIM: Color;
    const ELYSIUM_BLUE: Color;
    const ELYSIUM_PURPLE: Color;
    
    // Health & Status Colors
    const HEALTH_GREEN: Color;
    const HEALTH_GREEN_DARK: Color;
    const XP_PURPLE: Color;
    const XP_PURPLE_DARK: Color;
    const ENERGY_BLUE: Color;
    const ENERGY_BLUE_DARK: Color;
    
    // UI Background Colors
    const DARK_GLASS: Color;
    const DARKER_GLASS: Color;
    const LIGHT_GLASS: Color;
    
    // Accent Colors  
    const WARNING_ORANGE: Color;
    const DANGER_RED: Color;
    const SUCCESS_GREEN: Color;
    
    // Legacy compatibility
    const ELYSIUM_DESCENT_RED: Color;
    const ELYSIUM_DESCENT_RED_DIM: Color;
    const ELYSIUM_DESCENT_YELLOW: Color;
    const ELYSIUM_DESCENT_BLUE: Color;
}

impl ElysiumDescentColorPalette for Color {
    // Primary Brand Colors - Golden theme inspired by "Elysium"
    const ELYSIUM_GOLD: Color = Color::srgba(0.875, 0.667, 0.176, 1.0); // Rich gold
    const ELYSIUM_GOLD_DIM: Color = Color::srgba(0.647, 0.490, 0.129, 1.0); // Dimmed gold
    const ELYSIUM_BLUE: Color = Color::srgba(0.118, 0.565, 0.776, 1.0); // Deep sky blue
    const ELYSIUM_PURPLE: Color = Color::srgba(0.467, 0.278, 0.678, 1.0); // Royal purple
    
    // Health & Status Colors - Modern game UI palette
    const HEALTH_GREEN: Color = Color::srgba(0.235, 0.757, 0.353, 1.0); // Vibrant health green
    const HEALTH_GREEN_DARK: Color = Color::srgba(0.165, 0.529, 0.247, 1.0); // Dark health green
    const XP_PURPLE: Color = Color::srgba(0.576, 0.259, 0.816, 1.0); // Experience purple
    const XP_PURPLE_DARK: Color = Color::srgba(0.403, 0.182, 0.571, 1.0); // Dark XP purple
    const ENERGY_BLUE: Color = Color::srgba(0.118, 0.565, 0.776, 1.0); // Energy/mana blue
    const ENERGY_BLUE_DARK: Color = Color::srgba(0.082, 0.396, 0.543, 1.0); // Dark energy blue
    
    // UI Background Colors - Modern glassmorphism
    const DARK_GLASS: Color = Color::srgba(0.08, 0.12, 0.18, 0.85); // Dark translucent
    const DARKER_GLASS: Color = Color::srgba(0.05, 0.08, 0.12, 0.9); // Darker translucent
    const LIGHT_GLASS: Color = Color::srgba(0.15, 0.18, 0.25, 0.75); // Light translucent
    
    // Accent Colors
    const WARNING_ORANGE: Color = Color::srgba(1.0, 0.596, 0.0, 1.0); // Orange warning
    const DANGER_RED: Color = Color::srgba(0.925, 0.267, 0.267, 1.0); // Danger red
    const SUCCESS_GREEN: Color = Color::srgba(0.196, 0.804, 0.196, 1.0); // Success green
    
    // Legacy compatibility - using the gold as primary
    const ELYSIUM_DESCENT_RED: Color = Self::ELYSIUM_GOLD;
    const ELYSIUM_DESCENT_RED_DIM: Color = Self::ELYSIUM_GOLD_DIM;
    const ELYSIUM_DESCENT_YELLOW: Color = Self::ELYSIUM_GOLD;
    const ELYSIUM_DESCENT_BLUE: Color = Self::ELYSIUM_BLUE;
}
