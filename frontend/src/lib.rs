#![no_std]
#![allow(unused_imports)]

mod backend;
mod error;

pub use backend::*;
pub use error::*;

use pocket_knife_file_format::FileTable;

extern crate alloc;

use alloc::{rc::Rc, vec::Vec, boxed::Box, string::String, fmt::format, format};
use core::{mem::transmute, cell::RefCell, ops::DerefMut};
use embedded_graphics::{pixelcolor::{Rgb888, IntoStorage, Rgb565, raw::ToBytes}, geometry::{Point, OriginDimensions}, image::GetPixel, Pixel};
use rgb::RGB;
use slint::{platform::{software_renderer::{Rgb565Pixel, MinimalSoftwareWindow, RepaintBufferType}, Platform}, PhysicalSize, ModelRc, StandardListViewItem, SharedString, Image, SharedPixelBuffer, Rgb8Pixel};
use tinybmp::{Bmp, RawBmp, Bpp, RowOrder};

slint::include_modules!();

pub const SCREEN_WIDTH: u32 = 266;
pub const SCREEN_HEIGHT: u32 = 240;

pub const SCREEN_PIXELS: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

pub const INTERACT_SCROLL_SPEED_X: usize = 0;
pub const INTERACT_SCROLL_SPEED_Y: usize = 1;

#[derive(Clone)]
pub struct App<B: Backend> {
    pub backend: B,
    pub slint_window: Rc<MinimalSoftwareWindow>,
    pub ui: Rc<UI>,
    pub file_table: Rc<FileTable>,
}

impl <B: Backend> App<B> {
    pub fn new(backend: B) -> App<B> {
        let slint_window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        slint_window.set_size(PhysicalSize { width: SCREEN_WIDTH, height: SCREEN_HEIGHT });

        slint::platform::set_platform(B::slint_platform(slint_window.clone())).unwrap();

        let ui = Rc::new(UI::new().unwrap());

        let file_table = {
            let mut backend = backend.clone();
            let file_table = FileTable::read(&mut backend).unwrap();
            Rc::from(file_table)
        };

        {
            let slint_window = slint_window.clone();
            ui.on_request_redraw(move || slint_window.request_redraw());
        }
        {
            let mut backend = backend.clone();
            let file_table = file_table.clone();
            ui.on_load_image(move |filename| load_image(&mut backend, &file_table, filename.into()))
        }

        ui.set_fallback_image(Image::from_rgb8(SharedPixelBuffer::new(0, 0)));

        let filenames =
            file_table.0.keys()
                .map(SharedString::from)
                .map(StandardListViewItem::from)
                .collect::<Vec<_>>();

        ui.set_filenames(ModelRc::from(filenames.as_slice()));

        B::debug(format!("{:?}", filenames));

        ui.show().unwrap();

        App { slint_window, ui, backend, file_table }
    }

    // todo: only update changed region from renderer
    pub fn draw(&mut self) {
        self.update_interacts();
        slint::platform::update_timers_and_animations();
        self.slint_window.draw_if_needed(|renderer| {
            let mut buffer = [Rgb565Pixel(0); SCREEN_PIXELS];
            renderer.render(&mut buffer, SCREEN_WIDTH as usize);
            self.backend.blit(buffer);
        });
    }

    fn update_interacts(&self) {
        for (interact_id, interact) in [
            |ui: &UI, val: u32| ui.set_scroll_speed_x(val as f32),
            |ui: &UI, val: u32| ui.set_scroll_speed_y(val as f32),
        ].iter().enumerate() {
            if self.backend.interact_changed(interact_id) {
                B::debug(format!("interact changed: {}", interact_id));
                interact(&self.ui, self.backend.interact_read(interact_id));
            }
        }
    }
}

fn load_image(backend: &mut impl Backend, file_table: &FileTable, filename: String) -> Image {
    let bytes = file_table.open_file(backend, filename).unwrap();
    let bmp: Bmp<Rgb888> = Bmp::from_slice(&bytes).unwrap();
    let mut buffer: SharedPixelBuffer<Rgb8Pixel> = SharedPixelBuffer::new(bmp.size().width, bmp.size().height);
    {
        let mut buffer_mut = buffer.make_mut_slice().chunks_exact_mut(bmp.size().width as usize).collect::<Vec<_>>();
        for pixel in bmp.pixels() {
            buffer_mut[pixel.0.y as usize][pixel.0.x as usize] = Rgb8Pixel::from(pixel.1.to_ne_bytes());
        }
    }
    Image::from_rgb8(buffer)
}
