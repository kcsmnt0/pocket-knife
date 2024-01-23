#![no_std]
#![no_main]

#![feature(slice_as_chunks)]

#![allow(unused_imports)]

mod error;
mod gamepad;
mod pocket;

use error::*;
use gamepad::*;
use pocket::*;

use pocket_knife_frontend::*;

extern crate alloc;

use alloc::boxed::Box;
use alloc::{rc::Rc, vec};
use core::panic::PanicInfo;
use core::slice::from_raw_parts_mut;
use core::{mem::MaybeUninit, time::Duration};
use embedded_alloc::Heap;
use enumset::EnumSet;
use litex_openfpga::*;
use litex_pac::Peripherals;
use litex_pac::constants::CONFIG_CLOCK_FREQUENCY;
use riscv_rt::entry;
use slint::platform::Key;
use slint::{platform::{software_renderer::{MinimalSoftwareWindow, Rgb565Pixel, RepaintBufferType}, Platform, WindowAdapter, EventLoopProxy}, VecModel, StandardListViewItem, ModelRc, PlatformError, Window, EventLoopError};
use strum::IntoEnumIterator;

pub const HEAP_SIZE: usize = 32 * 1024 * 1024;
pub static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

#[global_allocator]
pub static HEAP: Heap = Heap::empty();

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic:");
    println!("{info}");
    loop {}
}

pub const IMAGE_ADDRESS: *const u8 = 0x42000000 as *const u8;

#[entry]
fn main() -> ! {
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) };

    let mut app = App::new(Pocket::default());
    let mut gamepad = Gamepad::default();

    loop {
        gamepad.update();
        for button in Button::iter() {
            if gamepad.state(button) == State::Pressed {
                app.slint_window.dispatch_event(slint::platform::WindowEvent::KeyPressed { text: button.key_text() })
            } else if gamepad.state(button) == State::Released {
                app.slint_window.dispatch_event(slint::platform::WindowEvent::KeyReleased { text: button.key_text() })
            }
        }

        app.draw();
    }
}
