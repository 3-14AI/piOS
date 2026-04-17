#![no_std]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

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

pub struct InitManager {
    services: BTreeMap<String, Service>,
}

impl Default for InitManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InitManager {
    pub fn new() -> Self {
        Self {
            services: BTreeMap::new(),
        }
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
        let deps_to_start;
        if let Some(service) = self.services.get(name) {
            if service.state == ServiceState::Running {
                return Ok(());
            }
            deps_to_start = service.unit.dependencies.clone();
        } else {
            return Err("Service not found");
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
        // Simple watchdog implementation
        // If a service fails and restart_on_failure is true, try to restart it
        let mut to_restart = Vec::new();
        for (name, service) in self.services.iter() {
            if service.state == ServiceState::Failed && service.unit.restart_on_failure {
                to_restart.push(name.clone());
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

    #[test]
    fn test_dependency_graph() {
        let mut init = InitManager::new();
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
        let mut init = InitManager::new();
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
        let mut init = InitManager::new();
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
}
