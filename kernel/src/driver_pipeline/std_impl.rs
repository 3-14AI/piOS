pub enum PipelineResult {
    Success,
    ErrorDeviceNotFound,
    ErrorSpecNotFound,
    ErrorGenerationFailed,
    ErrorVerificationFailed,
    ErrorLoadFailed,
}

pub struct DriverPipeline {
    pub is_initialized: bool,
}

impl Default for DriverPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl DriverPipeline {
    pub fn new() -> Self {
        DriverPipeline {
            is_initialized: true,
        }
    }

    pub fn execute_pipeline(
        &self,
        valid_device: bool,
        valid_spec: bool,
        valid_gen: bool,
        valid_verif: bool,
        valid_load: bool,
    ) -> PipelineResult {
        if !self.is_initialized {
            return PipelineResult::ErrorDeviceNotFound; // simplified
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_success() {
        let pipeline = DriverPipeline::new();
        match pipeline.execute_pipeline(true, true, true, true, true) {
            PipelineResult::Success => (),
            _ => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_pipeline_device_not_found() {
        let pipeline = DriverPipeline::new();
        match pipeline.execute_pipeline(false, true, true, true, true) {
            PipelineResult::ErrorDeviceNotFound => (),
            _ => panic!("Expected ErrorDeviceNotFound"),
        }
    }

    #[test]
    fn test_pipeline_spec_not_found() {
        let pipeline = DriverPipeline::new();
        match pipeline.execute_pipeline(true, false, true, true, true) {
            PipelineResult::ErrorSpecNotFound => (),
            _ => panic!("Expected ErrorSpecNotFound"),
        }
    }

    #[test]
    fn test_pipeline_generation_failed() {
        let pipeline = DriverPipeline::new();
        match pipeline.execute_pipeline(true, true, false, true, true) {
            PipelineResult::ErrorGenerationFailed => (),
            _ => panic!("Expected ErrorGenerationFailed"),
        }
    }

    #[test]
    fn test_pipeline_verification_failed() {
        let pipeline = DriverPipeline::new();
        match pipeline.execute_pipeline(true, true, true, false, true) {
            PipelineResult::ErrorVerificationFailed => (),
            _ => panic!("Expected ErrorVerificationFailed"),
        }
    }

    #[test]
    fn test_pipeline_load_failed() {
        let pipeline = DriverPipeline::new();
        match pipeline.execute_pipeline(true, true, true, true, false) {
            PipelineResult::ErrorLoadFailed => (),
            _ => panic!("Expected ErrorLoadFailed"),
        }
    }
}
