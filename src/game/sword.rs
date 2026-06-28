use bevy::prelude::*;

/// Visual style for weapon-specific slash effects.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SlashStyle {
    Standard, // Default slash visual
    Flame,    // Fire particles along slash (Flaming Edge)
    Trail,    // Persistent trail behind slash (Wind Whisker)
    Spark,    // Impact sparks on hit (Greatpurr)
    Leaf,     // Leaf particles on swing (Nine Lives)
}

/// A melee blade. Damage is the sword's base; passive gem upgrades add on top.
#[derive(Clone, Copy)]
pub struct Sword {
    pub name: &'static str,
    pub quirk: &'static str,
    pub icon: usize,      // index into GameArt.swords
    pub damage: i32,      // base damage per hit
    pub reach: f32,       // how far the slash extends
    pub arc: f32,         // half-angle of the swing cone, radians
    pub cooldown: f32,    // seconds between swings
    pub knockback: f32,   // push applied to hit enemies
    pub lifesteal: i32,   // hp gained per kill
    pub slash_wave: bool, // also fires a ranged slash projectile
    pub color: Color,     // slash + accent tint
    pub slash_style: SlashStyle, // visual effect style
    pub crit_chance: f32, // 0.0 to 1.0, chance for critical hit (1.5x damage)
}

/// Index 0 is always the starter sword.
pub const SWORDS: [Sword; 6] = [
    Sword {
        name: "Rusty Claw",
        quirk: "A trusty starter blade.",
        icon: 0,
        damage: 2,
        reach: 110.0, // Increased from 92 for better hit detection
        arc: 1.1,     // Increased from 0.95 for wider swing
        cooldown: 0.34,
        knockback: 80.0,
        lifesteal: 0,
        slash_wave: false,
        color: Color::srgb(0.85, 0.85, 0.9),
        slash_style: SlashStyle::Standard,
        crit_chance: 0.05, // 5% crit chance
    },
    Sword {
        name: "Tuna Slicer",
        quirk: "Fast, light, flurry of cuts.",
        icon: 1,
        damage: 2,
        reach: 84.0,
        arc: 0.8,
        cooldown: 0.16,
        knockback: 45.0,
        lifesteal: 0,
        slash_wave: false,
        color: Color::srgb(0.55, 0.8, 0.95),
        slash_style: SlashStyle::Standard,
        crit_chance: 0.10, // 10% crit chance
    },
    Sword {
        name: "Greatpurr",
        quirk: "Slow, huge arc, big knockback.",
        icon: 2,
        damage: 5,
        reach: 120.0,
        arc: 1.5,
        cooldown: 0.6,
        knockback: 220.0,
        lifesteal: 0,
        slash_wave: false,
        color: Color::srgb(0.85, 0.86, 0.92),
        slash_style: SlashStyle::Spark,
        crit_chance: 0.15, // 15% crit chance
    },
    Sword {
        name: "Flaming Edge",
        quirk: "Searing strikes hit harder.",
        icon: 3,
        damage: 4,
        reach: 100.0,
        arc: 1.0,
        cooldown: 0.32,
        knockback: 110.0,
        lifesteal: 0,
        slash_wave: false,
        color: Color::srgb(1.0, 0.55, 0.25),
        slash_style: SlashStyle::Flame,
        crit_chance: 0.10, // 10% crit chance
    },
    Sword {
        name: "Nine Lives",
        quirk: "Heals 1 HP per kill.",
        icon: 4,
        damage: 3,
        reach: 96.0,
        arc: 1.0,
        cooldown: 0.3,
        knockback: 90.0,
        lifesteal: 1,
        slash_wave: false,
        color: Color::srgb(0.45, 0.95, 0.6),
        slash_style: SlashStyle::Leaf,
        crit_chance: 0.08, // 8% crit chance
    },
    Sword {
        name: "Wind Whisker",
        quirk: "Each swing flings a slash-wave.",
        icon: 5,
        damage: 3,
        reach: 100.0,
        arc: 1.0,
        cooldown: 0.36,
        knockback: 100.0,
        lifesteal: 0,
        slash_wave: true,
        color: Color::srgb(0.6, 0.95, 0.95),
        slash_style: SlashStyle::Trail,
        crit_chance: 0.12, // 12% crit chance
    },
];

impl Sword {
    pub fn starter() -> Self {
        SWORDS[0]
    }
}
