use super::{ItemAliases, Recipe};
use crate::game::inventory::{
    string_to_item_err,
    tools::{ToolInfo, ToolMaterial, ToolType},
    Inventory, Item,
};

const PICKAXE_GRID: &str = "*|*|*|empty|stick|empty|empty|stick|empty";
const SHOVEL_GRID: &str = "*|stick|stick";
const AXE_GRID: &str = "*|*|*|stick|empty|stick";
const HOE_GRID: &str = "*|*|empty|stick|empty|stick";
const SWORD_GRID: &str = "*|*|stick";

fn recipe_grid_from_str(
    recipe_str: &str,
    w: usize,
    h: usize,
    ingredient: Item,
    item_aliases: &ItemAliases,
) -> Inventory {
    let mut grid = Inventory::empty_with_sz(w, h);

    let items: Vec<Item> = recipe_str
        .split("|")
        .filter_map(|s| {
            if s == "*" {
                return Some(ingredient);
            }
            let aliased = item_aliases.get(s);
            if aliased.is_some() {
                return aliased.copied();
            }
            string_to_item_err(s).ok()
        })
        .chain(std::iter::repeat(Item::Empty))
        .take(w * h)
        .collect();

    for ix in 0..w {
        for iy in 0..h {
            let index = iy * w + ix;
            grid.set_item(ix, iy, items[index]);
        }
    }

    grid
}

pub fn generate_tool_recipes(
    ingredient: &str,
    material: ToolMaterial,
    start_id: u16,
    item_aliases: &ItemAliases,
) -> Vec<Recipe> {
    let mut recipes = vec![];

    let ingredient = if let Some(ingredient) = item_aliases.get(ingredient).copied() {
        ingredient
    } else if let Ok(ingredient) = string_to_item_err(ingredient) {
        ingredient
    } else {
        return recipes;
    };

    let pickaxe = Recipe {
        ingredients: recipe_grid_from_str(PICKAXE_GRID, 3, 3, ingredient, item_aliases),
        reflect: false,
        shapeless: false,
        output: Item::Tool(start_id, ToolInfo::new_tool(ToolType::Pickaxe, material)),
    };
    recipes.push(pickaxe);

    let shovel = Recipe {
        ingredients: recipe_grid_from_str(SHOVEL_GRID, 1, 3, ingredient, item_aliases),
        reflect: false,
        shapeless: false,
        output: Item::Tool(start_id + 1, ToolInfo::new_tool(ToolType::Shovel, material)),
    };
    recipes.push(shovel);

    let axe = Recipe {
        ingredients: recipe_grid_from_str(AXE_GRID, 2, 3, ingredient, item_aliases),
        reflect: true,
        shapeless: false,
        output: Item::Tool(start_id + 2, ToolInfo::new_tool(ToolType::Axe, material)),
    };
    recipes.push(axe);

    let hoe = Recipe {
        ingredients: recipe_grid_from_str(HOE_GRID, 2, 3, ingredient, item_aliases),
        reflect: true,
        shapeless: false,
        output: Item::Tool(start_id + 3, ToolInfo::new_tool(ToolType::Hoe, material)),
    };
    recipes.push(hoe);

    let sword = Recipe {
        ingredients: recipe_grid_from_str(SWORD_GRID, 1, 3, ingredient, item_aliases),
        reflect: true,
        shapeless: false,
        output: Item::Tool(start_id + 4, ToolInfo::new_tool(ToolType::Sword, material)),
    };
    recipes.push(sword);

    recipes
}
