use crate::{column::ColumnSample, sim::SimChunk, Canvas, CONFIG};
use common::terrain::SpriteKind;
use noise::NoiseFn;
use rand::prelude::*;
use std::f32;
use vek::*;

fn close(x: f32, tgt: f32, falloff: f32) -> f32 {
    (1.0 - (x - tgt).abs() / falloff).max(0.0).powf(0.125)
}

const MUSH_FACT: f32 = 1.0e-4; // To balance everything around the mushroom spawning rate
const DEPTH_WATER_NORM: f32 = 15.0; // Water depth at which regular underwater sprites start spawning
pub fn apply_scatter_to(canvas: &mut Canvas, rng: &mut impl Rng) {
    use SpriteKind::*;
    #[allow(clippy::type_complexity)]
    // TODO: Add back all sprites we had before
    let scatter: &[(
        _,
        bool,
        fn(&SimChunk, &ColumnSample) -> (f32, Option<(f32, f32)>),
    )] = &[
        // (density, Option<(wavelen, threshold)>)
        // Flowers
        (BlueFlower, false, |c, col| {
            (
                close(c.temp, CONFIG.temperate_temp, 0.7).min(close(
                    c.humidity,
                    CONFIG.jungle_hum,
                    0.4,
                )) * col.tree_density
                    * MUSH_FACT
                    * 256.0,
                Some((256.0, 0.25)),
            )
        }),
        (PinkFlower, false, |c, col| {
            (
                close(c.temp, 0.0, 0.7).min(close(c.humidity, CONFIG.jungle_hum, 0.4))
                    * col.tree_density
                    * MUSH_FACT
                    * 350.0,
                Some((100.0, 0.1)),
            )
        }),
        (PurpleFlower, false, |c, col| {
            (
                close(c.temp, CONFIG.temperate_temp, 0.7).min(close(
                    c.humidity,
                    CONFIG.jungle_hum,
                    0.4,
                )) * col.tree_density
                    * MUSH_FACT
                    * 350.0,
                Some((100.0, 0.1)),
            )
        }),
        (RedFlower, false, |c, col| {
            (
                close(c.temp, CONFIG.tropical_temp, 0.7).min(close(
                    c.humidity,
                    CONFIG.jungle_hum,
                    0.4,
                )) * col.tree_density
                    * MUSH_FACT
                    * 350.0,
                Some((100.0, 0.1)),
            )
        }),
        (WhiteFlower, false, |c, col| {
            (
                close(c.temp, 0.0, 0.7).min(close(c.humidity, CONFIG.jungle_hum, 0.4))
                    * col.tree_density
                    * MUSH_FACT
                    * 350.0,
                Some((100.0, 0.1)),
            )
        }),
        (YellowFlower, false, |c, col| {
            (
                close(c.temp, 0.0, 0.7).min(close(c.humidity, CONFIG.jungle_hum, 0.4))
                    * col.tree_density
                    * MUSH_FACT
                    * 350.0,
                Some((100.0, 0.1)),
            )
        }),
        (Sunflower, false, |c, col| {
            (
                close(c.temp, 0.0, 0.7).min(close(c.humidity, CONFIG.jungle_hum, 0.4))
                    * col.tree_density
                    * MUSH_FACT
                    * 350.0,
                Some((100.0, 0.15)),
            )
        }),
        // Herbs and Spices
        (LingonBerry, false, |c, _| {
            (
                close(c.temp, 0.3, 0.4).min(close(c.humidity, CONFIG.jungle_hum, 0.5))
                    * MUSH_FACT
                    * 2.5,
                None,
            )
        }),
        (LeafyPlant, false, |c, _| {
            (
                close(c.temp, 0.3, 0.4).min(close(c.humidity, CONFIG.jungle_hum, 0.3))
                    * MUSH_FACT
                    * 4.0,
                None,
            )
        }),
        (Fern, false, |c, _| {
            (
                close(c.temp, 0.3, 0.4).min(close(c.humidity, CONFIG.forest_hum, 0.5))
                    * MUSH_FACT
                    * 0.25,
                Some((64.0, 0.2)),
            )
        }),
        (Blueberry, false, |c, _| {
            (
                close(c.temp, CONFIG.temperate_temp, 0.5).min(close(
                    c.humidity,
                    CONFIG.forest_hum,
                    0.5,
                )) * MUSH_FACT
                    * 0.3,
                None,
            )
        }),
        // Collectable Objects
        // Only spawn twigs in temperate forests
        (Twigs, false, |c, _| {
            ((c.tree_density - 0.5).max(0.0) * 1.0e-3, None)
        }),
        (Stones, false, |c, _| {
            ((c.rockiness - 0.5).max(0.0) * 1.0e-3, None)
        }),
        // Don't spawn Mushrooms in snowy regions
        (Mushroom, false, |c, _| {
            (
                close(c.temp, 0.3, 0.4).min(close(c.humidity, CONFIG.forest_hum, 0.35)) * MUSH_FACT,
                None,
            )
        }),
        // Grass
        (ShortGrass, false, |c, _| {
            (
                close(c.temp, 0.2, 0.65).min(close(c.humidity, CONFIG.jungle_hum, 0.4)) * 0.015,
                None,
            )
        }),
        (MediumGrass, false, |c, _| {
            (
                close(c.temp, 0.2, 0.6).min(close(c.humidity, CONFIG.jungle_hum, 0.4)) * 0.012,
                None,
            )
        }),
        (LongGrass, false, |c, _| {
            (
                close(c.temp, 0.3, 0.35).min(close(c.humidity, CONFIG.jungle_hum, 0.3)) * 0.15,
                Some((48.0, 0.2)),
            )
        }),
        // Jungle Sprites
        // (LongGrass, false, |c, col| {
        //     (
        //         close(c.temp, CONFIG.tropical_temp, 0.4).min(close(
        //             c.humidity,
        //             CONFIG.jungle_hum,
        //             0.6,
        //         )) * 0.08,
        //         Some((60.0, 5.0)),
        //     )
        // }),
        /*(WheatGreen, false, |c, col| {
            (
                close(c.temp, 0.4, 0.2).min(close(c.humidity, CONFIG.forest_hum, 0.1))
                    * MUSH_FACT
                    * 0.001,
                None,
            )
        }),*/
        (GrassSnow, false, |c, _| {
            (
                close(c.temp, CONFIG.snow_temp - 0.2, 0.4).min(close(
                    c.humidity,
                    CONFIG.forest_hum,
                    0.5,
                )) * 0.01,
                Some((48.0, 0.2)),
            )
        }),
        // Savanna Plants
        (SavannaGrass, false, |c, _| {
            (
                {
                    let savanna = close(c.temp, 1.0, 0.4).min(close(c.humidity, 0.2, 0.25));
                    let desert = close(c.temp, 1.0, 0.25).min(close(c.humidity, 0.0, 0.1));
                    (savanna - desert * 5.0).max(0.0)
                },
                Some((0.5, 0.2)),
            )
        }),
        (TallSavannaGrass, false, |c, _| {
            (
                {
                    let savanna = close(c.temp, 1.0, 0.4).min(close(c.humidity, 0.2, 0.25));
                    let desert = close(c.temp, 1.0, 0.25).min(close(c.humidity, 0.0, 0.1));
                    (savanna - desert * 5.0).max(0.0)
                },
                Some((12.5, 0.25)),
            )
        }),
        (RedSavannaGrass, false, |c, _| {
            (
                {
                    let savanna = close(c.temp, 1.0, 0.4).min(close(c.humidity, 0.2, 0.25));
                    let desert = close(c.temp, 1.0, 0.25).min(close(c.humidity, 0.0, 0.1));
                    (savanna - desert * 5.0).max(0.0)
                },
                Some((0.1, 0.1)),
            )
        }),
        (SavannaBush, false, |c, _| {
            (
                {
                    let savanna = close(c.temp, 1.0, 0.4).min(close(c.humidity, 0.2, 0.25));
                    let desert = close(c.temp, 1.0, 0.25).min(close(c.humidity, 0.0, 0.1));
                    (savanna - desert * 5.0).max(0.0)
                },
                Some((0.08, 0.05)),
            )
        }),
        // Desert Plants
        (DeadBush, false, |c, _| {
            (
                close(c.temp, 1.0, 0.95).min(close(c.humidity, 0.0, 0.3)) * MUSH_FACT * 7.5,
                None,
            )
        }),
        (LargeCactus, false, |c, _| {
            (
                close(c.temp, 1.0, 0.25).min(close(c.humidity, 0.0, 0.1)) * MUSH_FACT * 3.5,
                None,
            )
        }),
        (RoundCactus, false, |c, _| {
            (
                close(c.temp, 1.0, 0.25).min(close(c.humidity, 0.0, 0.1)) * MUSH_FACT * 5.5,
                None,
            )
        }),
        (ShortCactus, false, |c, _| {
            (
                close(c.temp, 1.0, 0.25).min(close(c.humidity, 0.0, 0.1)) * MUSH_FACT * 5.5,
                None,
            )
        }),
        (MedFlatCactus, false, |c, _| {
            (
                close(c.temp, 1.0, 0.25).min(close(c.humidity, 0.0, 0.1)) * MUSH_FACT * 5.5,
                None,
            )
        }),
        (ShortFlatCactus, false, |c, _| {
            (
                close(c.temp, 1.0, 0.25).min(close(c.humidity, 0.0, 0.1)) * MUSH_FACT * 5.5,
                None,
            )
        }),
        (Reed, false, |c, col| {
            (
                close(c.humidity, CONFIG.jungle_hum, 0.7)
                    * col
                        .water_dist
                        .map(|wd| Lerp::lerp(0.2, 0.0, (wd / 8.0).clamped(0.0, 1.0)))
                        .unwrap_or(0.0),
                Some((128.0, 0.5)),
            )
        }),
        // Underwater chests
        (ChestBurried, true, |_, col| {
            (
                MUSH_FACT
                    * 1.0e-6
                    * if col.alt < col.water_level - DEPTH_WATER_NORM + 30.0 {
                        1.0
                    } else {
                        0.0
                    },
                None,
            )
        }),
        // Underwater mud piles
        (Mud, true, |_, col| {
            (
                MUSH_FACT
                    * 1.0e-3
                    * if col.alt < col.water_level - DEPTH_WATER_NORM {
                        1.0
                    } else {
                        0.0
                    },
                None,
            )
        }),
        // Underwater grass
        (GrassBlue, true, |_, col| {
            (
                MUSH_FACT
                    * 250.0
                    * if col.alt < col.water_level - DEPTH_WATER_NORM {
                        1.0
                    } else {
                        0.0
                    },
                Some((100.0, 0.15)),
            )
        }),
        // seagrass
        (Seagrass, true, |c, col| {
            (
                close(c.temp, CONFIG.temperate_temp, 0.8)
                    * MUSH_FACT
                    * 300.0
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM + 18.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                Some((150.0, 0.3)),
            )
        }),
        // seagrass, coastal patches
        (Seagrass, true, |_, col| {
            (
                MUSH_FACT
                    * 600.0
                    * if col.water_level < CONFIG.sea_level && (col.water_level - col.alt) < 3.0 {
                        1.0
                    } else {
                        0.0
                    },
                Some((150.0, 0.4)),
            )
        }),
        // scattered seaweed (temperate species)
        (SeaweedTemperate, true, |c, col| {
            (
                close(c.temp, CONFIG.temperate_temp, 0.8)
                    * MUSH_FACT
                    * 50.0
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM + 11.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                Some((500.0, 0.75)),
            )
        }),
        // scattered seaweed (tropical species)
        (SeaweedTropical, true, |c, col| {
            (
                close(c.temp, 1.0, 0.95)
                    * MUSH_FACT
                    * 50.0
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM + 11.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                Some((500.0, 0.75)),
            )
        }),
        // Caulerpa lentillifera algae patch
        (SeaGrapes, true, |_, col| {
            (
                MUSH_FACT
                    * 250.0
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM + 10.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                Some((100.0, 0.15)),
            )
        }),
        // Caulerpa prolifera algae patch
        (WavyAlgae, true, |_, col| {
            (
                MUSH_FACT
                    * 250.0
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM + 10.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                Some((100.0, 0.15)),
            )
        }),
        // Mermaids' fan algae patch
        (MermaidsFan, true, |c, col| {
            (
                close(c.temp, 1.0, 0.95)
                    * MUSH_FACT
                    * 500.0
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM + 10.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                Some((50.0, 0.10)),
            )
        }),
        // Sea anemones
        (SeaAnemone, true, |c, col| {
            (
                close(c.temp, CONFIG.temperate_temp, 0.8)
                    * MUSH_FACT
                    * 125.0
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM - 9.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                Some((100.0, 0.3)),
            )
        }),
        // Giant Kelp
        (GiantKelp, true, |c, col| {
            (
                close(c.temp, CONFIG.temperate_temp, 0.8)
                    * MUSH_FACT
                    * 220.0
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM - 9.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                Some((200.0, 0.4)),
            )
        }),
        // Bull Kelp
        (BullKelp, true, |c, col| {
            (
                close(c.temp, CONFIG.temperate_temp, 0.7)
                    * MUSH_FACT
                    * 300.0
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM + 3.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                Some((75.0, 0.3)),
            )
        }),
        // Stony Corals
        (StonyCoral, true, |c, col| {
            (
                close(c.temp, 1.0, 0.9)
                    * MUSH_FACT
                    * 160.0
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM + 10.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                Some((120.0, 0.4)),
            )
        }),
        // Soft Corals
        (SoftCoral, true, |c, col| {
            (
                close(c.temp, 1.0, 0.9)
                    * MUSH_FACT
                    * 120.0
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM + 10.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                Some((120.0, 0.4)),
            )
        }),
        // Seashells
        (Seashells, true, |c, col| {
            (
                (c.rockiness - 0.5).max(0.0)
                    * 1.0e-3
                    * if col.water_level < CONFIG.sea_level
                        && col.alt < col.water_level - DEPTH_WATER_NORM + 20.0
                    {
                        1.0
                    } else {
                        0.0
                    },
                None,
            )
        }),
        (Stones, true, |c, col| {
            (
                (c.rockiness - 0.5).max(0.0)
                    * 1.0e-3
                    * if col.alt < col.water_level - DEPTH_WATER_NORM {
                        1.0
                    } else {
                        0.0
                    },
                None,
            )
        }),
    ];

    canvas.foreach_col(|canvas, wpos2d, col| {
        let underwater = col.water_level > col.alt;

        let kind = scatter
            .iter()
            .enumerate()
            .find_map(|(i, (kind, is_underwater, f))| {
                let (density, patch) = f(canvas.chunk(), col);
                let is_patch = patch
                    .map(|(wavelen, threshold)| {
                        canvas
                            .index()
                            .noise
                            .scatter_nz
                            .get(
                                wpos2d
                                    .map(|e| e as f64 / wavelen as f64 + i as f64 * 43.0)
                                    .into_array(),
                            )
                            .abs()
                            > 1.0 - threshold as f64
                    })
                    .unwrap_or(true);
                if density > 0.0
                    && is_patch
                    && rng.gen::<f32>() < density //RandomField::new(i as u32).chance(Vec3::new(wpos2d.x, wpos2d.y, 0), density)
                    && underwater == *is_underwater
                {
                    Some(*kind)
                } else {
                    None
                }
            });

        if let Some(kind) = kind {
            let alt = col.alt as i32;

            // Find the intersection between ground and air, if there is one near the
            // surface
            if let Some(solid_end) = (-4..8)
                .find(|z| {
                    canvas
                        .get(Vec3::new(wpos2d.x, wpos2d.y, alt + z))
                        .is_solid()
                })
                .and_then(|solid_start| {
                    (1..8).map(|z| solid_start + z).find(|z| {
                        !canvas
                            .get(Vec3::new(wpos2d.x, wpos2d.y, alt + z))
                            .is_solid()
                    })
                })
            {
                canvas.map(Vec3::new(wpos2d.x, wpos2d.y, alt + solid_end), |block| {
                    block.with_sprite(kind)
                });
            }
        }
    });
}
