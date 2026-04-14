#![no_main]

use libfuzzer_sys::fuzz_target;
use kernel::vfs::ext4;
use inference_runtime::InferenceEngine;

fuzz_target!(|data: &[u8]| {
    // Basic fuzzing for kernel components to validate they do not panic
    if data.len() > 10 {
        if let Ok(intent) = std::str::from_utf8(&data[..10]) {
            let mut engine = InferenceEngine::new();
            if let Ok(model) = engine.load_model_by_name(intent) {
               // Load model with fuzzed intent
               let mut engine_mut = InferenceEngine::new();
               let _ = engine_mut.init_execution_context(&model);
            }
        }

        let fs = ext4::Ext4::new();
        let _ = fs.mount();
    }
});
