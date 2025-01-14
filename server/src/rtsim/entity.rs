use super::*;
use common::{
    comp::inventory::loadout_builder::LoadoutBuilder,
    resources::Time,
    rtsim::{Memory, MemoryItem},
    store::Id,
    terrain::TerrainGrid,
};
use world::{
    civ::{Site, Track},
    util::RandomPerm,
    IndexRef, World,
};

pub struct Entity {
    pub is_loaded: bool,
    pub pos: Vec3<f32>,
    pub seed: u32,
    pub last_tick: u64,
    pub controller: RtSimController,

    pub brain: Brain,
}

const PERM_SPECIES: u32 = 0;
const PERM_BODY: u32 = 1;
const PERM_LOADOUT: u32 = 2;
const PERM_LEVEL: u32 = 3;
const PERM_GENUS: u32 = 4;

impl Entity {
    pub fn rng(&self, perm: u32) -> impl Rng { RandomPerm::new(self.seed + perm) }

    pub fn get_body(&self) -> comp::Body {
        match self.rng(PERM_GENUS).gen::<f32>() {
            // we want 5% airships, 45% birds, 50% humans
            // TODO: uncomment this to re-enable RtSim airships once physics is interpolated well
            // in multiplayer.
            //x if x < 0.05 => comp::Body::Ship(comp::ship::Body::DefaultAirship),
            x if x < 0.50 => {
                let species = *(&comp::bird_medium::ALL_SPECIES)
                    .choose(&mut self.rng(PERM_SPECIES))
                    .unwrap();
                comp::bird_medium::Body::random_with(&mut self.rng(PERM_BODY), &species).into()
            },
            _ => {
                let species = *(&comp::humanoid::ALL_SPECIES)
                    .choose(&mut self.rng(PERM_SPECIES))
                    .unwrap();
                comp::humanoid::Body::random_with(&mut self.rng(PERM_BODY), &species).into()
            },
        }
    }

    pub fn get_name(&self) -> String {
        use common::{generation::get_npc_name, npc::NPC_NAMES};
        let npc_names = NPC_NAMES.read();
        match self.get_body() {
            comp::Body::BirdMedium(b) => {
                get_npc_name(&npc_names.bird_medium, b.species).to_string()
            },
            comp::Body::BirdSmall(_) => "Warbler".to_string(),
            comp::Body::Dragon(b) => get_npc_name(&npc_names.dragon, b.species).to_string(),
            comp::Body::Humanoid(b) => get_npc_name(&npc_names.humanoid, b.species).to_string(),
            comp::Body::Ship(_) => "Veloren Air".to_string(),
            //TODO: finish match as necessary
            _ => unimplemented!(),
        }
    }

    pub fn get_level(&self) -> u32 {
        (self.rng(PERM_LEVEL).gen::<f32>().powi(2) * 15.0).ceil() as u32
    }

    pub fn get_loadout(&self) -> comp::inventory::loadout::Loadout {
        let mut rng = self.rng(PERM_LOADOUT);
        let main_tool = comp::Item::new_from_asset_expect(
            (&[
                "common.items.weapons.sword.wood-2",
                "common.items.weapons.sword.starter",
                "common.items.weapons.sword.wood-0",
                "common.items.weapons.bow.starter",
                "common.items.weapons.bow.hardwood-2",
            ])
                .choose(&mut rng)
                .unwrap(),
        );

        let back = match rng.gen_range(0..5) {
            0 => Some(comp::Item::new_from_asset_expect(
                "common.items.armor.agile.back",
            )),
            1 => Some(comp::Item::new_from_asset_expect(
                "common.items.npc_armor.back.backpack",
            )),
            2 => Some(comp::Item::new_from_asset_expect(
                "common.items.npc_armor.back.backpack_blue",
            )),
            3 => Some(comp::Item::new_from_asset_expect(
                "common.items.npc_armor.back.leather_blue",
            )),
            _ => None,
        };

        let lantern = match rng.gen_range(0..3) {
            0 => Some(comp::Item::new_from_asset_expect(
                "common.items.lantern.black_0",
            )),
            1 => Some(comp::Item::new_from_asset_expect(
                "common.items.lantern.blue_0",
            )),
            _ => Some(comp::Item::new_from_asset_expect(
                "common.items.lantern.red_0",
            )),
        };

        let chest = Some(comp::Item::new_from_asset_expect(
            "common.items.npc_armor.chest.leather_blue",
        ));
        let pants = Some(comp::Item::new_from_asset_expect(
            "common.items.npc_armor.pants.leather_blue",
        ));
        let shoulder = Some(comp::Item::new_from_asset_expect(
            "common.items.armor.swift.shoulder",
        ));

        LoadoutBuilder::build_loadout(self.get_body(), Some(main_tool), None, None)
            .back(back)
            .lantern(lantern)
            .chest(chest)
            .pants(pants)
            .shoulder(shoulder)
            .build()
    }

