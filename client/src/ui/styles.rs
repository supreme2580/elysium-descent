use bevy::prelude::*;

// ===== COLOR PALETTE =====

pub trait ElysiumDescentColorPalette {
    const ELYSIUM_DESCENT_RED: Color;
    const ELYSIUM_DESCENT_RED_DIM: Color;
    const ELYSIUM_DESCENT_YELLOW: Color;
    const ELYSIUM_DESCENT_BLUE: Color;
}

impl ElysiumDescentColorPalette for Color {
    const ELYSIUM_DESCENT_RED: Color = Color::srgba(1., 98. / 255., 81. / 255., 1.0);
    const ELYSIUM_DESCENT_RED_DIM: Color = Color::srgba(172. / 255., 64. / 255., 63. / 255., 1.0);
    const ELYSIUM_DESCENT_YELLOW: Color = Color::srgba(252. / 255., 226. / 255., 8. / 255., 1.0);
    const ELYSIUM_DESCENT_BLUE: Color = Color::srgba(8. / 255., 226. / 255., 252. / 255., 1.0);
}
