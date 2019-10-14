use std::time::{Duration, Instant};
use glium::glutin::event_loop::{EventLoop, ControlFlow};
use glium::glutin::event::{Event, StartCause};

pub enum LoopAction {
    Stop,
    Continue,
}

pub fn start_loop<F>(event_loop: EventLoop<()>, mut callback: F)->! where F: 'static + FnMut(&Vec<Event<()>>) -> LoopAction {
    let mut events_buffer = Vec::new();
    let mut next_frame_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        let run_callback = match event {
            Event::NewEvents(cause) => {
                match cause {
                    StartCause::ResumeTimeReached { .. } | StartCause::Init => {
                        true
                    },
                    _ => false
                }
            },
            _ => {
                events_buffer.push(event);
                false
            }
        };

        let action = if run_callback {
            let action = callback(&events_buffer);
            next_frame_time = Instant::now() + Duration::from_nanos(01);
            events_buffer.clear();
            action
        } else {
            LoopAction::Continue
        };

        match action {
            LoopAction::Continue => {
                *control_flow = ControlFlow::WaitUntil(next_frame_time);
            },
            LoopAction::Stop => *control_flow = ControlFlow::Exit
        }
    })
}