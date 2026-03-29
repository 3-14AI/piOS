pub struct CoGenerationFramework {
    pub is_running: bool,
    pub tests_generated: u64,
}

impl CoGenerationFramework {
    pub fn new() -> Self {
        Self {
            is_running: false,
            tests_generated: 0,
        }
    }

    pub fn generate_test(&mut self) {
        if self.is_running && self.tests_generated < 1000 {
            self.tests_generated += 1;
        }
    }

    pub fn start(&mut self) {
        if !self.is_running {
            self.is_running = true;
        }
    }

    pub fn stop(&mut self) {
        if self.is_running {
            self.is_running = false;
        }
    }
}

impl Default for CoGenerationFramework {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_co_generation() {
        let mut framework = CoGenerationFramework::new();
        assert_eq!(framework.is_running, false);
        assert_eq!(framework.tests_generated, 0);

        framework.start();
        assert_eq!(framework.is_running, true);

        framework.generate_test();
        assert_eq!(framework.tests_generated, 1);

        framework.stop();
        assert_eq!(framework.is_running, false);
    }
}
