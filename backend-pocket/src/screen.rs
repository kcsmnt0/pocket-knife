use pocket_knife_frontend::*;

use alloc::rc::Rc;
use core::slice::from_raw_parts_mut;
use litex_pac::{APF_VIDEO, Peripherals, constants::VIDEO_FRAMEBUFFER_BASE};
use slint::{platform::{software_renderer::{Rgb565Pixel, MinimalSoftwareWindow, RepaintBufferType}, WindowAdapter, Renderer}, Window, PhysicalSize};

pub const FRAMEBUFFER_ADDRESS: *mut Rgb565Pixel = VIDEO_FRAMEBUFFER_BASE as *mut Rgb565Pixel;

pub(crate) struct Screen(Rc<MinimalSoftwareWindow>);

impl Screen {
    pub(crate) fn new() -> Self {
        // todo: more clever buffering
        Screen(MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer))
    }
}

impl WindowAdapter for Screen {
    fn window(&self) -> &Window {
        self.0.window()
    }

    fn size(&self) -> PhysicalSize {
        return PhysicalSize {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
        }
    }

    fn renderer(&self) -> &dyn Renderer {
        self.0.renderer()
    }

    fn request_redraw(&self) {
        self.0.draw_if_needed(|renderer| {
            let mut buffer = [Rgb565Pixel(0); SCREEN_PIXELS];
            renderer.render(&mut buffer, SCREEN_WIDTH as usize);

            let video = unsafe { Peripherals::steal().APF_VIDEO };
            while video.video.read().vblank_triggered().bit() {
                // wait for vblank interval
            }

            let vram: &mut [Rgb565Pixel; SCREEN_PIXELS] = unsafe {
                from_raw_parts_mut(FRAMEBUFFER_ADDRESS, SCREEN_PIXELS)
            }.try_into().unwrap();

            // todo: use render return value to only update changed region
            vram.clone_from(&buffer);
        });
    }
}
