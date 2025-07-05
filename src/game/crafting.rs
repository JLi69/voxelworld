use super::inventory::{
    items_match, multiply_items, reduce_amt, string_to_item_err, Inventory, Item,
};
use crate::{
    impfile,
    voxel::{Block, FULL_BLOCK, SLAB, STAIR},
};
use std::collections::HashMap;

pub type ItemAliases = HashMap<String, Item>;

//Adds slab and stair items
fn add_block_variants(aliases: &mut ItemAliases, name: &str, item: Item) {
    if let Item::Block(block, _) = item {
        if block.shape() != FULL_BLOCK {
            return;
        }
        if block.is_fluid() {
            return;
        }
        if block.is_flat_item() {
            return;
        }
        if block.non_voxel_geometry() {
            return;
        }
        let mut slab = block;
        slab.set_shape(SLAB);
        let slab_name = format!("{name}_slab");
        aliases.insert(slab_name, Item::Block(slab, 1));

        let mut stair = block;
        stair.set_shape(STAIR);
        let stair_name = format!("{name}_stair");
        aliases.insert(stair_name, Item::Block(stair, 1));
    }
}

//Loads item aliases from impfile
pub fn load_item_aliases(path: &str) -> ItemAliases {
    let mut aliases = ItemAliases::new();

    let entries = impfile::parse_file(path);
    for e in entries {
        let vars = e.get_all_vars();
        for (name, val) in vars {
            if let Ok(item) = string_to_item_err(&val) {
                let reduced = reduce_amt(item);
                aliases.insert(name.clone(), reduced);
                add_block_variants(&mut aliases, &name, item);
            }
        }
    }

    aliases
}

pub struct Recipe {
    ingredients: Inventory,
    output: Item,
    reflect: bool,
}

impl Recipe {
    pub fn from_entry(entry: &impfile::Entry, item_aliases: &ItemAliases) -> Result<Self, ()> {
        let w = entry.get_var("width").parse::<usize>().unwrap_or(1);
        let h = entry.get_var("height").parse::<usize>().unwrap_or(1);
        let parsed_ingredients: Vec<Item> = entry
            .get_var("items")
            .split("|")
            .filter_map(|s| {
                let aliased = item_aliases.get(s);
                if aliased.is_some() {
                    return aliased.cloned();
                }
                string_to_item_err(s).ok()
            })
            .map(reduce_amt)
            .chain(std::iter::repeat(Item::Empty))
            .take(w * h)
            .collect();
        let output_amt = entry.get_var("amt").parse().unwrap_or(1);
        let output_str = entry.get_var("output");
        let parsed_output = if let Some(item) = item_aliases.get(&output_str) {
            *item
        } else {
            string_to_item_err(&output_str)?
        };
        let multiplied_output = multiply_items(parsed_output, output_amt);

        let mut grid = Inventory::empty_with_sz(w, h);
        for (i, item) in parsed_ingredients.iter().enumerate() {
            let ix = i % w;
            let iy = i / w;
            grid.set_item(ix, iy, *item);
        }

        Ok(Self {
            ingredients: grid,
            reflect: entry.get_var("reflect").parse::<bool>().unwrap_or(false),
            output: multiplied_output,
        })
    }

    fn check_match_pos(&self, crafting: &Inventory, x: usize, y: usize) -> bool {
        let w = self.ingredients.w();
        let h = self.ingredients.h();
        let xrange = x..(x + w);
        let yrange = y..(y + h);
        for ix in 0..crafting.w() {
            for iy in 0..crafting.h() {
                let matching = if xrange.contains(&ix) && yrange.contains(&iy) {
                    let ingredient = self.ingredients.get_item(ix - x, iy - y);
                    items_match(ingredient, crafting.get_item(ix, iy))
                } else {
                    crafting.get_item(ix, iy).is_empty()
                };

                if !matching {
                    return false;
                }
            }
        }
        true
    }

