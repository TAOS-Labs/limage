#![no_std]
#![cfg_attr(test, no_main)]

#![feature(abi_x86_interrupt)]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use core::panic::PanicInfo;

pub mod allocator;
pub mod dat;
pub mod dev;


/// Initialize the kernel.
pub fn init() {
  dev::framebuffer::fb0::init();
}

/// Halt the CPU.
pub fn hlt_loop() -> ! {
  loop {
    x86_64::instructions::hlt();
  }
}

/// Prints INFO to serial and framebuffer terminals.
#[macro_export]
macro_rules! info {
  ($($arg:tt)*) => {
    serial_info!($($arg)*);
    fb0_info!($($arg)*);
  };
}

/// Prints INFO to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! info_ln {
  ($($arg:tt)*) => {
    serial_info_ln!($($arg)*);
    fb0_info_ln!($($arg)*);
  };
}

/// Prints DEBUG to serial and framebuffer terminals.
#[macro_export]
macro_rules! debug {
  ($($arg:tt)*) => {
    serial_debug!($($arg)*);
    fb0_debug!($($arg)*);
  };
}

/// Prints DEBUG to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! debug_ln {
  ($($arg:tt)*) => {
    serial_debug_ln!($($arg)*);
    fb0_debug_ln!($($arg)*);
  };
}

/// Prints WARN to serial and framebuffer terminals.
#[macro_export]
macro_rules! warn {
  ($($arg:tt)*) => {
    serial_warn!($($arg)*);
    fb0_warn!($($arg)*);
  };
}

/// Prints WARN to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! warn_ln {
  ($($arg:tt)*) => {
    serial_warn_ln!($($arg)*);
    fb0_warn_ln!($($arg)*);
  };
}

/// Prints DANGER to serial and framebuffer terminals.
#[macro_export]
macro_rules! danger {
  ($($arg:tt)*) => {
    serial_danger!($($arg)*);
    fb0_danger!($($arg)*);
  };
}

/// Prints DANGER to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! danger_ln {
  ($($arg:tt)*) => {
    serial_danger_ln!($($arg)*);
    fb0_danger_ln!($($arg)*);
  };
}

pub trait Testable {
  fn run(&self) -> ();
}

impl<T> Testable for T
where
  T: Fn(),
{
  fn run(&self) {
    serial_print!("TEST: {}...\t", core::any::type_name::<T>());
    self();
    serial_print!("[ok]\n");
  }
}

#[no_mangle]
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_print!("INFO: Running {} tests...\n", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_danger_ln!("[failed]\n");
    serial_danger_ln!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_info_ln!("!!! RUNNING LIBRARY TESTS !!!");
    
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[test_case]
fn trivial_lib_assertion() {
    assert_eq!(1, 1);
}