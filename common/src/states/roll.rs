use crate::{
    comp::{CharacterState, InputKind, StateUpdate},
    states::{
        behavior::{CharacterBehavior, JoinData},
        utils::*,
    },
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Separated out to condense update portions of character state
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StaticData {
    /// How long until state should roll
    pub buildup_duration: Duration,
    /// How long state is rolling for
    pub movement_duration: Duration,
    /// How long it takes to recover from roll
    pub recover_duration: Duration,
    /// Affects the speed and distance of the roll
    pub roll_strength: f32,
    /// Affects whether you are immune to melee attacks while rolling
    pub immune_melee: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    /// Struct containing data that does not change over the course of the
    /// character state
    pub static_data: StaticData,
    /// Timer for each stage
    pub timer: Duration,
    /// What section the character stage is in
    pub stage_section: StageSection,
    /// Had weapon
    pub was_wielded: bool,
    /// Was sneaking
    pub was_sneak: bool,
    /// Was in state with combo
    pub was_combo: Option<(InputKind, u32)>,
}

impl CharacterBehavior for Data {
    fn behavior(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);

        // Smooth orientation
        handle_orientation(data, &mut update, 1.0);

        match self.stage_section {
            StageSection::Buildup => {
                handle_move(data, &mut update, 1.0);
                if self.timer < self.static_data.buildup_duration {
                    // Build up
                    update.character = CharacterState::Roll(Data {
                        timer: self
                            .timer
                            .checked_add(Duration::from_secs_f32(data.dt.0))
                            .unwrap_or_default(),
                        ..*self
                    });
                } else {
                    // Transitions to movement section of stage
                    update.character = CharacterState::Roll(Data {
                        timer: Duration::default(),
                        stage_section: StageSection::Movement,
                        ..*self
                    });
                }
            },
            StageSection::Movement => {
                // Update velocity
                handle_forced_movement(
                    data,
                    &mut update,
                    ForcedMovement::Forward {
                        strength: self.static_data.roll_strength
                            * ((1.0
                                - self.timer.as_secs_f32()
                                    / self.static_data.movement_duration.as_secs_f32())
                                / 2.0
                                + 0.5),
                    },
                    0.0,
                );

                if self.timer < self.static_data.movement_duration {
                    // Movement
                    update.character = CharacterState::Roll(Data {
                        timer: self
                            .timer
                            .checked_add(Duration::from_secs_f32(data.dt.0))
                            .unwrap_or_default(),
                        ..*self
                    });
                } else {
                    // Transitions to recover section of stage
                    update.character = CharacterState::Roll(Data {
                        timer: Duration::default(),
                        stage_section: StageSection::Recover,
                        ..*self
                    });
                }
            },
            StageSection::Recover => {
                if self.timer < self.static_data.recover_duration {
                    // Build up
                    update.character = CharacterState::Roll(Data {
                        timer: self
                            .timer
                            .checked_add(Duration::from_secs_f32(data.dt.0))
                            .unwrap_or_default(),
                        ..*self
                    });
                } else {
                    // Done
                    if let Some((input, stage)) = self.was_combo {
                        if input_is_pressed(data, input) {
                            handle_input(data, &mut update, input);
                            // If other states are introduced that progress through stages, add them
                            // here
                            if let CharacterState::ComboMelee(c) = &mut update.character {
                                c.stage = stage;
                            }
                        } else {
                            update.character = CharacterState::Wielding;
                        }
                    } else if self.was_wielded {
                        update.character = CharacterState::Wielding;
                    } else if self.was_sneak {
                        update.character = CharacterState::Sneak;
                    } else {
                        update.character = CharacterState::Idle;
                    }
                }
            },
            _ => {
                // If it somehow ends up in an incorrect stage section
                update.character = CharacterState::Idle;
            },
        }

        update
    }
}
