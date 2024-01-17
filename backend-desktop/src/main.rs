use i_slint_backend_winit::winit::dpi::{LogicalSize, Size};
use i_slint_backend_winit::winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use i_slint_backend_winit::winit::event_loop::{ControlFlow, EventLoop};
use i_slint_backend_winit::winit::keyboard::{KeyCode, PhysicalKey};
use i_slint_backend_winit::winit::window::WindowBuilder;
use i_slint_core::SharedString;
use i_slint_core::platform::{Key, Platform};
use i_slint_core::software_renderer::{MinimalSoftwareWindow, Rgb565Pixel};
use log::info;
use pixels::{Pixels, SurfaceTexture};
use rgb565::Rgb565;
use simple_logger::SimpleLogger;
use std::env;
use std::fs::{self, read_to_string};
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use pocket_knife_frontend::*;

struct Pocket {
    pixels: Pixels,
    slot1_file: Vec<u8>,
    interact_values: [u32; 16],
}

impl Backend for Pocket {
    fn slint_platform(window: Rc<MinimalSoftwareWindow>) -> Box<dyn Platform + 'static> {
        Box::new(SlintPlatform {
            window: window.clone(),
            start_time: SystemTime::now(),
        })
    }

    fn slot_read(&self, _slot_id: u16) -> Result<Vec<u8>, SlotReadError> {
        Ok(self.slot1_file.clone())
    }

    fn blit(&mut self, buffer: [Rgb565Pixel; SCREEN_PIXELS]) {
        let frame = self.pixels.frame_mut();
        for (i, pixel) in buffer.into_iter().enumerate() {
            let [r, g, b] = Rgb565::from_rgb565(pixel.0).to_srgb888_components();
            frame[i*4..i*4+4].copy_from_slice(&[r, g, b, u8::MAX]);
        }
        self.pixels.render().unwrap();
    }

    fn interact_read(&self, interact_id: usize) -> u32 {
        self.interact_values[interact_id]
    }
}

struct SlintPlatform {
    window: Rc<MinimalSoftwareWindow>,
    start_time: SystemTime,
}

impl Platform for SlintPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn i_slint_core::window::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        SystemTime::now().duration_since(self.start_time).unwrap()
    }
}

fn main() {
    SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();
    let args = &env::args().collect::<Vec<String>>();

    let event_loop = EventLoop::new().unwrap();

    let size = Size::Logical(LogicalSize { width: SCREEN_WIDTH as f64, height: SCREEN_HEIGHT as f64 });
    let window = WindowBuilder::new()
        .with_title("Pocket")
        .with_resizable(false)
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap();

    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let pixels = Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture).unwrap();

    let pocket = Pocket {
        pixels,
        slot1_file: fs::read(args[1].clone()).unwrap(),
        interact_values: load_interact_values(),
    };

    App::with_new(pocket, |mut app| {
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run(move |event, _control_flow| {
            match event {
                Event::WindowEvent { 
                    event: WindowEvent::KeyboardInput {
                        event: KeyEvent {
                            state,
                            physical_key,
                            ..
                        },
                        ..
                    },
                    ..
                } => {
                    if physical_key == PhysicalKey::Code(KeyCode::F5) && state.is_pressed() {
                        info!("reloading interact values");
                        app.backend.borrow_mut().interact_values = load_interact_values();
                    } else if let Some(text) = physical_key_text(physical_key) {
                        app.slint_window.dispatch_event(match state {
                            ElementState::Pressed => slint::platform::WindowEvent::KeyPressed { text },
                            ElementState::Released => slint::platform::WindowEvent::KeyReleased { text },
                        });
                    }
                },

                _ => { },
            }

            app.draw();
        }).unwrap();
    });
}

fn physical_key_text(physical_key: PhysicalKey) -> Option<SharedString> {
    match physical_key {
        PhysicalKey::Code(KeyCode::ArrowUp) => Some(SharedString::from(Key::UpArrow)),
        PhysicalKey::Code(KeyCode::ArrowDown) => Some(SharedString::from(Key::DownArrow)),
        PhysicalKey::Code(KeyCode::ArrowLeft) => Some(SharedString::from(Key::LeftArrow)),
        PhysicalKey::Code(KeyCode::ArrowRight) => Some(SharedString::from(Key::RightArrow)),
        PhysicalKey::Code(KeyCode::KeyA) => Some(SharedString::from("a")),
        PhysicalKey::Code(KeyCode::KeyB) => Some(SharedString::from("b")),
        PhysicalKey::Code(KeyCode::KeyX) => Some(SharedString::from("x")),
        PhysicalKey::Code(KeyCode::KeyY) => Some(SharedString::from("y")),
        PhysicalKey::Code(KeyCode::KeyL) => Some(SharedString::from("l")),
        PhysicalKey::Code(KeyCode::KeyR) => Some(SharedString::from("r")),
        PhysicalKey::Code(KeyCode::PageUp) => Some(SharedString::from(Key::PageUp)),
        PhysicalKey::Code(KeyCode::PageDown) => Some(SharedString::from(Key::PageDown)),
        PhysicalKey::Code(KeyCode::Home) => Some(SharedString::from(Key::Home)),
        PhysicalKey::Code(KeyCode::End) => Some(SharedString::from(Key::End)),
        PhysicalKey::Code(KeyCode::Escape) => Some(SharedString::from(Key::Escape)),
        PhysicalKey::Code(KeyCode::Enter) => Some(SharedString::from(Key::Return)),
        _ => None,
    }
}

fn load_interact_values() -> [u32; 16] {
    read_to_string("interact.values").unwrap()
        .lines()
        .map(|line| line.parse().unwrap())
        .collect::<Vec<_>>()
        .try_into().unwrap()
}
