use crate::{
    assets::{self, AssetExt},
    recipe::{default_recipe_book, RecipeInput},
    trade::Good,
};
use assets_manager::AssetGuard;
use hashbrown::HashMap;
use lazy_static::lazy_static;
use serde::Deserialize;
use tracing::info;

#[derive(Debug)]
struct Entry {
    probability: f32,
    item: String,
}

type Entries = Vec<(String, f32)>;
const PRICING_DEBUG: bool = false;

#[derive(Default, Debug)]
pub struct TradePricing {
    tools: Entries,
    armor: Entries,
    potions: Entries,
    food: Entries,
    ingredients: Entries,
    other: Entries,
    coin_scale: f32,
    //    rng: ChaChaRng,

    // get amount of material per item
    material_cache: HashMap<String, (Good, f32)>,
}

lazy_static! {
    static ref TRADE_PRICING: TradePricing = TradePricing::read();
}

struct ProbabilityFile {
    pub content: Vec<(f32, String)>,
}

impl assets::Asset for ProbabilityFile {
    type Loader = assets::LoadFrom<Vec<(f32, String)>, assets::RonLoader>;

    const EXTENSION: &'static str = "ron";
}

impl From<Vec<(f32, String)>> for ProbabilityFile {
    fn from(content: Vec<(f32, String)>) -> ProbabilityFile { Self { content } }
}

#[derive(Debug, Deserialize)]
struct TradingPriceFile {
    pub loot_tables: Vec<(f32, String)>,
    pub good_scaling: Vec<(Good, f32)>, // the amount of Good equivalent to the most common item
}

impl assets::Asset for TradingPriceFile {
    type Loader = assets::LoadFrom<TradingPriceFile, assets::RonLoader>;

    const EXTENSION: &'static str = "ron";
}

#[derive(Debug)]
struct RememberedRecipe {
    output: String,
    amount: u32,
    material_cost: f32,
    input: Vec<(String, u32)>,
}

impl TradePricing {
    const COIN_ITEM: &'static str = "common.items.utility.coins";
    const CRAFTING_FACTOR: f32 = 0.95;
    // increase price a bit compared to sum of ingredients
    const INVEST_FACTOR: f32 = 0.33;
    const UNAVAILABLE_PRICE: f32 = 1000000.0;

    // add this much of a non-consumed crafting tool price

    fn get_list(&self, good: Good) -> &Entries {
        match good {
            Good::Armor => &self.armor,
            Good::Tools => &self.tools,
            Good::Potions => &self.potions,
            Good::Food => &self.food,
            Good::Ingredients => &self.ingredients,
            _ => panic!("invalid good"),
        }
    }

    fn get_list_mut(&mut self, good: Good) -> &mut Entries {
        match good {
            Good::Armor => &mut self.armor,
            Good::Tools => &mut self.tools,
            Good::Potions => &mut self.potions,
            Good::Food => &mut self.food,
            Good::Ingredients => &mut self.ingredients,
            _ => panic!("invalid good"),
        }
    }

    fn get_list_by_path(&self, name: &str) -> &Entries {
        match name {
            _ if name.starts_with("common.items.crafting_ing.") => &self.ingredients,
            _ if name.starts_with("common.items.armor.") => &self.armor,
            _ if name.starts_with("common.items.glider.") => &self.other,
            _ if name.starts_with("common.items.weapons.") => &self.tools,
            _ if name.starts_with("common.items.consumable.") => &self.potions,
            _ if name.starts_with("common.items.food.") => &self.food,
            _ if name.starts_with("common.items.utility.") => &self.other,
            _ if name.starts_with("common.items.boss_drops.") => &self.other,
            _ if name.starts_with("common.items.ore.") => &self.ingredients,
            _ if name.starts_with("common.items.flowers.") => &self.ingredients,
            _ if name.starts_with("common.items.crafting_tools.") => &self.other,
            _ => {
                info!("unknown loot item {}", name);
                &self.other
            },
        }
    }

