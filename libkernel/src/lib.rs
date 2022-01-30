#![no_std]
#![no_main]
#![feature(core_panic)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod kernel;
pub mod panic;
mod serial;

mod platform;
#[cfg(test)]
mod test;

pub use kernel::Kernel;
