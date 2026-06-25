use super::state::{Modifier, RoomKind, RunState};
use rand::{Rng, SeedableRng, rngs::StdRng};

pub fn room_kind_for(state: &RunState) -> RoomKind {
    if state.room == state.rooms_per_floor {
        RoomKind::Boss
    } else if state.room == 2 {
        let mut rng = StdRng::seed_from_u64(state.room_seed ^ state.floor as u64);
        if rng.gen_bool(0.55) {
            RoomKind::Rest
        } else {
            RoomKind::Treasure
        }
    } else {
        RoomKind::Combat
    }
}

pub fn advance_room(state: &mut RunState) {
    // Tick down modifier durations
    for (_, remaining) in &mut state.active_modifiers {
        if *remaining > 0 {
            *remaining -= 1;
        }
    }

    state.room += 1;
    if state.room > state.rooms_per_floor {
        state.room = 1;
        state.floor += 1;
        state.room_seed = state.room_seed.wrapping_add(1);
    }
    state.current_room = room_kind_for(state);

    // Small chance to gain a random modifier on combat rooms
    if state.current_room == RoomKind::Combat {
        let mut rng = StdRng::seed_from_u64(state.room_seed ^ state.floor as u64 ^ state.room as u64);
        if rng.gen_bool(0.2) && state.active_modifiers.len() < 3 {
            let modifiers = [Modifier::GlassCannon, Modifier::SpeedDemon, Modifier::Vampire, Modifier::TreasureHunter];
            let pick = modifiers[rng.gen_range(0..modifiers.len())];
            if !state.has_modifier(pick) {
                state.active_modifiers.push((pick, 3)); // 3 room duration
            }
        }
    }
}
