use bevy::prelude::*;

// ===== COLOR PALETTE =====

pub trait ElysiumDescentColorPalette {
    const ELYSIUM_DESCENT_RED: Color;
    const ELYSIUM_DESCENT_RED_DIM: Color;
    const ELYSIUM_DESCENT_YELLOW: Color;
    const ELYSIUM_DESCENT_BLUE: Color;
}

impl ElysiumDescentColorPalette for Color {
    const ELYSIUM_DESCENT_RED: Color = Color::srgba(223. / 255., 170. / 255., 45. / 255., 1.0);
    const ELYSIUM_DESCENT_RED_DIM: Color = Color::srgba(223. / 255., 170. / 255., 45. / 255., 1.0);
    const ELYSIUM_DESCENT_YELLOW: Color = Color::srgba(223. / 255., 170. / 255., 45. / 255., 1.0);
    const ELYSIUM_DESCENT_BLUE: Color = Color::srgba(223. / 255., 170. / 255., 45. / 255., 1.0);
}
