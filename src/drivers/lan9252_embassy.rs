use crate::bindings::esc_cfg;
use crate::bindings::ESCREG_ALEVENT;
use crate::soes;

use crate::drivers::lan9252_cst::*;
use crate::esc_driver::EscDriver;

use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::{Config, Mode, Phase, Polarity, Spi};
use embassy_stm32::time::Hertz;
use embassy_time::{Duration, Timer};

use defmt::*;

pub struct Lan9252Blocking<'d> {
    spi: Spi<'d, Blocking>,
    cs: Output<'d>,
}

impl<'d> Lan9252Blocking<'d> {
    pub fn new(spi: Spi<'d, Blocking>, cs: Output<'d>) -> Self {
        Self { spi, cs }
    }

    /*
    pub fn init_global(driver: Self) {
        static mut DRIVER_INSTANCE: Option<Lan9252Blocking> = None;

        unsafe {
            DRIVER_INSTANCE = Some(driver);
            let drv = DRIVER_INSTANCE.as_mut().unwrap();

            drv.init();
            crate::drivers::esc_c::set_driver(drv); // Register for C

            //drv
        }
    }
     */

    fn write_32(&mut self, address: u16, val: u32) {
        let data = [
            ESC_CMD_SERIAL_WRITE,
            ((address >> 8) & 0xFF) as u8,
            (address & 0xFF) as u8,
            (val & 0xFF) as u8,
            ((val >> 8) & 0xFF) as u8,
            ((val >> 16) & 0xFF) as u8,
            ((val >> 24) & 0xFF) as u8,
        ];

        self.cs.set_low();
        self.spi.blocking_write(&data).unwrap();
        self.cs.set_high();
    }

    fn read_32(&mut self, address: u16) -> u32 {
        let tx = [
            ESC_CMD_FAST_READ,
            ((address >> 8) & 0xFF) as u8,
            (address & 0xFF) as u8,
            ESC_CMD_FAST_READ_DUMMY,
        ];
        let mut rx = [0u8; 4];

        self.cs.set_low();
        self.spi.blocking_write(&tx).unwrap();
        self.spi.blocking_transfer(&mut rx, &[0xFF; 4]).unwrap();
        self.cs.set_high();

        ((rx[3] as u32) << 24) | ((rx[2] as u32) << 16) | ((rx[1] as u32) << 8) | (rx[0] as u32)
    }

    /// CSR Read helper (like `ESC_read_csr` in C)
    fn read_csr(&mut self, address: u16, buf: &mut [u8]) {
        let len = buf.len() as u32;
        //defmt::info!("read_csr: addr=0x{:04X}, len={}", address, len);

        // Issue CSR read command
        //let mut value = (ESC_CSR_CMD_READ as u32) | ((len & 0x3) << 16) | (address as u32);
        let mut value = ESC_CSR_CMD_READ as u32 | ESC_CSR_CMD_SIZE(len) | (address as u32);

        //defmt::info!("Writing ESC_CSR_CMD_REG with value 0x{:08X}", value);
        self.write_32(ESC_CSR_CMD_REG, value);

        // Wait until not busy
        loop {
            value = self.read_32(ESC_CSR_CMD_REG);
            //defmt::info!("Polling ESC_CSR_CMD_REG: 0x{:08X}", value);
            if value & ESC_CSR_CMD_BUSY == 0 {
                break;
            }
        }

        // Read data
        value = self.read_32(ESC_CSR_DATA_REG);
        //defmt::info!("Read ESC_CSR_DATA_REG: 0x{:08X}", value);

        // Copy into buffer
        let bytes = value.to_le_bytes();
        buf.copy_from_slice(&bytes[..buf.len()]);
        //defmt::info!("Buffer after read: {:?}", buf);
    }

    /// CSR Write helper (like `ESC_write_csr` in C)
    fn write_csr(&mut self, address: u16, buf: &[u8]) {
        let len = buf.len() as u32;

        // Pack buf into u32 (little endian)
        let mut value: u32 = 0;
        for (i, b) in buf.iter().enumerate() {
            value |= (*b as u32) << (8 * i);
        }

        // Write data
        self.write_32(ESC_CSR_DATA_REG, value);

        // Issue CSR write command
        value = (ESC_CSR_CMD_WRITE as u32) | ESC_CSR_CMD_SIZE(buf.len() as u32) | (address as u32);
        self.write_32(ESC_CSR_CMD_REG, value);

        // Wait until not busy
        loop {
            value = self.read_32(ESC_CSR_CMD_REG);
            if value & ESC_CSR_CMD_BUSY == 0 {
                break;
            }
        }
    }

