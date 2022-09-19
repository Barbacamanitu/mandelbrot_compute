use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::{
    computer::{Computer, SampleLocation},
    gpu_interface::GPUInterface,
    math::UVec2,
    renderer::Renderer,
};

pub struct App {
    pub gpu: GPUInterface,
    pub computer: Computer,
    pub renderer: Renderer,
    pub sample_location: SampleLocation,
}

impl App {
    pub fn new(size: UVec2, window: &Window) -> App {
        let gpu = GPUInterface::new(window);
        let computer = Computer::new(size, &gpu);
        let renderer = Renderer::new(&gpu, size, window);
        App {
            gpu,
            computer,
            renderer: renderer,
            sample_location: SampleLocation::default(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => match input.state {
                winit::event::ElementState::Pressed => {
                    if input.virtual_keycode == Some(VirtualKeyCode::Left) {
                        self.sample_location.left();
                    }
                    if input.virtual_keycode == Some(VirtualKeyCode::Right) {
                        self.sample_location.right();
                    }

                    if input.virtual_keycode == Some(VirtualKeyCode::Up) {
                        self.sample_location.up();
                    }

                    if input.virtual_keycode == Some(VirtualKeyCode::Down) {
                        self.sample_location.down();
                    }

                    if input.virtual_keycode == Some(VirtualKeyCode::NumpadAdd) {
                        self.sample_location.zoom_in();
                    }

                    if input.virtual_keycode == Some(VirtualKeyCode::NumpadSubtract) {
                        self.sample_location.zoom_out();
                    }
                }
                winit::event::ElementState::Released => {}
            },
            _ => {}
        }
        return false;
    }
}
