use crate::serial::{Serial, COM1};
use core::fmt::write;

#[inline]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut serial = Serial(&COM1);
    serial.init();
    write(&mut serial, format_args!("{}\n", info)).expect("serial output");
    loop {}
}