    fn update_AlEvent(&mut self) {
        unsafe {
            // Read ALEVENT register into a temporary buffer
            let mut buf = [0u8; 2]; // u16 is 2 bytes
            self.read_csr(ESCREG_ALEVENT as u16, &mut buf);

            // Convert the buffer into u16 and handle endianness
            soes::ESCvar.ALevent = u16::from_le_bytes(buf); // assuming little-endian (like etohs) --> Need proper implementation of endianess
        }
    }

    /// ESC write process data RAM function
    fn write_pram(&mut self, mut address: u16, buf: &[u8]) {
        let mut len = buf.len() as u16;
        let mut byte_offset = 0usize;
        let mut fifo_cnt: u8;
        let mut first_byte_position: u8;
        let mut temp_len: u8;
        let mut data: [u8; 3] = [0; 3];

        // Abort any ongoing PRAM write
        self.write_32(ESC_PRAM_WR_CMD_REG, ESC_PRAM_CMD_ABORT);

        // Wait until not busy
        loop {
            let value = self.read_32(ESC_PRAM_WR_CMD_REG);
            if value & ESC_PRAM_CMD_BUSY == 0 {
                break;
            }
        }

        // Set address and length
        let value = ESC_PRAM_SIZE(len as u32) | ESC_PRAM_ADDR(address as u32);
        self.write_32(ESC_PRAM_WR_ADDR_LEN_REG, value);

        // Start PRAM write
        self.write_32(ESC_PRAM_WR_CMD_REG, ESC_PRAM_CMD_BUSY);

        // Wait for FIFO to be ready
        loop {
            let value = self.read_32(ESC_PRAM_WR_CMD_REG);
            if value & ESC_PRAM_CMD_AVAIL != 0 {
                fifo_cnt = ESC_PRAM_CMD_CNT(value) as u8;
                break;
            }
        }

        // Prepare first 32-bit value
        let mut value: u32 = 0;
        first_byte_position = (address & 0x03) as u8;
        temp_len = ((4 - first_byte_position) as u16).min(len) as u8;

        // Copy initial bytes into `value`
        for i in 0..temp_len as usize {
            value |= (buf[i] as u32) << (8 * (first_byte_position as usize + i));
        }

        // Write first value to FIFO
        self.write_32(ESC_PRAM_WR_FIFO_REG, value);

        len -= temp_len as u16;
        byte_offset += temp_len as usize;
        fifo_cnt = fifo_cnt.saturating_sub(1);

        // Select device (CS low)
        self.cs.set_low();

        // Send incrementing write command
        data[0] = ESC_CMD_SERIAL_WRITE;
        data[1] = ((ESC_PRAM_WR_FIFO_REG >> 8) & 0xFF) as u8;
        data[2] = (ESC_PRAM_WR_FIFO_REG & 0xFF) as u8;
        self.spi.blocking_write(&data).unwrap();

        // Continue writing remaining bytes
        while len > 0 {
            temp_len = len.min(4) as u8;
            let mut value: u32 = 0;

            for i in 0..temp_len as usize {
                value |= (buf[byte_offset + i] as u32) << (8 * i);
            }

            let data_from_value = value.to_le_bytes();
            self.spi.blocking_write(&data_from_value).unwrap();

            fifo_cnt = fifo_cnt.saturating_sub(1);
            len -= temp_len as u16;
            byte_offset += temp_len as usize;
        }

        // Unselect device (CS high)
        self.cs.set_high();
    }

