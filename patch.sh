sed -i 's/if self.memory_allocations < 0xffff_ffff_ffff_ffff {/self.memory_allocations = self.memory_allocations.saturating_add(1);/g' kernel/src/telemetry.rs
sed -i '/self.memory_allocations += 1;/d' kernel/src/telemetry.rs
sed -i '/if self.ipc_messages_sent < 0xffff_ffff_ffff_ffff {/c\        self.ipc_messages_sent = self.ipc_messages_sent.saturating_add(1);' kernel/src/telemetry.rs
sed -i '/self.ipc_messages_sent += 1;/d' kernel/src/telemetry.rs
sed -i '/if self.page_faults < 0xffff_ffff_ffff_ffff {/c\        self.page_faults = self.page_faults.saturating_add(1);' kernel/src/telemetry.rs
sed -i '/self.page_faults += 1;/d' kernel/src/telemetry.rs
