use std::fmt::Display;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FoodInfo {
    //Health restored
    pub health: i32,
    //Stamina restored (percent)
    pub stamina: u8,
}

impl FoodInfo {
    pub fn new(health_restored: i32, stamina_restored: u8) -> Self {
        Self {
            health: health_restored,
            stamina: stamina_restored,
        }
    }

    pub fn get_stamina_perc(&self) -> f32 {
        self.stamina as f32 / 100.0
    }
}

impl Display for FoodInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.health, self.stamina)
    }
}

pub fn string_to_food_info(s: &str) -> Result<FoodInfo, ()> {
    let tokens: Vec<String> = s.split("/").map(|s| s.to_string()).collect();

    if tokens.len() != 2 {
        return Err(());
    }

    let health = tokens[0].parse::<i32>().map_err(|_| ())?;
    let stamina = tokens[1].parse::<u8>().map_err(|_| ())?;

    Ok(FoodInfo::new(health, stamina))
}
