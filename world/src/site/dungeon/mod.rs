use super::SpawnRules;
use crate::{
    block::block_from_structure,
    column::ColumnSample,
    sim::WorldSim,
    site::{namegen::NameGen, BlockMask},
    util::{attempt, Grid, RandomField, Sampler, CARDINALS, DIRS},
    IndexRef,
};

use common::{
    assets::{AssetExt, AssetHandle},
    astar::Astar,
    comp::{
        inventory::loadout_builder,
        {self},
    },
    generation::{ChunkSupplement, EntityInfo},
    lottery::Lottery,
    store::{Id, Store},
    terrain::{Block, BlockKind, SpriteKind, Structure, StructuresGroup, TerrainChunkSize},
    vol::{BaseVol, ReadVol, RectSizedVol, RectVolSize, WriteVol},
};
use core::{f32, hash::BuildHasherDefault};
use fxhash::FxHasher64;
use lazy_static::lazy_static;
use rand::prelude::*;
use serde::Deserialize;
use vek::*;

pub struct Dungeon {
    name: String,
    origin: Vec2<i32>,
    alt: i32,
    seed: u32,
    #[allow(dead_code)]
    noise: RandomField,
    floors: Vec<Floor>,
    difficulty: u32,
}

pub struct GenCtx<'a, R: Rng> {
    sim: Option<&'a WorldSim>,
    rng: &'a mut R,
}

#[derive(Deserialize)]
pub struct Colors {
    pub stone: (u8, u8, u8),
}

const ALT_OFFSET: i32 = -2;

impl Dungeon {
    #[allow(clippy::let_and_return)] // TODO: Pending review in #587
    pub fn generate(wpos: Vec2<i32>, sim: Option<&WorldSim>, rng: &mut impl Rng) -> Self {
        let mut ctx = GenCtx { sim, rng };
        let difficulty = ctx.rng.gen_range(0..6);
        let floors = 3 + difficulty / 2;
        let this = Self {
            name: {
                let name = NameGen::location(ctx.rng).generate();
                match ctx.rng.gen_range(0..5) {
                    0 => format!("{} Dungeon", name),
                    1 => format!("{} Lair", name),
                    2 => format!("{} Crib", name),
                    3 => format!("{} Catacombs", name),
                    _ => format!("{} Pit", name),
                }
            },
            origin: wpos - TILE_SIZE / 2,
            alt: ctx
                .sim
                .and_then(|sim| sim.get_alt_approx(wpos))
                .unwrap_or(0.0) as i32
                + 6,
            seed: ctx.rng.gen(),
            noise: RandomField::new(ctx.rng.gen()),
            floors: (0..floors)
                .scan(Vec2::zero(), |stair_tile, level| {
                    let (floor, st) =
                        Floor::generate(&mut ctx, *stair_tile, level as i32, difficulty);
                    *stair_tile = st;
                    Some(floor)
                })
                .collect(),
            difficulty,
        };

        this
    }

    pub fn name(&self) -> &str { &self.name }

    pub fn get_origin(&self) -> Vec2<i32> { self.origin }

    pub fn radius(&self) -> f32 { 200.0 }

    #[allow(clippy::needless_update)] // TODO: Pending review in #587
    pub fn spawn_rules(&self, wpos: Vec2<i32>) -> SpawnRules {
        SpawnRules {
            trees: wpos.distance_squared(self.origin) > 64i32.pow(2),
            ..SpawnRules::default()
        }
    }

    pub fn difficulty(&self) -> u32 { self.difficulty }

