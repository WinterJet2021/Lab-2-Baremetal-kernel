#![no_main]
#![no_std]

use core::arch::asm;
use core::fmt::Write;
use core::panic::PanicInfo;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use bootloader_api::info::{FrameBuffer, PixelFormat, Optional};

use kernel::serial;
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight, RasterizedChar};


#[cfg(not(test))]
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
    // Log stack pointer (RSP)
    let rsp: u64;
    unsafe { asm!("mov {}, rsp", out(reg) rsp); }
    let _ = writeln!(serial(), "RSP: 0x{:x}", rsp);

    // Check framebuffer presence
    let has_fb = matches!(boot_info.framebuffer, Optional::Some(_));
    let _ = writeln!(serial(), "Boot info received (framebuffer present: {})", has_fb);

    // Extract framebuffer safely
    let fb = match boot_info.framebuffer.as_mut() {
        Some(fb) => fb,
        None => {
            let _ = writeln!(serial(), "No framebuffer provided by bootloader.");
            loop {}
        }
    };

    // Clear background to black
    clear_fb(fb, RGBA(0, 0, 0, 0));

    // ---- Draw “Hello, world!” centered ----
    const TEXT: &str = "Hello, world Tuey!";
    let (tw, th) = text_size(TEXT);
    let info = fb.info();
    let cx = info.width.saturating_sub(tw) / 2;
    let cy = info.height.saturating_sub(th) / 2;

    draw_text(fb, cx, cy, TEXT, RGBA(255, 255, 255, 0)); // white

    let _ = writeln!(serial(), "Drew \"{TEXT}\" at ({cx},{cy}). Entering wait loop...");
    loop {}
}

/* ================= drawing helpers ================= */

#[derive(Clone, Copy)]
struct RGBA(u8, u8, u8, u8);

fn clear_fb(buffer: &mut FrameBuffer, color: RGBA) {
    let info = buffer.info();
    for y in 0..info.height {
        for x in 0..info.width {
            write_pixel(buffer, x, y, color);
        }
    }
}

// Return width/height (in pixels) for a string with our chosen font/spacing
fn text_size(s: &str) -> (usize, usize) {
    let mut w = 0;
    let mut h = 0;
    for ch in s.chars() {
        if let Some(r) = get_raster(ch, FontWeight::Regular, RasterHeight::Size16) {
            w += r.width() + 1; // 1px tracking
            h = h.max(r.height());
        } else {
            // fallback width for missing glyphs
            w += 8;
            h = h.max(16);
        }
    }
    (w.saturating_sub(1), h) // drop last tracking pixel
}

fn draw_text(buffer: &mut FrameBuffer, mut x: usize, y: usize, s: &str, color: RGBA) {
    for ch in s.chars() {
        if let Some(r) = get_raster(ch, FontWeight::Regular, RasterHeight::Size16) {
            draw_raster_char(buffer, x, y, &r, color);
            x += r.width() + 1; // tracking
        } else {
            // simple 6x10 box as fallback glyph
            draw_box(buffer, x, y, 6, 10, color);
            x += 7;
        }
    }
}

fn draw_raster_char(
    buffer: &mut FrameBuffer,
    x0: usize,
    y0: usize,
    ch: &RasterizedChar,
    color: RGBA,
) {
    // Blend the glyph’s intensity onto our solid color (simple threshold)
    for (dy, row) in ch.raster().iter().enumerate() {
        for (dx, &intensity) in row.iter().enumerate() {
            if intensity > 8 { // treat as “on” pixel
                write_pixel(buffer, x0 + dx, y0 + dy, color);
            }
        }
    }
}

fn draw_box(buffer: &mut FrameBuffer, x0: usize, y0: usize, w: usize, h: usize, color: RGBA) {
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            write_pixel(buffer, x, y, color);
        }
    }
}

fn write_pixel(buffer: &mut FrameBuffer, x: usize, y: usize, RGBA(r, g, b, a): RGBA) {
    let info = buffer.info();
    if x >= info.width || y >= info.height {
        return;
    }

    let pixel_offset = y * usize::from(info.stride) + x;
    let bpp = usize::from(info.bytes_per_pixel);
    let byte_offset = pixel_offset * bpp;

    // Arrange channels for the framebuffer’s pixel format
    let mut color = [0u8; 4];
    match info.pixel_format {
        PixelFormat::Rgb => { color[0] = r; color[1] = g; color[2] = b; color[3] = a; }
        PixelFormat::Bgr => { color[0] = b; color[1] = g; color[2] = r; color[3] = a; }
        other => {
            let _ = writeln!(serial(), "Unsupported pixel format: {:?}", other);
            return;
        }
    }

    let buf = buffer.buffer_mut();
    let end = byte_offset + bpp;
    if end <= buf.len() {
        buf[byte_offset..end].copy_from_slice(&color[..bpp]);
    }
}