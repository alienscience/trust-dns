// Copyright 2015-2017 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![cfg(not(windows))]
#![cfg(feature = "dns-over-https")]

extern crate chrono;
extern crate futures;
extern crate log;
extern crate native_tls;
extern crate rustls;
extern crate tokio;
extern crate trust_dns;
extern crate trust_dns_https;
extern crate trust_dns_native_tls;
extern crate trust_dns_proto;
extern crate trust_dns_server;

mod server_harness;

use std::env;
use std::fs::File;
use std::io::*;
use std::net::*;

use rustls::internal::msgs::codec::Codec;
use rustls::Certificate;
use tokio::runtime::current_thread::Runtime;
use trust_dns::client::*;
use trust_dns_https::HttpsClientStreamBuilder;
use trust_dns_proto::xfer::DnsExchange;

use server_harness::{named_test_harness, query_a};

#[test]
fn test_example_https_toml_startup() {
    named_test_harness("dns_over_tls.toml", move |_, tls_port| {
        let mut cert_der = vec![];
        let server_path = env::var("TDNS_SERVER_SRC_ROOT").unwrap_or_else(|_| ".".to_owned());
        println!("using server src path: {}", server_path);

        File::open(&format!(
            "{}/tests/named_test_configs/sec/example.cert",
            server_path
        )).expect("failed to open cert")
        .read_to_end(&mut cert_der)
        .expect("failed to read cert");

        let mut io_loop = Runtime::new().unwrap();
        let addr: SocketAddr = ("127.0.0.1", tls_port)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();

        let mut tls_conn_builder = HttpsClientStreamBuilder::new();
        let cert = to_trust_anchor(&cert_der);
        tls_conn_builder.add_ca(cert);
        let mp = tls_conn_builder.build(addr, "ns.example.com".to_string());
        let (exchange, handle) = DnsExchange::connect(mp);
        let (bg, mut client) = ClientFuture::from_exchange(exchange, handle);

        // ipv4 should succeed
        io_loop.spawn(bg);
        query_a(&mut io_loop, &mut client);

        let addr: SocketAddr = ("127.0.0.1", tls_port)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        let mut tls_conn_builder = HttpsClientStreamBuilder::new();
        let cert = to_trust_anchor(&cert_der);
        tls_conn_builder.add_ca(cert);
        let mp = tls_conn_builder.build(addr, "ns.example.com".to_string());
        let (exchange, handle) = DnsExchange::connect(mp);
        let (bg, mut client) = ClientFuture::from_exchange(exchange, handle);
        io_loop.spawn(bg);

        // ipv6 should succeed
        query_a(&mut io_loop, &mut client);

        assert!(true);
    })
}

fn to_trust_anchor(cert_der: &[u8]) -> Certificate {
    Certificate::read_bytes(cert_der).unwrap()
}
