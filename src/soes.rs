use cty::c_void;

use crate::bindings::*;
use crate::esc_driver::EscDriver;
use core::slice;

use core::cell::Cell;
use core::ffi::CStr;
use core::mem::MaybeUninit;

#[no_mangle]
pub extern "C" fn DPRINT_RUST(msg: *const u8) {
    unsafe {
        if !msg.is_null() {
            if let Ok(s) = CStr::from_ptr(msg as *const i8).to_str() {
                defmt::info!("{}", s);
            }
        }
    }
}

pub const fn max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

#[no_mangle]
pub static mut ESCvar: _ESCvar = unsafe { MaybeUninit::zeroed().assume_init() };

#[no_mangle]
pub static mut MBXcontrol: [_MBXcontrol; MBXBUFFERS as usize] =
    unsafe { MaybeUninit::zeroed().assume_init() };

pub const MBX_SIZE: usize = MBXBUFFERS as usize * max(MBXSIZE as usize, MBXSIZEBOOT as usize);
#[no_mangle]
pub static mut MBX: [u8; MBX_SIZE] = [0; MBX_SIZE];

#[no_mangle]
pub static mut SMmap2: [_SMmap; MAX_MAPPINGS_SM2 as usize] =
    unsafe { MaybeUninit::zeroed().assume_init() };

#[no_mangle]
pub static mut SMmap3: [_SMmap; MAX_MAPPINGS_SM3 as usize] =
    unsafe { MaybeUninit::zeroed().assume_init() };

/// EtherCAT Slave abstraction wrapping SOES and the LAN9252 driver
//pub struct EcatSlave<D: EscDriver> {
//    driver: D,
//    config: esc_cfg,
//}
pub struct EcatSlave {
    cfg: esc_cfg,
    // Global variables
    //mbx: [[u8; MAX_MBXSIZE]; MBXBUFFERS],
    //sm2_map: [_SMmap; MAX_MAPPINGS_SM2],
    //sm3_map: [_SMmap; MAX_MAPPINGS_SM3],
    //esc_var: _ESCvar,

    // Private
    watchdog: i32,

    // Optional PDO buffers
    //#[cfg(MAX_MAPPINGS_SM2 > 0)]
    rxpdo: [u8; MAX_RXPDO_SIZE as usize],
    //#[cfg(MAX_MAPPINGS_SM3 > 0)]
    txpdo: [u8; MAX_TXPDO_SIZE as usize],

    //IO callbacks
    output_cb: Option<fn()>,
    input_cb: Option<fn()>,
}

impl EcatSlave {
    pub fn new(cfg: esc_cfg_t) -> Self {
        let esc_var: _ESCvar = unsafe { MaybeUninit::zeroed().assume_init() };

        Self {
            cfg,
            //esc_var,
            watchdog: cfg.watchdog_cnt as i32,
            rxpdo: [0u8; MAX_RXPDO_SIZE as usize],
            txpdo: [0u8; MAX_TXPDO_SIZE as usize],
            output_cb: None,
            input_cb: None,
        }
    }

    /// Register a custom output callback
    pub fn set_output_cb(&mut self, cb: fn()) {
        self.output_cb = Some(cb);
    }

    /// Register a custom input callback
    pub fn set_input_cb(&mut self, cb: fn()) {
        self.input_cb = Some(cb);
    }

    /// Initialize the EtherCAT slave stack (equivalent of `ecat_slv_init`)
    pub fn init(&mut self) {
        unsafe {
            defmt::info!("Slave stack init started");

            // Watchdog
            let watchdog = self.cfg.watchdog_cnt;
            defmt::debug!("Watchdog count: {}", watchdog);

            // Stack + hardware init
            ESC_config(&mut self.cfg as *mut esc_cfg);

            // Wait until ESC startup done
            loop {
                unsafe {
                    ESC_read(
                        ESCREG_DLSTATUS as u16,
                        &mut ESCvar.DLstatus as *mut u16 as *mut core::ffi::c_void,
                        core::mem::size_of::<u16>() as u16,
                    );

                    // Convert endianness if needed (C etohs)
                    ESCvar.DLstatus = u16::from_le(ESCvar.DLstatus);
                }

                if (ESCvar.DLstatus & 0x0001) != 0 {
                    defmt::info!("ESC started up (0x{:04x})", ESCvar.DLstatus);
                    break;
                }
            }

            // TODO: add FOE_init, EOE_init if enabled in SOES build

            // Reset ESC to init state
            ESC_ALstatus(ESCinit as u8);
            defmt::info!("Writing AL status to ESCInit");
            ESC_ALerror(ALERR_NONE as u16);
            defmt::info!("Remove errors");
            ESC_stopmbx();
            defmt::info!("Stopping mailbox");
            ESC_stopinput();
            defmt::info!("Stopping input");
            ESC_stopoutput();
            defmt::info!("Stopping output");
        }
    }

