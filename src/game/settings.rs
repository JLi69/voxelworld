#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CloudDisplay {
    Fancy,
    Flat,
    Disabled,
}

pub struct Settings {
    pub cloud_display: CloudDisplay,
}

impl Settings {
    pub fn default() -> Self {
        Self {
            cloud_display: CloudDisplay::Fancy,
        }
    }
}