    pub fn tick(&mut self, time: &Time, terrain: &TerrainGrid, world: &World, index: &IndexRef) {
        let tgt_site = self.brain.tgt.or_else(|| {
            world
                .civs()
                .sites
                .iter()
                .filter(|s| match self.get_body() {
                    comp::Body::Humanoid(_) => s.1.is_settlement() | s.1.is_castle(),
                    comp::Body::Ship(_) => s.1.is_settlement(),
                    _ => s.1.is_dungeon(),
                })
                .filter(|_| thread_rng().gen_range(0i32..4) == 0)
                .min_by_key(|(_, site)| {
                    let wpos = site.center * TerrainChunk::RECT_SIZE.map(|e| e as i32);
                    let dist = wpos.map(|e| e as f32).distance(self.pos.xy()) as u32;
                    dist + if dist < 96 { 100_000 } else { 0 }
                })
                .map(|(id, _)| id)
        });
        self.brain.tgt = tgt_site;

        tgt_site.map(|tgt_site| {
            let site = &world.civs().sites[tgt_site];

            let destination_name = site
                .site_tmp
                .map_or("".to_string(), |id| index.sites[id].name().to_string());

            let wpos = site.center * TerrainChunk::RECT_SIZE.map(|e| e as i32);
            let dist = wpos.map(|e| e as f32).distance(self.pos.xy()) as u32;

            if dist < 64 {
                self.brain.tgt = None;
            }

            let travel_to = self.pos.xy()
                + Vec3::from(
                    (wpos.map(|e| e as f32 + 0.5) - self.pos.xy())
                        .try_normalized()
                        .unwrap_or_else(Vec2::zero),
                ) * 64.0;
            let travel_to_alt = world
                .sim()
                .get_alt_approx(travel_to.map(|e| e as i32))
                .unwrap_or(0.0) as i32;
            let travel_to = terrain
                .find_space(Vec3::new(
                    travel_to.x as i32,
                    travel_to.y as i32,
                    travel_to_alt,
                ))
                .map(|e| e as f32)
                + Vec3::new(0.5, 0.5, 0.0);
            self.controller.travel_to = Some((travel_to, destination_name));
            self.controller.speed_factor = 0.70;
        });

        // Forget old memories
        self.brain
            .memories
            .retain(|memory| memory.time_to_forget > time.0);
    }
}

#[derive(Default)]
pub struct Brain {
    tgt: Option<Id<Site>>,
    track: Option<(Track, usize)>,
    memories: Vec<Memory>,
}

impl Brain {
    pub fn add_memory(&mut self, memory: Memory) { self.memories.push(memory); }

    pub fn remembers_character(&self, name_to_remember: &str) -> bool {
        self.memories.iter().any(|memory| matches!(&memory.item, MemoryItem::CharacterInteraction { name, .. } if name == name_to_remember))
    }

    pub fn remembers_fight_with_character(&self, name_to_remember: &str) -> bool {
        self.memories.iter().any(|memory| matches!(&memory.item, MemoryItem::CharacterFight { name, .. } if name == name_to_remember))
    }
}
