#![deny(clippy::clone_on_ref_ptr)]

use hashbrown::HashMap;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::Serialize;
use std::{error::Error, fs::File, io::Write};
use structopt::StructOpt;

use comp::item::{
    armor::{ArmorKind, Protection},
    ItemDesc, ItemKind, ItemTag, Quality,
};
use veloren_common::{assets::ASSETS_PATH, comp};

#[derive(StructOpt)]
struct Cli {
    /// Available arguments: "armor_stats", "weapon_stats"
    function: String,
}

#[derive(Serialize)]
struct FakeItemDef {
    name: String,
    description: String,
    kind: ItemKind,
    quality: Quality,
    tags: Vec<ItemTag>,
}

impl FakeItemDef {
    fn new(
        name: String,
        description: String,
        kind: ItemKind,
        quality: Quality,
        tags: Vec<ItemTag>,
    ) -> Self {
        Self {
            name,
            description,
            kind,
            quality,
            tags,
        }
    }
}

fn armor_stats() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path("armorstats.csv")?;

    let headers: HashMap<String, usize> = rdr
        .headers()
        .expect("Failed to read CSV headers")
        .iter()
        .enumerate()
        .map(|(i, x)| (x.to_string(), i))
        .collect();

    for record in rdr.records() {
        for item in comp::item::Item::new_from_asset_glob("common.items.armor.*")
            .expect("Failed to iterate over item folders!")
        {
            match item.kind() {
                comp::item::ItemKind::Armor(armor) => {
                    match armor.kind {
                        ArmorKind::Bag(_) => {
                            continue;
                        },
                        _ => {},
                    }

                    if let Ok(ref record) = record {
                        if item.item_definition_id()
                            == record.get(headers["Path"]).expect("No file path in csv?")
                        {
                            let protection =
                                if let Some(protection_raw) = record.get(headers["Protection"]) {
                                    if protection_raw == "Invincible" {
                                        Protection::Invincible
                                    } else {
                                        let value: f32 = protection_raw.parse().unwrap();
                                        Protection::Normal(value)
                                    }
                                } else {
                                    eprintln!(
                                        "Could not unwrap protection value for {:?}",
                                        item.item_definition_id()
                                    );
                                    Protection::Normal(0.0)
                                };

                            let poise_resilience = if let Some(poise_resilience_raw) =
                                record.get(headers["Poise Resilience"])
                            {
                                if poise_resilience_raw == "Invincible" {
                                    Protection::Invincible
                                } else {
                                    let value: f32 = poise_resilience_raw.parse().unwrap();
                                    Protection::Normal(value)
                                }
                            } else {
                                eprintln!(
                                    "Could not unwrap poise protection value for {:?}",
                                    item.item_definition_id()
                                );
                                Protection::Normal(0.0)
                            };

                            let kind = armor.kind.clone();
                            let armor =
                                comp::item::armor::Armor::new(kind, protection, poise_resilience);
                            let quality = if let Some(quality_raw) = record.get(headers["Quality"])
                            {
                                match quality_raw {
                                    "Low" => comp::item::Quality::Low,
                                    "Common" => comp::item::Quality::Common,
                                    "Moderate" => comp::item::Quality::Moderate,
                                    "High" => comp::item::Quality::High,
                                    "Epic" => comp::item::Quality::Epic,
                                    "Legendary" => comp::item::Quality::Legendary,
                                    "Artifact" => comp::item::Quality::Artifact,
                                    "Debug" => comp::item::Quality::Debug,
                                    _ => {
                                        eprintln!(
                                            "Unknown quality variant for {:?}",
                                            item.item_definition_id()
                                        );
                                        comp::item::Quality::Debug
                                    },
                                }
                            } else {
                                eprintln!(
                                    "Could not unwrap quality for {:?}",
                                    item.item_definition_id()
                                );
                                comp::item::Quality::Debug
                            };

                            let description = record
                                .get(headers["Description"])
                                .expect(&format!(
                                    "Error unwrapping description for {:?}",
                                    item.item_definition_id()
                                ))
                                .replace("\\'", "'");

                            let fake_item = FakeItemDef::new(
                                item.name().to_string(),
                                description.to_string(),
                                ItemKind::Armor(armor),
                                quality,
                                item.tags().to_vec(),
                            );

                            let pretty_config = PrettyConfig::new()
                                .with_depth_limit(4)
                                .with_separate_tuple_members(true)
                                .with_decimal_floats(true)
                                .with_enumerate_arrays(true);

                            let mut path = ASSETS_PATH.clone();
                            for part in item.item_definition_id().split(".") {
                                path.push(part);
                            }
                            path.set_extension("ron");

                            let path_str = path.to_str().expect("File path not unicode?!");
                            let mut writer = File::create(path_str)?;
                            write!(
                                writer,
                                "ItemDef{}",
                                to_string_pretty(&fake_item, pretty_config)?.replace("\\'", "'")
                            )?;
                        }
                    }
                },
                _ => println!("Skipping non-armor item: {:?}\n", item),
            }
        }
    }

    Ok(())
}

