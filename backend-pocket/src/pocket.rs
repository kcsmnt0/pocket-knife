use crate::Error;
use pocket_knife_frontend::*;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::{vec, format};
use core::cell::RefCell;
use core::slice::{from_raw_parts_mut, from_raw_parts};
use chrono::NaiveDateTime;
use embedded_io::{ErrorType, Read, Seek, SeekFrom};
use litex_openfpga::{println, SlintPlatform, UART, File};
use litex_pac::Peripherals;
use litex_pac::constants::{CONFIG_CLOCK_FREQUENCY, VIDEO_FRAMEBUFFER_BASE};
use slint::platform::Platform;
use slint::platform::software_renderer::{MinimalSoftwareWindow, Rgb565Pixel};

pub const FRAMEBUFFER_ADDRESS: *mut Rgb565Pixel = VIDEO_FRAMEBUFFER_BASE as *mut Rgb565Pixel;

#[derive(Clone)]
pub struct Pocket {
    seek_position: Rc<RefCell<u32>>,
}

impl Backend for Pocket {
    fn debug(message: String) {
        println!("{}", message);
    }

    fn slint_platform(window: Rc<MinimalSoftwareWindow>) -> Box<dyn Platform + 'static> {
        Box::new(SlintPlatform::new(
            window,
            CONFIG_CLOCK_FREQUENCY,
        ))
    }

    fn blit(&self, buffer: [Rgb565Pixel; SCREEN_PIXELS]) {
        let video = unsafe { Peripherals::steal().APF_VIDEO };
        while !video.video.read().vblank_triggered().bit() {
            // wait for vblank interval
        }
        let vram: &mut [Rgb565Pixel; SCREEN_PIXELS] = unsafe {
            from_raw_parts_mut(FRAMEBUFFER_ADDRESS, SCREEN_PIXELS)
        }.try_into().unwrap();
        vram.clone_from(&buffer);
    }

    fn interact_read(&self, interact_id: usize) -> u32 {
        let interacts = unsafe { Peripherals::steal().APF_INTERACT };
        match interact_id {
            0 => interacts.interact0.read().bits(),
            1 => interacts.interact1.read().bits(),
            2 => interacts.interact2.read().bits(),
            3 => interacts.interact3.read().bits(),
            4 => interacts.interact4.read().bits(),
            5 => interacts.interact5.read().bits(),
            6 => interacts.interact6.read().bits(),
            7 => interacts.interact7.read().bits(),
            8 => interacts.interact8.read().bits(),
            9 => interacts.interact9.read().bits(),
            10 => interacts.interact10.read().bits(),
            11 => interacts.interact11.read().bits(),
            12 => interacts.interact12.read().bits(),
            13 => interacts.interact13.read().bits(),
            14 => interacts.interact14.read().bits(),
            15 => interacts.interact15.read().bits(),
            _ => panic!("invalid interact"),
        }
    }

    fn interact_changed(&self, interact_id: usize) -> bool {
        let interacts = unsafe { Peripherals::steal().APF_INTERACT };
        match interact_id {
            0 => interacts.interact_changed0.read().bits() != 0,
            1 => interacts.interact_changed1.read().bits() != 0,
            2 => interacts.interact_changed2.read().bits() != 0,
            3 => interacts.interact_changed3.read().bits() != 0,
            4 => interacts.interact_changed4.read().bits() != 0,
            5 => interacts.interact_changed5.read().bits() != 0,
            6 => interacts.interact_changed6.read().bits() != 0,
            7 => interacts.interact_changed7.read().bits() != 0,
            8 => interacts.interact_changed8.read().bits() != 0,
            9 => interacts.interact_changed9.read().bits() != 0,
            10 => interacts.interact_changed10.read().bits() != 0,
            11 => interacts.interact_changed11.read().bits() != 0,
            12 => interacts.interact_changed12.read().bits() != 0,
            13 => interacts.interact_changed13.read().bits() != 0,
            14 => interacts.interact_changed14.read().bits() != 0,
            15 => interacts.interact_changed15.read().bits() != 0,
            _ => panic!("invalid interact"),
        }
    }

    fn now(&self) -> NaiveDateTime {
        let rtc = unsafe { Peripherals::steal().APF_RTC };
        let time = rtc.unix_seconds.read().unix_seconds().bits();
        NaiveDateTime::from_timestamp_opt(time as i64, 0).unwrap()
    }
}

// never does partial reads
impl Read for Pocket {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        println!(
            "read: buffer address {:X?}, length {}, data address {:X?}",
            buffer as *mut [u8],
            buffer.len(),
            *self.seek_position.borrow(),
        );

        File::request_read(
            *self.seek_position.borrow(),
            buffer.len() as u32,
            buffer as *mut [u8] as *mut u8 as u32,
            1,
        );

        println!("read requested");

        File::block_op_complete();

        println!("buffer bytes: {:X?}", buffer);

        *self.seek_position.borrow_mut() += buffer.len() as u32;

        Ok(buffer.len())
    }

    fn read_exact(&mut self, buffer: &mut [u8]) -> Result<(), embedded_io::ReadExactError<Self::Error>> {
        self.read(buffer).map_err(embedded_io::ReadExactError::Other)?;
        Ok(())
    }
}

impl Seek for Pocket {
    fn seek(&mut self, target: SeekFrom) -> Result<u64, Self::Error> {
        let mut seek_position = self.seek_position.borrow_mut();
        *seek_position = match target {
            SeekFrom::Current(offset) => seek_position.wrapping_add_signed(offset as i32),
            SeekFrom::Start(offset) => offset as u32,
            SeekFrom::End(offset) => {
                File::size(1).wrapping_add_signed(offset as i32)
            },
        };
        Ok(*seek_position as u64)
    }
}

impl ErrorType for Pocket {
    type Error = Error;
}

impl Default for Pocket {
    fn default() -> Self {
        Pocket {
            seek_position: Rc::new(RefCell::new(0)),
        }
    }
}