    pub fn apply_to<'a>(
        &'a self,
        index: IndexRef,
        wpos2d: Vec2<i32>,
        mut get_column: impl FnMut(Vec2<i32>) -> Option<&'a ColumnSample<'a>>,
        vol: &mut (impl BaseVol<Vox = Block> + RectSizedVol + ReadVol + WriteVol),
    ) {
        lazy_static! {
            pub static ref ENTRANCES: AssetHandle<StructuresGroup> =
                Structure::load_group("dungeon_entrances");
        }

        let entrances = ENTRANCES.read();
        let entrance = &entrances[self.seed as usize % entrances.len()];

        for y in 0..vol.size_xy().y as i32 {
            for x in 0..vol.size_xy().x as i32 {
                let offs = Vec2::new(x, y);

                let wpos2d = wpos2d + offs;
                let rpos = wpos2d - self.origin;

                // Apply the dungeon entrance
                let col_sample = if let Some(col) = get_column(offs) {
                    col
                } else {
                    continue;
                };
                for z in entrance.get_bounds().min.z..entrance.get_bounds().max.z {
                    let wpos = Vec3::new(offs.x, offs.y, self.alt + z + ALT_OFFSET);
                    let spos = Vec3::new(rpos.x - TILE_SIZE / 2, rpos.y - TILE_SIZE / 2, z);
                    if let Some(block) = entrance
                        .get(spos)
                        .ok()
                        .copied()
                        .map(|sb| {
                            block_from_structure(
                                index,
                                sb,
                                spos,
                                self.origin,
                                self.seed,
                                col_sample,
                                // TODO: Take environment into account.
                                Block::air,
                            )
                        })
                        .unwrap_or(None)
                    {
                        let _ = vol.set(wpos, block);
                    }
                }

                // Apply the dungeon internals
                let mut z = self.alt + ALT_OFFSET;
                for floor in &self.floors {
                    z -= floor.total_depth();

                    let mut sampler = floor.col_sampler(
                        index,
                        rpos,
                        z,
                        // TODO: Take environment into account.
                        Block::air,
                    );

                    for rz in 0..floor.total_depth() {
                        if let Some(block) = sampler(rz).finish() {
                            let _ = vol.set(Vec3::new(offs.x, offs.y, z + rz), block);
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::or_fun_call)] // TODO: Pending review in #587
    pub fn apply_supplement<'a>(
        &'a self,
        // NOTE: Used only for dynamic elements like chests and entities!
        dynamic_rng: &mut impl Rng,
        wpos2d: Vec2<i32>,
        _get_column: impl FnMut(Vec2<i32>) -> Option<&'a ColumnSample<'a>>,
        supplement: &mut ChunkSupplement,
    ) {
        let rpos = wpos2d - self.origin;
        let area = Aabr {
            min: rpos,
            max: rpos + TerrainChunkSize::RECT_SIZE.map(|e| e as i32),
        };

        // Add waypoint
        let pos = self.origin.map2(FLOOR_SIZE, |e, sz| e + sz as i32 / 2);
        if area.contains_point(pos - self.origin) {
            supplement.add_entity(
                EntityInfo::at(Vec3::new(pos.x as f32, pos.y as f32, self.alt as f32) + 0.5)
                    .into_waypoint(),
            );
        }

        let mut z = self.alt + ALT_OFFSET;
        for floor in &self.floors {
            z -= floor.total_depth();
            let origin = Vec3::new(self.origin.x, self.origin.y, z);
            floor.apply_supplement(dynamic_rng, area, origin, supplement);
        }
    }
}

const TILE_SIZE: i32 = 13;

#[derive(Clone)]
pub enum Tile {
    UpStair(Id<Room>),
    DownStair(Id<Room>),
    Room(Id<Room>),
    Tunnel,
    Solid,
}

impl Tile {
    fn is_passable(&self) -> bool {
        matches!(
            self,
            Tile::UpStair(_) | Tile::DownStair(_) | Tile::Room(_) | Tile::Tunnel
        )
    }
}

pub struct Room {
    seed: u32,
    loot_density: f32,
    enemy_density: Option<f32>,
    miniboss: bool,
    boss: bool,
    area: Rect<i32, i32>,
    height: i32,
    pillars: Option<i32>, // Pillars with the given separation
    difficulty: u32,
}

struct Floor {
    tile_offset: Vec2<i32>,
    tiles: Grid<Tile>,
    rooms: Store<Room>,
    solid_depth: i32,
    hollow_depth: i32,
    #[allow(dead_code)]
    stair_tile: Vec2<i32>,
    final_level: bool,
    difficulty: u32,
}

const FLOOR_SIZE: Vec2<i32> = Vec2::new(18, 18);

impl Floor {
    fn generate(
        ctx: &mut GenCtx<impl Rng>,
        stair_tile: Vec2<i32>,
        level: i32,
        difficulty: u32,
    ) -> (Self, Vec2<i32>) {
        const MAX_WIDTH: u32 = 4;
        let floors = 3 + difficulty / 2;
        let final_level = level == floors as i32 - 1;
        let width = (2 + difficulty / 2).min(MAX_WIDTH);
        let height = (15 + difficulty * 3).min(30);

        let new_stair_tile = if final_level {
            Vec2::zero()
        } else {
            std::iter::from_fn(|| {
                Some(FLOOR_SIZE.map(|sz| ctx.rng.gen_range(-sz / 2 + 2..sz / 2 - 1)))
            })
            .filter(|pos| *pos != stair_tile)
            .take(8)
            .max_by_key(|pos| (*pos - stair_tile).map(|e| e.abs()).sum())
            .unwrap()
        };

        let tile_offset = -FLOOR_SIZE / 2;
        let mut this = Floor {
            tile_offset,
            tiles: Grid::new(FLOOR_SIZE, Tile::Solid),
            rooms: Store::default(),
            solid_depth: if level == 0 { 80 } else { 32 },
            hollow_depth: 30,
            stair_tile: new_stair_tile - tile_offset,
            final_level,
            difficulty,
        };

        const STAIR_ROOM_HEIGHT: i32 = 13;
        // Create rooms for entrance and exit
        let upstair_room = this.create_room(Room {
            seed: ctx.rng.gen(),
            loot_density: 0.0,
            enemy_density: None,
            miniboss: false,
            boss: false,
            area: Rect::from((stair_tile - tile_offset - 1, Extent2::broadcast(3))),
            height: STAIR_ROOM_HEIGHT,
            pillars: None,
            difficulty,
        });
        if final_level {
            // Boss room
            this.create_room(Room {
                seed: ctx.rng.gen(),
                loot_density: 0.0,
                enemy_density: Some((0.0002 * difficulty as f32).min(0.001)), // Minions!
                miniboss: false,
                boss: true,
                area: Rect::from((
                    new_stair_tile - tile_offset - MAX_WIDTH as i32 - 1,
                    Extent2::broadcast(width as i32 * 2 + 1),
                )),
                height: height as i32,
                pillars: Some(2),
                difficulty,
            });
        } else {
            // Create downstairs room
            let downstair_room = this.create_room(Room {
                seed: ctx.rng.gen(),
                loot_density: 0.0,
                enemy_density: None,
                miniboss: false,
                boss: false,
                area: Rect::from((new_stair_tile - tile_offset - 1, Extent2::broadcast(3))),
                height: STAIR_ROOM_HEIGHT,
                pillars: None,
                difficulty,
            });
            this.tiles.set(
                new_stair_tile - tile_offset,
                Tile::DownStair(downstair_room),
            );
        }
        this.tiles
            .set(stair_tile - tile_offset, Tile::UpStair(upstair_room));

        this.create_rooms(ctx, level, 7);
        // Create routes between all rooms
        let room_areas = this.rooms.values().map(|r| r.area).collect::<Vec<_>>();
        for a in room_areas.iter() {
            for b in room_areas.iter() {
                this.create_route(ctx, a.center(), b.center());
            }
        }

        (this, new_stair_tile)
    }

    fn create_room(&mut self, room: Room) -> Id<Room> {
        let area = room.area;
        let id = self.rooms.insert(room);
        for x in 0..area.extent().w {
            for y in 0..area.extent().h {
                self.tiles
                    .set(area.position() + Vec2::new(x, y), Tile::Room(id));
            }
        }
        id
    }

    fn create_rooms(&mut self, ctx: &mut GenCtx<impl Rng>, level: i32, n: usize) {
        let dim_limits = (3, 6);

        for _ in 0..n {
            let area = match attempt(64, || {
                let sz = Vec2::<i32>::zero().map(|_| ctx.rng.gen_range(dim_limits.0..dim_limits.1));
                let pos = FLOOR_SIZE.map2(sz, |floor_sz, room_sz| {
                    ctx.rng.gen_range(0..floor_sz + 1 - room_sz)
                });
                let area = Rect::from((pos, Extent2::from(sz)));
                let area_border = Rect::from((pos - 1, Extent2::from(sz) + 2)); // The room, but with some personal space

                // Ensure no overlap
                if self
                    .rooms
                    .values()
                    .any(|r| r.area.collides_with_rect(area_border))
                {
                    return None;
                }

                Some(area)
            }) {
                Some(area) => area,
                None => return,
            };
            let mut dynamic_rng = rand::thread_rng();

            match dynamic_rng.gen_range(0..5) {
                0 => self.create_room(Room {
                    seed: ctx.rng.gen(),
                    loot_density: 0.000025 + level as f32 * 0.00015,
                    enemy_density: None,
                    miniboss: true,
                    boss: false,
                    area,
                    height: ctx.rng.gen_range(15..20),
                    pillars: Some(4),
                    difficulty: self.difficulty,
                }),
                _ => self.create_room(Room {
                    seed: ctx.rng.gen(),
                    loot_density: 0.000025 + level as f32 * 0.00015,
                    enemy_density: Some(0.001 + level as f32 * 0.00006),
                    miniboss: false,
                    boss: false,
                    area,
                    height: ctx.rng.gen_range(10..15),
                    pillars: if ctx.rng.gen_range(0..4) == 0 {
                        Some(4)
                    } else {
                        None
                    },
                    difficulty: self.difficulty,
                }),
            };
        }
    }

    #[allow(clippy::unnested_or_patterns)] // TODO: Pending review in #587
    fn create_route(&mut self, _ctx: &mut GenCtx<impl Rng>, a: Vec2<i32>, b: Vec2<i32>) {
        let heuristic = move |l: &Vec2<i32>| (l - b).map(|e| e.abs()).reduce_max() as f32;
        let neighbors = |l: &Vec2<i32>| {
            let l = *l;
            CARDINALS
                .iter()
                .map(move |dir| l + dir)
                .filter(|pos| self.tiles.get(*pos).is_some())
        };
        let transition = |_a: &Vec2<i32>, b: &Vec2<i32>| match self.tiles.get(*b) {
            Some(Tile::Room(_)) | Some(Tile::Tunnel) => 1.0,
            Some(Tile::Solid) => 25.0,
            Some(Tile::UpStair(_)) | Some(Tile::DownStair(_)) => 0.0,
            _ => 100000.0,
        };
        let satisfied = |l: &Vec2<i32>| *l == b;
        // We use this hasher (FxHasher64) because
        // (1) we don't care about DDOS attacks (ruling out SipHash);
        // (2) we don't care about determinism across computers (we could use AAHash);
        // (3) we have 8-byte keys (for which FxHash is fastest).
        let mut astar = Astar::new(
            20000,
            a,
            heuristic,
            BuildHasherDefault::<FxHasher64>::default(),
        );
        let path = astar
            .poll(
                FLOOR_SIZE.product() as usize + 1,
                heuristic,
                neighbors,
                transition,
                satisfied,
            )
            .into_path()
            .expect("No route between locations - this shouldn't be able to happen");

        for pos in path.iter() {
            if let Some(tile @ Tile::Solid) = self.tiles.get_mut(*pos) {
                *tile = Tile::Tunnel;
            }
        }
    }

    #[allow(clippy::match_single_binding)] // TODO: Pending review in #587
    fn apply_supplement(
        &self,
        // NOTE: Used only for dynamic elements like chests and entities!
        dynamic_rng: &mut impl Rng,
        area: Aabr<i32>,
        origin: Vec3<i32>,
        supplement: &mut ChunkSupplement,
    ) {
        /*
        // Add stair waypoint
        let stair_rcenter =
            Vec3::from((self.stair_tile + self.tile_offset).map(|e| e * TILE_SIZE + TILE_SIZE / 2));

        if area.contains_point(stair_rcenter.xy()) {
            let offs = Vec2::new(
                dynamic_rng.gen_range(-1.0..1.0),
                dynamic_rng.gen_range(-1.0..1.0),
            )
            .try_normalized()
            .unwrap_or_else(Vec2::unit_y)
                * (TILE_SIZE as f32 / 2.0 - 4.0);
            if !self.final_level {
                supplement.add_entity(
                    EntityInfo::at((origin + stair_rcenter).map(|e| e as f32)
            + Vec3::from(offs))             .into_waypoint(),
                );
            }
        }
        */

        for x in area.min.x..area.max.x {
            for y in area.min.y..area.max.y {
                let tile_pos = Vec2::new(x, y).map(|e| e.div_euclid(TILE_SIZE)) - self.tile_offset;
                let wpos2d = origin.xy() + Vec2::new(x, y);
                if let Some(Tile::Room(room)) = self.tiles.get(tile_pos) {
                    let room = &self.rooms[*room];

                    let tile_wcenter = origin
                        + Vec3::from(
                            Vec2::new(x, y)
                                .map(|e| e.div_euclid(TILE_SIZE) * TILE_SIZE + TILE_SIZE / 2),
                        );

                    let tile_is_pillar = room
                        .pillars
                        .map(|pillar_space| {
                            tile_pos
                                .map(|e| e.rem_euclid(pillar_space) == 0)
                                .reduce_and()
                        })
                        .unwrap_or(false);

                    if room
                        .enemy_density
                        .map(|density| dynamic_rng.gen_range(0..density.recip() as usize) == 0)
                        .unwrap_or(false)
                        && !tile_is_pillar
                    {
                        // Bad
                        let chosen = match room.difficulty {
                            0 => {
                                Lottery::<String>::load_expect(match dynamic_rng.gen_range(0..4) {
                                    0 => "common.loot_tables.loot_table_humanoids",
                                    1 => "common.loot_tables.loot_table_armor_cloth",
                                    _ => "common.loot_tables.loot_table_weapon_common",
                                })
                            },
                            1 => {
                                Lottery::<String>::load_expect(match dynamic_rng.gen_range(0..4) {
                                    0 => "common.loot_tables.loot_table_humanoids",
                                    1 => "common.loot_tables.loot_table_armor_light",
                                    _ => "common.loot_tables.loot_table_weapon_uncommon",
                                })
                            },
                            2 => {
                                Lottery::<String>::load_expect(match dynamic_rng.gen_range(0..4) {
                                    0 => "common.loot_tables.loot_table_humanoids",
                                    1 => "common.loot_tables.loot_table_armor_heavy",
                                    _ => "common.loot_tables.loot_table_weapon_rare",
                                })
                            },
                            3 => {
                                Lottery::<String>::load_expect(match dynamic_rng.gen_range(0..10) {
                                    0 => "common.loot_tables.loot_table_humanoids",
                                    1 => "common.loot_tables.loot_table_armor_heavy",
                                    2 => "common.loot_tables.loot_table_weapon_rare",
                                    _ => "common.loot_tables.loot_table_cultists",
                                })
                            },
                            4 => {
                                Lottery::<String>::load_expect(match dynamic_rng.gen_range(0..6) {
                                    0 => "common.loot_tables.loot_table_humanoids",
                                    1 => "common.loot_tables.loot_table_armor_misc",
                                    2 => "common.loot_tables.loot_table_weapon_rare",
                                    _ => "common.loot_tables.loot_table_cultists",
                                })
                            },
                            5 => {
                                Lottery::<String>::load_expect(match dynamic_rng.gen_range(0..5) {
                                    0 => "common.loot_tables.loot_table_humanoids",
                                    1 => "common.loot_tables.loot_table_armor_misc",
                                    2 => "common.loot_tables.loot_table_weapon_rare",
                                    _ => "common.loot_tables.loot_table_cultists",
                                })
                            },
                            _ => Lottery::<String>::load_expect(
                                "common.loot_tables.loot_table_armor_misc",
                            ),
                        };
                        let chosen = chosen.read();
                        let chosen = chosen.choose();
                        //let is_giant =
                        // RandomField::new(room.seed.wrapping_add(1)).chance(Vec3::from(tile_pos),
                        // 0.2) && !room.boss;
                        let entity = EntityInfo::at(
                            tile_wcenter.map(|e| e as f32)
                            // Randomly displace them a little
                            + Vec3::<u32>::iota()
                                .map(|e| (RandomField::new(room.seed.wrapping_add(10 + e)).get(Vec3::from(tile_pos)) % 32) as i32 - 16)
                                .map(|e| e as f32 / 16.0),
                        )
                        //.do_if(is_giant, |e| e.into_giant())
                        .with_alignment(comp::Alignment::Enemy)
                        .with_loadout_config(loadout_builder::LoadoutConfig::CultistAcolyte)
                        .with_skillset_config(common::skillset_builder::SkillSetConfig::CultistAcolyte)
                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen))
                        .with_level(dynamic_rng.gen_range((room.difficulty as f32).powf(1.25) + 3.0..(room.difficulty as f32).powf(1.5) + 4.0).round() as u16);
                        let entity = match room.difficulty {
                            0 => entity
                                .with_body(comp::Body::BipedSmall(
                                    comp::biped_small::Body::random_with(
                                        dynamic_rng,
                                        &comp::biped_small::Species::Gnarling,
                                    ),
                                ))
                                .with_name("Gnarling")
                                .with_loadout_config(loadout_builder::LoadoutConfig::Gnarling)
                                .with_skillset_config(
                                    common::skillset_builder::SkillSetConfig::Gnarling,
                                )
                                .with_loot_drop(comp::Item::new_from_asset_expect(chosen))
                                .with_main_tool(comp::Item::new_from_asset_expect(
                                    match dynamic_rng.gen_range(0..5) {
                                        0 => {
                                            "common.items.npc_weapons.biped_small.gnarling.\
                                             adlet_bow"
                                        },
                                        1 => {
                                            "common.items.npc_weapons.biped_small.gnarling.\
                                             gnoll_staff"
                                        },
                                        _ => {
                                            "common.items.npc_weapons.biped_small.gnarling.\
                                             wooden_spear"
                                        },
                                    },
                                )),
                            1 => entity
                                .with_body(comp::Body::BipedSmall(
                                    comp::biped_small::Body::random_with(
                                        dynamic_rng,
                                        &comp::biped_small::Species::Adlet,
                                    ),
                                ))
                                .with_name("Adlet")
                                .with_loadout_config(loadout_builder::LoadoutConfig::Adlet)
                                .with_skillset_config(
                                    common::skillset_builder::SkillSetConfig::Adlet,
                                )
                                .with_loot_drop(comp::Item::new_from_asset_expect(chosen))
                                .with_main_tool(comp::Item::new_from_asset_expect(
                                    match dynamic_rng.gen_range(0..5) {
                                        0 => "common.items.npc_weapons.biped_small.adlet.adlet_bow",
                                        1 => {
                                            "common.items.npc_weapons.biped_small.adlet.gnoll_staff"
                                        },
                                        _ => {
                                            "common.items.npc_weapons.biped_small.adlet.\
                                             wooden_spear"
                                        },
                                    },
                                )),
                            2 => entity
                                .with_body(comp::Body::BipedSmall(
                                    comp::biped_small::Body::random_with(
                                        dynamic_rng,
                                        &comp::biped_small::Species::Sahagin,
                                    ),
                                ))
                                .with_name("Sahagin")
                                .with_loadout_config(loadout_builder::LoadoutConfig::Sahagin)
                                .with_skillset_config(
                                    common::skillset_builder::SkillSetConfig::Sahagin,
                                )
                                .with_loot_drop(comp::Item::new_from_asset_expect(chosen))
                                .with_main_tool(comp::Item::new_from_asset_expect(
                                    match dynamic_rng.gen_range(0..5) {
                                        0 => {
                                            "common.items.npc_weapons.biped_small.sahagin.adlet_bow"
                                        },
                                        1 => {
                                            "common.items.npc_weapons.biped_small.sahagin.\
                                             gnoll_staff"
                                        },
                                        _ => {
                                            "common.items.npc_weapons.biped_small.sahagin.\
                                             wooden_spear"
                                        },
                                    },
                                )),
                            3 => entity
                                .with_body(comp::Body::BipedSmall(
                                    comp::biped_small::Body::random_with(
                                        dynamic_rng,
                                        &comp::biped_small::Species::Haniwa,
                                    ),
                                ))
                                .with_name("Haniwa")
                                .with_loadout_config(loadout_builder::LoadoutConfig::Haniwa)
                                .with_skillset_config(
                                    common::skillset_builder::SkillSetConfig::Haniwa,
                                )
                                .with_loot_drop(comp::Item::new_from_asset_expect(chosen))
                                .with_main_tool(comp::Item::new_from_asset_expect(
                                    match dynamic_rng.gen_range(0..5) {
                                        0 => {
                                            "common.items.npc_weapons.biped_small.haniwa.adlet_bow"
                                        },
                                        1 => {
                                            "common.items.npc_weapons.biped_small.haniwa.\
                                             gnoll_staff"
                                        },
                                        _ => {
                                            "common.items.npc_weapons.biped_small.haniwa.\
                                             wooden_spear"
                                        },
                                    },
                                )),
                            4 => entity
                                .with_body(comp::Body::BipedSmall(
                                    comp::biped_small::Body::random_with(
                                        dynamic_rng,
                                        &comp::biped_small::Species::Myrmidon,
                                    ),
                                ))
                                .with_name("Myrmidon")
                                .with_loadout_config(loadout_builder::LoadoutConfig::Myrmidon)
                                .with_skillset_config(
                                    common::skillset_builder::SkillSetConfig::Myrmidon,
                                )
                                .with_loot_drop(comp::Item::new_from_asset_expect(chosen))
                                .with_main_tool(comp::Item::new_from_asset_expect(
                                    match dynamic_rng.gen_range(0..5) {
                                        0 => {
                                            "common.items.npc_weapons.biped_small.myrmidon.\
                                             adlet_bow"
                                        },
                                        1 => {
                                            "common.items.npc_weapons.biped_small.myrmidon.\
                                             gnoll_staff"
                                        },
                                        _ => {
                                            "common.items.npc_weapons.biped_small.myrmidon.\
                                             wooden_spear"
                                        },
                                    },
                                )),
                            5 => match dynamic_rng.gen_range(0..6) {
                                0 => entity
                                    .with_body(comp::Body::Humanoid(comp::humanoid::Body::random()))
                                    .with_name("Cultist Warlock")
                                    .with_loadout_config(loadout_builder::LoadoutConfig::Warlock)
                                    .with_skillset_config(
                                        common::skillset_builder::SkillSetConfig::Warlock,
                                    )
                                    .with_loot_drop(comp::Item::new_from_asset_expect(chosen))
                                    .with_main_tool(comp::Item::new_from_asset_expect(
                                        "common.items.weapons.staff.cultist_staff",
                                    )),
                                1 => entity
                                    .with_body(comp::Body::Object(comp::object::Body::Crossbow))
                                    .with_name("Possessed Turret".to_string())
                                    .with_loot_drop(comp::Item::new_from_asset_expect(
                                        "common.items.crafting_ing.twigs",
                                    )),
                                _ => entity
                                    .with_name("Cultist Warlord")
                                    .with_loadout_config(loadout_builder::LoadoutConfig::Warlord)
                                    .with_skillset_config(
                                        common::skillset_builder::SkillSetConfig::Warlord,
                                    )
                                    .with_loot_drop(comp::Item::new_from_asset_expect(chosen))
                                    .with_main_tool(comp::Item::new_from_asset_expect(
                                        match dynamic_rng.gen_range(0..5) {
                                            0 => "common.items.weapons.axe.malachite_axe-0",
                                            1..=2 => "common.items.weapons.sword.cultist",
                                            3 => "common.items.weapons.hammer.cultist_purp_2h-0",
                                            _ => "common.items.weapons.bow.bone-1",
                                        },
                                    )),
                            },
                            _ => entity.with_name("Humanoid").with_main_tool(
                                comp::Item::new_from_asset_expect(
                                    "common.items.weapons.bow.bone-1",
                                ),
                            ),
                        };
                        supplement.add_entity(entity);
                    }

                    if room.boss {
                        let boss_spawn_tile = room.area.center();
                        // Don't spawn the boss in a pillar
                        let boss_tile_is_pillar = room
                            .pillars
                            .map(|pillar_space| {
                                boss_spawn_tile
                                    .map(|e| e.rem_euclid(pillar_space) == 0)
                                    .reduce_and()
                            })
                            .unwrap_or(false);
                        let boss_spawn_tile =
                            boss_spawn_tile + if boss_tile_is_pillar { 1 } else { 0 };

                        if tile_pos == boss_spawn_tile && tile_wcenter.xy() == wpos2d {
                            let chosen = match room.difficulty {
                                0 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_weapon_uncommon",
                                ),
                                1 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_weapon_uncommon",
                                ),
                                2 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_armor_heavy",
                                ),
                                3 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_weapon_rare",
                                ),
                                4 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_miniboss",
                                ),
                                5 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_mindflayer",
                                ),
                                _ => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_armor_misc",
                                ),
                            };
                            let chosen = chosen.read();
                            let chosen = chosen.choose();
                            let entity = match room.difficulty {
                                0 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::BipedLarge(
                                            comp::biped_large::Body::random_with(
                                                dynamic_rng,
                                                &comp::biped_large::Species::Harvester,
                                            ),
                                        ))
                                        .with_name("Harvester".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                ],
                                1 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::BipedLarge(
                                            comp::biped_large::Body::random_with(
                                                dynamic_rng,
                                                &comp::biped_large::Species::Yeti,
                                            ),
                                        ))
                                        .with_name("Yeti".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                ],
                                2 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::BipedLarge(
                                            comp::biped_large::Body::random_with(
                                                dynamic_rng,
                                                &comp::biped_large::Species::Tidalwarrior,
                                            ),
                                        ))
                                        .with_name("Tidal Warrior".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                ],
                                3 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::Golem(
                                            comp::golem::Body::random_with(
                                                dynamic_rng,
                                                &comp::golem::Species::ClayGolem,
                                            ),
                                        ))
                                        .with_name("Clay Golem".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                ],
                                4 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::BipedLarge(
                                            comp::biped_large::Body::random_with(
                                                dynamic_rng,
                                                &comp::biped_large::Species::Minotaur,
                                            ),
                                        ))
                                        .with_name("Minotaur".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                ],
                                5 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::BipedLarge(
                                            comp::biped_large::Body::random_with(
                                                dynamic_rng,
                                                &comp::biped_large::Species::Mindflayer,
                                            ),
                                        ))
                                        .with_name("Mindflayer".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen))
                                        .with_skillset_config(
                                            common::skillset_builder::SkillSetConfig::Mindflayer,
                                        ),
                                ],
                                _ => {
                                    vec![EntityInfo::at(tile_wcenter.map(|e| e as f32)).with_body(
                                        comp::Body::QuadrupedSmall(
                                            comp::quadruped_small::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_small::Species::Sheep,
                                            ),
                                        ),
                                    )]
                                },
                            };

                            for entity in entity {
                                supplement.add_entity(
                                    entity
                                        .with_level(
                                            dynamic_rng
                                                .gen_range(
                                                    (room.difficulty as f32).powf(1.25) + 3.0
                                                        ..(room.difficulty as f32).powf(1.5) + 4.0,
                                                )
                                                .round()
                                                as u16
                                                * 5,
                                        )
                                        .with_alignment(comp::Alignment::Enemy),
                                );
                            }
                        }
                    }
                    if room.miniboss {
                        let miniboss_spawn_tile = room.area.center();
                        // Don't spawn the miniboss in a pillar
                        let miniboss_tile_is_pillar = room
                            .pillars
                            .map(|pillar_space| {
                                miniboss_spawn_tile
                                    .map(|e| e.rem_euclid(pillar_space) == 0)
                                    .reduce_and()
                            })
                            .unwrap_or(false);
                        let miniboss_spawn_tile =
                            miniboss_spawn_tile + if miniboss_tile_is_pillar { 1 } else { 0 };

                        if tile_pos == miniboss_spawn_tile && tile_wcenter.xy() == wpos2d {
                            let chosen = match room.difficulty {
                                0 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_animal_parts",
                                ),
                                1 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_animal_parts",
                                ),
                                2 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_animal_parts",
                                ),
                                3 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_weapon_rare",
                                ),
                                4 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_weapon_rare",
                                ),
                                5 => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_miniboss",
                                ),
                                _ => Lottery::<String>::load_expect(
                                    "common.loot_tables.loot_table_armor_misc",
                                ),
                            };
                            let chosen = chosen.read();
                            let chosen = chosen.choose();
                            let entity = match room.difficulty {
                                0 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::QuadrupedMedium(
                                            comp::quadruped_medium::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_medium::Species::Bonerattler,
                                            ),
                                        ))
                                        .with_name("Bonerattler".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                ],
                                1 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::QuadrupedMedium(
                                            comp::quadruped_medium::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_medium::Species::Bonerattler,
                                            )
                                        ))
                                        .with_name("Bonerattler".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(
                                            chosen
                                        ));
                                    3
                                ],
                                2 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::QuadrupedLow(
                                            comp::quadruped_low::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_low::Species::Hakulaq,
                                            ),
                                        ))
                                        .with_name("Hakulaq".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                        EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::QuadrupedLow(
                                            comp::quadruped_low::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_low::Species::Hakulaq,
                                            ),
                                        ))
                                        .with_name("Hakulaq".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                        EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::QuadrupedLow(
                                            comp::quadruped_low::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_low::Species::Hakulaq,
                                            ),
                                        ))
                                        .with_name("Hakulaq".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                        EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::QuadrupedLow(
                                            comp::quadruped_low::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_low::Species::Hakulaq,
                                            ),
                                        ))
                                        .with_name("Hakulaq".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                        EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::QuadrupedLow(
                                            comp::quadruped_low::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_low::Species::Hakulaq,
                                            ),
                                        ))
                                        .with_name("Hakulaq".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                        EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::QuadrupedLow(
                                            comp::quadruped_low::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_low::Species::Hakulaq,
                                            ),
                                        ))
                                        .with_name("Hakulaq".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                ],
                                3 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::Humanoid(
                                            comp::humanoid::Body::random(),
                                        ))
                                        .with_name("Animal Trainer".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen))
                                        .with_loadout_config(loadout_builder::LoadoutConfig::CultistAcolyte)
                                        .with_skillset_config(
                                            common::skillset_builder::SkillSetConfig::CultistAcolyte
                                        )
                                        .with_main_tool(comp::Item::new_from_asset_expect(
                                            match dynamic_rng.gen_range(0..6) {
                                                0 => "common.items.weapons.axe.malachite_axe-0",
                                                1..=2 => "common.items.weapons.sword.cultist",
                                                3 => {
                                                    "common.items.weapons.hammer.cultist_purp_2h-0"
                                                },
                                                4 => "common.items.weapons.staff.cultist_staff",
                                                _ => "common.items.weapons.bow.bone-1",
                                            },
                                        )),
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::QuadrupedMedium(
                                            comp::quadruped_medium::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_medium::Species::Darkhound,
                                            ),
                                        ))
                                        .with_name("Tamed Darkhound".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::QuadrupedMedium(
                                            comp::quadruped_medium::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_medium::Species::Darkhound,
                                            ),
                                        ))
                                        .with_name("Tamed Darkhound".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                ],
                                4 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::BipedLarge(
                                            comp::biped_large::Body::random_with(
                                                dynamic_rng,
                                                &comp::biped_large::Species::Dullahan,
                                            ),
                                        ))
                                        .with_name("Dullahan Guard".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                ],
                                5 => vec![
                                    EntityInfo::at(tile_wcenter.map(|e| e as f32))
                                        .with_body(comp::Body::Golem(
                                            comp::golem::Body::random_with(
                                                dynamic_rng,
                                                &comp::golem::Species::StoneGolem,
                                            ),
                                        ))
                                        .with_name("Stonework Defender".to_string())
                                        .with_loot_drop(comp::Item::new_from_asset_expect(chosen)),
                                ],
                                _ => {
                                    vec![EntityInfo::at(tile_wcenter.map(|e| e as f32)).with_body(
                                        comp::Body::QuadrupedSmall(
                                            comp::quadruped_small::Body::random_with(
                                                dynamic_rng,
                                                &comp::quadruped_small::Species::Sheep,
                                            ),
                                        ),
                                    )]
                                },
                            };

                            for entity in entity {
                                supplement.add_entity(
                                    entity
                                        .with_level(
                                            dynamic_rng
                                                .gen_range(
                                                    (room.difficulty as f32).powf(1.25) + 3.0
                                                        ..(room.difficulty as f32).powf(1.5) + 4.0,
                                                )
                                                .round()
                                                as u16
                                                * 5,
                                        )
                                        .with_alignment(comp::Alignment::Enemy),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn total_depth(&self) -> i32 { self.solid_depth + self.hollow_depth }

    fn nearest_wall(&self, rpos: Vec2<i32>) -> Option<Vec2<i32>> {
        let tile_pos = rpos.map(|e| e.div_euclid(TILE_SIZE));

        DIRS.iter()
            .map(|dir| tile_pos + *dir)
            .filter(|other_tile_pos| {
                self.tiles
                    .get(*other_tile_pos)
                    .filter(|tile| tile.is_passable())
                    .is_none()
            })
            .map(|other_tile_pos| {
                rpos.clamped(
                    other_tile_pos * TILE_SIZE,
                    (other_tile_pos + 1) * TILE_SIZE - 1,
                )
            })
            .min_by_key(|nearest| rpos.distance_squared(*nearest))
    }

    // Find orientation of a position relative to another position
    #[allow(clippy::collapsible_if)]
    fn relative_ori(pos1: Vec2<i32>, pos2: Vec2<i32>) -> u8 {
        if (pos1.x - pos2.x).abs() < (pos1.y - pos2.y).abs() {
            if pos1.y > pos2.y { 4 } else { 8 }
        } else {
            if pos1.x > pos2.x { 2 } else { 6 }
        }
    }

    #[allow(clippy::unnested_or_patterns)] // TODO: Pending review in #587
    fn col_sampler<'a>(
        &'a self,
        index: IndexRef<'a>,
        pos: Vec2<i32>,
        _floor_z: i32,
        mut with_sprite: impl FnMut(SpriteKind) -> Block,
    ) -> impl FnMut(i32) -> BlockMask + 'a {
        let rpos = pos - self.tile_offset * TILE_SIZE;
        let tile_pos = rpos.map(|e| e.div_euclid(TILE_SIZE));
        let tile_center = tile_pos * TILE_SIZE + TILE_SIZE / 2;
        let rtile_pos = rpos - tile_center;

        let colors = &index.colors.site.dungeon;

        let vacant = BlockMask::new(with_sprite(SpriteKind::Empty), 1);

        let make_staircase = move |pos: Vec3<i32>, radius: f32, inner_radius: f32, stretch: f32| {
            let stone = BlockMask::new(Block::new(BlockKind::Rock, colors.stone.into()), 5);

            if (pos.xy().magnitude_squared() as f32) < inner_radius.powi(2) {
                stone
            } else if (pos.xy().magnitude_squared() as f32) < radius.powi(2) {
                if ((pos.x as f32).atan2(pos.y as f32) / (f32::consts::PI * 2.0) * stretch
                    + pos.z as f32)
                    .rem_euclid(stretch)
                    < 1.5
                {
                    stone
                } else {
                    vacant
                }
            } else {
                BlockMask::nothing()
            }
        };

        let wall_thickness = 3.0;
        let dist_to_wall = self
            .nearest_wall(rpos)
            .map(|nearest| (nearest.distance_squared(rpos) as f32).sqrt())
            .unwrap_or(TILE_SIZE as f32);
        let tunnel_dist =
            1.0 - (dist_to_wall - wall_thickness).max(0.0) / (TILE_SIZE as f32 - wall_thickness);

        let floor_sprite = if RandomField::new(7331).chance(Vec3::from(pos), 0.001) {
            BlockMask::new(
                with_sprite(
                    match (RandomField::new(1337).get(Vec3::from(pos)) / 2) % 30 {
                        0 => SpriteKind::Apple,
                        1 => SpriteKind::VeloriteFrag,
                        2 => SpriteKind::Velorite,
                        3..=8 => SpriteKind::Mushroom,
                        9..=15 => SpriteKind::FireBowlGround,
                        _ => SpriteKind::ShortGrass,
                    },
                ),
                1,
            )
        } else if let Some(Tile::Room(room)) | Some(Tile::DownStair(room)) =
            self.tiles.get(tile_pos)
        {
            let room = &self.rooms[*room];
            if RandomField::new(room.seed).chance(Vec3::from(pos), room.loot_density * 0.5) {
                BlockMask::new(with_sprite(SpriteKind::Chest), 1)
            } else {
                vacant
            }
        } else {
            vacant
        };

        let tunnel_height = if self.final_level { 16.0 } else { 8.0 };
        let pillar_thickness: i32 = 4;

        move |z| match self.tiles.get(tile_pos) {
            Some(Tile::Solid) => BlockMask::nothing(),
            Some(Tile::Tunnel) => {
                let light_offset: i32 = 7;
                if (dist_to_wall - wall_thickness) as i32 == 1
                    && rtile_pos.map(|e| e % light_offset == 0).reduce_bitxor()
                    && z == 1
                {
                    let ori =
                        Floor::relative_ori(rpos, self.nearest_wall(rpos).unwrap_or_default());
                    let furniture = SpriteKind::WallSconce;
                    BlockMask::new(Block::air(furniture).with_ori(ori).unwrap(), 1)
                } else if dist_to_wall >= wall_thickness
                    && (z as f32) < tunnel_height * (1.0 - tunnel_dist.powi(4))
                {
                    if z == 0 { floor_sprite } else { vacant }
                } else {
                    BlockMask::nothing()
                }
            },
            Some(Tile::Room(room)) | Some(Tile::DownStair(room))
                if dist_to_wall < wall_thickness
                    || z as f32
                        >= self.rooms[*room].height as f32 * (1.0 - tunnel_dist.powi(4)) =>
            {
                BlockMask::nothing()
            },

            Some(Tile::Room(room)) | Some(Tile::DownStair(room))
                if self.rooms[*room]
                    .pillars
                    .map(|pillar_space| {
                        tile_pos
                            .map(|e| e.rem_euclid(pillar_space) == 0)
                            .reduce_and()
                            && rtile_pos.map(|e| e as f32).magnitude_squared()
                                < (pillar_thickness as f32 + 0.5).powi(2)
                    })
                    .unwrap_or(false) =>
            {
                if z == 1 && rtile_pos.product() == 0 && rtile_pos.sum().abs() == pillar_thickness {
                    let ori = Floor::relative_ori(rtile_pos, Vec2::zero());
                    let furniture = SpriteKind::WallSconce;
                    BlockMask::new(Block::air(furniture).with_ori(ori).unwrap(), 1)
                } else if z < self.rooms[*room].height
                    && rtile_pos.map(|e| e as f32).magnitude_squared()
                        > (pillar_thickness as f32 - 0.5).powi(2)
                {
                    vacant
                } else {
                    BlockMask::nothing()
                }
            }

            Some(Tile::Room(_)) => {
                let light_offset = 7;
                if z == 0 {
                    floor_sprite
                } else if dist_to_wall as i32 == 4
                    && rtile_pos.map(|e| e % light_offset == 0).reduce_bitxor()
                    && z == 1
                {
                    let ori = Floor::relative_ori(
                        rpos,
                        self.nearest_wall(rpos).unwrap_or_else(Vec2::zero),
                    );
                    let furniture = SpriteKind::WallSconce;
                    BlockMask::new(Block::air(furniture).with_ori(ori).unwrap(), 1)
                } else {
                    vacant
                }
            },
            Some(Tile::DownStair(_)) => {
                make_staircase(Vec3::new(rtile_pos.x, rtile_pos.y, z), 0.0, 0.5, 9.0)
                    .resolve_with(vacant)
            },
            Some(Tile::UpStair(room)) => {
                let inner_radius: f32 = 0.5;
                let stretch = 9;
                let block = make_staircase(
                    Vec3::new(rtile_pos.x, rtile_pos.y, z),
                    TILE_SIZE as f32 / 2.0,
                    inner_radius,
                    stretch as f32,
                );
                let furniture = SpriteKind::WallSconce;
                let ori = Floor::relative_ori(Vec2::zero(), rtile_pos);
                if z < self.rooms[*room].height {
                    block.resolve_with(vacant)
                } else if z % stretch == 0 && rtile_pos.x == 0 && rtile_pos.y == -TILE_SIZE / 2 {
                    BlockMask::new(Block::air(furniture).with_ori(ori).unwrap(), 1)
                } else {
                    make_staircase(
                        Vec3::new(rtile_pos.x, rtile_pos.y, z),
                        TILE_SIZE as f32 / 2.0,
                        inner_radius,
                        stretch as f32,
                    )
                }
            },
            None => BlockMask::nothing(),
        }
    }
}
