use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::app::App;

mod app;
mod computer;
mod gpu_interface;
mod math;
mod renderer;

fn main() {
    let size = UVec2::new(1024, 1024);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(size.x, size.y))
        .with_title("GPU_Automata")
        .with_position(PhysicalPosition::new(0, 0))
        .build(&event_loop)
        .unwrap();
    let mut app = App::new(size, &window);

    event_loop.run(move |event, _, control_flow| {
        //sim.renderer.handle_events(&event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                //.Handle gui events

                if !app.handle_event(event) {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            app.renderer.resize(*physical_size, &mut app.gpu);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so we have to dereference it twice
                            app.renderer.resize(**new_inner_size, &mut app.gpu);
                        }
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let mandelbrot = app
                    .computer
                    .run(&app.gpu, &app.sample_location.to_mandlebrot_params(180));
                match app.renderer.render(&app.gpu, mandelbrot) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        app.renderer.resize(app.gpu.size, &mut app.gpu)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
