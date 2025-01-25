use crate::voxel::{orientation_to_normal, rotate_orientation, Block, World, EMPTY_BLOCK};
use cgmath::{InnerSpace, Vector3};

//Axis aligned bounding box (this hitbox is aligned with the x, y, z axis)
pub struct Hitbox {
    pub position: Vector3<f32>,
    pub dimensions: Vector3<f32>,
}

//A hitbox made up of multiple hitboxes, used for partial blocks (stairs)
pub enum CompositeHitbox {
    Single(Hitbox),
    Double(Hitbox, Hitbox),
    Triple(Hitbox, Hitbox, Hitbox),
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

    pub fn slab_hitbox(orientation: u8, x: i32, y: i32, z: i32) -> Self {
        let fx = x as f32 + 0.5;
        let fy = y as f32 + 0.5;
        let fz = z as f32 + 0.5;
        let norm = orientation_to_normal(orientation);
        Hitbox::new(
            fx - norm.x as f32 * 0.25,
            fy - norm.y as f32 * 0.25,
            fz - norm.z as f32 * 0.25,
            1.0 - norm.x.abs() as f32 * 0.5,
            1.0 - norm.y.abs() as f32 * 0.5,
            1.0 - norm.z.abs() as f32 * 0.5,
        )
    }

    pub fn corner_hitbox(orientation1: u8, orientation2: u8, x: i32, y: i32, z: i32) -> Self {
        let fx = x as f32 + 0.5;
        let fy = y as f32 + 0.5;
        let fz = z as f32 + 0.5;
        let norm1 = orientation_to_normal(orientation1);
        let norm2 = orientation_to_normal(orientation2);
        Hitbox::new(
            fx - norm1.x as f32 * 0.25 - norm2.x as f32 * 0.25,
            fy - norm1.y as f32 * 0.25 - norm2.y as f32 * 0.25,
            fz - norm1.z as f32 * 0.25 - norm2.z as f32 * 0.25,
            1.0 - norm1.x.abs() as f32 * 0.5 - norm2.x.abs() as f32 * 0.5,
            1.0 - norm1.y.abs() as f32 * 0.5 - norm2.y.abs() as f32 * 0.5,
            1.0 - norm1.z.abs() as f32 * 0.5 - norm2.z.abs() as f32 * 0.5,
        )
    }

