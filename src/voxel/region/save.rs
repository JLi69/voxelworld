use super::{Region, REGION_SIZE};
use crate::game::save::CHUNK_PATH;
use std::{fs::File, io::Write, path::Path};

fn region_file_name(x: i32, y: i32, z: i32) -> String {
    "region_".to_string()
        + x.to_string().as_str()
        + "_"
        + y.to_string().as_str()
        + "_"
        + z.to_string().as_str()
}

impl Region {
    fn get_chunk_data(&self, ix: usize, iy: usize, iz: usize) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        if let Some(chunk) = self.get_relative_chunk(ix, iy, iz) {
            chunk.get_chunk_bytes()
        } else {
            (0xffffu16.to_be_bytes().to_vec(), vec![], vec![])
        }
    }

    pub fn save_region(&self, worldpath: &str) -> Result<(), String> {
        let chunk_dir_path = worldpath.to_string() + CHUNK_PATH;
        let chunk_path = chunk_dir_path.clone() + region_file_name(self.x, self.y, self.z).as_str();
        if !Path::new(&chunk_dir_path).exists() {
            if let Err(msg) = std::fs::create_dir_all(&chunk_dir_path) {
                eprintln!("E: Failed to create chunk dir");
                eprintln!("{msg}");
                return Err(chunk_path);
            }
        }

        let mut data_to_write = vec![];
        for ix in 0..REGION_SIZE {
            for iy in 0..REGION_SIZE {
                for iz in 0..REGION_SIZE {
                    let (len, counts, data) = self.get_chunk_data(ix, iy, iz);
                    data_to_write.extend(len);
                    data_to_write.extend((ix as u16).to_be_bytes());
                    data_to_write.extend((iy as u16).to_be_bytes());
                    data_to_write.extend((iz as u16).to_be_bytes());
                    data_to_write.extend(counts);
                    data_to_write.extend(data);
                }
            }
        }

        match File::create(&chunk_path) {
            Ok(mut file) => {
                let data_sz = data_to_write.len() as u32;
                let data_sz_bytes = data_sz.to_be_bytes();
                if let Err(msg) = file.write_all(&data_sz_bytes) {
                    eprintln!("Error when saving {}, {}, {}", self.x, self.y, self.z);
                    eprintln!("E: {msg}");
                    return Err(chunk_path);
                }
                if let Err(msg) = file.write_all(&data_to_write) {
                    eprintln!("Error when saving {}, {}, {}", self.x, self.y, self.z);
                    eprintln!("E: {msg}");
                    return Err(chunk_path);
                }
            }
            Err(msg) => {
                eprintln!("Failed to save chunk {}, {}, {}", self.x, self.y, self.z);
                eprintln!("{msg}");
                return Err(chunk_path);
            }
        }

        Ok(())
    }
}

fn merge_regions(updated: &mut Region, original: &Region) {
    for ix in 0..REGION_SIZE {
        for iy in 0..REGION_SIZE {
            for iz in 0..REGION_SIZE {
                if updated.get_relative_chunk(ix, iy, iz).is_some() {
                    continue;
                }

                if let Some(chunk) = original.get_relative_chunk(ix, iy, iz) {
                    updated.set_relative_chunk(ix, iy, iz, Some(chunk.clone()));
                }
            }
        }
    }
}

pub fn serialize_region(world_dir_path: &str, region: &Region) -> Result<(), String> {
    let mut region_clone = region.clone();
    let x = region.x;
    let y = region.y;
    let z = region.z;

    if let Some(original) = Region::load_region(world_dir_path, x, y, z) {
        merge_regions(&mut region_clone, &original);
    }

    region_clone.save_region(world_dir_path)
}
