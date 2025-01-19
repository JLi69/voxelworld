use crate::voxel::{orientation_to_normal, Block};
use cgmath::Vector3;

//Axis aligned bounding box (this hitbox is aligned with the x, y, z axis)
pub struct Hitbox {
    pub position: Vector3<f32>,
    pub dimensions: Vector3<f32>,
}

//Returns the height of the fluid based on its geometry
fn get_fluid_height(geometry: u8) -> f32 {
    if geometry <= 7 {
        return geometry as f32 / 8.0;
    }

    1.0
}

impl Hitbox {
    //sx, sy, sz must be positive!
    pub fn new(x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32) -> Self {
        assert!(sx > 0.0);
        assert!(sy > 0.0);
        assert!(sz > 0.0);
        Self {
            position: Vector3::new(x, y, z),
            dimensions: Vector3::new(sx, sy, sz),
        }
    }

    //Create a hitbox from voxel coordinates to represent a block
    //This function assumes that the block is a full voxel
    pub fn from_block(x: i32, y: i32, z: i32) -> Self {
        let fx = x as f32 + 0.5;
        let fy = y as f32 + 0.5;
        let fz = z as f32 + 0.5;
        Hitbox::new(fx, fy, fz, 1.0, 1.0, 1.0)
    }

    //Create a hitbox from voxel coordinates and also block data
    //Will attempt to use the block geometry to determine an appropriate hitbox
    pub fn from_block_data(x: i32, y: i32, z: i32, block: Block) -> Self {
        let fx = x as f32 + 0.5;
        let fy = y as f32 + 0.5;
        let fz = z as f32 + 0.5;

        if block.is_fluid() {
            let height = get_fluid_height(block.geometry);
            Hitbox::new(fx, fy - (1.0 - height) / 2.0, fz, 1.0, height, 1.0)
        } else {
            match block.shape() {
                1 => {
                    //Slab
                    let norm = orientation_to_normal(block.orientation());
                    Hitbox::new(
                        fx - norm.x as f32 * 0.25,
                        fy - norm.y as f32 * 0.25,
                        fz - norm.z as f32 * 0.25,
                        1.0 - norm.x.abs() as f32 * 0.5,
                        1.0 - norm.y.abs() as f32 * 0.5,
                        1.0 - norm.z.abs() as f32 * 0.5,
                    )
                }
                _ => Hitbox::new(fx, fy, fz, 1.0, 1.0, 1.0),
            }
        }
    }

    //Create a hitbox from Vector3, we assume that size has positive dimensions
    pub fn from_vecs(pos: Vector3<f32>, size: Vector3<f32>) -> Self {
        assert!(size.x > 0.0);
        assert!(size.y > 0.0);
        assert!(size.z > 0.0);
        Self {
            position: pos,
            dimensions: size,
        }
    }

    //Check for intersection between this hitbox and another hitbox
    pub fn intersects(&self, other: &Self) -> bool {
        (self.position.x - other.position.x).abs() < (self.dimensions.x + other.dimensions.x) / 2.0
            && (self.position.y - other.position.y).abs()
                < (self.dimensions.y + other.dimensions.y) / 2.0
            && (self.position.z - other.position.z).abs()
                < (self.dimensions.z + other.dimensions.z) / 2.0
    }
}
