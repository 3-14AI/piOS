#!/bin/bash
sed -i 's/tid != self.current_tid/tid != self.current_tids[0]/g' kernel/src/scheduler.rs
sed -i 's/let cur = self.current_tid;/let cur = self.current_tids[0];/g' kernel/src/scheduler.rs
sed -i 's/self.current_tid = next;/let mut next_tids = self.current_tids; next_tids[0] = next; self.current_tids = next_tids;/g' kernel/src/scheduler.rs