fn weapon_stats() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path("weaponstats.csv")?;

    let headers: HashMap<String, usize> = rdr
        .headers()
        .expect("Failed to read CSV headers")
        .iter()
        .enumerate()
        .map(|(i, x)| (x.to_string(), i))
        .collect();

    for record in rdr.records() {
        let mut items: Vec<comp::Item> = comp::Item::new_from_asset_glob("common.items.weapons.*")
            .expect("Failed to iterate over item folders!");
        items.extend(
            comp::Item::new_from_asset_glob("common.items.npc_weapons.*")
                .expect("Failed to iterate over npc weapons!"),
        );
        for item in items.iter() {
            match item.kind() {
                comp::item::ItemKind::Tool(tool) => {
                    if let Ok(ref record) = record {
                        if item.item_definition_id()
                            == record.get(headers["Path"]).expect("No file path in csv?")
                        {
                            let kind = tool.kind;
                            let equip_time_secs: f32 = record
                                .get(headers["Equip Time (s)"])
                                .expect(&format!(
                                    "Error unwrapping equip time for {:?}",
                                    item.item_definition_id()
                                ))
                                .parse()
                                .expect(&format!("Not a u32? {:?}", item.item_definition_id()));
                            let power: f32 = record
                                .get(headers["Power"])
                                .expect(&format!(
                                    "Error unwrapping power for {:?}",
                                    item.item_definition_id()
                                ))
                                .parse()
                                .expect(&format!("Not a f32? {:?}", item.item_definition_id()));
                            let poise_strength: f32 = record
                                .get(headers["Poise Strength"])
                                .expect(&format!(
                                    "Error unwrapping poise power for {:?}",
                                    item.item_definition_id()
                                ))
                                .parse()
                                .expect(&format!("Not a f32? {:?}", item.item_definition_id()));

                            let speed: f32 = record
                                .get(headers["Speed"])
                                .expect(&format!(
                                    "Error unwrapping speed for {:?}",
                                    item.item_definition_id()
                                ))
                                .parse()
                                .expect(&format!("Not a f32? {:?}", item.item_definition_id()));

                            let hands = if let Some(hands_raw) = record.get(headers["Hands"]) {
                                match hands_raw {
                                    "One" | "1" | "1h" => comp::item::tool::Hands::One,
                                    "Two" | "2" | "2h" => comp::item::tool::Hands::Two,
                                    _ => {
                                        eprintln!(
                                            "Unknown hand variant for {:?}",
                                            item.item_definition_id()
                                        );
                                        comp::item::tool::Hands::Two
                                    },
                                }
                            } else {
                                eprintln!(
                                    "Could not unwrap hand for {:?}",
                                    item.item_definition_id()
                                );
                                comp::item::tool::Hands::Two
                            };

                            let crit_chance: f32 = record
                                .get(headers["Crit Chance"])
                                .expect(&format!(
                                    "Error unwrapping crit_chance for {:?}",
                                    item.item_definition_id()
                                ))
                                .parse()
                                .expect(&format!("Not a f32? {:?}", item.item_definition_id()));

                            let crit_mult: f32 = record
                                .get(headers["Crit Mult"])
                                .expect(&format!(
                                    "Error unwrapping crit_mult for {:?}",
                                    item.item_definition_id()
                                ))
                                .parse()
                                .expect(&format!("Not a f32? {:?}", item.item_definition_id()));

                            let tool = comp::item::tool::Tool::new(
                                kind,
                                hands,
                                equip_time_secs,
                                power,
                                poise_strength,
                                speed,
                                crit_chance,
                                crit_mult,
                            );

                            let quality = if let Some(quality_raw) = record.get(headers["Quality"])
                            {
                                match quality_raw {
                                    "Low" => comp::item::Quality::Low,
                                    "Common" => comp::item::Quality::Common,
                                    "Moderate" => comp::item::Quality::Moderate,
                                    "High" => comp::item::Quality::High,
                                    "Epic" => comp::item::Quality::Epic,
                                    "Legendary" => comp::item::Quality::Legendary,
                                    "Artifact" => comp::item::Quality::Artifact,
                                    "Debug" => comp::item::Quality::Debug,
                                    _ => {
                                        eprintln!(
                                            "Unknown quality variant for {:?}",
                                            item.item_definition_id()
                                        );
                                        comp::item::Quality::Debug
                                    },
                                }
                            } else {
                                eprintln!(
                                    "Could not unwrap quality for {:?}",
                                    item.item_definition_id()
                                );
                                comp::item::Quality::Debug
                            };

                            let description = record.get(headers["Description"]).expect(&format!(
                                "Error unwrapping description for {:?}",
                                item.item_definition_id()
                            ));

                            let fake_item = FakeItemDef::new(
                                item.name().to_string(),
                                description.to_string(),
                                ItemKind::Tool(tool),
                                quality,
                                item.tags().to_vec(),
                            );

                            let pretty_config = PrettyConfig::new()
                                .with_depth_limit(4)
                                .with_separate_tuple_members(true)
                                .with_decimal_floats(true)
                                .with_enumerate_arrays(true);

                            let mut path = ASSETS_PATH.clone();
                            for part in item.item_definition_id().split(".") {
                                path.push(part);
                            }
                            path.set_extension("ron");

                            let path_str = path.to_str().expect("File path not unicode?!");
                            let mut writer = File::create(path_str)?;
                            write!(
                                writer,
                                "ItemDef{}",
                                to_string_pretty(&fake_item, pretty_config)?.replace("\\'", "'")
                            )?;
                        }
                    }
                },
                _ => println!("Skipping non-weapon item: {:?}\n", item),
            }
        }
    }

    Ok(())
}

