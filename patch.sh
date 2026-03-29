sed -i '/impl MockDevice {/i impl Default for MockDevice {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n' userland/services/net_stack/src/lib.rs
sed -i '/impl WasmNetStack {/i impl Default for WasmNetStack {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n' userland/services/net_stack/src/lib.rs
