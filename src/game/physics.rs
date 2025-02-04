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
            //Fence
            76 => Some(Self::new(fx, fy + 0.25, fz, 1.0, 1.5, 1.0)),
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

    fn from_block_orientation(x: i32, y: i32, z: i32, sz: f32, block: Block) -> Self {
        let fx = x as f32 + 0.5;
        let fy = y as f32 + 0.5;
        let fz = z as f32 + 0.5;
        let norm = orientation_to_normal(block.orientation());
        Self::new(
            fx - norm.x as f32 * sz / 2.0,
            fy - norm.y as f32 * sz / 2.0,
            fz - norm.z as f32 * sz / 2.0,
            1.0 - norm.x.abs() as f32 * sz,
            1.0 - norm.y.abs() as f32 * sz,
            1.0 - norm.z.abs() as f32 * sz,
        )
    }

    //Returns a block's bounding box
    pub fn from_block_bbox(x: i32, y: i32, z: i32, block: Block) -> Self {
        let fx = x as f32 + 0.5;
        let fy = y as f32 + 0.5;
        let fz = z as f32 + 0.5;
        match block.id {
            //Ladder and seeds
            75 => Self::from_block_orientation(x, y, z, 0.9, block),
            //Seeds
            77 => {
                let mut bbox = Self::from_block_orientation(x, y, z, 0.9, block);
                bbox.dimensions.x *= 0.8;
                bbox.dimensions.z *= 0.8;
                bbox
            }
            //Wheat
            50..=52 => {
                let sz = 1.0 / 16.0 * 2.0f32.powi(block.id as i32 - 50 + 1);
                let mut bbox = Self::from_block_orientation(x, y, z, 1.0 - sz, block);
                bbox.dimensions.x *= 0.8;
                bbox.dimensions.z *= 0.8;
                bbox
            }
            //Fully grown wheat, sapling, grass
            47 | 49 | 53 => {
                let sz = 15.0 / 16.0;
                let mut bbox = Self::from_block_orientation(x, y, z, 1.0 - sz, block);
                bbox.dimensions.x *= 0.8;
                bbox.dimensions.z *= 0.8;
                bbox
            }
            //Mushroom, flowers
            48 | 55 => {
                let mut bbox = Self::from_block_orientation(x, y, z, 0.5, block);
                bbox.dimensions.x *= 0.3;
                bbox.dimensions.z *= 0.3;
                bbox
            }
            54 | 56 => {
                let mut bbox = Self::from_block_orientation(x, y, z, 0.2, block);
                bbox.dimensions.x *= 0.3;
                bbox.dimensions.z *= 0.3;
                bbox
            }
            //Sugar cane
            69 => Self::new(fx, fy, fz, 0.8, 1.0, 0.8),
            //Torches
            71..=74 => match block.orientation() {
                0 | 3 => {
                    let mut bbox = Self::from_block_orientation(x, y, z, 5.5 / 16.0, block);
                    bbox.dimensions.x *= 3.0 / 16.0;
                    bbox.dimensions.z *= 3.0 / 16.0;
                    bbox
                }
                1 | 4 => {
                    let mut bbox = Self::from_block_orientation(x, y, z, 10.0 / 16.0, block);
                    bbox.dimensions.y *= 10.5 / 16.0;
                    bbox.dimensions.z *= 4.0 / 16.0;
                    bbox
                }
                2 | 5 => {
                    let mut bbox = Self::from_block_orientation(x, y, z, 10.0 / 16.0, block);
                    bbox.dimensions.y *= 10.5 / 16.0;
                    bbox.dimensions.x *= 4.0 / 16.0;
                    bbox
                }
                _ => Self::from_block(x, y, z),
            },
            //Fence
            76 => Self::new(fx, fy, fz, 0.5, 1.0, 0.5),
            _ => composite_to_bbox(Hitbox::from_block_data(x, y, z, block)),
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

    pub fn min(&self) -> Vector3<f32> {
        self.position - self.dimensions
    }

    pub fn max(&self) -> Vector3<f32> {
        self.position + self.dimensions
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

//Converts a composite hitbox into a bounding box
pub fn composite_to_bbox(composite_hitbox: CompositeHitbox) -> Hitbox {
    match composite_hitbox {
        CompositeHitbox::Single(b) => b,
        CompositeHitbox::Double(b1, b2) => {
            let minx = b1.min().x.min(b2.min().x);
            let miny = b1.min().y.min(b2.min().y);
            let minz = b1.min().z.min(b2.min().z);
            let maxx = b1.max().x.max(b2.max().x);
            let maxy = b1.max().y.max(b2.max().y);
            let maxz = b1.max().z.max(b2.max().z);
            Hitbox::new(
                (minx + maxx) / 2.0,
                (miny + maxy) / 2.0,
                (minz + maxz) / 2.0,
                (maxx - minx) / 2.0,
                (maxy - miny) / 2.0,
                (maxz - minz) / 2.0,
            )
        }
        CompositeHitbox::Triple(b1, b2, b3) => {
            let minx = b1.min().x.min(b2.min().x).min(b3.min().x);
            let miny = b1.min().y.min(b2.min().y).min(b3.min().y);
            let minz = b1.min().z.min(b2.min().z).min(b3.min().z);
            let maxx = b1.max().x.max(b2.max().x).max(b3.max().x);
            let maxy = b1.max().y.max(b2.max().y).max(b3.max().y);
            let maxz = b1.max().z.max(b2.max().z).max(b3.max().z);
            Hitbox::new(
                (minx + maxx) / 2.0,
                (miny + maxy) / 2.0,
                (minz + maxz) / 2.0,
                (maxx - minx) / 2.0,
                (maxy - miny) / 2.0,
                (maxz - minz) / 2.0,
            )
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

pub fn least_dist(pos: Vector3<f32>, hitbox: &Hitbox) -> f32 {
    let maxpoint = hitbox.position + hitbox.dimensions / 2.0;
    let minpoint = hitbox.position - hitbox.dimensions / 2.0;
    let dx = (pos.x - maxpoint.x).max(minpoint.x - pos.x).max(0.0);
    let dy = (pos.y - maxpoint.y).max(minpoint.y - pos.y).max(0.0);
    let dz = (pos.z - maxpoint.z).max(minpoint.z - pos.z).max(0.0);
    (dx * dx + dy * dy + dz * dz).sqrt()
}

//This function uses ray marching to determine if a ray starting from a position
//going in a direction intersects a hitbox
pub fn ray_intersects_box(pos: Vector3<f32>, dir: Vector3<f32>, hitbox: &Hitbox) -> bool {
    if dir.magnitude() == 0.0 {
        return false;
    }

    let mut current_pos = pos;
    let mut dist = least_dist(current_pos, hitbox);
    let mut min_dist = least_dist(current_pos, hitbox);
    while min_dist >= dist && dist > 0.01 {
        current_pos += dir.normalize() * dist;
        min_dist = min_dist.min(least_dist(current_pos, hitbox));
        dist = least_dist(current_pos, hitbox);
    }

    dist <= 0.01
}
