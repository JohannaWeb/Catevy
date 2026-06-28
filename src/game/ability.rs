use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Active skills the player triggers with a key. Dash is the starter; the rest
/// are found in rooms and slotted onto Q / F.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ability {
    Dash,
    Whirlwind,
    WarCry,
    Hairball,
    SecondWind,
    Zoomies, // Cat: 3x speed for 2s, afterimage trail
    Purr,    // Cat: Charm nearby enemies 3s
    CatNap,  // Cat: Heal 2 HP over 3s, immobile
}

impl Ability {
    pub fn name(self) -> &'static str {
        match self {
            Ability::Dash => "Dash",
            Ability::Whirlwind => "Whirlwind",
            Ability::WarCry => "War Cry",
            Ability::Hairball => "Hairball",
            Ability::SecondWind => "Second Wind",
            Ability::Zoomies => "Zoomies",
            Ability::Purr => "Purr",
            Ability::CatNap => "Cat Nap",
        }
    }

    pub fn cooldown(self) -> f32 {
        match self {
            Ability::Dash => 1.1,
            Ability::Whirlwind => 3.0,
            Ability::WarCry => 5.0,
            Ability::Hairball => 2.2,
            Ability::SecondWind => 14.0,
            Ability::Zoomies => 12.0,
            Ability::Purr => 8.0,
            Ability::CatNap => 20.0,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Ability::Dash => Color::srgb(0.5, 0.9, 1.0),
            Ability::Whirlwind => Color::srgb(0.85, 0.95, 1.0),
            Ability::WarCry => Color::srgb(1.0, 0.65, 0.3),
            Ability::Hairball => Color::srgb(0.85, 0.7, 0.5),
            Ability::SecondWind => Color::srgb(0.5, 0.95, 0.6),
            Ability::Zoomies => Color::srgb(0.9, 0.6, 1.0),
            Ability::Purr => Color::srgb(1.0, 0.7, 0.8),
            Ability::CatNap => Color::srgb(0.6, 0.8, 1.0),
        }
    }
}

/// One equipped ability plus its cooldown timer.
pub struct AbilitySlot {
    pub ability: Ability,
    pub cd: Timer,
}

impl AbilitySlot {
    pub fn new(ability: Ability) -> Self {
        let mut cd = Timer::from_seconds(ability.cooldown(), TimerMode::Once);
        let dur = cd.duration();
        cd.tick(dur); // start ready
        Self { ability, cd }
    }
}

/// Key bound to ability slot 0/1/2.
pub const ABILITY_KEYS: [KeyCode; 3] = [KeyCode::Space, KeyCode::KeyQ, KeyCode::KeyF];
pub const ABILITY_KEY_LABELS: [&str; 3] = ["Space", "Q", "F"];

/// Abilities that can drop in rooms (Dash is granted at the start instead).
pub const FOUND_ABILITIES: [Ability; 7] = [
    Ability::Whirlwind,
    Ability::WarCry,
    Ability::Hairball,
    Ability::SecondWind,
    Ability::Zoomies,
    Ability::Purr,
    Ability::CatNap,
];
