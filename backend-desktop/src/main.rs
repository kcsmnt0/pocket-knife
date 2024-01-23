#![feature(slice_as_chunks)]

mod pocket;
use pocket::*;

use pocket_knife_frontend::*;

use i_slint_backend_winit::winit::dpi::{LogicalSize, Size};
use i_slint_backend_winit::winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use i_slint_backend_winit::winit::event_loop::{ControlFlow, EventLoop};
use i_slint_backend_winit::winit::keyboard::{KeyCode, PhysicalKey};
use i_slint_backend_winit::winit::window::WindowBuilder;
use i_slint_core::SharedString;
use i_slint_core::platform::Key;
use pixels::{Pixels, SurfaceTexture};
use std::cell::RefCell;
use std::env;
use std::fs::{read_to_string, File};
use std::rc::Rc;

fn main() {
    // SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();

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
    let pixels = Rc::new(RefCell::new(Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture).unwrap()));

    let filesystem_image = Rc::new(RefCell::new(File::open(args[1].clone()).unwrap()));

    let interact_values = Rc::new(RefCell::new(load_interact_values().map(|value| Interact { value, changed: true })));

    let pocket = Pocket {
        pixels,
        filesystem_image,
        interact_values,
    };

    let mut app = App::new(pocket);

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run(move |event, control_flow| {
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
                    println!("reloading interact values");
                    let new_values = load_interact_values();
                    let mut values = app.backend.interact_values.borrow_mut();
                    for (i, old_value) in values.as_mut().iter_mut().enumerate() {
                        if old_value.value != new_values[i] {
                            old_value.value = new_values[i];
                            old_value.changed = true;
                        }
                    }
                } else if physical_key == PhysicalKey::Code(KeyCode::F12) && state.is_pressed() {
                    control_flow.exit();
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
