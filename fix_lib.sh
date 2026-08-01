sed -i 's/pub use dns_client::\*;//' userland/services/net_stack/src/lib.rs
sed -i 's/pub use http_client::\*;//' userland/services/net_stack/src/lib.rs
echo "pub use dns_client::*;" >> userland/services/net_stack/src/lib.rs
echo "pub use http_client::*;" >> userland/services/net_stack/src/lib.rs
