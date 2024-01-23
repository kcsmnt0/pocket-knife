use crate::SCREEN_PIXELS;

use alloc::{string::String, rc::Rc, format, boxed::Box};
use chrono::{Datelike, NaiveDateTime, Timelike};
use core::cell::RefCell;
use embedded_io::{Read, Seek};
use slint::platform::{software_renderer::{MinimalSoftwareWindow, Rgb565Pixel}, Platform};

pub trait Backend: 'static + Read + Seek + Clone {
    fn debug(message: String);
    fn slint_platform(window: Rc<MinimalSoftwareWindow>) -> Box<dyn Platform + 'static>;
    fn blit(&self, buffer: [Rgb565Pixel; SCREEN_PIXELS]);
    fn interact_read(&self, interact_id: usize) -> u32;
    fn interact_changed(&self, interact_id: usize) -> bool;
    fn now(&self) -> NaiveDateTime;
}
