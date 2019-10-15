use cgmath::prelude::*;
use cgmath::Vector3;
use cgmath::Matrix3;
use cgmath::Matrix4;

use crate::controls::*;
use glium::glutin::event::{ KeyboardInput, ElementState };
use glium::Surface;

#[derive(Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    velocity: Vector3<f32>,

    /*pub forward: Vector3<f32>,
    pub right: Vector3<f32>,
    pub up: Vector3<f32>,*/

    pub orientation: Matrix3<f32>,

    mx_delta: f32,
    my_delta: f32,

    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    move_boost: bool
}

impl Camera {
    pub fn new(position: Vector3<f32>, forward: Vector3<f32>) -> Camera {
        const WORLD_UP: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);
        
        let forward_norm = forward.normalize();
        let right = forward_norm.cross(WORLD_UP);
        let up = right.cross(forward);

        let orientation = Matrix3::from_cols(up, right, forward_norm);

        Camera {
            position,
            velocity: Vector3::zero(),
            orientation: orientation,

            mx_delta: 0.0,
            my_delta: 0.0,

            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            move_boost: false
        }
    }

    pub fn handle_mouse(&mut self, mx: f32, my: f32) {
        self.mx_delta += mx;
        self.my_delta += my;
    }

    pub fn handle_keyboard(&mut self, event: &KeyboardInput) {
        let pressed = event.state == ElementState::Pressed;
        
        match event.virtual_keycode {
            Some(KEY_FORWARD) => self.move_forward = pressed,
            Some(KEY_BACKWARD) => self.move_backward = pressed,
            Some(KEY_LEFT) => self.move_left = pressed,
            Some(KEY_RIGHT) => self.move_right = pressed,
            Some(KEY_UP) => self.move_up = pressed,
            Some(KEY_DOWN) => self.move_down = pressed,
            Some(KEY_BOOST) => self.move_boost = pressed,
            _ => ()
        }
    }

    pub fn update(&mut self) {
        use std::f32::consts::PI;

        // Movement
        const TICK_STEP: f32 = 1.0 / 60.0;
        const MOVEMENT_SPEED: f32 = 32.0;

        let mut acceleration = Vector3::zero();

        if self.move_forward  { acceleration += self.orientation.z * TICK_STEP; }
        if self.move_backward { acceleration -= self.orientation.z * TICK_STEP; }
        if self.move_left     { acceleration -= self.orientation.y * TICK_STEP; }
        if self.move_right    { acceleration += self.orientation.y * TICK_STEP; }
        if self.move_up       { acceleration += self.orientation.x * TICK_STEP; }
        if self.move_down     { acceleration -= self.orientation.x * TICK_STEP; }

        let norm_accel = acceleration.normalize();
        if  norm_accel.x.is_finite() { // prevent NaNs...
            acceleration = norm_accel;
        }

        if self.move_boost { acceleration *= 2.0; }

        self.velocity += acceleration;

        let drag = TICK_STEP + 1.0;
        self.velocity /= drag;

        self.position += self.velocity * TICK_STEP * TICK_STEP * MOVEMENT_SPEED;

        // Rotation
        self.orientation = self.orientation * legacy::orthorotate(
            Vector3::new(
                self.orientation.y.z * 0.1,
                (-self.mx_delta * PI / 180.0) * MOUSE_SENSITIVITY / 100.0,
                ( self.my_delta * PI / 180.0) * MOUSE_SENSITIVITY / 100.0
            )
        );

        self.mx_delta = 0.0;
        self.my_delta = 0.0;
    }

    pub fn get_view_matrix(&self, delta: f32) -> Matrix4<f32> {
        const TICK_STEP: f32 = 1.0 / 60.0;
        const MOVEMENT_SPEED: f32 = 32.0;
        let translation = self.position + self.velocity * TICK_STEP * TICK_STEP * MOVEMENT_SPEED * delta;

        Matrix4::new(
            self.orientation.y.x, self.orientation.x.x, -self.orientation.z.x, 0.0,
            self.orientation.y.y, self.orientation.x.y, -self.orientation.z.y, 0.0,
            self.orientation.y.z, self.orientation.x.z, -self.orientation.z.z, 0.0,
                             0.0,                  0.0,                   0.0, 1.0
        ) * Matrix4::from_translation(-translation)
    }

    pub fn get_perspective_matrix(target: &glium::Frame) -> Matrix4<f32> {
        let (width, height) = target.get_dimensions();
        let aspect_ratio = width as f32 / height as f32;

        cgmath::perspective(cgmath::Deg(90.0), aspect_ratio, 0.1, 1024.0)
    }
}

mod legacy {
    use cgmath::Vector3;
    use cgmath::Matrix3;

    pub fn orthorotate(rot: Vector3<f32>) -> Matrix3<f32> {
        let c = Vector3::new(rot.x.cos(), rot.y.cos(), rot.z.cos());
        let s = Vector3::new(rot.x.sin(), rot.y.sin(), rot.z.sin());

        Matrix3::new(
            s.x*s.z*s.y + c.x*c.z, -c.x*s.z*s.y + s.x*c.z, s.z*c.y,
                         -s.x*c.y,                c.x*c.y,    s.y,
            s.x*c.z*s.y - c.x*s.z, -c.x*c.z*s.y - s.x*s.z, c.z*c.y
        )
    }
}