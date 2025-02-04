use crate::{
    character::CharacterId,
    comp,
    rtsim::RtSimEntity,
    trade::{TradeAction, TradeId},
    uid::Uid,
    util::Dir,
    Explosion,
};
use comp::{
    invite::{InviteKind, InviteResponse},
    item::Item,
    Ori, Pos,
};
use specs::Entity as EcsEntity;
use std::{collections::VecDeque, ops::DerefMut, sync::Mutex};
use vek::*;

pub type SiteId = u64;

pub enum LocalEvent {
    /// Applies upward force to entity's `Vel`
    Jump(EcsEntity, f32),
    /// Applies the `impulse` to `entity`'s `Vel`
    ApplyImpulse {
        entity: EcsEntity,
        impulse: Vec3<f32>,
    },
    /// Applies `vel` velocity to `entity`
    Boost { entity: EcsEntity, vel: Vec3<f32> },
}

#[allow(clippy::large_enum_variant)] // TODO: Pending review in #587
pub enum ServerEvent {
    Explosion {
        pos: Vec3<f32>,
        explosion: Explosion,
        owner: Option<Uid>,
    },
    Damage {
        entity: EcsEntity,
        change: comp::HealthChange,
    },
    PoiseChange {
        entity: EcsEntity,
        change: comp::PoiseChange,
        kb_dir: Vec3<f32>,
    },
    Delete(EcsEntity),
    Destroy {
        entity: EcsEntity,
        cause: comp::HealthSource,
    },
    InventoryManip(EcsEntity, comp::InventoryManip),
    GroupManip(EcsEntity, comp::GroupManip),
    Respawn(EcsEntity),
    Shoot {
        entity: EcsEntity,
        dir: Dir,
        body: comp::Body,
        light: Option<comp::LightEmitter>,
        projectile: comp::Projectile,
        gravity: Option<comp::Gravity>,
        speed: f32,
        object: Option<comp::Object>,
    },
    Shockwave {
        properties: comp::shockwave::Properties,
        pos: Pos,
        ori: Ori,
    },
    Knockback {
        entity: EcsEntity,
        impulse: Vec3<f32>,
    },
    BeamSegment {
        properties: comp::beam::Properties,
        pos: Pos,
        ori: Ori,
    },
    LandOnGround {
        entity: EcsEntity,
        vel: Vec3<f32>,
    },
    EnableLantern(EcsEntity),
    DisableLantern(EcsEntity),
    NpcInteract(EcsEntity, EcsEntity),
    InviteResponse(EcsEntity, InviteResponse),
    InitiateInvite(EcsEntity, Uid, InviteKind),
    ProcessTradeAction(EcsEntity, TradeId, TradeAction),
    Mount(EcsEntity, EcsEntity),
    Unmount(EcsEntity),
    Possess(Uid, Uid),
    /// Inserts default components for a character when loading into the game
    InitCharacterData {
        entity: EcsEntity,
        character_id: CharacterId,
    },
    UpdateCharacterData {
        entity: EcsEntity,
        components: (
            comp::Body,
            comp::Stats,
            comp::Inventory,
            Option<comp::Waypoint>,
        ),
    },
    ExitIngame {
        entity: EcsEntity,
    },
    // TODO: to avoid breakage when adding new fields, perhaps have an `NpcBuilder` type?
    CreateNpc {
        pos: comp::Pos,
        stats: comp::Stats,
        health: comp::Health,
        poise: comp::Poise,
        loadout: comp::inventory::loadout::Loadout,
        body: comp::Body,
        agent: Option<comp::Agent>,
        alignment: comp::Alignment,
        scale: comp::Scale,
        home_chunk: Option<comp::HomeChunk>,
        drop_item: Option<Item>,
        rtsim_entity: Option<RtSimEntity>,
    },
    CreateWaypoint(Vec3<f32>),
    ClientDisconnect(EcsEntity),
    ChunkRequest(EcsEntity, Vec2<i32>),
    ChatCmd(EcsEntity, String),
    /// Send a chat message to the player from an npc or other player
    Chat(comp::UnresolvedChatMsg),
    Aura {
        entity: EcsEntity,
        aura_change: comp::AuraChange,
    },
    Buff {
        entity: EcsEntity,
        buff_change: comp::BuffChange,
    },
    EnergyChange {
        entity: EcsEntity,
        change: comp::EnergyChange,
    },
    ComboChange {
        entity: EcsEntity,
        change: i32,
    },
    RequestSiteInfo {
        entity: EcsEntity,
        id: SiteId,
    },
}

pub struct EventBus<E> {
    queue: Mutex<VecDeque<E>>,
}

impl<E> Default for EventBus<E> {
    fn default() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
        }
    }
}

impl<E> EventBus<E> {
    pub fn emitter(&self) -> Emitter<E> {
        Emitter {
            bus: self,
            events: VecDeque::new(),
        }
    }

    pub fn emit_now(&self, event: E) { self.queue.lock().unwrap().push_back(event); }

    pub fn recv_all(&self) -> impl ExactSizeIterator<Item = E> {
        std::mem::replace(self.queue.lock().unwrap().deref_mut(), VecDeque::new()).into_iter()
    }
}

pub struct Emitter<'a, E> {
    bus: &'a EventBus<E>,
    events: VecDeque<E>,
}

impl<'a, E> Emitter<'a, E> {
    pub fn emit(&mut self, event: E) { self.events.push_back(event); }

    pub fn append(&mut self, other: &mut VecDeque<E>) { self.events.append(other) }
}

impl<'a, E> Drop for Emitter<'a, E> {
    fn drop(&mut self) { self.bus.queue.lock().unwrap().append(&mut self.events); }
}
