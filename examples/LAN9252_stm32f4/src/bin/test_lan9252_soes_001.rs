#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::{Config, Mode, Phase, Polarity, Spi};
use embassy_stm32::time::Hertz;
use embassy_time::{Duration, Timer};

use core::ptr;
use SOES_rs::bindings::esc_cfg;
use SOES_rs::drivers::set_driver;
use SOES_rs::drivers::Lan9252Blocking;
use SOES_rs::esc_driver::EscDriver;
use SOES_rs::soes;

use core::cell::RefCell;
//static DRIVER: RefCell<Option<Lan9252Blocking<'static>>> = RefCell::new(None);

#[repr(C)]
pub struct _Objects {
    pub serial: u32,
    pub Key1: u8,
    pub Key2: u8,
    pub Counter: u32,
    pub LedIn: u8,
}

// global variable expected by soes-c
#[no_mangle]
pub static mut Obj: _Objects = _Objects {
    serial: 0,
    Key1: 0,
    Key2: 0,
    Counter: 0,
    LedIn: 0,
};

// Dummy esc_cfg for test
fn dummy_esc_cfg() -> esc_cfg {
    esc_cfg {
        user_arg: ptr::null_mut(),
        use_interrupt: 0,
        watchdog_cnt: 2000,
        skip_default_initialization: false,
        set_defaults_hook: None,
        pre_state_change_hook: None,
        post_state_change_hook: None,
        application_hook: None,
        safeoutput_override: None,
        pre_object_download_hook: None,
        post_object_download_hook: None,
        pre_object_upload_hook: None,
        post_object_upload_hook: None,
        rxpdo_override: None,
        txpdo_override: None,
        esc_hw_interrupt_enable: None,
        esc_hw_interrupt_disable: None,
        esc_hw_eep_handler: None,
        esc_check_dc_handler: None,
    }
}

fn my_outputs() {
    unsafe {
        if (Obj.LedIn != 0) {
            defmt::info!("LED ON \r\n");
        } else {
            defmt::info!("LED OFF \r\n");
        }
    }
}

fn my_inputs() {
    unsafe {
        Obj.Counter += 1;
    }
}

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

    defmt::info!("Hello, World!");

    let mut driver = Lan9252Blocking::new(spi, cs);
    let drv_ptr: *mut dyn EscDriver = &mut driver;

    unsafe {
        let drv_ref: &mut dyn EscDriver = &mut *drv_ptr;
        set_driver(drv_ref);
    }
    let mut ecat_slv = soes::EcatSlave::new(dummy_esc_cfg());

    ecat_slv.set_output_cb(my_outputs);
    ecat_slv.set_input_cb(my_inputs);
    ecat_slv.init();
    ecat_slv.pdi_debug();

    loop {
        ecat_slv.run();
    }
}
