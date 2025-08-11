use super::{Region, REGION_SIZE_I32};
use crate::{
    game::save::CHUNK_PATH,
    voxel::{Block, Chunk, tile_data::TileData}, bin_data::{ByteStream, parse_binary_data},
};
use std::{fs::File, io::Read};

fn read_bytes(bytes: &mut [u8], file: &mut File) {
    if let Err(msg) = file.read(bytes) {
        eprintln!("Error when loading chunk: {msg}");
    }
}

fn get_val<T: Copy>(data: &[T], index: usize) -> Option<T> {
    if index >= data.len() {
        None
    } else {
        Some(data[index])
    }
}

fn bytes_to_u16(bytes: &[u8]) -> Vec<u16> {
    let mut vals = vec![0u16; bytes.len() / 2];
    for (i, v) in vals.iter_mut().enumerate() {
        let index = i * 2;
        let a = get_val(bytes, index).unwrap_or(0);
        let b = get_val(bytes, index + 1).unwrap_or(0);
        *v = ((a as u16) << 8) | (b as u16);
    }
    vals
}

fn region_from_bytes(data: &[u16], x: i32, y: i32, z: i32) -> Region {
    let mut region = Region::new(x, y, z);

    if data.is_empty() {
        return region;
    }

    let mut index = 0;
    while index < data.len() {
        let len = get_val(data, index).unwrap_or(0);
        index += 1;
        let ix = get_val(data, index).unwrap_or(0);
        index += 1;
        let iy = get_val(data, index).unwrap_or(0);
        index += 1;
        let iz = get_val(data, index).unwrap_or(0);
        index += 1;
        //Ungenerated
        if len == 0xffff {
            continue;
        }

        let mut blocks = vec![];
        let start = index;
        for i in 0..len {
            let count = get_val(data, start + i as usize).unwrap_or(0);
            let block_data = get_val(data, start + i as usize + len as usize).unwrap_or(0);
            let id = (block_data >> 8) as u8;
            let geometry = (block_data & 0xff) as u8;
            let mut block = Block::new_id_orientation(id, geometry);
            if block.is_fluid() && block.geometry == 0 {
                block.geometry = 7;
            }
            blocks.push((count, block));
            index += 2;
        }

        let chunkx = x * REGION_SIZE_I32 + ix as i32;
        let chunky = y * REGION_SIZE_I32 + iy as i32;
        let chunkz = z * REGION_SIZE_I32 + iz as i32;
        let chunk = Chunk::from_rle(chunkx, chunky, chunkz, &blocks);
        region.set_chunk(chunkx, chunky, chunkz, Some(chunk));
    }

    region
}

impl Region {
    pub fn load_region(world_dir_path: &str, x: i32, y: i32, z: i32) -> Option<Self> {
        let path = world_dir_path.to_string()
            + CHUNK_PATH
            + "region_"
            + x.to_string().as_str()
            + "_"
            + y.to_string().as_str()
            + "_"
            + z.to_string().as_str();

        match File::open(&path) {
            Ok(mut file) => {
                //Block data
                let mut sz_bytes = [0u8; size_of::<u32>()];
                read_bytes(&mut sz_bytes, &mut file);
                let sz = u32::from_be_bytes(sz_bytes);
                let mut block_bytes = vec![0u8; sz as usize];
                read_bytes(&mut block_bytes, &mut file);
                let chunk_data = bytes_to_u16(&block_bytes); 
                let mut region = region_from_bytes(&chunk_data, x, y, z);
                
                let mut sz_bytes = [0u8; size_of::<u32>()];
                read_bytes(&mut sz_bytes, &mut file);
                let sz = u32::from_be_bytes(sz_bytes);
                //No tile data to read, return
                if sz == 0 {
                    return Some(region);
                }

                let mut tile_data_bytes = vec![0u8; sz as usize];
                read_bytes(&mut tile_data_bytes, &mut file);
                let mut byte_stream = ByteStream::new(tile_data_bytes);
                let parsed = parse_binary_data(&mut byte_stream);
                if let Some(tile_data_list) = parsed.get("tile_data") {
                    tile_data_list.iter()
                        .filter_map(TileData::from_data_table)
                        .for_each(|((x, y, z), tile_data)| {
                            region.set_tile_data(x, y, z, tile_data);
                        });
                }

                Some(region)
            }
            Err(_msg) => None,
        }
    }
}