    fn get_list_by_path_mut(&mut self, name: &str) -> &mut Entries {
        match name {
            _ if name.starts_with("common.items.crafting_ing.") => &mut self.ingredients,
            _ if name.starts_with("common.items.armor.") => &mut self.armor,
            _ if name.starts_with("common.items.glider.") => &mut self.other,
            _ if name.starts_with("common.items.weapons.") => &mut self.tools,
            _ if name.starts_with("common.items.consumable.") => &mut self.potions,
            _ if name.starts_with("common.items.food.") => &mut self.food,
            _ if name.starts_with("common.items.utility.") => &mut self.other,
            _ if name.starts_with("common.items.boss_drops.") => &mut self.other,
            _ if name.starts_with("common.items.ore.") => &mut self.ingredients,
            _ if name.starts_with("common.items.flowers.") => &mut self.ingredients,
            _ if name.starts_with("common.items.crafting_tools.") => &mut self.other,
            _ => {
                info!("unknown loot item {}", name);
                &mut self.other
            },
        }
    }

    fn read() -> Self {
        fn add(entryvec: &mut Entries, itemname: &str, probability: f32) {
            let val = entryvec.iter_mut().find(|j| *j.0 == *itemname);
            if let Some(r) = val {
                if PRICING_DEBUG {
                    info!("Update {} {}+{}", r.0, r.1, probability);
                }
                r.1 += probability;
            } else {
                if PRICING_DEBUG {
                    info!("New {} {}", itemname, probability);
                }
                entryvec.push((itemname.to_string(), probability));
            }
        }
        fn sort_and_normalize(entryvec: &mut Entries, scale: f32) {
            if !entryvec.is_empty() {
                entryvec.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                let rescale = scale / entryvec.last().unwrap().1;
                for i in entryvec.iter_mut() {
                    i.1 *= rescale;
                }
            }
        }
        fn get_scaling(contents: &AssetGuard<TradingPriceFile>, good: Good) -> f32 {
            contents
                .good_scaling
                .iter()
                .find(|i| i.0 == good)
                .map(|i| i.1)
                .unwrap_or(1.0)
        }

        let mut result = TradePricing::default();
        let files = TradingPriceFile::load_expect("common.item_price_calculation");
        let contents = files.read();
        for i in contents.loot_tables.iter() {
            if PRICING_DEBUG {
                info!(?i);
            }
            let loot = ProbabilityFile::load_expect(&i.1);
            for j in loot.read().content.iter() {
                add(&mut result.get_list_by_path_mut(&j.1), &j.1, i.0 * j.0);
            }
        }

        // Apply recipe book
        let book = default_recipe_book().read();
        let mut ordered_recipes: Vec<RememberedRecipe> = Vec::new();
        for (_, r) in book.iter() {
            ordered_recipes.push(RememberedRecipe {
                output: r.output.0.id().into(),
                amount: r.output.1,
                material_cost: TradePricing::UNAVAILABLE_PRICE,
                input: r
                    .inputs
                    .iter()
                    .filter(|i| matches!(i.0, RecipeInput::Item(_)))
                    .map(|i| {
                        (
                            if let RecipeInput::Item(it) = &i.0 {
                                it.id().into()
                            } else {
                                panic!("recipe logic broken");
                            },
                            i.1,
                        )
                    })
                    .collect(),
            });
        }
        // look up price (inverse frequency) of an item
        fn price_lookup(s: &TradePricing, name: &str) -> f32 {
            let vec = s.get_list_by_path(name);
            vec.iter()
                .find(|(n, _)| n == name)
                .map(|(_, freq)| 1.0 / freq)
                // even if we multiply by INVEST_FACTOR we need to remain above UNAVAILABLE_PRICE (add 1.0 to compensate rounding errors)
                .unwrap_or(TradePricing::UNAVAILABLE_PRICE/TradePricing::INVEST_FACTOR+1.0)
        }
        fn calculate_material_cost(s: &TradePricing, r: &RememberedRecipe) -> f32 {
            r.input
                .iter()
                .map(|(name, amount)| {
                    price_lookup(s, name) * (*amount as f32).max(TradePricing::INVEST_FACTOR)
                })
                .sum()
        }
        // re-look up prices and sort the vector by ascending material cost, return
        // whether first cost is finite
        fn price_sort(s: &TradePricing, vec: &mut Vec<RememberedRecipe>) -> bool {
            if !vec.is_empty() {
                for e in vec.iter_mut() {
                    e.material_cost = calculate_material_cost(s, e);
                }
                vec.sort_by(|a, b| a.material_cost.partial_cmp(&b.material_cost).unwrap());
                //info!(?vec);
                vec.first().unwrap().material_cost < TradePricing::UNAVAILABLE_PRICE
            } else {
                false
            }
        }
        // re-evaluate prices based on crafting tables
        // (start with cheap ones to avoid changing material prices after evaluation)
        while price_sort(&result, &mut ordered_recipes) {
            ordered_recipes.retain(|e| {
                if e.material_cost < TradePricing::UNAVAILABLE_PRICE {
                    let actual_cost = calculate_material_cost(&result, e);
                    add(
                        &mut result.get_list_by_path_mut(&e.output),
                        &e.output,
                        (e.amount as f32) / actual_cost * TradePricing::CRAFTING_FACTOR,
                    );
                    false
                } else {
                    true
                }
            });
            //info!(?ordered_recipes);
        }

        let good_list = [
            Good::Armor,
            Good::Tools,
            Good::Potions,
            Good::Food,
            Good::Ingredients,
        ];
        for &g in good_list.iter() {
            sort_and_normalize(result.get_list_mut(g), get_scaling(&contents, g));
            let mut materials = result
                .get_list(g)
                .iter()
                .map(|i| (i.0.clone(), (g, 1.0 / i.1)))
                .collect::<Vec<_>>();
            result.material_cache.extend(materials.drain(..));
        }
        result.coin_scale = get_scaling(&contents, Good::Coin);
        result
    }

