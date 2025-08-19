#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CloudDisplay {
    Fancy,
    Flat,
    Disabled,
}

pub const MIN_RENDER_DIST: u32 = 3;
pub const DEFAULT_RENDER_DIST: u32 = 7;
pub const MAX_RENDER_DIST: u32 = 20;

pub struct Settings {
    pub cloud_display: CloudDisplay,
    pub render_distance: u32,
}

impl Settings {
    pub fn default() -> Self {
        Self {
            cloud_display: CloudDisplay::Fancy,
            render_distance: DEFAULT_RENDER_DIST,
        }
    }

    pub fn get_range(&self) -> u32 {
        self.render_distance.clamp(MIN_RENDER_DIST, MAX_RENDER_DIST)
    }
}
