use super::{Block, Chunk, CHUNK_SIZE, CHUNK_SIZE_I32, EMPTY_BLOCK};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Light {
    //[4 bits sky][4 bits red][4 bits green][4 bits blue]
    light_data: u16,
}

#[derive(Clone, Copy)]
pub struct LightUpdate {
    //If any of these values are None, then that means no update
    pub sky: Option<u16>,
    pub r: Option<u16>,
    pub g: Option<u16>,
    pub b: Option<u16>,
}

#[derive(Clone, Copy)]
pub struct LightSrc {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

impl LightSrc {
    pub fn new(rval: u16, gval: u16, bval: u16) -> Self {
        Self {
            r: rval,
            g: gval,
            b: bval,
        }
    }

    //Returns the rgb values as f32 (between 0.0 -> 1.0)
    pub fn rgb_f32(&self) -> (f32, f32, f32) {
        (
            self.r as f32 / 15.0,
            self.g as f32 / 15.0,
            self.b as f32 / 15.0,
        )
    }
}

pub type LU = LightUpdate;

impl LightUpdate {
    pub fn new(
        skyupdate: Option<u16>,
        rupdate: Option<u16>,
        gupdate: Option<u16>,
        bupdate: Option<u16>,
    ) -> Self {
        Self {
            sky: skyupdate,
            r: rupdate,
            g: gupdate,
            b: bupdate,
        }
    }
}

macro_rules! get_channel {
    ($channel:ident, $bits:expr) => {
        pub fn $channel<T: From<u16>>(&self) -> T {
            ((self.light_data >> $bits) & 0xf).into()
        }
    };
}

macro_rules! set_channel {
    ($channel:ident, $bits:expr) => {
        pub fn $channel<T: Into<u16>>(&mut self, v: T) {
            self.light_data &= !(0xf << $bits);
            self.light_data |= (v.into() << $bits);
        }
    };
}

//Unused macro, for getting maximum of client and block light
macro_rules! get_client {
    ($fn_name: ident, $channel:ident) => {
        pub fn $fn_name<T: From<u16> + Ord>(&self) -> T {
            self.sky::<T>().max(self.$channel())
        }
    };
}

impl Light {
    //All channels are set to 0 (black)
    pub fn black() -> Self {
        Self { light_data: 0 }
    }

    //All channels are set to the corresponding values
    pub fn new<T: Into<u16>>(sky: T, r: T, g: T, b: T) -> Self {
        Self {
            light_data: (sky.into() << 12) | (r.into() << 8) | (g.into() << 4) | b.into(),
        }
    }

    //Get the channel values
    get_channel!(blue, 0);
    get_channel!(green, 4);
    get_channel!(red, 8);
    get_channel!(sky, 12);

    //Set the channel values
    set_channel!(set_blue, 0);
    set_channel!(set_green, 4);
    set_channel!(set_red, 8);
    set_channel!(set_sky, 12); 

    //Returns the rgb values as a tuple (without the sky)
    pub fn get_rgb<T: From<u16>>(&self) -> (T, T, T) {
        (self.red(), self.green(), self.blue())
    }

    //rgb channels
    pub fn r(&self) -> u16 {
        self.red()
    }

    pub fn g(&self) -> u16 {
        self.green()
    }

    pub fn b(&self) -> u16 {
        self.blue()
    }

    //Sky
    pub fn skylight(&self) -> u16 {
        self.sky()
    }

    //Update channels
    pub fn update(&mut self, light_update: LU) {
        if let Some(sky) = light_update.sky {
            self.set_sky(sky);
        }

        if let Some(r) = light_update.r {
            self.set_red(r);
        }

        if let Some(g) = light_update.g {
            self.set_green(g);
        }

        if let Some(b) = light_update.b {
            self.set_blue(b);
        }
    }
}

//Contains the y coordinate of the tallest block in a 16 x 16 region
//The value is set to None if no such block is found
pub struct SkyLightMap {
    //In chunk coordinates
    pub x: i32,
    pub z: i32,
    heights: Vec<Option<i32>>,
    count: usize,
}

//Returns whether sky light can pass through a block
pub fn skylight_can_pass(block: Block) -> bool {
    //If it is a fluid or is leaves, then it blocks sky light
    if block.is_fluid() || block.id == 7 {
        return false;
    }

    block.transparent() || block.shape() != 0 || block.id == EMPTY_BLOCK
}

impl SkyLightMap {
    pub fn new(chunkx: i32, chunkz: i32) -> Self {
        Self {
            x: chunkx,
            z: chunkz,
            heights: vec![],
            count: 0,
        }
    }

    //x and z are in world coordinates
    pub fn get(&self, x: i32, z: i32) -> Option<i32> {
        if self.heights.is_empty() {
            return None;
        }

        let ix = x - self.x * CHUNK_SIZE_I32;
        let iz = z - self.z * CHUNK_SIZE_I32;
        //Out of bounds
        if ix < 0 || iz < 0 || ix >= CHUNK_SIZE_I32 || iz >= CHUNK_SIZE_I32 {
            return None;
        }
        let index = (ix + iz * CHUNK_SIZE_I32) as usize;
        self.heights[index]
    }

