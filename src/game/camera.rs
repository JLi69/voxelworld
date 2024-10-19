use cgmath::{Deg, InnerSpace, Matrix4, Vector3, Vector4};

use crate::impfile;

const DEFAULT_ZNEAR: f32 = 0.1;
const DEFAULT_ZFAR: f32 = 1000.0;
//In degrees
const DEFAULT_FOV: f32 = 75.0;

pub struct Camera {
    pub position: Vector3<f32>,
    pub yaw: f32,   //In degrees
    pub pitch: f32, //In degrees
    pub znear: f32,
    pub zfar: f32,
    pub fovy: f32, //In degrees
}

impl Camera {
    //Creates a new camera at (x, y, z)
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vector3::new(x, y, z),
            yaw: 0.0,
            pitch: 0.0,
            znear: DEFAULT_ZNEAR,
            zfar: DEFAULT_ZFAR,
            fovy: DEFAULT_FOV,
        }
    }

    //Rotate the camera based on mouse movement
    pub fn rotate(&mut self, dmousex: f32, dmousey: f32, sensitivity: f32) {
        self.yaw += dmousex * sensitivity;
        self.pitch += dmousey * sensitivity;
        //Clamp the pitch
        self.pitch = self.pitch.clamp(-90.0, 90.0);
    }

    //Returns the view matrix for the camera
    pub fn get_view(&self) -> Matrix4<f32> {
        Matrix4::from_angle_x(Deg(self.pitch))
            * Matrix4::from_angle_y(Deg(self.yaw))
            * Matrix4::from_translation(-self.position)
    }

    //Forward vector for camera
    pub fn forward(&self) -> Vector3<f32> {
        let dir = Matrix4::from_angle_y(Deg(-self.yaw))
            * Matrix4::from_angle_x(Deg(-self.pitch))
            * Vector4::new(0.0, 0.0, -1.0, 1.0);
        Vector3::new(dir.x, dir.y, dir.z).normalize()
    }

    //Right vector for camera
    pub fn right(&self) -> Vector3<f32> {
        Vector3::new(0.0, 1.0, 0.0)
            .cross(self.forward())
            .normalize()
    }

    //Up vector for camera
    pub fn up(&self) -> Vector3<f32> {
        self.forward().cross(self.right()).normalize()
    }

    //Returns fovy in degrees for camera
    pub fn get_fovy(&self) -> Deg<f32> {
        Deg(self.fovy)
    }

    pub fn to_entry(&self) -> impfile::Entry {
        let mut entry = impfile::Entry::new("camera");

        entry.add_float("x", self.position.x);
        entry.add_float("y", self.position.y);
        entry.add_float("z", self.position.z);
        entry.add_float("yaw", self.yaw);
        entry.add_float("pitch", self.pitch);

        entry
    }

    pub fn from_entry(entry: &impfile::Entry) -> Self {
        let x = entry.get_var("x").parse::<f32>().unwrap_or(0.0);
        let y = entry.get_var("y").parse::<f32>().unwrap_or(0.0);
        let z = entry.get_var("z").parse::<f32>().unwrap_or(0.0);
        Self { 
            position: Vector3::new(x, y, z), 
            yaw: entry.get_var("yaw").parse::<f32>().unwrap_or(0.0), 
            pitch: entry.get_var("pitch").parse::<f32>().unwrap_or(0.0), 
            znear: DEFAULT_ZNEAR, 
            zfar: DEFAULT_ZFAR, 
            fovy: DEFAULT_FOV, 
        }
    }
}
