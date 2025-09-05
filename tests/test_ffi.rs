extern crate SOES_rs; // ton crate
use core::ptr;
use SOES_rs::*;

unsafe extern "C" fn dummy_hook() {
    // Called if application_hook is triggered
}

/// Basic FFI test: create an esc_cfg and call ESC_init
#[test]
fn test_esc_init() {
    let mut cfg = SOES_rs::bindings::esc_cfg {
        user_arg: ptr::null_mut(),
        use_interrupt: 0,
        watchdog_cnt: 1234,
        skip_default_initialization: false,
        set_defaults_hook: None,
        pre_state_change_hook: None,
        post_state_change_hook: None,
        application_hook: Some(dummy_hook),
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
    };

    unsafe {
        SOES_rs::bindings::ESC_init(&mut cfg as *mut SOES_rs::bindings::esc_cfg);
    }

    // No panic = success. We just ensure linkage and execution works.
}