    /// ESC read process data RAM function
    fn read_pram(&mut self, mut address: u16, buf: &mut [u8]) {
        let mut len = buf.len() as u16;
        let mut byte_offset = 0usize;
        let mut fifo_cnt: u8;
        let mut first_byte_position: u8;
        let mut temp_len: u8;
        let mut data: [u8; 4] = [0; 4];

        // Abort any ongoing PRAM read
        self.write_32(ESC_PRAM_RD_CMD_REG, ESC_PRAM_CMD_ABORT);

        // Wait until not busy
        loop {
            let value = self.read_32(ESC_PRAM_RD_CMD_REG);
            if value & ESC_PRAM_CMD_BUSY == 0 {
                break;
            }
        }

        // Set address and length
        let value = ESC_PRAM_SIZE(len as u32) | ESC_PRAM_ADDR(address as u32);
        self.write_32(ESC_PRAM_RD_ADDR_LEN_REG, value);

        // Start PRAM read
        self.write_32(ESC_PRAM_RD_CMD_REG, ESC_PRAM_CMD_BUSY);

        // Wait for FIFO to be ready
        loop {
            let value = self.read_32(ESC_PRAM_RD_CMD_REG);
            if value & ESC_PRAM_CMD_AVAIL != 0 {
                fifo_cnt = ESC_PRAM_CMD_CNT(value) as u8;
                break;
            }
        }

        // Read first 32-bit value from FIFO
        let mut value = self.read_32(ESC_PRAM_RD_FIFO_REG);
        fifo_cnt -= 1;

        first_byte_position = (address & 0x03) as u8;
        temp_len = ((4 - first_byte_position) as u16).min(len) as u8;

        buf[byte_offset..(byte_offset + temp_len as usize)].copy_from_slice(
            &value.to_le_bytes()
                [first_byte_position as usize..(first_byte_position + temp_len) as usize],
        );

        len -= temp_len as u16;
        byte_offset += temp_len as usize;

        // Select device (CS low)
        self.cs.set_low();

        // Send command for FIFO read
        data[0] = ESC_CMD_FAST_READ;
        data[1] = ((ESC_PRAM_RD_FIFO_REG >> 8) & 0xFF) as u8;
        data[2] = (ESC_PRAM_RD_FIFO_REG & 0xFF) as u8;
        data[3] = ESC_CMD_FAST_READ_DUMMY;
        self.spi.blocking_write(&data).unwrap();

        // Continue reading remaining bytes
        while len > 0 {
            temp_len = len.min(4) as u8;

            let mut temp_buf = [0u8; 4];
            self.spi
                .blocking_transfer(&mut temp_buf, &[0xFF; 4])
                .unwrap();

            buf[byte_offset..(byte_offset + temp_len as usize)]
                .copy_from_slice(&temp_buf[..temp_len as usize]);

            fifo_cnt = fifo_cnt.saturating_sub(1);
            len -= temp_len as u16;
            byte_offset += temp_len as usize;
        }

        // Unselect device (CS high)
        self.cs.set_high();
    }
}

impl<'d> EscDriver for Lan9252Blocking<'d> {
    fn init(&mut self) {
        self.write_32(ESC_RESET_CTRL_REG, ESC_RESET_CTRL_RST);

        loop {
            let value = self.read_32(ESC_CSR_CMD_REG);
            if (value & ESC_RESET_CTRL_RST) == 0 {
                break;
            }
        }

        let value = self.read_32(LAN9252_BYTE_TEST);
        //defmt::info!("Test Byte: {:#010x}", value);
    }

    fn reset(&mut self) {
        // TODO
    }

    /// Write to ESC memory (CSR or PRAM depending on address)
    fn write(&mut self, mut address: u16, mut buf: &[u8]) {
        // If address is >= 0x1000 → PRAM write (not implemented yet)
        if address >= 0x1000 {
            //defmtdefmt::warn!("PRAM write not implemented yet");
            self.write_pram(address, buf);
            return;
        }

        while !buf.is_empty() {
            // Max 4 bytes at a time
            let mut size = buf.len().min(4);

            // Align size to LAN9252 rules
            if address & 0x01 != 0 {
                size = 1;
            } else if address & 0x02 != 0 {
                size = if size & 0x01 != 0 { 1 } else { 2 };
            } else if size == 3 {
                size = 1;
            }

            self.write_csr(address, &buf[..size]);

            // Advance
            buf = &buf[size..];
            address += size as u16;
        }
        /* To mimic the ET1100 always providing AlEvent on every read or write */
        self.update_AlEvent();
    }

    /// Read from ESC memory (CSR or PRAM depending on address)
    fn read(&mut self, mut address: u16, mut buf: &mut [u8]) {
        //defmtdefmt::info!("read: addr=0x{:04X}, total_len={}", address, buf.len());

        if address >= 0x1000 {
            self.read_pram(address, buf);
            return;
        }

        while !buf.is_empty() {
            let mut size = buf.len().min(4);

            // Align to LAN9252 CSR rules
            if address & (1 << 0) != 0 {
                size = 1;
            } else if address & (1 << 1) != 0 {
                size = if size & 1 != 0 { 1 } else { 2 };
            } else if size == 3 {
                size = 1;
            }

            //defmt::info!("Reading chunk: addr=0x{:04X}, size={}", address, size);

            self.read_csr(address, &mut buf[..size]);

            buf = &mut buf[size..];
            address += size as u16;
        }

        //defmtdefmt::info!("Finished read at addr=0x{:04X}", address);
        /* To mimic the ET1100 always providing AlEvent on every read or write */
        self.update_AlEvent();
    }
}
