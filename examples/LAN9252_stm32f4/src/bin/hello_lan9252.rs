#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::{Config, Mode, Phase, Polarity, Spi};
use embassy_stm32::time::Hertz;
use embassy_time::{Duration, Timer};

pub struct Lan9252<'d> {
    spi: Spi<'d, Blocking>,
    cs: Output<'d>,
}

impl<'d> Lan9252<'d> {
    pub fn new(spi: Spi<'d, Blocking>, cs: Output<'d>) -> Self {
        Self { spi, cs }
    }

    pub fn write_32(&mut self, address: u16, val: u32) {
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

    pub fn read_32(&mut self, address: u16) -> u32 {
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
    pub fn esc_init(&mut self) {
        // Reset
        self.write_32(ESC_RESET_CTRL_REG, ESC_RESET_CTRL_RST);

        // wait until ready
        loop {
            let value = self.read_32(ESC_CSR_CMD_REG);
            if (value & ESC_RESET_CTRL_RST) == 0 {
                break;
            }
        }

        // Config IRQ
        // self.write_32(LAN9252_IRQ_CFG, 0x111);

        // Enable IRQ
        // self.write_32(LAN9252_INT_EN, 1);

        // Enable ESC interrupts
        // ESC_ALeventmaskwrite(ALEVENT_MASK);

        // Test byte
        let value = self.read_32(LAN9252_BYTE_TEST);
        defmt::info!("Test Byte: {:#010x}", value);
    }
}

// commandes LAN9252
// Helper: BIT macro
#[inline(always)]
const fn BIT(n: u32) -> u32 {
    1u32 << n
}

// ESC SPI Commands
pub const ESC_CMD_SERIAL_WRITE: u8 = 0x02;
pub const ESC_CMD_SERIAL_READ: u8 = 0x03;
pub const ESC_CMD_FAST_READ: u8 = 0x0B;
pub const ESC_CMD_RESET_SQI: u8 = 0xFF;

pub const ESC_CMD_FAST_READ_DUMMY: u8 = 1;
pub const ESC_CMD_ADDR_INC: u8 = (BIT(6) as u8);

// PRAM registers
pub const ESC_PRAM_RD_FIFO_REG: u16 = 0x000;
pub const ESC_PRAM_WR_FIFO_REG: u16 = 0x020;
pub const ESC_PRAM_RD_ADDR_LEN_REG: u16 = 0x308;
pub const ESC_PRAM_RD_CMD_REG: u16 = 0x30C;
pub const ESC_PRAM_WR_ADDR_LEN_REG: u16 = 0x310;
pub const ESC_PRAM_WR_CMD_REG: u16 = 0x314;

// PRAM command flags
pub const ESC_PRAM_CMD_BUSY: u32 = BIT(31);
pub const ESC_PRAM_CMD_ABORT: u32 = BIT(30);

#[inline(always)]
pub const fn ESC_PRAM_CMD_CNT(x: u32) -> u32 {
    (x >> 8) & 0x1F
}

pub const ESC_PRAM_CMD_AVAIL: u32 = BIT(0);

#[inline(always)]
pub const fn ESC_PRAM_SIZE(x: u32) -> u32 {
    x << 16
}

#[inline(always)]
pub const fn ESC_PRAM_ADDR(x: u32) -> u32 {
    x << 0
}

// CSR registers
pub const ESC_CSR_DATA_REG: u16 = 0x300;
pub const ESC_CSR_CMD_REG: u16 = 0x304;

pub const ESC_CSR_CMD_BUSY: u32 = BIT(31);
pub const ESC_CSR_CMD_READ: u32 = BIT(31) | BIT(30);
pub const ESC_CSR_CMD_WRITE: u32 = BIT(31);

#[inline(always)]
pub const fn esc_csr_cmd_size(len: u32) -> u32 {
    match len {
        1 => 0 << 16,
        2 => 1 << 16,
        4 => 2 << 16,
        _ => 0 << 16, // fallback, never expect this
    }
}

// Reset control
pub const ESC_RESET_CTRL_REG: u16 = 0x1F8;
pub const ESC_RESET_CTRL_RST: u32 = BIT(6);

// Sync status
pub const ESCREG_SYNC0_STATUS: u16 = 0x098E;

// LAN9252 direct registers
pub const LAN9252_IRQ_CFG: u16 = 0x54;
pub const LAN9252_INT_EN: u16 = 0x5C;
pub const LAN9252_INT_STS: u16 = 0x58;
pub const LAN9252_BYTE_TEST: u16 = 0x64;

// ALEVENT_MASK
pub const ALEVENT_MASK: u32 =
    ESCREG_ALEVENT_CONTROL | ESCREG_ALEVENT_SMCHANGE | ESCREG_ALEVENT_SM0 | ESCREG_ALEVENT_SM1;

// ⚠️
pub const ESCREG_ALEVENT_CONTROL: u32 = BIT(0);
pub const ESCREG_ALEVENT_SMCHANGE: u32 = BIT(1);
pub const ESCREG_ALEVENT_SM0: u32 = BIT(2);
pub const ESCREG_ALEVENT_SM1: u32 = BIT(3);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default()); // ⚠️ didn't check if clock config is correct

    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);
    spi_config.mode = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };

    let mut spi = Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, spi_config);

    let mut cs = Output::new(p.PB6, Level::High, Speed::VeryHigh);
    let mut lan = Lan9252::new(spi, cs);

    defmt::info!("Hello, World!");
    lan.esc_init();

    loop {}
}
