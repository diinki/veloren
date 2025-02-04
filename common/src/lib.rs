#![deny(unsafe_code)]
#![allow(incomplete_features)]
#![type_length_limit = "1664759"]
#![allow(clippy::option_map_unit_fn)]
#![deny(clippy::clone_on_ref_ptr)]
#![feature(
    arbitrary_enum_discriminant,
    associated_type_defaults,
    bool_to_option,
    const_checked_int_methods,
    const_generics,
    fundamental,
    iter_map_while,
    label_break_value,
    option_expect_none,
    option_unwrap_none,
    option_zip,
    trait_alias,
    type_alias_impl_trait
)]

/// Re-exported crates
pub use uuid;

// modules
#[cfg(not(target_arch = "wasm32"))]
pub mod assets;
#[cfg(not(target_arch = "wasm32"))] pub mod astar;
#[cfg(not(target_arch = "wasm32"))]
pub mod character;
#[cfg(not(target_arch = "wasm32"))] pub mod clock;
#[cfg(not(target_arch = "wasm32"))] pub mod cmd;
pub mod combat;
pub mod comp;
#[cfg(not(target_arch = "wasm32"))]
pub mod consts;
#[cfg(not(target_arch = "wasm32"))]
pub mod effect;
#[cfg(not(target_arch = "wasm32"))] pub mod event;
#[cfg(not(target_arch = "wasm32"))]
pub mod explosion;
#[cfg(not(target_arch = "wasm32"))]
pub mod figure;
#[cfg(not(target_arch = "wasm32"))]
pub mod generation;
#[cfg(not(target_arch = "wasm32"))] pub mod grid;
#[cfg(not(target_arch = "wasm32"))]
pub mod lottery;
#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
pub mod npc;
#[cfg(not(target_arch = "wasm32"))]
pub mod outcome;
#[cfg(not(target_arch = "wasm32"))] pub mod path;
#[cfg(not(target_arch = "wasm32"))] pub mod ray;
#[cfg(not(target_arch = "wasm32"))]
pub mod recipe;
#[cfg(not(target_arch = "wasm32"))]
pub mod region;
pub mod resources;
#[cfg(not(target_arch = "wasm32"))] pub mod rtsim;
#[cfg(not(target_arch = "wasm32"))]
pub mod skillset_builder;
pub mod slowjob;
#[cfg(not(target_arch = "wasm32"))]
pub mod spiral;
#[cfg(not(target_arch = "wasm32"))]
pub mod states;
#[cfg(not(target_arch = "wasm32"))] pub mod store;
#[cfg(not(target_arch = "wasm32"))]
pub mod terrain;
#[cfg(not(target_arch = "wasm32"))] pub mod time;
#[cfg(not(target_arch = "wasm32"))] pub mod trade;
#[cfg(not(target_arch = "wasm32"))] pub mod typed;
pub mod uid;
#[cfg(not(target_arch = "wasm32"))] pub mod util;
#[cfg(not(target_arch = "wasm32"))] pub mod vol;
#[cfg(not(target_arch = "wasm32"))]
pub mod volumes;

pub use combat::DamageSource;
#[cfg(not(target_arch = "wasm32"))]
pub use combat::{Damage, GroupTarget, Knockback, KnockbackDir};
#[cfg(not(target_arch = "wasm32"))]
pub use comp::inventory::loadout_builder::LoadoutBuilder;
#[cfg(not(target_arch = "wasm32"))]
pub use explosion::{Explosion, RadiusEffect};
#[cfg(not(target_arch = "wasm32"))]
pub use skillset_builder::SkillSetBuilder;
