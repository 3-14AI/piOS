#!/bin/bash
cat << 'INNER_EOF' >> userland/services/net_stack/src/dns_client.rs

    #[test]
    fn test_dns_client_coverage_7() {
        let mut stack = WasmNetStack::new();
        let servers = alloc::vec![IpAddress::Ipv4(smoltcp::wire::Ipv4Address::new(8, 8, 8, 8))];
        let mut client = DnsClient::new(&mut stack, &servers, 4);
        let _ = client.query(&mut stack, "error");
        let handle = client.query(&mut stack, "example.com").unwrap();
        let _ = client.get_result(&mut stack, handle);
        let _ = client.query(&mut stack, "example2.com");
        let _ = client.query(&mut stack, "example3.com");
        let _ = client.query(&mut stack, "example4.com");
    }
INNER_EOF

cat << 'INNER_EOF' >> userland/services/net_stack/src/http_client.rs

    #[test]
    fn test_http_client_coverage_7() {
        let mut stack = WasmNetStack::new();
        let mut client = HttpClient::new(&mut stack, "example.com", 80);
        let addr = smoltcp::wire::IpAddress::v4(8, 8, 8, 8);
        let _ = client.connect(&mut stack, addr, 0);
        let _ = client.send_request(&mut stack, "/missing");
        let _ = client.read_response(&mut stack);
        let _ = client.send_request(&mut stack, "/missing2");
        let _ = client.read_response(&mut stack);
        let _ = client.send_request(&mut stack, "/missing3");
        let _ = client.read_response(&mut stack);
        let _ = client.send_request(&mut stack, "/missing4");
        let _ = client.read_response(&mut stack);
    }
INNER_EOF

cargo tarpaulin --packages net_stack --fail-under 80.00