    /// Print some ESC registers for debugging PDI
    pub fn pdi_debug(&mut self) {
        let mut value: u8 = 0;
        let mut read_print = |addr: u16, name: &str| {
            unsafe {
                ESC_read(
                    addr as u16,
                    &mut value as *mut _ as *mut core::ffi::c_void,
                    core::mem::size_of::<u8>() as u16,
                );
            }
            defmt::info!("{} [0x{:04X}]: {}", name, addr, value);
        };

        defmt::info!("[ESC debug]");
        read_print(0x0000, "Type");
        read_print(0x0001, "Revision");
        read_print(0x0004, "FMMU count");
        read_print(0x0005, "Sync Managers count");
        read_print(0x0006, "RAM size");
    }

    /// Polling function
    pub fn poll(&mut self) {
        // Read local time

        unsafe {
            ESC_read(
                ESCREG_LOCALTIME as u16,
                &mut ESCvar.Time as *mut u32 as *mut core::ffi::c_void,
                core::mem::size_of::<u32>() as u16,
            );
            ESCvar.Time = u32::from_le(ESCvar.Time); //not sure in need but i need to handle endianess properly :(
                                                     //defmt::info!("ESC Local Time: {} µs", ESCvar.Time);

            /* Check the state machine */
            ESC_state();

            /* Check the SM activation event */
            ESC_sm_act_event();

            /* Check mailboxes */
            // Minimal mailbox handling     //need to implement ESC_download_pre_objecthandler etc
            if ESC_mbxprocess() > 0 {
                ESC_coeprocess();
            }

            /* Call emulated eeprom handler if set */
            if let Some(handler) = ESCvar.esc_hw_eep_handler {
                handler();
            }
        }
    }

    /* Function to update local I/O, call read ethercat outputs, call
     * write ethercat inputs. Implement watch-dog counter to count-out if we have
     * made state change affecting the App.state.
     */
    pub fn dig_process(&mut self, flags: u8) {
        // Handle watchdog
        if (flags & DIG_PROCESS_WD_FLAG) > 0 {
            if self.watchdog > 0 {
                self.watchdog -= 1;
            }

            if self.watchdog <= 0 && (unsafe { ESCvar.App.state } & APPSTATE_OUTPUT as u8) > 0 {
                defmt::warn!("DIG_process watchdog expired");
                unsafe {
                    ESC_ALstatusgotoerror((ESCsafeop | ESCerror) as u8, ALERR_WATCHDOG as u16);
                }
            } else if (unsafe { ESCvar.App.state } & APPSTATE_OUTPUT as u8) == 0 {
                self.watchdog = unsafe { ESCvar.watchdogcnt };
            }
        }

        // Handle Outputs
        if (flags & DIG_PROCESS_OUTPUTS_FLAG) > 0 {
            if (unsafe { ESCvar.App.state } & APPSTATE_OUTPUT as u8) > 0
                && (unsafe { ESCvar.ALevent } & ESCREG_ALEVENT_SM2 as u16) != 0
            {
                self.rxpdo_update();
                self.watchdog = unsafe { ESCvar.watchdogcnt };
                if let Some(cb) = self.output_cb {
                    cb();
                } else {
                    defmt::warn!("ESC: No Output cb defined!");
                }
            } else if (unsafe { ESCvar.ALevent } & ESCREG_ALEVENT_SM2 as u16) != 0 {
                self.rxpdo_update();
            }
        }

        // Call application
        if (flags & DIG_PROCESS_APP_HOOK_FLAG) > 0 {
            unsafe {
                if let Some(hook) = ESCvar.application_hook {
                    hook();
                }
            }
        }

        // Handle Inputs
        if (flags & DIG_PROCESS_INPUTS_FLAG) > 0 {
            if unsafe { ESCvar.App.state } > 0 {
                if let Some(cb) = self.input_cb {
                    cb();
                } else {
                    defmt::warn!("ESC: No input cb defined!");
                }
                self.txpdo_update();
            }
        }
    }

