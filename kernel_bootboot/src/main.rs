#![no_std]
#![no_main]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    paraos_libkernel::panic::panic(info);
}

#[repr(C)]
struct Bootboot {
    magic: [u8; 4],
    size: u32,
    protocol: u8,
    framebuffer_type: u8,
    numcores: u16,
    bspid: u16,
    timezone: i16,
    datetime: [u8; 8usize],
    initrd_ptr: u64,
    initrd_size: u64,
    fb_ptr: *mut u8,
    fb_size: u32,
    fb_width: u32,
    fb_height: u32,
    fb_scanline: u32,
}

extern "C" {
    static BOOTBOOT: Bootboot;
}

use spin::{Barrier, Once};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    static START: Once<Barrier> = Once::new();
    paraos_libkernel::Kernel::new(unsafe { BOOTBOOT.bspid as u32 }, &START).run();
    loop {}
}