fn main() {
    let args = Cli::from_args();
    if args.function.eq_ignore_ascii_case("armor-stats") {
        if get_input(
            "
-------------------------------------------------------------------------------
|                                 DISCLAIMER                                  |
-------------------------------------------------------------------------------
|                                                                             |
|   This script will wreck the RON files for armor if it messes up.           |
|   You might want to save a back up of the weapon files or be prepared to    |
|   use `git checkout HEAD -- ../assets/common/items/armor/*` if needed.      |
|   If this script does mess up your files, please fix it. Otherwise your     |
|   files will be yeeted away and you will get a bonk on the head.            |
|                                                                             |
-------------------------------------------------------------------------------

In order for this script to work, you need to have first run the csv exporter.
Once you have armorstats.csv you can make changes to stats, quality, and
description in your preferred editor. Save the csv file and then run this
script to import your changes back to RON.

Would you like to continue? (y/n)
> ",
        )
        .to_lowercase()
            == "y".to_string()
        {
            if let Err(e) = armor_stats() {
                println!("Error: {}\n", e)
            }
        }
    } else if args.function.eq_ignore_ascii_case("weapon-stats") {
        if get_input(
            "
-------------------------------------------------------------------------------
|                                 DISCLAIMER                                  |
-------------------------------------------------------------------------------
|                                                                             |
|   This script will wreck the RON files for weapons if it messes up.         |
|   You might want to save a back up of the weapon files or be prepared to    |
|   use `git checkout HEAD -- ../assets/common/items/weapons/*` if needed.    |
|   If this script does mess up your files, please fix it. Otherwise your     |
|   files will be yeeted away and you will get a bonk on the head.            |
|                                                                             |
-------------------------------------------------------------------------------

In order for this script to work, you need to have first run the csv exporter.
Once you have weaponstats.csv you can make changes to stats, quality, and
description in your preferred editor. Save the csv file and then run this
script to import your changes back to RON.

Would you like to continue? (y/n)
> ",
        )
        .to_lowercase()
            == "y".to_string()
        {
            if let Err(e) = weapon_stats() {
                println!("Error: {}\n", e)
            }
        }
    } else {
        println!("Invalid argument, available arguments:\n\"armor-stats\"\n\"weapon-stats\"\n\"")
    }
}

pub fn get_input(prompt: &str) -> String {
    // Function to get input from the user with a prompt
    let mut input = String::new();
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Error reading input");

    String::from(input.trim())
}
