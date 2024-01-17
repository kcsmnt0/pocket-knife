#![no_std]

extern crate alloc;

use core::{mem::transmute, cell::RefCell, borrow::BorrowMut};

use alloc::{rc::Rc, vec::Vec, boxed::Box};
use embedded_graphics::{pixelcolor::{Rgb565, IntoStorage}, geometry::Point, image::GetPixel};
use slint::{platform::{software_renderer::{Rgb565Pixel, MinimalSoftwareWindow, RepaintBufferType}, Platform}, PhysicalSize};
use tinybmp::Bmp;

mod error;
pub use error::*;

slint::include_modules!();

pub const SCREEN_WIDTH: u32 = 266;
pub const SCREEN_HEIGHT: u32 = 240;

pub const SCREEN_PIXELS: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

pub const INTERACT_SCROLL_SPEED_X: usize = 0;
pub const INTERACT_SCROLL_SPEED_Y: usize = 1;

pub trait Backend {
    fn slint_platform(window: Rc<MinimalSoftwareWindow>) -> Box<dyn Platform + 'static>;
    fn blit(&mut self, buffer: [Rgb565Pixel; SCREEN_PIXELS]);
    fn slot_read(&self, slot_id: u16) -> Result<Vec<u8>, SlotReadError>;
    fn interact_read(&self, interact_id: usize) -> u32;
}

pub struct App<B: Backend + 'static> {
    pub slint_window: Rc<MinimalSoftwareWindow>,
    pub backend: Rc<RefCell<B>>,
    pub ui: UI,
}

impl <B: Backend + 'static> App<B> {
    pub fn with_new<T>(backend: B, f: impl FnOnce(App<B>) -> T) -> T {
        let backend = Rc::new(RefCell::new(backend));
        let slint_window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        slint_window.set_size(PhysicalSize { width: SCREEN_WIDTH, height: SCREEN_HEIGHT });

        slint::platform::set_platform(B::slint_platform(slint_window.clone())).unwrap();

        let ui = UI::new().unwrap();

        {
            let slint_window = slint_window.clone();
            ui.on_request_redraw(move || slint_window.request_redraw());
        }
        {
            let backend = backend.clone();
            ui.on_scroll_speed_x(move || backend.borrow().interact_read(INTERACT_SCROLL_SPEED_X) as i32);
        }
        {
            let backend = backend.clone();
            ui.on_scroll_speed_y(move || backend.borrow().interact_read(INTERACT_SCROLL_SPEED_Y) as i32);
        }

        ui.show().unwrap();

        let mut app = App { slint_window, backend, ui };
        app.draw_menu();
        f(app)
    }

    // todo: only update changed region from renderer
    pub fn draw(&mut self) {
        slint::platform::update_timers_and_animations();
        if self.ui.global::<Data>().get_menu_active() {
            self.draw_menu();
        } else {
            self.draw_image();
        }
    }

    fn draw_menu(&mut self) {
        self.slint_window.draw_if_needed(|renderer| {
            let mut buffer = [Rgb565Pixel(0); SCREEN_PIXELS];
            renderer.render(&mut buffer, SCREEN_WIDTH as usize);
            (*self.backend).borrow_mut().borrow_mut().blit(buffer);
        });
    }

    fn draw_image(&mut self) {
        let buffer = load_image(self.backend.as_ref(), 1, |bmp| {
            let scroll_x = self.ui.global::<Data>().get_scroll_x();
            let scroll_y = self.ui.global::<Data>().get_scroll_y();
            let mut buffer = [[Rgb565Pixel(0); SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize];
            for i in 0..SCREEN_WIDTH {
                for j in 0..SCREEN_HEIGHT {
                    if let Some(pixel) = bmp.pixel(Point::new(i as i32 + scroll_x, j as i32 + scroll_y)) {
                        buffer[j as usize][i as usize] = Rgb565Pixel(pixel.into_storage());
                    }
                }
            }
            buffer
        }).unwrap();

        (*self.backend).borrow_mut().borrow_mut().blit(unsafe { transmute(buffer) });
    }
}

fn load_image<T>(backend: &RefCell<impl Backend>, slot_id: u16, f: impl FnOnce(Bmp<Rgb565>) -> T) -> Result<T, ImageReadError> {
    let bytes = backend.borrow().slot_read(slot_id).map_err(ImageReadError::SlotReadError)?;
    Bmp::from_slice(&bytes[..]).map(f).map_err(ImageReadError::ParseError)
}
