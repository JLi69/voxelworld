use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToolMaterial {
    Wood,
    Stone,
    Iron,
    Gold,
    Diamond,
    Aqua,
    Rainbow,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToolType {
    Pickaxe,
    Shovel,
    Axe,
    Hoe,
    Sword,
}

const fn get_material_speed(material: ToolMaterial) -> f32 {
    match material {
        ToolMaterial::Wood => 2.0,
        ToolMaterial::Stone => 4.0,
        ToolMaterial::Iron => 6.0,
        ToolMaterial::Gold => 8.0,
        ToolMaterial::Diamond => 10.0,
        ToolMaterial::Rainbow | ToolMaterial::Aqua => 12.0,
    }
}

const fn get_material_durability(material: ToolMaterial) -> u32 {
    match material {
        ToolMaterial::Wood => 64,
        ToolMaterial::Stone => 128,
        ToolMaterial::Iron => 256,
        ToolMaterial::Gold => 384,
        ToolMaterial::Diamond => 1680,
        ToolMaterial::Aqua => 800,
        ToolMaterial::Rainbow => 2048,
    }
}

const fn get_material_attack(material: ToolMaterial) -> u32 {
    match material {
        ToolMaterial::Wood => 0,
        ToolMaterial::Stone => 1,
        ToolMaterial::Iron => 2,
        ToolMaterial::Gold => 3,
        ToolMaterial::Diamond => 4,
        ToolMaterial::Rainbow | ToolMaterial::Aqua => 5,
    }
}

const fn get_tool_base(tool_type: ToolType) -> u32 {
    match tool_type {
        ToolType::Pickaxe => 2,
        ToolType::Axe => 3,
        ToolType::Sword => 4,
        _ => 1,
    }
}

const fn tool_atk_multiplier(tool_type: ToolType) -> f32 {
    match tool_type {
        ToolType::Pickaxe => 0.5,
        ToolType::Axe => 0.75,
        ToolType::Sword => 1.0,
        _ => 0.0,
    }
}

const fn tool_atk(tool_type: ToolType, material: ToolMaterial) -> u32 {
    let multiplier = tool_atk_multiplier(tool_type);
    let add = multiplier * get_material_attack(material) as f32;
    get_tool_base(tool_type) + add as u32
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
        "aqua" => Ok(ToolMaterial::Aqua),
        "rainbow" => Ok(ToolMaterial::Rainbow),
        _ => Err(()),
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ToolInfo {
    pub durability: u32,
    pub max_durability: u32,
    pub speed: f32,
    pub tool_type: ToolType,
    pub attack: u32,
}

impl ToolInfo {
    pub fn new_tool(tool: ToolType, material: ToolMaterial) -> Self {
        Self {
            durability: get_material_durability(material),
            max_durability: get_material_durability(material),
            speed: get_material_speed(material),
            tool_type: tool,
            attack: tool_atk(tool, material),
        }
    }

    pub fn reduce_info(&self) -> Self {
        Self {
            durability: self.max_durability,
            max_durability: self.max_durability,
            speed: self.speed,
            tool_type: self.tool_type,
            attack: self.attack,
        }
    }

    pub fn update_durability(&mut self, amt: u32) {
        if amt > self.durability {
            self.durability = 0;
            return;
        }
        self.durability -= amt;
    }
}

impl Display for ToolInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}/{}",
            self.durability,
            self.max_durability,
            self.speed,
            tool_type_to_string(self.tool_type),
            self.attack,
        )
    }
}

fn parse_tool_info(tokens: &[String]) -> Result<ToolInfo, ()> {
    let info_durability = tokens[0].parse::<u32>().map_err(|_| ())?;
    let info_max_durability = tokens[1].parse::<u32>().map_err(|_| ())?;
    let info_speed = tokens[2].parse::<f32>().map_err(|_| ())?;
    let info_tool_type = string_to_tool_type(&tokens[3])?;
    let info_tool_atk = tokens[4].parse::<u32>().map_err(|_| ())?;

    Ok(ToolInfo {
        durability: info_durability,
        max_durability: info_max_durability,
        speed: info_speed,
        tool_type: info_tool_type,
        attack: info_tool_atk,
    })
}

fn parse_tool_info_material(tokens: &[String]) -> Result<ToolInfo, ()> {
    let material = string_to_tool_material(&tokens[0])?;
    let info_tool_type = string_to_tool_type(&tokens[1])?;
    Ok(ToolInfo::new_tool(info_tool_type, material))
}

pub fn string_to_tool_info(s: &str) -> Result<ToolInfo, ()> {
    let tokens: Vec<String> = s.split("/").map(|s| s.to_string()).collect();

    if tokens.len() == 5 {
        parse_tool_info(&tokens)
    } else if tokens.len() == 2 {
        parse_tool_info_material(&tokens)
    } else {
        Err(())
    }
}