    /// Write local process data to Sync Manager 3 (TXPDO).
    pub fn txpdo_update(&mut self) {
        unsafe {
            if let Some(override_fn) = ESCvar.txpdo_override {
                override_fn();
            } else {
                if MAX_MAPPINGS_SM3 > 0 {
                    COE_pdoPack(
                        self.txpdo.as_mut_ptr(),
                        ESCvar.sm3mappings,
                        SMmap3.as_mut_ptr(),
                    );
                }
                ESC_write(
                    ESC_SM3_sma as u16,
                    self.txpdo.as_mut_ptr() as *mut core::ffi::c_void,
                    ESCvar.ESC_SM3_sml,
                );
            }
        }
    }

    /// Read Sync Manager 2 (RXPDO) into local process data.
    pub fn rxpdo_update(&mut self) {
        unsafe {
            if let Some(override_fn) = ESCvar.rxpdo_override {
                override_fn();
            } else {
                ESC_read(
                    ESC_SM2_sma as u16,
                    self.rxpdo.as_mut_ptr() as *mut core::ffi::c_void,
                    ESCvar.ESC_SM2_sml,
                );

                if MAX_MAPPINGS_SM2 > 0 {
                    COE_pdoUnpack(
                        self.rxpdo.as_mut_ptr(),
                        ESCvar.sm2mappings,
                        SMmap2.as_mut_ptr(),
                    );
                }
            }
        }
    }

    pub fn print_al_error(&self) {
        unsafe {
            if ESCvar.ALerror != 0 {
                defmt::warn!("AL Error 0x{:04X}\r\n", ESCvar.ALerror);
            }
        }
    }

    pub fn run(&mut self) {
        self.poll();
        self.dig_process(
            DIG_PROCESS_WD_FLAG
                | DIG_PROCESS_OUTPUTS_FLAG
                | DIG_PROCESS_APP_HOOK_FLAG
                | DIG_PROCESS_INPUTS_FLAG,
        );

        self.print_al_error()
    }
}

/// ESC C bindings will call this --> Will need proper implementation enventually
#[no_mangle]
pub extern "C" fn APP_safeoutput() {
    defmt::info!("APP_safeoutput() called");
}

use cty::{c_uchar, c_uint, c_ushort, size_t};

#[no_mangle]
pub extern "C" fn ESC_download_pre_objecthandler(
    index: c_ushort,
    subindex: c_uchar,
    _data: *mut c_void,
    _size: size_t,
    _flags: c_ushort,
) -> c_uint {
    defmt::warn!("ESC_download_pre_objecthandler called for index {:04x}, subindex {:02x}, but not implemented", index, subindex);
    0
}

#[no_mangle]
pub extern "C" fn ESC_download_post_objecthandler(
    index: c_ushort,
    subindex: c_uchar,
    _flags: c_ushort,
) -> c_uint {
    defmt::warn!("ESC_download_post_objecthandler called for index {:04x}, subindex {:02x}, but not implemented", index, subindex);
    0
}

#[no_mangle]
pub extern "C" fn ESC_upload_pre_objecthandler(
    index: c_ushort,
    subindex: c_uchar,
    _data: *mut c_void,
    _size: size_t,
    _flags: c_ushort,
) -> c_uint {
    defmt::warn!("ESC_upload_pre_objecthandler called for index {:04x}, subindex {:02x}, but not implemented", index, subindex);
    0
}

#[no_mangle]
pub extern "C" fn ESC_upload_post_objecthandler(
    index: c_ushort,
    subindex: c_uchar,
    _flags: c_ushort,
) -> c_uint {
    defmt::warn!("ESC_upload_post_objecthandler called for index {:04x}, subindex {:02x}, but not implemented", index, subindex);
    0
}

pub const DIG_PROCESS_INPUTS_FLAG: u8 = 0x01;
pub const DIG_PROCESS_OUTPUTS_FLAG: u8 = 0x02;
pub const DIG_PROCESS_WD_FLAG: u8 = 0x04;
pub const DIG_PROCESS_APP_HOOK_FLAG: u8 = 0x08;

#[inline(always)]
pub const fn is_rxpdo(index: u16) -> bool {
    index >= 0x1600 && index < 0x1800
}

#[inline(always)]
pub const fn is_txpdo(index: u16) -> bool {
    index >= 0x1A00 && index < 0x1C00
}
