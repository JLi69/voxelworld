use cgmath::{Deg, Matrix4, Vector3, Vector4};

pub const DEFAULT_CAMERA_SPEED: f32 = 4.0;

pub struct Camera {
    pub speed: f32,
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub yaw: f32,   //In degrees
    pub pitch: f32, //In degrees
}

impl Camera {
    //Creates a new camera at (x, y, z)
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            speed: DEFAULT_CAMERA_SPEED,
            position: Vector3::new(x, y, z),
            direction: Vector3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
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
        Vector3::new(dir.x, dir.y, dir.z)
    }

    //Returns position of camera
    pub fn position(&self) -> Vector3<f32> {
        self.position
    }
}
