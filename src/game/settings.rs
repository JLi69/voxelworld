use crate::impfile::{self, Entry};
use std::{fs::File, io::Write};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CloudDisplay {
    Fancy,
    Flat,
    Disabled,
}

fn string_to_cloud_display(s: &str) -> Result<CloudDisplay, ()> {
    match s {
        "fancy" => Ok(CloudDisplay::Fancy),
        "flat" => Ok(CloudDisplay::Flat),
        "disabled" => Ok(CloudDisplay::Disabled),
        _ => Err(()),
    }
}

fn cloud_display_to_string(cloud_display: CloudDisplay) -> String {
    match cloud_display {
        CloudDisplay::Fancy => "fancy".to_string(),
        CloudDisplay::Flat => "flat".to_string(),
        CloudDisplay::Disabled => "disabled".to_string(),
    }
}

pub const MIN_RENDER_DIST: u32 = 3;
pub const DEFAULT_RENDER_DIST: u32 = 7;
pub const MAX_RENDER_DIST: u32 = 16;

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

    pub fn load(path: &str) -> Self {
        let entries = impfile::parse_file(path);
        if entries.len() != 1 {
            return Self::default();
        }

        let cloud_display = string_to_cloud_display(&entries[0].get_var("cloud_display"))
            .unwrap_or(CloudDisplay::Fancy);
        Self {
            cloud_display,
            render_distance: entries[0]
                .get_var("render_distance")
                .parse::<u32>()
                .unwrap_or(DEFAULT_RENDER_DIST)
                .clamp(MIN_RENDER_DIST, MAX_RENDER_DIST),
        }
    }

    pub fn save(&self, path: &str) {
        let mut entry = Entry::new("settings");
        entry.add_integer("render_distance", self.render_distance as i64);
        entry.add_string(
            "cloud_display",
            &cloud_display_to_string(self.cloud_display),
        );

        let settings_entry_str = entry.to_impfile_string();
        let res = match File::create(path) {
            Ok(mut settings_file) => {
                impfile::write_comment(&mut settings_file, "Game settings");
                settings_file.write_all(settings_entry_str.as_bytes())
            }
            Err(msg) => Err(msg),
        };

        if let Err(msg) = res {
            eprintln!("E: Failed to save settings: {msg}");
        }
    }
}
