#![no_main]

use libfuzzer_sys::fuzz_target;
use kernel::mass_storage::UsbMassStorageDriver;
use kernel::input::UsbHidDriver;
use kernel::usb::Urb;
use kernel::vfs::ext4::Ext4;
use inference_runtime::InferenceEngine;

fuzz_target!(|data: &[u8]| {
    if data.len() >= 512 {
        let mut ms_drv = UsbMassStorageDriver::new();
        ms_drv.init();
        let mut buffer = [0u8; 512];
        let _ = ms_drv.read_blocks(0, 1, &mut buffer);
        let _ = ms_drv.write_blocks(0, 1, &data[..512]);
    }

    if data.len() >= 8 {
        let mut hid_drv = UsbHidDriver::new(1);
        let mut urb = Urb::new(1, data.as_ptr() as usize, data.len());
        urb.actual_length = data.len();
        let _ = hid_drv.handle_urb(&urb);
    }

    let mut fs = Ext4::new();
    let _ = fs.mount();

    if data.len() > 10 {
        if let Ok(intent) = std::str::from_utf8(&data[..10]) {
            let mut engine = InferenceEngine::new();
            if let Ok(model) = engine.load_model_by_name(intent) {
               let mut engine_mut = InferenceEngine::new();
               let _ = engine_mut.init_execution_context(&model);
            }
        }
    }
});
