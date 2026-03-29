extern crate alloc;
use alloc::vec::Vec;
use vstd::prelude::*;

verus! {
    /// Pipeline result representing success or failure
    pub enum PipelineResult {
        Success,
        ErrorDeviceNotFound,
        ErrorSpecNotFound,
        ErrorGenerationFailed,
        ErrorVerificationFailed,
        ErrorLoadFailed,
    }

    /// Verifiable Pipeline Executor
    pub struct DriverPipeline {
        pub is_initialized: bool,
    }

    impl DriverPipeline {
        pub fn new() -> (p: Self)
            ensures p.is_initialized == true
        {
            DriverPipeline { is_initialized: true }
        }

        pub fn execute_pipeline(&self, valid_device: bool, valid_spec: bool, valid_gen: bool, valid_verif: bool, valid_load: bool) -> (res: PipelineResult)
            requires
                self.is_initialized == true,
            ensures
                (valid_device && valid_spec && valid_gen && valid_verif && valid_load) ==> res is Success,
                (!valid_device) ==> res is ErrorDeviceNotFound,
                (valid_device && !valid_spec) ==> res is ErrorSpecNotFound,
                (valid_device && valid_spec && !valid_gen) ==> res is ErrorGenerationFailed,
                (valid_device && valid_spec && valid_gen && !valid_verif) ==> res is ErrorVerificationFailed,
                (valid_device && valid_spec && valid_gen && valid_verif && !valid_load) ==> res is ErrorLoadFailed,
        {
            if !valid_device {
                PipelineResult::ErrorDeviceNotFound
            } else if !valid_spec {
                PipelineResult::ErrorSpecNotFound
            } else if !valid_gen {
                PipelineResult::ErrorGenerationFailed
            } else if !valid_verif {
                PipelineResult::ErrorVerificationFailed
            } else if !valid_load {
                PipelineResult::ErrorLoadFailed
            } else {
                PipelineResult::Success
            }
        }

        pub fn execute_with_retry(
            &self,
            valid_device: bool,
            valid_spec: bool,
            valid_gens: &[bool],
            valid_verifs: &[bool],
            valid_load: bool,
        ) -> (res: PipelineResult)
            requires
                self.is_initialized == true,
            ensures
                (!valid_device) ==> res is ErrorDeviceNotFound,
                (valid_device && !valid_spec) ==> res is ErrorSpecNotFound,
                (valid_device && valid_spec && (valid_gens@.len() == 0 || valid_gens@.len() != valid_verifs@.len())) ==> res is ErrorGenerationFailed,
        {
            if !valid_device {
                return PipelineResult::ErrorDeviceNotFound;
            }
            if !valid_spec {
                return PipelineResult::ErrorSpecNotFound;
            }

            let max_retries: usize = valid_gens.len();
            if max_retries == 0 || max_retries != valid_verifs.len() {
                return PipelineResult::ErrorGenerationFailed;
            }

            let mut i: usize = 0;
            let mut final_gen = false;
            let mut final_verif = false;

            while i < max_retries
                invariant
                    i <= max_retries,
                    valid_gens@.len() == max_retries as int,
                    valid_verifs@.len() == max_retries as int,
                decreases max_retries - i,
            {
                final_gen = valid_gens[i];
                final_verif = valid_verifs[i];

                if final_gen && final_verif {
                    break;
                }
                i += 1;
            }

            if !final_gen {
                PipelineResult::ErrorGenerationFailed
            } else if !final_verif {
                PipelineResult::ErrorVerificationFailed
            } else if !valid_load {
                PipelineResult::ErrorLoadFailed
            } else {
                PipelineResult::Success
            }
        }
    }
}
