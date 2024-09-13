use super::KeyState;
use cgmath::{Deg, InnerSpace, Matrix4, Vector3, Vector4};

pub const DEFAULT_CAMERA_SPEED: f32 = 4.0;

pub struct Camera {
    pub speed: f32,
    position: Vector3<f32>,
    direction: Vector3<f32>,
    yaw: f32,   //In degrees
    pitch: f32, //In degrees
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

    pub fn strafe(&mut self, left: KeyState, right: KeyState) {
        self.direction.x = 0.0;

        if left.is_held() {
            self.direction.x += -1.0;
        }

        if right.is_held() {
            self.direction.x += 1.0;
        }
    }

    pub fn fly(&mut self, down: KeyState, up: KeyState) {
        self.direction.y = 0.0;

        if down.is_held() {
            self.direction.y += -1.0;
        }

        if up.is_held() {
            self.direction.y += 1.0;
        }
    }

    pub fn move_forward(&mut self, forward: KeyState, backward: KeyState) {
        self.direction.z = 0.0;

        if forward.is_held() {
            self.direction.z += -1.0;
        }

        if backward.is_held() {
            self.direction.z += 1.0;
        }
    }

    pub fn calculate_velocity(&self) -> Vector3<f32> {
        let mut vel = Vector3::new(0.0, 0.0, 0.0);

        let dirxz = Vector3::new(self.direction.x, 0.0, self.direction.z);
        if dirxz.magnitude() > 0.0 {
            vel += dirxz.normalize() * self.speed;
        }

        let diry = Vector3::new(0.0, self.direction.y, 0.0);
        if diry.magnitude() > 0.0 {
            vel += diry * self.speed;
        }

        let vel_transformed =
            Matrix4::from_angle_y(Deg(-self.yaw)) * Vector4::new(vel.x, vel.y, vel.z, 1.0);

        Vector3::new(vel_transformed.x, vel_transformed.y, vel_transformed.z)
    }

    pub fn update(&mut self, dt: f32) {
        self.position += self.calculate_velocity() * dt;
    }

    pub fn rotate(&mut self, dmousex: f32, dmousey: f32, sensitivity: f32) {
        //Rotate the camera based on mouse movement
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

    pub fn forward(&self) -> Vector3<f32> {
        let dir = Matrix4::from_angle_y(Deg(-self.yaw))
            * Matrix4::from_angle_x(Deg(-self.pitch))
            * Vector4::new(0.0, 0.0, -1.0, 1.0);
        Vector3::new(dir.x, dir.y, dir.z)
    }

    pub fn position(&self) -> Vector3<f32> {
        self.position
    }
}
