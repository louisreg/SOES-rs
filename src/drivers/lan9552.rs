use crate::bindings::esc_cfg;
use crate::esc_driver::EscDriver;
use embedded_hal::blocking::spi::{Transfer, Write};

pub struct Lan9252;

impl<SPI> EscDriver<SPI> for Lan9252
where
    SPI: Transfer<u8> + Write<u8>,
{
    fn init(&mut self, cfg: &esc_cfg, spi: &mut SPI) {
        unsafe { crate::bindings::ESC_init(cfg as *const _) }
    }

    fn read(&self, spi: &mut SPI, address: u16, buf: &mut [u8]) {
        unsafe { crate::bindings::ESC_read(address, buf.as_mut_ptr() as *mut _, buf.len() as u16) }
    }

    fn write(&self, spi: &mut SPI, address: u16, buf: &[u8]) {
        unsafe { crate::bindings::ESC_write(address, buf.as_ptr() as *const _, buf.len() as u16) }
    }
}
