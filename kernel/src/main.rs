#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::fmt::Write;
use bootloader_api::info::{FrameBuffer, PixelFormat};
use kernel::serial;
use noto_sans_mono_bitmap::{FontWeight, get_raster, RasterHeight, RasterizedChar};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = writeln!(serial(), "PANIC: {info}");
    loop {}
}
const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB kernel stack size
    config
};
entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    writeln!(serial(), "Entered kernel with boot info: {boot_info:?}").unwrap();

    let vga_buffer = boot_info.framebuffer.as_mut().unwrap();
    // vga_buffer.buffer_mut().fill(0);

    let bitmap_char = get_raster('O', FontWeight::Regular, RasterHeight::Size16).unwrap();
    let next = write_rendered_char(vga_buffer, 0, 512, bitmap_char);
    write_rendered_char(vga_buffer, next.0, next.1, get_raster('S', FontWeight::Regular, RasterHeight::Size16).unwrap());

    writeln!(serial(), "Entering kernel wait loop...").unwrap();

    loop {}
}

fn write_rendered_char(buffer:&mut FrameBuffer, x_pos:usize, y_pos:usize, rendered_char: RasterizedChar) -> (usize, usize) {
    for (y, row) in rendered_char.raster().iter().enumerate() {
        for (x, byte) in row.iter().enumerate() {
            write_pixel(buffer, x_pos + x, y_pos + y, *byte);
        }
    }
    (x_pos + rendered_char.width(), y_pos)
}

fn write_pixel(buffer:&mut FrameBuffer, x: usize, y: usize, intensity: u8) {
    let mut info = buffer.info();
    let pixel_offset = y * usize::from(info.stride) + x;
    let color = match info.pixel_format {
        PixelFormat::Rgb => [intensity, intensity, intensity / 2, 0],
        PixelFormat::Bgr => [intensity / 2, intensity, intensity, 0],
        other => {
            info.pixel_format = PixelFormat::Rgb;
            panic!("pixel format {:?} not supported in logger", other)
        }
    };
    let bytes_per_pixel = info.bytes_per_pixel;
    let byte_offset = pixel_offset * usize::from(bytes_per_pixel);

    buffer.buffer_mut()[byte_offset..(byte_offset + usize::from(bytes_per_pixel))]
        .copy_from_slice(&color[..usize::from(bytes_per_pixel)]);
}