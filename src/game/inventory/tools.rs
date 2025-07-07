use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToolMaterial {
    Wood,
    Stone,
    Iron,
    Gold,
    Diamond,
    Rainbow,
}

const fn get_material_speed(material: ToolMaterial) -> f32 {
    match material {
        ToolMaterial::Wood => 2.0,
        ToolMaterial::Stone => 4.0,
        ToolMaterial::Iron => 6.0,
        ToolMaterial::Gold => 8.0,
        ToolMaterial::Diamond => 10.0,
        ToolMaterial::Rainbow => 12.0,
    }
}

const fn get_material_durability(material: ToolMaterial) -> u32 {
    match material {
        ToolMaterial::Wood => 64,
        ToolMaterial::Stone => 128,
        ToolMaterial::Iron => 256,
        ToolMaterial::Gold => 384,
        ToolMaterial::Diamond => 1680,
        ToolMaterial::Rainbow => 2048,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    Pickaxe,
    Shovel,
    Axe,
    Hoe,
    Sword,
}

pub fn tool_type_to_string(tool_type: ToolType) -> String {
    match tool_type {
        ToolType::Pickaxe => "pickaxe",
        ToolType::Shovel => "shovel",
        ToolType::Axe => "axe",
        ToolType::Hoe => "hoe",
        ToolType::Sword => "sword",
    }
    .to_string()
}

pub fn string_to_tool_type(s: &str) -> Result<ToolType, ()> {
    match s {
        "pickaxe" => Ok(ToolType::Pickaxe),
        "shovel" => Ok(ToolType::Shovel),
        "axe" => Ok(ToolType::Axe),
        "hoe" => Ok(ToolType::Hoe),
        "sword" => Ok(ToolType::Sword),
        _ => Err(()),
    }
}

pub fn string_to_tool_material(s: &str) -> Result<ToolMaterial, ()> {
    match s {
        "wood" => Ok(ToolMaterial::Wood),
        "stone" => Ok(ToolMaterial::Stone),
        "iron" => Ok(ToolMaterial::Iron),
        "gold" => Ok(ToolMaterial::Gold),
        "diamond" => Ok(ToolMaterial::Diamond),
        "rainbow" => Ok(ToolMaterial::Rainbow),
        _ => Err(()),
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ToolInfo {
    pub durability: u32,
    pub max_durability: u32,
    pub speed: f32,
    pub tool_type: ToolType,
}

impl ToolInfo {
    pub fn new_tool(tool: ToolType, material: ToolMaterial) -> Self {
        Self {
            durability: get_material_durability(material),
            max_durability: get_material_durability(material),
            speed: get_material_speed(material),
            tool_type: tool,
        }
    }

    pub fn reduce_info(&self) -> Self {
        Self {
            durability: self.max_durability,
            max_durability: self.max_durability,
            speed: self.speed,
            tool_type: self.tool_type,
        }
    }
}

impl Display for ToolInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            self.durability,
            self.max_durability,
            self.speed,
            tool_type_to_string(self.tool_type),
        )
    }
}

fn parse_tool_info(tokens: &[String]) -> Result<ToolInfo, ()> {
    let info_durability = tokens[0].parse::<u32>().map_err(|_| ())?;
    let info_max_durability = tokens[1].parse::<u32>().map_err(|_| ())?;
    let info_speed = tokens[2].parse::<f32>().map_err(|_| ())?;
    let info_tool_type = string_to_tool_type(&tokens[3])?;

    Ok(ToolInfo {
        durability: info_durability,
        max_durability: info_max_durability,
        speed: info_speed,
        tool_type: info_tool_type,
    })
}

fn parse_tool_info_material(tokens: &[String]) -> Result<ToolInfo, ()> {
    let material = string_to_tool_material(&tokens[0])?;
    let info_tool_type = string_to_tool_type(&tokens[1])?;
    Ok(ToolInfo::new_tool(info_tool_type, material))
}

pub fn string_to_tool_info(s: &str) -> Result<ToolInfo, ()> {
    let tokens: Vec<String> = s.split("/").map(|s| s.to_string()).collect();

    if tokens.len() == 4 {
        parse_tool_info(&tokens)
    } else if tokens.len() == 2 {
        parse_tool_info_material(&tokens)
    } else {
        Err(())
    }
}
