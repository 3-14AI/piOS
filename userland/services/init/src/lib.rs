#![no_std]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use nl_sh::NlShell;
use inference_runtime::{InferenceEngine, Model, Tensor};
use alloc::boxed::Box;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Failed,
    Stopping,
}

#[derive(Debug, Clone)]
pub struct UnitFile {
    pub name: String,
    pub dependencies: Vec<String>,
    pub exec_start: String,
    pub restart_on_failure: bool,
}

pub struct Service {
    pub unit: UnitFile,
    pub state: ServiceState,
    pub restart_count: u32,
}

pub fn parse_semantic_deps(output: &str) -> Vec<String> {
    let trimmed = output.trim().trim_matches(char::from(0));
    if let Some(deps_str) = trimmed.strip_prefix("dependencies=") {
        return deps_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    alloc::vec![]
}

pub fn parse_semantic_restart(output: &str) -> bool {
    let trimmed = output.trim().trim_matches(char::from(0));
    trimmed == "restart=true"
}

pub trait SemanticProvider {
    fn resolve_dependencies(&mut self, name: &str) -> Vec<String>;
    fn analyze_crash(&mut self, name: &str) -> bool;
}

pub struct AiSemanticProvider {
    engine: InferenceEngine,
    model: Model,
}

impl AiSemanticProvider {
    pub fn new() -> Result<Self, &'static str> {
        let mut engine = InferenceEngine::new();
        let model = engine
            .load_model_by_name("init_supervisor")
            .map_err(|_| "Failed to load init supervisor model")?;
        Ok(Self { engine, model })
    }
}

impl SemanticProvider for AiSemanticProvider {
    fn resolve_dependencies(&mut self, name: &str) -> Vec<String> {
        let ctx = match self.engine.init_execution_context(&self.model) {
            Ok(c) => c,
            Err(_) => return alloc::vec![],
        };

        let tensor = Tensor::new(name.as_bytes().to_vec(), alloc::vec![name.len()]);
        let _ = self.engine.set_input(ctx, 0, &tensor);
        let _ = self.engine.compute(ctx);

        let mut out = [0u8; 64];
        if let Ok(bytes_written) = self.engine.get_output(ctx, 0, &mut out) {
            let output_str = core::str::from_utf8(&out[..bytes_written]).unwrap_or("");
            return parse_semantic_deps(output_str);
        }

        alloc::vec![]
    }

    fn analyze_crash(&mut self, name: &str) -> bool {
        let ctx = match self.engine.init_execution_context(&self.model) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let tensor = Tensor::new(name.as_bytes().to_vec(), alloc::vec![name.len()]);
        let _ = self.engine.set_input(ctx, 0, &tensor);
        let _ = self.engine.compute(ctx);

        let mut out = [0u8; 64];
        if let Ok(bytes_written) = self.engine.get_output(ctx, 0, &mut out) {
            let output_str = core::str::from_utf8(&out[..bytes_written]).unwrap_or("");
            return parse_semantic_restart(output_str);
        }

        false
    }
}

pub struct InitManager {
    services: BTreeMap<String, Service>,
    pub default_shell: Option<NlShell>,
    pub semantic_provider: Box<dyn SemanticProvider>,
}

