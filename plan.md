1. Update `userland/services/net_stack/Cargo.toml` to include `proto-dns` and `socket-dns` in `smoltcp` features. This has already been mostly done but making sure `Cargo.lock` hasn't had unintended changes.
2. Implement a `dns.rs` module in `userland/services/net_stack/src/dns.rs`. It will provide a `DnsResolver` struct wrapping a `DnsSocket` and logic to start DNS queries and parse responses.
3. Implement an `http.rs` module in `userland/services/net_stack/src/http.rs`. It will provide an `HttpClient` struct wrapping a `TcpSocket` and logic to form basic HTTP GET requests and read the response.
4. Integrate `DnsResolver` and `HttpClient` into `WasmNetStack` in `userland/services/net_stack/src/lib.rs`.
5. Add unit tests for both DNS resolving and HTTP client logic using `WasmNetStack`.
6. Run `pre_commit_instructions` and make sure formatting, tests, and coverage are met.
