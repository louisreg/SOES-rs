use crate::bindings::esc_cfg;
use embedded_hal::blocking::spi::{Transfer, Write};

pub trait EscDriver<SPI>
where
    SPI: Transfer<u8> + Write<u8>,
{
    fn init(&mut self, cfg: &esc_cfg, spi: &mut SPI);

    fn read(&self, spi: &mut SPI, address: u16, buf: &mut [u8]);

    fn write(&self, spi: &mut SPI, address: u16, buf: &[u8]);

    fn reset(&self, spi: &mut SPI) {}
}
