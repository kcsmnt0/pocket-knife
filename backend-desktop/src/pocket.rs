use pocket_knife_frontend::{Backend, SCREEN_PIXELS};

use chrono::{offset::Local, NaiveDateTime};
use i_slint_core::{software_renderer::{MinimalSoftwareWindow, Rgb565Pixel}, platform::Platform};
use pixels::Pixels;
use rgb565::Rgb565;
use std::{rc::Rc, time::{SystemTime, Duration}, fs::File, io::{Read, Seek}, cell::RefCell};

#[derive(Clone)]
pub struct Pocket {
    pub pixels: Rc<RefCell<Pixels>>,
    pub filesystem_image: Rc<RefCell<File>>,
    pub interact_values: Rc<RefCell<[Interact; 16]>>,
}

#[derive(Clone, Copy)]
pub struct Interact {
    pub value: u32,
    pub changed: bool,
}

struct SlintPlatform {
    window: Rc<MinimalSoftwareWindow>,
    start_time: SystemTime,
}

impl Backend for Pocket {
    fn debug(message: String) {
        println!("{}", message);
    }

    fn slint_platform(window: Rc<MinimalSoftwareWindow>) -> Box<dyn Platform + 'static> {
        Box::new(SlintPlatform {
            window,
            start_time: SystemTime::now(),
        })
    }

    fn blit(&self, buffer: [Rgb565Pixel; SCREEN_PIXELS]) {
        {
            let mut pixels = self.pixels.borrow_mut();
            let frame = pixels.frame_mut();
            for (i, pixel) in buffer.into_iter().enumerate() {
                let [r, g, b] = Rgb565::from_rgb565(pixel.0).to_srgb888_components();
                frame[i*4..i*4+4].copy_from_slice(&[r, g, b, u8::MAX]);
            }
        }
        self.pixels.borrow().render().unwrap();
    }

    fn interact_read(&self, interact_id: usize) -> u32 {
        self.interact_values.borrow()[interact_id].value
    }

    fn interact_changed(&self, interact_id: usize) -> bool {
        let value = self.interact_values.borrow()[interact_id].changed;
        self.interact_values.borrow_mut().as_mut()[interact_id].changed = false;
        value
    }

    fn now(&self) -> NaiveDateTime {
        Local::now().naive_local()
    }
}

impl embedded_io::ErrorType for Pocket {
    type Error = std::io::Error;
}

impl embedded_io::Read for Pocket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.filesystem_image.borrow_mut().read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), embedded_io::ReadExactError<Self::Error>> {
        self.filesystem_image.borrow_mut().read_exact(buf).map_err(From::from)
    }
}

impl embedded_io::Seek for Pocket {
    fn seek(&mut self, pos: embedded_io::SeekFrom) -> Result<u64, Self::Error> {
        self.filesystem_image.borrow_mut().seek(pos.into())
    }

    fn rewind(&mut self) -> Result<(), Self::Error> {
        self.filesystem_image.borrow_mut().rewind()
    }

    fn stream_position(&mut self) -> Result<u64, Self::Error> {
        self.filesystem_image.borrow_mut().stream_position()
    }
}

impl Platform for SlintPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn i_slint_core::window::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        SystemTime::now().duration_since(self.start_time).unwrap()
    }
}