    //Create a hitbox from voxel coordinates and also block data
    //Will attempt to use the block geometry to determine an appropriate hitbox
    //Hitbox is used to determine which hitbox to return for a stair, which is
    pub fn from_block_data(x: i32, y: i32, z: i32, block: Block) -> CompositeHitbox {
        let fx = x as f32 + 0.5;
        let fy = y as f32 + 0.5;
        let fz = z as f32 + 0.5;

        if block.is_fluid() {
            let height = get_fluid_height(block.geometry);
            let hitbox = Self::new(fx, fy - (1.0 - height) / 2.0, fz, 1.0, height, 1.0);
            return CompositeHitbox::Single(hitbox);
        }

        let hitbox = match block.id {
            //Ladder
            75 => {
                let norm = orientation_to_normal(block.orientation());
                Some(Self::new(
                    fx - norm.x as f32 * 0.3,
                    fy - norm.y as f32 * 0.3,
                    fz - norm.z as f32 * 0.3,
                    1.0 - norm.x.abs() as f32 * 0.6,
                    1.0 - norm.y.abs() as f32 * 0.6,
                    1.0 - norm.z.abs() as f32 * 0.6,
                ))
            }
            _ => None,
        };

        if let Some(hitbox) = hitbox {
            return CompositeHitbox::Single(hitbox);
        }

        let reflection = if block.reflection() == 0 { 0 } else { 3 };

        match block.shape() {
            1 => CompositeHitbox::Single(Self::slab_hitbox(block.orientation(), x, y, z)),
            2 => CompositeHitbox::Double(
                Self::slab_hitbox(reflection, x, y, z),
                Self::slab_hitbox(block.orientation(), x, y, z),
            ),
            3 => {
                let rotated = rotate_orientation(block.orientation());
                CompositeHitbox::Double(
                    Self::slab_hitbox(reflection, x, y, z),
                    Self::corner_hitbox(block.orientation(), rotated, x, y, z),
                )
            }
            4 => {
                let rotated = rotate_orientation(block.orientation());
                CompositeHitbox::Triple(
                    Self::slab_hitbox(reflection, x, y, z),
                    Self::slab_hitbox(block.orientation(), x, y, z),
                    Self::slab_hitbox(rotated, x, y, z),
                )
            }
            _ => CompositeHitbox::Single(Self::new(fx, fy, fz, 1.0, 1.0, 1.0)),
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

pub fn composite_to_hitbox(composite_hitbox: CompositeHitbox, hitbox: &Hitbox) -> Hitbox {
    match composite_hitbox {
        CompositeHitbox::Single(b) => b,
        CompositeHitbox::Double(b1, b2) => {
            if b2.intersects(hitbox) {
                b2
            } else {
                b1
            }
        }
        CompositeHitbox::Triple(b1, b2, b3) => {
            if b3.intersects(hitbox) {
                b3
            } else if b2.intersects(hitbox) {
                b2
            } else {
                b1
            }
        }
    }
}

pub fn scan_block_hitbox<T>(
    hitbox: &Hitbox,
    world: &World,
    ix: i32,
    iy: i32,
    iz: i32,
    range: i32,
    skip_fn: T,
) -> Option<Hitbox>
where
    T: Fn(Block) -> bool,
{
    for x in (ix - range)..=(ix + range) {
        for y in (iy - range)..=(iy + range) {
            for z in (iz - range)..=(iz + range) {
                let block = world.get_block(x, y, z);
                if skip_fn(block) {
                    continue;
                }

                let composite_hitbox = Hitbox::from_block_data(x, y, z, block);
                let block_hitbox = composite_to_hitbox(composite_hitbox, hitbox);

                if !hitbox.intersects(&block_hitbox) {
                    continue;
                }

                return Some(block_hitbox);
            }
        }
    }

    None
}

pub fn scan_block_full_hitbox<T>(
    hitbox: &Hitbox,
    world: &World,
    ix: i32,
    iy: i32,
    iz: i32,
    range: i32,
    skip_fn: T,
) -> Option<Hitbox>
where
    T: Fn(Block) -> bool,
{
    for x in (ix - range)..=(ix + range) {
        for y in (iy - range)..=(iy + range) {
            for z in (iz - range)..=(iz + range) {
                let block = world.get_block(x, y, z);
                if skip_fn(block) {
                    continue;
                }

                let block_hitbox = Hitbox::from_block(x, y, z);

                if !hitbox.intersects(&block_hitbox) {
                    continue;
                }

                return Some(block_hitbox);
            }
        }
    }

    None
}

pub fn get_block_collision(world: &World, hitbox: &Hitbox) -> Option<Hitbox> {
    let ix = hitbox.position.x.floor() as i32;
    let iy = hitbox.position.y.floor() as i32;
    let iz = hitbox.position.z.floor() as i32;

    let mut hit: Option<Hitbox> = None;
    let mut min_dist = 999.0;
    for x in (ix - 2)..=(ix + 2) {
        for y in (iy - 2)..=(iy + 2) {
            for z in (iz - 2)..=(iz + 2) {
                let block = world.get_block(x, y, z);
                if block.id == EMPTY_BLOCK {
                    continue;
                }

                if block.no_hitbox() {
                    continue;
                }

                let composite_hitbox = Hitbox::from_block_data(x, y, z, block);
                let block_hitbox = composite_to_hitbox(composite_hitbox, hitbox);

                if !hitbox.intersects(&block_hitbox) {
                    continue;
                }

                if (block_hitbox.position - hitbox.position).magnitude() > min_dist {
                    continue;
                }

                min_dist = (block_hitbox.position - hitbox.position).magnitude();
                hit = Some(block_hitbox);
            }
        }
    }

    hit
}
