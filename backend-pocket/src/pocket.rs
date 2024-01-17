use core::slice::from_raw_parts_mut;

use alloc::boxed::Box;
use alloc::rc::Rc;
use litex_pac::constants::CONFIG_CLOCK_FREQUENCY;
use pocket_knife_frontend::*;

use alloc::vec;
use alloc::vec::Vec;
use litex_openfpga::{println, SlintPlatform};
use litex_pac::Peripherals;
use slint::platform::Platform;
use slint::platform::software_renderer::{MinimalSoftwareWindow, Rgb565Pixel};

use crate::FRAMEBUFFER_ADDRESS;

pub struct Pocket(pub Peripherals);

impl Backend for Pocket {
    fn slint_platform(window: Rc<MinimalSoftwareWindow>) -> Box<dyn Platform + 'static> {
        Box::new(SlintPlatform::new(
            window.clone(),
            CONFIG_CLOCK_FREQUENCY,
        ))
    }

    // todo: check if file size is too large for heap
    fn slot_read(&self, slot_id: u16) -> Result<Vec<u8>, SlotReadError> {
        self.0.APF_BRIDGE.slot_id.write(|w| w.slot_id().variant(slot_id));
        while self.0.APF_BRIDGE.slot_id.read().slot_id().bits() != 1 {
            // wait for slot change
        }

        let file_bytes_len = self.0.APF_BRIDGE.file_size.read().file_size().bits();
        let (file_ptr, file_len, file_cap) = vec![0u8; file_bytes_len as usize].into_raw_parts();

        self.0.APF_BRIDGE.data_offset.write(|w| w.data_offset().variant(0));
        self.0.APF_BRIDGE.transfer_length.write(|w| w.transfer_length().variant(file_bytes_len));
        self.0.APF_BRIDGE.ram_data_address.write(|w| w.ram_data_address().variant(file_ptr as u32));
        self.0.APF_BRIDGE.request_read.write(|w| w.request_read().bit(true));

        let result_code = self.0.APF_BRIDGE.command_result_code.read().command_result_code().bits();
        if result_code != 0 {
            return Err(if result_code == 1 { SlotReadError::InvalidSlot } else { SlotReadError::Unknown });
        }

        while !self.0.APF_BRIDGE.status.read().status().bit() {
            // wait for read to start
        }

        self.0.APF_BRIDGE.transfer_length.write(|w| w.transfer_length().variant(0));
        self.0.APF_BRIDGE.ram_data_address.write(|w| w.ram_data_address().variant(0));
        self.0.APF_BRIDGE.request_read.write(|w| w.request_read().bit(true));

        while !self.0.APF_BRIDGE.status.read().status().bit() {
            // wait for first read to finish
        }

        Ok(unsafe { Vec::from_raw_parts(file_ptr, file_len, file_cap) })
    }

    fn blit(&mut self, buffer: [Rgb565Pixel; SCREEN_PIXELS]) {
        while !self.0.APF_VIDEO.video.read().vblank_triggered().bit() {
            // wait for vblank interval
        }
        let vram: &mut [Rgb565Pixel; SCREEN_PIXELS] = unsafe {
            from_raw_parts_mut(FRAMEBUFFER_ADDRESS, SCREEN_PIXELS)
        }.try_into().unwrap();
        vram.clone_from(&buffer);
    }

    fn interact_read(&self, interact_id: usize) -> u32 {
        match interact_id {
            0 => self.0.APF_INTERACT.interact0.read().bits(),
            1 => self.0.APF_INTERACT.interact1.read().bits(),
            2 => self.0.APF_INTERACT.interact2.read().bits(),
            3 => self.0.APF_INTERACT.interact3.read().bits(),
            4 => self.0.APF_INTERACT.interact4.read().bits(),
            5 => self.0.APF_INTERACT.interact5.read().bits(),
            6 => self.0.APF_INTERACT.interact6.read().bits(),
            7 => self.0.APF_INTERACT.interact7.read().bits(),
            8 => self.0.APF_INTERACT.interact8.read().bits(),
            9 => self.0.APF_INTERACT.interact9.read().bits(),
            10 => self.0.APF_INTERACT.interact10.read().bits(),
            11 => self.0.APF_INTERACT.interact11.read().bits(),
            12 => self.0.APF_INTERACT.interact12.read().bits(),
            13 => self.0.APF_INTERACT.interact13.read().bits(),
            14 => self.0.APF_INTERACT.interact14.read().bits(),
            15 => self.0.APF_INTERACT.interact15.read().bits(),
            _ => panic!("invalid interact"),
        }
    }
}