    fn check_match_pos_reflected(&self, crafting: &Inventory, x: usize, y: usize) -> bool {
        let w = self.ingredients.w();
        let h = self.ingredients.h();
        let xrange = x..(x + w);
        let yrange = y..(y + h);
        for ix in 0..crafting.w() {
            for iy in 0..crafting.h() {
                let matching = if xrange.contains(&ix) && yrange.contains(&iy) {
                    let ingredient = self.ingredients.get_item(w - 1 - (ix - x), iy - y);
                    items_match(ingredient, crafting.get_item(ix, iy))
                } else {
                    crafting.get_item(ix, iy).is_empty()
                };

                if !matching {
                    return false;
                }
            }
        }
        true
    }

    pub fn check_match(&self, crafting: &Inventory) -> bool {
        let w = self.ingredients.w();
        let h = self.ingredients.h();
        for x in 0..=(crafting.w() - w) {
            for y in 0..=(crafting.h() - h) {
                if self.check_match_pos(crafting, x, y) {
                    return true;
                }
                if self.reflect && self.check_match_pos_reflected(crafting, x, y) {
                    return true;
                }
            }
        }
        false
    }
}

//Table of crafting recipes
pub struct RecipeTable {
    recipes: Vec<Recipe>,
}

fn generate_slab_recipe(block: Block) -> Recipe {
    let mut slab = block;
    slab.set_shape(SLAB);
    let mut grid = Inventory::empty_with_sz(3, 1);
    grid.set_item(0, 0, Item::Block(block, 1));
    grid.set_item(1, 0, Item::Block(block, 1));
    grid.set_item(2, 0, Item::Block(block, 1));
    Recipe {
        ingredients: grid,
        output: Item::Block(slab, 6),
        reflect: false,
    }
}

//Generates stair recipe
fn generate_stair_recipe(block: Block) -> Recipe {
    let mut stair = block;
    stair.set_shape(STAIR);
    stair.set_orientation(2);
    let mut grid = Inventory::empty_with_sz(3, 3);
    grid.set_item(0, 0, Item::Block(block, 1));
    grid.set_item(0, 1, Item::Block(block, 1));
    grid.set_item(1, 1, Item::Block(block, 1));
    grid.set_item(0, 2, Item::Block(block, 1));
    grid.set_item(1, 2, Item::Block(block, 1));
    grid.set_item(2, 2, Item::Block(block, 1));
    Recipe {
        ingredients: grid,
        output: Item::Block(stair, 8),
        reflect: true,
    }
}

impl RecipeTable {
    pub fn new() -> Self {
        Self { recipes: vec![] }
    }

    pub fn load_recipes(&mut self, item_alias_path: &str, recipe_path: &str) {
        let item_aliases = load_item_aliases(item_alias_path);
        self.recipes = impfile::parse_file(recipe_path)
            .iter()
            .filter_map(|e| Recipe::from_entry(e, &item_aliases).ok())
            .collect();
        let mut auto_generated_recipes = vec![];
        for item in item_aliases.values().copied() {
            if let Item::Block(block, _) = item {
                if block.shape() != FULL_BLOCK {
                    continue;
                }
                if block.is_fluid() {
                    continue;
                }
                if block.is_flat_item() {
                    continue;
                }
                if block.non_voxel_geometry() {
                    continue;
                }
                //Generate slab recipes
                let slab_recipe = generate_slab_recipe(block);
                auto_generated_recipes.push(slab_recipe);
                //Generate stair recipes
                let stair_recipe = generate_stair_recipe(block);
                auto_generated_recipes.push(stair_recipe);
            }
        }
        self.recipes.extend(auto_generated_recipes);
    }

    //Returns option for an output
    pub fn get_output(&self, crafting: &Inventory) -> Option<Item> {
        for recipe in &self.recipes {
            if recipe.check_match(crafting) {
                return Some(recipe.output);
            }
        }

        None
    }
}
