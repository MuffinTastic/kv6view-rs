#![feature(div_duration)]
use std::time::{Duration, Instant};

use clap::{Arg, ArgMatches, App};

#[macro_use]
extern crate glium;
use glium::glutin;
use glium::glutin::event::{Event, WindowEvent, DeviceEvent, ElementState};
use glium::glutin::event_loop::EventLoop;
use glium::{Display, Surface};

use cgmath::prelude::*;
use cgmath::Vector3;
use cgmath::Matrix4;

mod eventutil;
mod controls;
mod camera;
mod shaders;
mod kv6;

use camera::Camera;

#[derive(Debug, Copy, Clone)]
struct Vertex {
    position: [f32; 3]
}
implement_vertex!(Vertex, position);

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

struct Viewer {
    focused: bool,

    camera: Camera,
    
    program: glium::Program,

    light_dir: Vector3<f32>,
    light_kv6: kv6::KV6Model,
    show_light: bool,
    
    user_kv6: kv6::KV6Model,
    aos_team_color: Vector3<f32>
}

fn set_capture(display: &Display, capture: bool) {
    display.gl_window().window().set_cursor_grab(capture).unwrap();
    display.gl_window().window().set_cursor_visible(!capture);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("kv6view-rs")
        .version("1.0")
        .about("View KV6 models in OpenGL with Rust")
        .arg(Arg::with_name("file")
            .required(true)
            .index(1))
        .arg(Arg::with_name("aos-team-color")
            .long("aos-team")
            .help("Replace voxels colored 0,0,0 with this color.")
            .number_of_values(3)
            .required(false))
        .get_matches();

    let event_loop = EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size((WINDOW_WIDTH, WINDOW_HEIGHT).into())
        .with_title("KV6View");
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).expect("Error creating display");

    set_capture(&display, true);

    let viewer = init_data(matches, &display)?;
    run_loop(viewer, event_loop, display);
    Ok(())
}

fn init_data(matches: ArgMatches, display: &Display) -> Result<Viewer, Box<dyn std::error::Error>> {
    let camera = Camera::new(
        Vector3::new(0.0, 32.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0).normalize()
    );

    let mut aos_team_color = Vector3::new(0.0, 0.0, 0.0);
    if let Some(color_string) = matches.values_of("aos-team-color") {
        let values_str: Vec<&str> = color_string.collect();
        let values = values_str.iter().map(|s| s.parse::<u8>().map(|i| i as f32))
            .collect::<Result<Vec<f32>, std::num::ParseIntError>>()?;
        // guaranteed to be 3 elements by Arg match
        aos_team_color = Vector3::new(values[0], values[1], values[2]);
    }

    let program = glium::Program::from_source(display,
        &shaders::VERTEX_SHADER_SRC,
        &shaders::FRAGMENT_SHADER_SRC,
        None)?;

    let light_kv6 = kv6::KV6Model::from_file("kv6/light.kv6", display)?;
    // file match guaranteed (required), unwrap
    let user_kv6 = kv6::KV6Model::from_file(matches.value_of("file").unwrap(), display)?;

    Ok(Viewer {
        focused: true,

        camera,

        program,
        light_dir: (Vector3::new(0.0, 0.0, 0.0) - Vector3::new(-128.0, -128.0, 64.0)).normalize(),
        light_kv6,
        show_light: true,

        user_kv6,
        aos_team_color
    })
}

fn run_loop(mut viewer: Viewer, event_loop: EventLoop<()>, display: Display) {
    let MS_PER_UPDATE = Duration::new(1, 0).div_f64(60.0); // can't make const Durations in rust?

    let mut prev_time = Instant::now();
    let mut lag = Duration::new(0, 0);

    let mut ticks = 0;
    let mut frames = 0;

    eventutil::start_loop(event_loop, move |events| {
        let mut action = eventutil::LoopAction::Continue;

        lag += prev_time.elapsed();
        prev_time = Instant::now();

        for event in events {
            handle_event(&mut viewer, event, &display, &mut action);
        }

        while lag >= MS_PER_UPDATE {
            update(&mut viewer);

            lag -= MS_PER_UPDATE;
            ticks += 1;
            if ticks >= 60 {
                ticks = 0;
                frames = 0;
            }
        }

        frames += 1;

        render(&mut viewer, &display, lag.div_duration_f32(MS_PER_UPDATE));

        return action;
    });
}

fn handle_event(viewer: &mut Viewer, event: &Event<()>, display: &Display, action: &mut eventutil::LoopAction) {
    match event {
        Event::WindowEvent { event, ..} => match event {
            WindowEvent::CloseRequested =>
                *action = eventutil::LoopAction::Stop,
            WindowEvent::Focused(focus) => {
                viewer.focused = *focus;
                set_capture(&display, *focus);
            },
            WindowEvent::KeyboardInput { input, .. } => if viewer.focused {
                viewer.camera.handle_keyboard(input);

                let pressed = input.state == ElementState::Pressed;
                match input.virtual_keycode {
                    Some(controls::KEY_EXIT) => if pressed { *action = eventutil::LoopAction::Stop; },
                    Some(controls::KEY_MOVE_LIGHT) => if pressed { viewer.light_dir = -viewer.camera.orientation.z; },
                    Some(controls::KEY_SHOW_LIGHT) => if pressed { viewer.show_light = !viewer.show_light; },
                    _ => (),
                }
            },
            _ => (),
        },
        Event::DeviceEvent { event, .. } => match event {
            DeviceEvent::MouseMotion { delta } => if viewer.focused {
                viewer.camera.handle_mouse(delta.0 as f32, delta.1 as f32)
            },
            _ => ()
        }
        _ => (),
    }
}

// stub, but possibly useful in the future
fn update(viewer: &mut Viewer) {
    viewer.camera.update();
}

fn render(viewer: &mut Viewer, display: &Display, delta: f32) {
    let mut target = display.draw();

    target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);

    let params = glium::DrawParameters {
        depth: glium::draw_parameters::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        .. Default::default()
    };

    let perspective: [[f32; 4]; 4] = Camera::get_perspective_matrix(&target).into();
    let view: [[f32; 4]; 4] = viewer.camera.get_view_matrix(delta).into();
    let model: [[f32; 4]; 4] = Matrix4::from_value(1.0).into(); // identity
    let light_dir: [f32; 3] = viewer.light_dir.into();
    let aos_team_color: [f32; 3] = viewer.aos_team_color.into();

    target.draw(&viewer.user_kv6.vertex_buffer, &viewer.user_kv6.indices, &viewer.program,
        &uniform! { perspective: perspective, view: view, model: model, light_dir: light_dir, aos_team_color: aos_team_color },
        &params).unwrap();

    if viewer.show_light {
        let model: [[f32; 4]; 4] = Matrix4::from_translation(-viewer.light_dir * 128.0).into();
        let light_dir: [f32; 3] = (-viewer.light_dir).into(); // so that it's lit on the side facing the user's kv6

        target.draw(&viewer.light_kv6.vertex_buffer, &viewer.light_kv6.indices, &viewer.program,
            &uniform! { perspective: perspective, view: view, model: model, light_dir: light_dir, aos_team_color: aos_team_color },
            &params).unwrap();
    }

    target.finish().unwrap();
}