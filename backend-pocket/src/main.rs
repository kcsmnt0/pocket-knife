#![no_std]
#![no_main]

#![feature(vec_into_raw_parts)]

#![allow(unused_imports)]

mod gamepad;
mod pocket;

use gamepad::*;
use pocket::*;

use pocket_knife_frontend::*;

use alloc::vec::Vec;
use alloc::{rc::Rc, vec};
use slint::platform::Key;
use strum::IntoEnumIterator;
use core::{mem::MaybeUninit, time::Duration};
use core::panic::PanicInfo;
use core::slice::from_raw_parts_mut;
use embedded_alloc::Heap;
use enumset::EnumSet;
use litex_pac::constants::{CONFIG_CLOCK_FREQUENCY, VIDEO_FRAMEBUFFER_BASE};
use slint::{platform::{software_renderer::{MinimalSoftwareWindow, Rgb565Pixel, RepaintBufferType}, Platform, WindowAdapter, EventLoopProxy}, VecModel, StandardListViewItem, ModelRc, PlatformError, Window, EventLoopError};

extern crate alloc;

use alloc::boxed::Box;
use litex_openfpga::*;
use litex_pac::Peripherals;
use riscv_rt::entry;

pub const FRAMEBUFFER_ADDRESS: *mut Rgb565Pixel = VIDEO_FRAMEBUFFER_BASE as *mut Rgb565Pixel;

pub const HEAP_SIZE: usize = 32 * 1024 * 1024; // 32MiB
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
    let peripherals = unsafe { Peripherals::steal() };

    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) };

    let mut gamepad = Gamepad::default();

    App::with_new(Pocket(peripherals), move |mut app| {
        loop {
            gamepad.update(&app.backend.borrow().0);
            for button in Button::iter() {
                if gamepad.state(button) == State::Pressed {
                    app.slint_window.dispatch_event(slint::platform::WindowEvent::KeyPressed { text: button.key_text() })
                } else if gamepad.state(button) == State::Released {
                    app.slint_window.dispatch_event(slint::platform::WindowEvent::KeyReleased { text: button.key_text() })
                }
            }

            app.draw();
        }
    })
}