    fn random_item_impl(&self, good: Good, amount: f32) -> String {
        if good == Good::Coin {
            TradePricing::COIN_ITEM.into()
        } else {
            let table = self.get_list(good);
            let upper = table.len();
            let lower = table
                .iter()
                .enumerate()
                .find(|i| i.1.1 * amount >= 1.0)
                .map(|i| i.0)
                .unwrap_or(upper - 1);
            let index = (rand::random::<f32>() * ((upper - lower) as f32)).floor() as usize + lower;
            //.gen_range(lower..upper);
            table.get(index).unwrap().0.clone()
        }
    }

    pub fn random_item(good: Good, amount: f32) -> String {
        TRADE_PRICING.random_item_impl(good, amount)
    }

    pub fn get_material(item: &str) -> (Good, f32) {
        if item == TradePricing::COIN_ITEM {
            (Good::Coin, 1.0 / TRADE_PRICING.coin_scale)
        } else {
            TRADE_PRICING
                .material_cache
                .get(item)
                .cloned()
                .unwrap_or((Good::Terrain(crate::terrain::BiomeKind::Void), 0.0))
        }
    }

    #[cfg(test)]
    fn instance() -> &'static Self { &TRADE_PRICING }

    #[cfg(test)]
    fn print_sorted(&self) {
        fn printvec(x: &str, e: &[(String, f32)]) {
            println!("{}", x);
            for i in e.iter() {
                println!("{} {}", i.0, 1.0 / i.1);
            }
        }
        printvec("Armor", &self.armor);
        printvec("Tools", &self.tools);
        printvec("Potions", &self.potions);
        printvec("Food", &self.food);
        printvec("Ingredients", &self.ingredients);
        println!("{} {}", TradePricing::COIN_ITEM, self.coin_scale);
    }
}

#[cfg(test)]
mod tests {
    use crate::{comp::inventory::trade_pricing::TradePricing, trade::Good};
    use tracing::{info, Level};
    use tracing_subscriber::{
        filter::{EnvFilter, LevelFilter},
        FmtSubscriber,
    };

    fn init() {
        FmtSubscriber::builder()
            .with_max_level(Level::ERROR)
            .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
            .init();
    }

    #[test]
    fn test_prices() {
        init();
        info!("init");

        TradePricing::instance().print_sorted();
        info!("Armor 5 {}", TradePricing::random_item(Good::Armor, 5.0));
        info!("Armor 5 {}", TradePricing::random_item(Good::Armor, 5.0));
        info!("Armor 5 {}", TradePricing::random_item(Good::Armor, 5.0));
        info!("Armor 5 {}", TradePricing::random_item(Good::Armor, 5.0));
        info!("Armor 5 {}", TradePricing::random_item(Good::Armor, 5.0));
    }
}
