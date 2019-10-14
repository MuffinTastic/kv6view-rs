use cgmath::prelude::*;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix4;

use crate::controls::*;
use glium::glutin::event::{ KeyboardInput, ElementState };
use glium::Surface;

#[derive(Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    velocity: Vector3<f32>,

    pub forward: Vector3<f32>,
    pub right: Vector3<f32>,
    pub up: Vector3<f32>,

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

        Camera {
            position,
            velocity: Vector3::zero(),

            forward: forward_norm,
            right,
            up,

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

        if self.move_forward  { acceleration += self.forward * TICK_STEP; }
        if self.move_backward { acceleration -= self.forward * TICK_STEP; }
        if self.move_left     { acceleration -= self.right * TICK_STEP;   }
        if self.move_right    { acceleration += self.right * TICK_STEP;   }
        if self.move_up       { acceleration += self.up * TICK_STEP;      }
        if self.move_down     { acceleration -= self.up * TICK_STEP;      }

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
        legacy::orthorotate(self.right.z * 0.1,
            (-self.my_delta * PI / 180.0) * MOUSE_SENSITIVITY / 100.0,
            ( self.mx_delta * PI / 180.0) * MOUSE_SENSITIVITY / 100.0,
            &mut self.right, &mut self.up, &mut self.forward);

        /*self.forward = (Vector3::new(0.0, 0.0, 0.0) - self.position).normalize();
        self.forward = if self.forward.is_finite() { self.forward } else { Vector3::new(1.0, 0.0, 0.0) };
        self.right = self.forward.cross(Vector3::new(0.0, 0.0, 1.0));
        self.up = self.right.cross(self.forward);*/

        self.mx_delta = 0.0;
        self.my_delta = 0.0;
    }

    pub fn get_view_matrix(&self, delta: f32) -> Matrix4<f32> {
        const TICK_STEP: f32 = 1.0 / 60.0;
        const MOVEMENT_SPEED: f32 = 32.0;
        let translation = self.position + self.velocity * TICK_STEP * TICK_STEP * MOVEMENT_SPEED * delta;

        Matrix4 {
            x: Vector4::new(self.right.x, self.up.x, -self.forward.x, 0.0),
            y: Vector4::new(self.right.y, self.up.y, -self.forward.y, 0.0),
            z: Vector4::new(self.right.z, self.up.z, -self.forward.z, 0.0),
            w: Vector4::new(0.0, 0.0, 0.0, 1.0)
        } * Matrix4::from_translation(-translation)
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

    // voxlap convenience function. i probably don't need it, but uh, it's here anyway
    pub fn orthorotate(mut ox: f32, mut oy: f32, mut oz: f32,
        ist: &mut Vector3<f32>,
        ihe: &mut Vector3<f32>,
        ifo: &mut Vector3<f32>)
    {
        let mut f: f32; let mut t: f32;
        let mut rr = Matrix3::new(0.0,0.0,0.0,
                                  0.0,0.0,0.0, // annoying
                                  0.0,0.0,0.0);

        let dx = ox.sin(); ox = ox.cos();
        let dy = oy.sin(); oy = oy.cos();
        let dz = oz.sin(); oz = oz.cos();

        
        f = ox*oz; t = dx*dz; rr.x.x =  t*dy + f; rr.z.y = -f*dy - t;
        f = ox*dz; t = dx*oz; rr.x.y = -f*dy + t; rr.z.x =  t*dy - f;
        rr.x.z = dz*oy; rr.y.x = -dx*oy; rr.y.y = ox*oy; rr.z.z = oz*oy; rr.y.z = dy;
        ox = ist.x; oy = ihe.x; oz = ifo.x;
        ist.x = ox*rr.x.x + oy*rr.y.x + oz*rr.z.x;
        ihe.x = ox*rr.x.y + oy*rr.y.y + oz*rr.z.y;
        ifo.x = ox*rr.x.z + oy*rr.y.z + oz*rr.z.z;
        ox = ist.y; oy = ihe.y; oz = ifo.y;
        ist.y = ox*rr.x.x + oy*rr.y.x + oz*rr.z.x;
        ihe.y = ox*rr.x.y + oy*rr.y.y + oz*rr.z.y;
        ifo.y = ox*rr.x.z + oy*rr.y.z + oz*rr.z.z;
        ox = ist.z; oy = ihe.z; oz = ifo.z;
        ist.z = ox*rr.x.x + oy*rr.y.x + oz*rr.z.x;
        ihe.z = ox*rr.x.y + oy*rr.y.y + oz*rr.z.y;
        ifo.z = ox*rr.x.z + oy*rr.y.z + oz*rr.z.z;
    }
}