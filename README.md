# SOES-rs

**SOES-rs** is a Rust implementation (or rather, a Rust wrapper) of **SOES** (Simple Open EtherCAT Slave). The project aims to gradually “oxidize” the original C code into a modern, object-oriented, and idiomatic Rust version.

---

## ⚠️ Warning

This project is **highly experimental**.  

- I am **not** an expert in Rust or C, and my EtherCAT knowledge is limited.  
- This project is primarily for **educational purposes**, to learn embedded Rust and better understand EtherCAT.  
- **Use at your own risk.** The code is not production-ready or fully tested.  
- I recommend keeping a working SOES C project to **verify results** obtained with SOES-rs.  

---

## Project Goals

The main goal is to create a **100% Rust version of SOES** while preserving as much functionality as possible.  

Current focus:  
1. **ESC and ESC-COE** (`esc.c` and `esc_coe.c`).  
2. Future work: **ESC-FOE**, **ESC-EEPROM** (`esc_foe.c` and `esc_eep.c`).  
3. **ESC EoE (Ethernet over EtherCAT)** is **not planned** at this stage.  

Additional goals:  
- Introduce a **driver abstraction layer** (e.g., for LAN9252) to enable **mocking for local tests**.  
- Gradually implement **unit and functional tests**.  
- Eventually support **async Rust**, compatible with frameworks like **Embassy**.  

---

## Key Features

- Rust-native wrapper over SOES with an **object-oriented design**.  
- Focus on safety where possible, though **`unsafe` code is currently necessary** for bindings and low-level access.  
- Low-level driver abstraction to separate hardware access from the EtherCAT protocol logic.  
- Modular structure to gradually replace C code with idiomatic Rust.  

---

## Current Status

- Initial wrapper over `esc.c` and `esc_coe.c`.  
- Unsafe code heavily used for direct memory access and C bindings.  
- Read/write of **process data via LAN9252 SPI** implemented.  
- Partial support for **SDO handling**.  
- Logging through `defmt`.  
- Tests and async support are **planned** but not implemented yet.  

---

## Dependencies

- `embassy-stm32` for hardware abstraction (optional, for async support later).  
- `defmt` for logging/debugging.  
- `cty` for C type definitions in bindings.  

---

## Usage

## Mini Usage Example

This example demonstrates a basic setup of the `SOES-rs` Rust EtherCAT slave stack with a blocking SPI driver for the LAN9252, including simple input/output callbacks.

```rust
#![no_std]
#![no_main]

use SOES_rs::bindings::esc_cfg;
use SOES_rs::drivers::{set_driver, Lan9252Blocking};
use SOES_rs::esc_driver::EscDriver;
use SOES_rs::soes;

//Whatever else you need

#[repr(C)]
pub struct _Objects {
    pub serial: u32,
    pub Key1: u8,
    pub Key2: u8,
    pub Counter: u32,
    pub LedIn: u8,
}

// Global variable expected by SOES C code
#[no_mangle]
pub static mut Obj: _Objects = _Objects {
    serial: 0,
    Key1: 0,
    Key2: 0,
    Counter: 0,
    LedIn: 0,
};

// Dummy ESC configuration for initialization
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

// Output callback example
fn my_outputs() {
    unsafe {
        if Obj.LedIn != 0 {
            defmt::info!("LED ON");
        } else {
            defmt::info!("LED OFF");
        }
    }
}

// Input callback example
fn my_inputs() {
    unsafe {
        Obj.Counter += 1;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    //Configure you MCU, spi etc here

    defmt::info!("Initializing EtherCAT Slave...");

    // Initialize LAN9252 driver
    let mut driver = Lan9252Blocking::new(spi, cs);
    let drv_ptr: *mut dyn EscDriver = &mut driver;

    //unfortunately needed until "ESC_read"/"ESC_write" is expected by soes-c
    unsafe {
        let drv_ref: &mut dyn EscDriver = &mut *drv_ptr;
        set_driver(drv_ref);
    }

    // Initialize EtherCAT slave stack
    let mut ecat_slv = soes::EcatSlave::new(dummy_esc_cfg());
    ecat_slv.set_output_cb(my_outputs);
    ecat_slv.set_input_cb(my_inputs);
    ecat_slv.init();
    ecat_slv.pdi_debug();

    // Main loop
    loop {
        ecat_slv.run();
    }
}

```

## Roadmap

- Remove **esc.c** and **esc_coe** from bindings.
- **ESC-FOE and ESC-EEPROM** implementation.  
- **Unit and functional tests** using mock drivers.  
- **Transition more C functions to safe Rust**.  
- **Develop async interface** compatible with Embassy.  
- **Optional:** support EtherCAT EoE in the future.  

---

## Changelog

### Initial Comit
- Rust wrapper over SOES C library.  
- Blocking SPI driver implemented for LAN9252.  
- CSR read/write functions and process data handling implemented.  
- Unsafe Rust bindings for ESC variables (`ESCvar`, `MBX`, etc.).  
- Logging integrated with `defmt`.  
- Working yet very basic/unsafe example

