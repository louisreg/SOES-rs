use crate::esc_driver::EscDriver;

// static driver accessible aux bindings C
static mut DRIVER: Option<&'static mut dyn EscDriver> = None;

/// Set the global ESC driver
pub fn set_driver(driver: &'static mut dyn EscDriver) {
    unsafe {
        DRIVER = Some(driver);
    }
}

/// ESC C bindings will call this
#[no_mangle]
pub extern "C" fn ESC_write(address: u16, buf: *const u8, len: usize) {
    unsafe {
        if let Some(driver) = DRIVER.as_mut() {
            let slice = core::slice::from_raw_parts(buf, len);
            driver.write(address, slice);
        }
    }
}

#[no_mangle]
pub extern "C" fn ESC_read(address: u16, buf: *mut u8, len: usize) {
    unsafe {
        if let Some(driver) = DRIVER.as_mut() {
            let slice = core::slice::from_raw_parts_mut(buf, len);
            driver.read(address, slice);
        }
    }
}