    //x and z are in world coordinates
    pub fn set(&mut self, x: i32, z: i32, val: Option<i32>) {
        let ix = x - self.x * CHUNK_SIZE_I32;
        let iz = z - self.z * CHUNK_SIZE_I32;
        //Out of bounds
        if ix < 0 || iz < 0 || ix >= CHUNK_SIZE_I32 || iz >= CHUNK_SIZE_I32 {
            return;
        }

        let current = self.get(x, z);
        //If val is Some then increase the number of values in the map by 1
        //otherwise decrease it by 1 if we are changing a value in the map
        //from Some to None or None to Some
        if val.is_some() && current.is_none() {
            self.count += 1;
        } else if val.is_none() && current.is_some() {
            self.count -= 1;
        }

        //Allocate or deallocate memory
        if self.count > 0 && self.heights.is_empty() {
            self.heights = vec![None; CHUNK_SIZE * CHUNK_SIZE];
        } else if self.count == 0 && !self.heights.is_empty() {
            self.heights = vec![];
        }

        let index = (ix + iz * CHUNK_SIZE_I32) as usize;
        self.heights[index] = val;
    }

    //set the values of the skylight map using a chunk
    pub fn init_map_from_chunk(&mut self, chunk: &Chunk) {
        let chunkpos = chunk.get_chunk_pos();

        if chunkpos.x != self.x || chunkpos.z != self.z {
            return;
        }

        if chunk.is_empty() {
            return;
        }

        let chunkx = chunkpos.x * CHUNK_SIZE_I32;
        let chunky = chunkpos.y * CHUNK_SIZE_I32;
        let chunkz = chunkpos.z * CHUNK_SIZE_I32;
        for x in chunkx..(chunkx + CHUNK_SIZE_I32) {
            for z in chunkz..(chunkz + CHUNK_SIZE_I32) {
                if let Some(height) = self.get(x, z) {
                    if height >= chunky + CHUNK_SIZE_I32 {
                        continue;
                    }
                }

                for y in (chunky..(chunky + CHUNK_SIZE_I32)).rev() {
                    let block = chunk.get_block(x, y, z);
                    //If the block is transparent or the block is not a full block,
                    //then ignore it
                    //Otherwise, if it is a fluid or leaves, stop
                    if skylight_can_pass(block) {
                        continue;
                    }

                    if let Some(height) = self.get(x, z) {
                        self.set(x, z, Some(height.max(y)));
                    } else {
                        self.set(x, z, Some(y));
                    }

                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_set_red() {
        let mut light = Light::black();
        for v in (0..16u16).rev() {
            light.set_red(v);
            assert_eq!(light.r(), v);
            assert_eq!(light.g(), 0);
            assert_eq!(light.b(), 0);
            assert_eq!(light.skylight(), 0);
        }
    }

    #[test]
    pub fn test_set_green() {
        let mut light = Light::black();
        for v in (0..16u16).rev() {
            light.set_green(v);
            assert_eq!(light.r(), 0);
            assert_eq!(light.g(), v);
            assert_eq!(light.b(), 0);
            assert_eq!(light.skylight(), 0);
        }
    }

    #[test]
    pub fn test_set_blue() {
        let mut light = Light::black();
        for v in (0..16u16).rev() {
            light.set_blue(v);
            assert_eq!(light.r(), 0);
            assert_eq!(light.g(), 0);
            assert_eq!(light.b(), v);
            assert_eq!(light.skylight(), 0);
        }
    }

    #[test]
    pub fn test_set_sky() {
        let mut light = Light::black();
        for v in (0..16u16).rev() {
            light.set_sky(v);
            assert_eq!(light.r(), 0);
            assert_eq!(light.g(), 0);
            assert_eq!(light.b(), 0);
            assert_eq!(light.skylight(), v);
        }
    }

    #[test]
    pub fn test_light_creation() {
        let light = Light::new(4u16, 1u16, 2u16, 3u16);
        assert_eq!(light.skylight(), 4);
        assert_eq!(light.r(), 1);
        assert_eq!(light.g(), 2);
        assert_eq!(light.b(), 3);
        assert_eq!(light.get_rgb(), (1, 2, 3));
        assert_eq!(light.get_client(), (4, 4, 4));
    }

    #[test]
    pub fn test_set() {
        let mut light = Light::black();
        assert_eq!(light.skylight(), 0);
        assert_eq!(light.r(), 0);
        assert_eq!(light.g(), 0);
        assert_eq!(light.b(), 0);

        light.set_sky(8u16);
        light.set_red(2u16);
        light.set_green(8u16);
        light.set_blue(14u16);
        assert_eq!(light.get_rgb(), (2, 8, 14));
        assert_eq!(light.get_client(), (8, 8, 14));
    }
}
