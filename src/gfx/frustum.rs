use crate::game::physics::Hitbox;
use crate::game::Camera;
use cgmath::{InnerSpace, Vector3};

//A plane only needs to be defined by a distance to the origin and a normal vector
pub struct Plane {
    pub dist: f32,          //Distance to the origin
    pub norm: Vector3<f32>, //Normal vector
}

impl Plane {
    //Creates a new plane from a distance and a normal vector
    pub fn new(d: f32, n: Vector3<f32>) -> Self {
        Self {
            dist: d,
            norm: n.normalize(),
        }
    }

    //Creates a new plane from a point on the plane and a normal vector
    pub fn from_point(p: Vector3<f32>, n: Vector3<f32>) -> Self {
        let d = cgmath::dot(n.normalize(), p);
        Self::new(d, n)
    }

    //Returns the signed distance that v has with the plane
    //it is positive if it is in front of the plane, otherwise it is negative
    pub fn signed_dist(&self, v: Vector3<f32>) -> f32 {
        cgmath::dot(v, self.norm) - self.dist
    }

    //Returns true if least part of the AABB is in front of the plane
    pub fn aabb_in_front(&self, hitbox: &Hitbox) -> bool {
        let extent = hitbox.dimensions / 2.0;
        let norm_abs = Vector3::new(self.norm.x.abs(), self.norm.y.abs(), self.norm.z.abs());
        let r = cgmath::dot(extent, norm_abs);
        let dist = self.signed_dist(hitbox.position);
        -r <= dist
    }
}

pub struct Frustum {
    near: Plane,
    far: Plane,
    left: Plane,
    right: Plane,
    top: Plane,
    bottom: Plane,
}

impl Frustum {
    //Creates a new frustum from a camera and an aspect ratio
    pub fn new(cam: &Camera, aspect: f32) -> Self {
        //Calculate the fov in radians
        let fovy_rad = cam.fovy.to_radians() / 2.0;
        let half_hfar = fovy_rad.tan() * cam.zfar;
        let half_wfar = half_hfar * aspect;

        //Calculate plane normals
        let left_n = cam
            .up()
            .cross(cam.forward() * cam.zfar - cam.right() * half_wfar)
            .normalize();
        let right_n = (cam.forward() * cam.zfar + cam.right() * half_wfar)
            .cross(cam.up())
            .normalize();
        let top_n = cam
            .right()
            .cross(cam.forward() * cam.zfar + cam.up() * half_hfar)
            .normalize();
        let bottom_n = (cam.forward() * cam.zfar - cam.up() * half_hfar)
            .cross(cam.right())
            .normalize();

        Self {
            near: Plane::from_point(cam.position + cam.forward() * cam.znear, cam.forward()),
            far: Plane::from_point(cam.position + cam.forward() * cam.zfar, -cam.forward()),
            left: Plane::from_point(cam.position, left_n),
            right: Plane::from_point(cam.position, right_n),
            top: Plane::from_point(cam.position, top_n),
            bottom: Plane::from_point(cam.position, bottom_n),
        }
    }

    //Returns true if an AABB intersects with the frustum
    pub fn intersects(&self, aabb: &Hitbox) -> bool {
        self.near.aabb_in_front(aabb)
            && self.far.aabb_in_front(aabb)
            && self.left.aabb_in_front(aabb)
            && self.right.aabb_in_front(aabb)
            && self.top.aabb_in_front(aabb)
            && self.bottom.aabb_in_front(aabb)
    }
}
