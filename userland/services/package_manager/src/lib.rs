#![no_std]
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

extern crate alloc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>, // list of package names
    pub wasm_blob: Vec<u8>,
}

pub struct Repository {
    pub packages: BTreeMap<String, Package>,
}

pub struct RemoteRepository {
    pub host: String,
    pub port: u16,
}

impl RemoteRepository {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
        }
    }

    pub fn download_package(&self, name: &str) -> Result<Package, String> {
        let mut stack = net_stack::WasmNetStack::new();
        let mut client = net_stack::HttpClient::new(&mut stack, &self.host, self.port);

        // Mock connection. In a real system, we'd resolve DNS and connect to the actual IP.
        let mock_ip = smoltcp::wire::IpAddress::v4(8, 8, 8, 8);
        if client.connect(&mut stack, mock_ip, 12345).is_err() {
            // It will fail because of the MockDevice, but we proceed with the logic structure
        }

        let path = alloc::format!("/packages/{}.wasm", name);
        if client.send_request(&mut stack, &path).is_err() {
            // Return mock error if network is not actually available
            return Err(alloc::format!(
                "Failed to send request for package {}",
                name
            ));
        }

        match client.read_raw_response(&mut stack) {
            Ok(raw_response) => {
                match net_stack::HttpClient::parse_raw_response(&raw_response) {
                    Ok(body) => {
                        Ok(Package {
                            name: name.to_string(),
                            version: "1.0.0".to_string(), // In a real system, fetch metadata first
                            dependencies: Vec::new(),
                            wasm_blob: body,
                        })
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository {
    pub fn new() -> Self {
        Self {
            packages: BTreeMap::new(),
        }
    }

    pub fn add_package(&mut self, pkg: Package) {
        self.packages.insert(pkg.name.clone(), pkg);
    }

    pub fn get_package(&self, name: &str) -> Option<&Package> {
        self.packages.get(name)
    }
}

pub struct PackageManager {
    pub repo: Repository,
    pub installed: BTreeSet<String>,
}

impl PackageManager {
    pub fn new(repo: Repository) -> Self {
        Self {
            repo,
            installed: BTreeSet::new(),
        }
    }

    pub fn resolve_dependencies(
        &self,
        target: &str,
        resolved: &mut BTreeSet<String>,
        resolving: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        if resolved.contains(target) {
            return Ok(());
        }
        if resolving.contains(target) {
            return Err(alloc::format!(
                "Circular dependency detected involving {}",
                target
            ));
        }

        let pkg = self
            .repo
            .get_package(target)
            .ok_or_else(|| alloc::format!("Package {} not found in repository", target))?;

        resolving.insert(target.to_string());

        for dep in &pkg.dependencies {
            self.resolve_dependencies(dep, resolved, resolving)?;
        }

        resolving.remove(target);
        resolved.insert(target.to_string());

        Ok(())
    }

    pub fn install(&mut self, target: &str) -> Result<Vec<Package>, String> {
        let mut resolved = BTreeSet::new();
        let mut resolving = BTreeSet::new();

        self.resolve_dependencies(target, &mut resolved, &mut resolving)?;

        let mut to_install = Vec::new();
        for pkg_name in resolved {
            if !self.installed.contains(&pkg_name) {
                let pkg = self.repo.get_package(&pkg_name).unwrap().clone();
                to_install.push(pkg);
                // In a real sandboxed app store, we would verify signatures and
                // prepare the sandbox here.
                self.installed.insert(pkg_name);
            }
        }

        Ok(to_install)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_resolve_dependencies() {
        let mut repo = Repository::new();
        repo.add_package(Package {
            name: "libc".to_string(),
            version: "1.0".to_string(),
            dependencies: vec![],
            wasm_blob: vec![],
        });
        repo.add_package(Package {
            name: "utils".to_string(),
            version: "1.0".to_string(),
            dependencies: vec!["libc".to_string()],
            wasm_blob: vec![],
        });
        repo.add_package(Package {
            name: "app".to_string(),
            version: "1.0".to_string(),
            dependencies: vec!["utils".to_string(), "libc".to_string()],
            wasm_blob: vec![],
        });

        let mut pm = PackageManager::new(repo);
        let installed = pm.install("app").expect("Failed to install");

        assert_eq!(installed.len(), 3);
        assert!(installed.iter().any(|p| p.name == "libc"));
        assert!(installed.iter().any(|p| p.name == "utils"));
        assert!(installed.iter().any(|p| p.name == "app"));
        assert_eq!(pm.installed.len(), 3);
    }

    #[test]
    fn test_missing_dependency() {
        let mut repo = Repository::new();
        repo.add_package(Package {
            name: "app".to_string(),
            version: "1.0".to_string(),
            dependencies: vec!["missing_lib".to_string()],
            wasm_blob: vec![],
        });

        let mut pm = PackageManager::new(repo);
        let err = pm
            .install("app")
            .expect_err("Should fail due to missing dependency");
        assert_eq!(err, "Package missing_lib not found in repository");
    }

    #[test]
    fn test_remote_repository_download_error() {
        let remote = RemoteRepository::new("example.com", 80);
        let result = remote.download_package("testpkg");
        // Because the net_stack uses MockDevice internally which does not really send,
        // it returns Err("Failed to send request for package testpkg")
        assert!(result.is_err());
    }

    #[test]
    fn test_circular_dependency() {
        let mut repo = Repository::new();
        repo.add_package(Package {
            name: "a".to_string(),
            version: "1.0".to_string(),
            dependencies: vec!["b".to_string()],
            wasm_blob: vec![],
        });
        repo.add_package(Package {
            name: "b".to_string(),
            version: "1.0".to_string(),
            dependencies: vec!["a".to_string()],
            wasm_blob: vec![],
        });

        let mut pm = PackageManager::new(repo);
        let err = pm
            .install("a")
            .expect_err("Should fail due to circular dependency");
        assert!(err.contains("Circular dependency detected"));
    }
}
