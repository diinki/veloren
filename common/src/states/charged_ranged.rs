use crate::{
    combat::{
        Attack, AttackDamage, AttackEffect, CombatBuff, CombatEffect, CombatRequirement, Damage,
        DamageSource, GroupTarget, Knockback, KnockbackDir,
    },
    comp::{
        projectile, Body, CharacterState, EnergyChange, EnergySource, Gravity, LightEmitter,
        Projectile, StateUpdate,
    },
    event::ServerEvent,
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
    /// How long the weapon needs to be prepared for
    pub buildup_duration: Duration,
    /// How long it takes to charge the weapon to max damage and knockback
    pub charge_duration: Duration,
    /// How long the state has until exiting
    pub recover_duration: Duration,
    /// How much energy is drained per second when charging
    pub energy_drain: f32,
    /// How much damage is dealt with no charge
    pub initial_damage: f32,
    /// How much the damage scales as it is charged
    pub scaled_damage: f32,
    /// How much knockback there is with no charge
    pub initial_knockback: f32,
    /// How much the knockback scales as it is charged
    pub scaled_knockback: f32,
    /// Speed stat of the weapon
    pub speed: f32,
    /// Projectile information
    pub projectile_body: Body,
    pub projectile_light: Option<LightEmitter>,
    pub projectile_gravity: Option<Gravity>,
    pub initial_projectile_speed: f32,
    pub scaled_projectile_speed: f32,
    /// Move speed efficiency
    pub move_speed: f32,
    /// What key is used to press ability
    pub ability_info: AbilityInfo,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    /// Struct containing data that does not change over the course of the
    /// character state
    pub static_data: StaticData,
    /// Timer for each stage
    pub timer: Duration,
    /// What section the character stage is in
    pub stage_section: StageSection,
    /// Whether the attack fired already
    pub exhausted: bool,
}

impl CharacterBehavior for Data {
    fn behavior(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);

        handle_move(data, &mut update, self.static_data.move_speed);
        handle_jump(data, &mut update);

        match self.stage_section {
            StageSection::Buildup => {
                if self.timer < self.static_data.buildup_duration {
                    // Build up
                    update.character = CharacterState::ChargedRanged(Data {
                        timer: self
                            .timer
                            .checked_add(Duration::from_secs_f32(data.dt.0))
                            .unwrap_or_default(),
                        ..*self
                    });
                } else {
                    // Transitions to swing section of stage
                    update.character = CharacterState::ChargedRanged(Data {
                        timer: Duration::default(),
                        stage_section: StageSection::Charge,
                        ..*self
                    });
                }
            },
            StageSection::Charge => {
                if !input_is_pressed(data, self.static_data.ability_info.input) && !self.exhausted {
                    let charge_frac = (self.timer.as_secs_f32()
                        / self.static_data.charge_duration.as_secs_f32())
                    .min(1.0);
                    let knockback = AttackEffect::new(
                        Some(GroupTarget::OutOfGroup),
                        CombatEffect::Knockback(Knockback {
                            strength: self.static_data.initial_knockback
                                + charge_frac * self.static_data.scaled_knockback,
                            direction: KnockbackDir::Away,
                        }),
                    )
                    .with_requirement(CombatRequirement::AnyDamage);
                    let buff = CombatEffect::Buff(CombatBuff::default_physical());
                    let damage = AttackDamage::new(
                        Damage {
                            source: DamageSource::Projectile,
                            value: self.static_data.initial_damage as f32
                                + charge_frac * self.static_data.scaled_damage as f32,
                        },
                        Some(GroupTarget::OutOfGroup),
                    )
                    .with_effect(buff);
                    let (crit_chance, crit_mult) =
                        get_crit_data(data, self.static_data.ability_info);
                    let attack = Attack::default()
                        .with_damage(damage)
                        .with_crit(crit_chance, crit_mult)
                        .with_effect(knockback)
                        .with_combo_increment();

                    // Fire
                    let projectile = Projectile {
                        hit_solid: vec![projectile::Effect::Stick],
                        hit_entity: vec![
                            projectile::Effect::Attack(attack),
                            projectile::Effect::Vanish,
                        ],
                        time_left: Duration::from_secs(15),
                        owner: Some(*data.uid),
                        ignore_group: true,
                    };
                    update.server_events.push_front(ServerEvent::Shoot {
                        entity: data.entity,
                        dir: data.inputs.look_dir,
                        body: self.static_data.projectile_body,
                        projectile,
                        light: self.static_data.projectile_light,
                        gravity: self.static_data.projectile_gravity,
                        speed: self.static_data.initial_projectile_speed
                            + charge_frac * self.static_data.scaled_projectile_speed,
                        object: None,
                    });

                    update.character = CharacterState::ChargedRanged(Data {
                        timer: Duration::default(),
                        stage_section: StageSection::Recover,
                        exhausted: true,
                        ..*self
                    });
                } else if self.timer < self.static_data.charge_duration
                    && input_is_pressed(data, self.static_data.ability_info.input)
                {
                    // Charges
                    update.character = CharacterState::ChargedRanged(Data {
                        timer: self
                            .timer
                            .checked_add(Duration::from_secs_f32(
                                data.dt.0 * self.static_data.speed,
                            ))
                            .unwrap_or_default(),
                        ..*self
                    });

                    // Consumes energy if there's enough left and RMB is held down
                    update.energy.change_by(EnergyChange {
                        amount: -(self.static_data.energy_drain as f32
                            * data.dt.0
                            * self.static_data.speed) as i32,
                        source: EnergySource::Ability,
                    });
                } else if input_is_pressed(data, self.static_data.ability_info.input) {
                    // Holds charge
                    update.character = CharacterState::ChargedRanged(Data {
                        timer: self
                            .timer
                            .checked_add(Duration::from_secs_f32(
                                data.dt.0 * self.static_data.speed,
                            ))
                            .unwrap_or_default(),
                        ..*self
                    });

                    // Consumes energy if there's enough left and RMB is held down
                    update.energy.change_by(EnergyChange {
                        amount: -(self.static_data.energy_drain as f32
                            * data.dt.0
                            * self.static_data.speed
                            / 5.0) as i32,
                        source: EnergySource::Ability,
                    });
                }
            },
            StageSection::Recover => {
                if self.timer < self.static_data.recover_duration {
                    // Recovers
                    update.character = CharacterState::ChargedRanged(Data {
                        timer: self
                            .timer
                            .checked_add(Duration::from_secs_f32(data.dt.0))
                            .unwrap_or_default(),
                        ..*self
                    });
                } else {
                    // Done
                    update.character = CharacterState::Wielding;
                }
            },
            _ => {
                // If it somehow ends up in an incorrect stage section
                update.character = CharacterState::Wielding;
            },
        }

        // At end of state logic so an interrupt isn't overwritten
        if !input_is_pressed(data, self.static_data.ability_info.input) {
            handle_state_interrupt(data, &mut update, false);
        }

        update
    }
}
