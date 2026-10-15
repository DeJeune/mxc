// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Allows direct HTTPS egress to one caller-selected IPv4 address.

use std::error::Error;
use std::io::{self, Write};
use std::net::Ipv4Addr;

use mxc_sdk::v1::policy::{
    NetworkAction, NetworkEgressPolicy, NetworkIngressPolicy, NetworkPeerPolicy, NetworkPolicy,
    NetworkPortPolicy, NetworkProtocol, NetworkRulePolicy,
};
use mxc_sdk::v1::{self, ContainerRequest, Containment, WaitResult};

fn run() -> Result<i32, Box<dyn Error>> {
    let ip: Ipv4Addr = std::env::var("MXC_DESTINATION_IP")?.parse()?;
    let host = std::env::var("MXC_DESTINATION_HOST")?;
    if host.is_empty()
        || !host
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-')
    {
        return Err(io::Error::other("MXC_DESTINATION_HOST must be an HTTPS hostname").into());
    }
    let request = ContainerRequest {
        containment: Containment::ProcessContainer(Default::default()),
        network: Some(NetworkPolicy {
            egress: Some(NetworkEgressPolicy {
                default: Some(NetworkAction::Deny),
                allow: Some(vec![NetworkRulePolicy {
                    to: Some(vec![NetworkPeerPolicy::new(format!("{ip}/32"))]),
                    ports: Some(vec![NetworkPortPolicy {
                        protocol: Some(NetworkProtocol::Tcp),
                        port: Some(443),
                        ..Default::default()
                    }]),
                }]),
                ..Default::default()
            }),
            ingress: Some(NetworkIngressPolicy {
                default: Some(NetworkAction::Deny),
                host_loopback: Some(NetworkAction::Deny),
            }),
            ..Default::default()
        }),
        timeout_ms: Some(30_000),
        ..ContainerRequest::new(format!(
            "curl.exe --fail --show-error --silent --max-time 20 --proxy \"\" \
             --resolve {host}:443:{ip} https://{host}/"
        ))
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
