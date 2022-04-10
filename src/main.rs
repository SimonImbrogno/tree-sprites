#![feature(generic_const_exprs)]
#![feature(slice_index_methods)]
#![feature(variant_count)]
#![allow(unused_parens)]

use std::fmt::Debug;
use std::time::Duration;

use log::{debug, error};
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

mod debug;
mod game;
mod render;
mod timer;

use debug::DebugTimers;
use game::game_state::{GameState, Input};
use render::RenderState;
use timer::{AverageDurationTimer, DurationTimer, TargetTimer, Timer, TimerState};

fn process_window_event(event: WindowEvent, render_state: &mut RenderState, input_state: &mut Input, control_flow: &mut ControlFlow) {
    match event {
        WindowEvent::CloseRequested                            => *control_flow = ControlFlow::Exit,
        WindowEvent::Resized(physical_size)                    => render_state.resize(physical_size),
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => render_state.resize(*new_inner_size),

        WindowEvent::KeyboardInput { input, .. } => {

            if let Some(keycode) = input.virtual_keycode {
                if input.state == ElementState::Pressed {
                    match keycode {
                        VirtualKeyCode::A => input_state.left  = true,
                        VirtualKeyCode::D => input_state.right = true,
                        VirtualKeyCode::S => input_state.down  = true,
                        VirtualKeyCode::W => input_state.up    = true,

                        VirtualKeyCode::LControl => input_state.zoom_in  = true,
                        VirtualKeyCode::Space    => input_state.zoom_out = true,

                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }

                if input.state == ElementState::Released {
                    match keycode {
                        VirtualKeyCode::A => input_state.left  = false,
                        VirtualKeyCode::D => input_state.right = false,
                        VirtualKeyCode::S => input_state.down  = false,
                        VirtualKeyCode::W => input_state.up    = false,

                        VirtualKeyCode::P => input_state.pause = !input_state.pause,
                        VirtualKeyCode::G => input_state.show_grid = !input_state.show_grid,
                        VirtualKeyCode::H => input_state.show_dual = !input_state.show_dual,
                        VirtualKeyCode::T => input_state.show_trees = !input_state.show_trees,

                        VirtualKeyCode::LControl => input_state.zoom_in  = false,
                        VirtualKeyCode::Space    => input_state.zoom_out = false,

                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }
        },

        _ => {}
    }
}

type RenderResult = Result<(), wgpu::SurfaceError>;

fn handle_render_result(render_result: RenderResult, render_state: &mut RenderState, window: &winit::window::Window, control_flow: &mut ControlFlow) {
    match render_result {
        // Success, party time.
        Ok(_) => {}

        // Surface lost, try to reconfigure.
        Err(wgpu::SurfaceError::Lost) => {
            debug!("Surface lost: Attemting to reconfigure.");
            let size = window.inner_size();
            render_state.resize(size);
        },

        // Something crazy happened, no memory for surface, bail!
        Err(wgpu::SurfaceError::OutOfMemory) => {
            debug!("Render surface OOM: Quitting.");
            *control_flow = ControlFlow::Exit;
        },

        // Other errors should resolve themselves by next frame.
        Err(e) => {
            error!("Failed to render: {:?}", e);
        },
    }
}

const UPS_TARGET: u64 = 120;
const FPS_TARGET: u64 = 120;

fn main() {
    //Duration constructor is unstable as constfn;
    let update_target_dt = Duration::from_secs_f32(1.0 / (UPS_TARGET as f32));
    let frame_target_dt = Duration::from_secs_f32(1.0 / (FPS_TARGET as f32));

    env_logger::init();
    debug!("Logger initialized");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut game_state = Box::new(GameState::new());
    let mut render_state = pollster::block_on(RenderState::new(&window, &game_state));

    let mut input = Input::default();

    let mut dbgt = DebugTimers::new();

    // let running_timer = DurationTimer::new();
    let mut update_timer = TargetTimer::new(update_target_dt);

    let mut window_title_update_timer = TargetTimer::new(Duration::from_secs_f32(0.25));

    let mut loop_timer = DurationTimer::new();
    let mut sim_time = Duration::from_secs(0);
    let mut accumulator = Duration::from_secs(0);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { window_id, event } => {
                if window_id == window.id() {
                    process_window_event(event, &mut render_state, &mut input, control_flow);
                }
            },
            Event::MainEventsCleared => {
                let mut elapsed = update_timer.elapsed();
                update_timer.reset();

                if (elapsed.as_secs_f32() - update_target_dt.as_secs_f32()).abs() < 0.0002 {
                    elapsed = update_target_dt;
                }

                accumulator += elapsed;

                let mut count = 0;

                while accumulator > update_target_dt {
                    count += 1;

                    sim_time    += update_target_dt;
                    accumulator -= update_target_dt;

                    input.dt = update_target_dt * 2;
                    input.t = sim_time;

                    measure!(dbgt.long_avg_update_timer, {
                        measure!(dbgt.avg_update_timer, {
                            game_state.update(&input);
                        });
                    });
                }

                if count > 1 {
                    debug!("+{} updates...", count);
                }

                // NOTE: Timing happens internally
                let render_result = render_state.try_render(&game_state, &mut dbgt);
                handle_render_result(render_result, &mut render_state, &window, control_flow);

                let loop_time = loop_timer.elapsed();
                loop_timer.reset();

                if let TimerState::Ready(_) = window_title_update_timer.check() {
                    window_title_update_timer.reset();

                    let rps = 1.0 / loop_time.as_secs_f32();

                    let avg_ut = dbgt.avg_update_timer.average().as_micros();
                    let avg_rt = dbgt.avg_render_timer.average().as_micros();
                    let avg_total = avg_rt + avg_ut;

                    let ups_budget_usage   = (avg_ut as f32 / frame_target_dt.as_micros() as f32) * 100.0;
                    let fps_budget_usage   = (avg_rt as f32 / frame_target_dt.as_micros() as f32) * 100.0;
                    let total_budget_usage = (avg_total as f32 / frame_target_dt.as_micros() as f32) * 100.0;

                    window.set_title(&format!(
                        "RPS {:.0} ({:.02}μs : {:.02}%) - UPS ({:.02}μs : {:.02}%) --- Total: {:.02}μs : {:.02}%",
                        rps, avg_rt, fps_budget_usage,
                        avg_ut, ups_budget_usage,
                        avg_total, total_budget_usage
                    ));
                }
            }
            _ => {}
        }
    });
}
