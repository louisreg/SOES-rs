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
pub const fn ESC_CSR_CMD_SIZE(x: u32) -> u32 {
    x << 16
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
