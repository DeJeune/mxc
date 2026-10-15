// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Connects to one TCP listener through host loopback.

use std::error::Error;
use std::io::{self, Write};

use mxc_sdk::v1::policy::{
    NetworkAction, NetworkEgressPolicy, NetworkIngressPolicy, NetworkPeerPolicy, NetworkPolicy,
    NetworkPortPolicy, NetworkProtocol, NetworkRulePolicy,
};
use mxc_sdk::v1::{self, ContainerRequest, Containment, WaitResult};

fn run() -> Result<i32, Box<dyn Error>> {
    let request = ContainerRequest {
        containment: Containment::ProcessContainer(Default::default()),
        network: Some(NetworkPolicy {
            egress: Some(NetworkEgressPolicy {
                default: Some(NetworkAction::Deny),
                allow: Some(vec![NetworkRulePolicy {
                    to: Some(vec![NetworkPeerPolicy::new("127.0.0.1/32")]),
                    ports: Some(vec![NetworkPortPolicy {
                        protocol: Some(NetworkProtocol::Tcp),
                        port: Some(8081),
                        ..Default::default()
                    }]),
                }]),
                ..Default::default()
            }),
            ingress: Some(NetworkIngressPolicy {
                default: Some(NetworkAction::Deny),
                host_loopback: Some(NetworkAction::Allow),
            }),
            ..Default::default()
        }),
        timeout_ms: Some(30_000),
        ..ContainerRequest::new(
            "curl.exe --fail --show-error --silent --max-time 10 --proxy \"\" http://127.0.0.1:8081/",
        )
    };
    let result = v1::run(request, Default::default())?;

    io::stdout().write_all(&result.stdout)?;
    io::stderr().write_all(&result.stderr)?;
    for warning in result.warnings {
        eprintln!("warning: {warning}");
    }
    Ok(match result.outcome {
        WaitResult::Exited(code) => code,
        WaitResult::TimedOut => {
            eprintln!("workload timed out");
            124
        }
    })
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("MXC error: {error}");
            std::process::exit(1);
        }
    }
}
