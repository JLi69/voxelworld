use cgmath::Vector3;

//Axis aligned bounding box (this hitbox is aligned with the x, y, z axis)
pub struct Hitbox {
    pub position: Vector3<f32>,
    pub dimensions: Vector3<f32>,
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
    pub fn from_block(x: i32, y: i32, z: i32) -> Self {
        let fx = x as f32 + 0.5;
        let fy = y as f32 + 0.5;
        let fz = z as f32 + 0.5;
        Hitbox::new(fx, fy, fz, 1.0, 1.0, 1.0)
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