impl InitManager {
    pub fn new() -> Result<Self, &'static str> {
        let provider = AiSemanticProvider::new()?;
        Ok(Self {
            services: BTreeMap::new(),
            default_shell: None,
            semantic_provider: Box::new(provider),
        })
    }

    pub fn new_with_provider(provider: Box<dyn SemanticProvider>) -> Self {
        Self {
            services: BTreeMap::new(),
            default_shell: None,
            semantic_provider: provider,
        }
    }

    pub fn setup_default_shell(&mut self) -> Result<(), &'static str> {
        let shell = NlShell::new()?;
        self.default_shell = Some(shell);

        self.load_unit(UnitFile {
            name: "nl_sh".to_string(),
            dependencies: alloc::vec![],
            exec_start: "nl_sh".to_string(),
            restart_on_failure: true,
        });

        self.start_service("nl_sh")
    }

    pub fn load_unit(&mut self, unit: UnitFile) {
        let name = unit.name.clone();
        self.services.insert(
            name,
            Service {
                unit,
                state: ServiceState::Stopped,
                restart_count: 0,
            },
        );
    }

    pub fn start_service(&mut self, name: &str) -> Result<(), &'static str> {
        let mut deps_to_start;
        if let Some(service) = self.services.get(name) {
            if service.state == ServiceState::Running {
                return Ok(());
            }
            deps_to_start = service.unit.dependencies.clone();
        } else {
            return Err("Service not found");
        }

        // Semantically resolve missing dependencies
        let semantic_deps = self.semantic_provider.resolve_dependencies(name);
        for s_dep in semantic_deps {
            if !deps_to_start.contains(&s_dep) {
                deps_to_start.push(s_dep);
            }
        }

        for dep in deps_to_start {
            self.start_service(&dep)?;
        }

        if let Some(service) = self.services.get_mut(name) {
            service.state = ServiceState::Starting;
            // Here we would actually spawn the WASM component or process
            service.state = ServiceState::Running;
        }

        Ok(())
    }

    pub fn stop_service(&mut self, name: &str) -> Result<(), &'static str> {
        if let Some(service) = self.services.get_mut(name) {
            service.state = ServiceState::Stopping;
            // Here we would actually kill/stop the WASM component or process
            service.state = ServiceState::Stopped;
            Ok(())
        } else {
            Err("Service not found")
        }
    }

    pub fn graceful_shutdown(&mut self) {
        let names: Vec<String> = self.services.keys().cloned().collect();
        for name in names.iter().rev() {
            let _ = self.stop_service(name);
        }
    }

    pub fn watchdog_tick(&mut self) {
        // AI-driven watchdog implementation
        // If a service fails, check restart_on_failure. If false, ask AI if it should be restarted anyway.
        let mut to_restart = Vec::new();
        for (name, service) in self.services.iter() {
            if service.state == ServiceState::Failed {
                if service.unit.restart_on_failure {
                    to_restart.push(name.clone());
                } else {
                    // Temporarily release borrow to call self method
                }
            }
        }

        // Second pass for semantic analysis to avoid borrow checker issues
        for name in self.services.keys().cloned().collect::<Vec<_>>() {
            let state = self.services.get(&name).unwrap().state;
            let restart = self.services.get(&name).unwrap().unit.restart_on_failure;
            if state == ServiceState::Failed && !restart {
                if self.semantic_provider.analyze_crash(&name) {
                    to_restart.push(name);
                }
            }
        }

        for name in to_restart {
            let _ = self.start_service(&name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    struct MockSemanticProvider;
    impl SemanticProvider for MockSemanticProvider {
        fn resolve_dependencies(&mut self, name: &str) -> Vec<String> {
            if name == "web_unresolved" {
                alloc::vec!["db".to_string()]
            } else {
                alloc::vec![]
            }
        }

        fn analyze_crash(&mut self, name: &str) -> bool {
            if name == "critical_daemon" {
                true
            } else {
                false
            }
        }
    }

    fn create_test_init() -> InitManager {
        InitManager::new_with_provider(Box::new(MockSemanticProvider))
    }

    #[test]
    fn test_parse_semantic_deps() {
        assert_eq!(parse_semantic_deps("dependencies=db,cache\0"), alloc::vec!["db", "cache"]);
        assert_eq!(parse_semantic_deps("dependencies=db  ,  cache \0"), alloc::vec!["db", "cache"]);
        assert_eq!(parse_semantic_deps("unknown"), alloc::vec![] as Vec<String>);
    }

    #[test]
    fn test_parse_semantic_restart() {
        assert_eq!(parse_semantic_restart("restart=true\0"), true);
        assert_eq!(parse_semantic_restart("restart=false\0"), false);
        assert_eq!(parse_semantic_restart("unknown"), false);
    }

    #[test]
    fn test_setup_default_shell() {
        let mut init = create_test_init();
        assert_eq!(init.setup_default_shell(), Ok(()));
        assert!(init.default_shell.is_some());
        assert_eq!(
            init.services.get("nl_sh").unwrap().state,
            ServiceState::Running
        );
    }

    #[test]
    fn test_dependency_graph() {
        let mut init = create_test_init();
        init.load_unit(UnitFile {
            name: "db".to_string(),
            dependencies: vec![],
            exec_start: "db.wasm".to_string(),
            restart_on_failure: true,
        });
        init.load_unit(UnitFile {
            name: "web".to_string(),
            dependencies: vec!["db".to_string()],
            exec_start: "web.wasm".to_string(),
            restart_on_failure: true,
        });

        assert_eq!(init.start_service("web"), Ok(()));
        assert_eq!(
            init.services.get("db").unwrap().state,
            ServiceState::Running
        );
        assert_eq!(
            init.services.get("web").unwrap().state,
            ServiceState::Running
        );
    }

    #[test]
    fn test_graceful_shutdown() {
        let mut init = create_test_init();
        init.load_unit(UnitFile {
            name: "service1".to_string(),
            dependencies: vec![],
            exec_start: "s1.wasm".to_string(),
            restart_on_failure: false,
        });

        init.start_service("service1").unwrap();
        assert_eq!(
            init.services.get("service1").unwrap().state,
            ServiceState::Running
        );

        init.graceful_shutdown();
        assert_eq!(
            init.services.get("service1").unwrap().state,
            ServiceState::Stopped
        );
    }

    #[test]
    fn test_watchdog() {
        let mut init = create_test_init();
        init.load_unit(UnitFile {
            name: "service1".to_string(),
            dependencies: vec![],
            exec_start: "s1.wasm".to_string(),
            restart_on_failure: true,
        });

        init.start_service("service1").unwrap();
        init.services.get_mut("service1").unwrap().state = ServiceState::Failed;

        init.watchdog_tick();

        assert_eq!(
            init.services.get("service1").unwrap().state,
            ServiceState::Running
        );
    }

    #[test]
    fn test_semantic_crash_analysis() {
        let mut init = create_test_init();
        init.load_unit(UnitFile {
            name: "critical_daemon".to_string(),
            dependencies: vec![],
            exec_start: "crit.wasm".to_string(),
            restart_on_failure: false, // Normally wouldn't restart
        });

        init.start_service("critical_daemon").unwrap();
        init.services.get_mut("critical_daemon").unwrap().state = ServiceState::Failed;

        // Watchdog tick should use semantic analysis to restart it anyway
        init.watchdog_tick();

        assert_eq!(
            init.services.get("critical_daemon").unwrap().state,
            ServiceState::Running
        );
    }

    #[test]
    fn test_semantic_dependency_resolution() {
        let mut init = create_test_init();
        init.load_unit(UnitFile {
            name: "db".to_string(),
            dependencies: vec![],
            exec_start: "db.wasm".to_string(),
            restart_on_failure: true,
        });
        init.load_unit(UnitFile {
            name: "web_unresolved".to_string(),
            dependencies: vec![], // Missing db dependency
            exec_start: "web.wasm".to_string(),
            restart_on_failure: true,
        });

        // This will semantically inject "db" as a dependency
        assert_eq!(init.start_service("web_unresolved"), Ok(()));

        assert_eq!(
            init.services.get("db").unwrap().state,
            ServiceState::Running
        );
        assert_eq!(
            init.services.get("web_unresolved").unwrap().state,
            ServiceState::Running
        );
    }
}
