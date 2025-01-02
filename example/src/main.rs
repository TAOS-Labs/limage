#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(gk::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
extern crate gk;

use limine::BaseRevision;
use limine::request::{RequestsEndMarker, RequestsStartMarker};
use core::panic::PanicInfo;


#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".requests_start_marker"]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[link_section = ".requests_end_marker"]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();


#[no_mangle]
unsafe extern "C" fn _start() -> ! {  
    assert!(BASE_REVISION.is_supported());
    gk::init();

    #[cfg(test)]
    {
        serial_info_ln!("!!! RUNNING BINARY TESTS !!!");
        test_main();
    }

    // individual device output
    fb0_info_ln!("hello framebuffer");
    serial_info_ln!("hello serial");

    // combined device output
    info_ln!("information for all devices");
    debug_ln!("debug for all devices");
    warn_ln!("warning for all devices");
    danger_ln!("danger for all devices");

    gk::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn rust_panic(info: &PanicInfo) -> ! {
    serial_danger_ln!("{}", info);
    gk::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn rust_panic(info: &PanicInfo) -> ! {
    serial_danger_ln!("{}", info);
    gk::test_panic_handler(info);
}

#[test_case]
fn trivial_main_assertion() {
    assert_eq!(1, 1);
}