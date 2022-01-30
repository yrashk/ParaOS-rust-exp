#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

mod bootboot;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    paraos_libkernel::panic::panic(info);
}

extern "C" {
    static BOOTBOOT: bootboot::Bootboot;
}

use spin::{Barrier, Once};

use buddy_system_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::empty();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    static START: Once<Barrier> = Once::new();
    paraos_libkernel::Kernel::new(unsafe { BOOTBOOT.bspid as u32 }, &START).run(|| unsafe {
        for entry in &BOOTBOOT.memory_mappings()[1..] {
            if entry.is_free() && entry.addr() != 0x0 {
                HEAP_ALLOCATOR
                    .lock()
                    .add_to_heap(entry.addr(), entry.addr() + entry.size());
            }
        }
        let mut serial = paraos_libkernel::serial::Serial::new(&paraos_libkernel::serial::COM1);
        serial.init();
        core::fmt::write(
            &mut serial,
            format_args!(
                "Free memory: {}MB\n",
                HEAP_ALLOCATOR.lock().stats_total_bytes() / (1024 * 1024)
            ),
        )
        .expect("serial output");
    });
    loop {}
}
