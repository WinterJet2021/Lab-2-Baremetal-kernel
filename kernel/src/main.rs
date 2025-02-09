#![no_main] // disable all Rust-level entry points
#![no_std] // don't link the Rust standard library

use core::arch::asm;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::fmt::Write;
use core::panic::PanicInfo;
use bootloader_api::info::{FrameBuffer, PixelFormat};
use kernel::serial;
use noto_sans_mono_bitmap::{FontWeight, get_raster, RasterHeight, RasterizedChar};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
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
    let rip = x86_64::registers::read_rip().as_u64();
    writeln!(serial(), "RIP: 0x{:x}", rip).unwrap();
    let rsp: u64;
    unsafe {
        asm!("mov {}, rsp", out(reg) rsp);
    }
    writeln!(serial(), "RSP: 0x{:x}", rsp).unwrap();
    
    writeln!(serial(), "Entered kernel with boot info: {boot_info:?}").unwrap();

    let vga_buffer = boot_info.framebuffer.as_mut().unwrap();
    for x in 0..vga_buffer.info().width {
        for y in 0..vga_buffer.info().height {
            write_pixel(vga_buffer, x, y, RGBA((x % 256) as u8, (y%256) as u8, ((x + y) % 256) as u8, 0));
        }
    }

    let bitmap_char = get_raster('O', FontWeight::Regular, RasterHeight::Size16).unwrap();
    let next = write_rendered_char(vga_buffer, 0, 512, bitmap_char);
    write_rendered_char(vga_buffer, next.0, next.1, get_raster('S', FontWeight::Regular, RasterHeight::Size16).unwrap());

    writeln!(serial(), "Entering kernel wait loop...").unwrap();

    loop {
        // This infinite loop keeps the kernel running
    }
}

struct Cursor(usize, usize);
struct RGBA(u8, u8, u8, u8);

fn write_rendered_char(buffer:&mut FrameBuffer, x_pos:usize, y_pos:usize, rendered_char: RasterizedChar) -> Cursor {
    for (y, row) in rendered_char.raster().iter().enumerate() {
        for (x, byte) in row.iter().enumerate() {
            write_pixel(buffer, x_pos + x, y_pos + y, intensity_to_rgba(*byte));
        }
    }
    Cursor(x_pos + rendered_char.width(), y_pos)
}

fn intensity_to_rgba(intensity: u8) -> RGBA { RGBA(intensity >> 1, intensity, intensity >> 1, 0) }

fn write_pixel(buffer:&mut FrameBuffer, x: usize, y: usize, rgba_color: RGBA) {
    let mut info = buffer.info();
    let pixel_offset = y * usize::from(info.stride) + x;
    let color = match info.pixel_format {
        PixelFormat::Rgb => [rgba_color.0, rgba_color.1, rgba_color.2, rgba_color.3],
        PixelFormat::Bgr => [rgba_color.0, rgba_color.1, rgba_color.2, rgba_color.3],
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