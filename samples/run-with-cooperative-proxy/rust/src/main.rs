// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Uses cooperative proxy environment variables with a direct egress rule.

use std::error::Error;
use std::io::{self, Write};
use std::net::Ipv4Addr;

use mxc_sdk::v1::policy::{
    NetworkAction, NetworkEgressPolicy, NetworkIngressPolicy, NetworkPeerPolicy, NetworkPolicy,
    NetworkPortPolicy, NetworkProtocol, NetworkRulePolicy,
};
use mxc_sdk::v1::{self, ContainerRequest, Containment, WaitResult};

fn run() -> Result<i32, Box<dyn Error>> {
    let ip: Ipv4Addr = match std::env::var("MXC_PROXY_IP") {
        Ok(value) => value.parse()?,
        Err(std::env::VarError::NotPresent) => Ipv4Addr::LOCALHOST,
        Err(error) => return Err(error.into()),
    };
    let local = if ip.is_loopback() {
        NetworkAction::Allow
    } else {
        NetworkAction::Deny
    };
    let proxy = format!("http://{ip}:8080");
    let request = ContainerRequest {
        containment: Containment::ProcessContainer(Default::default()),
        network: Some(NetworkPolicy {
            egress: Some(NetworkEgressPolicy {
                default: Some(NetworkAction::Deny),
                allow: Some(vec![NetworkRulePolicy {
                    to: Some(vec![NetworkPeerPolicy::new(format!("{ip}/32"))]),
                    ports: Some(vec![NetworkPortPolicy {
                        protocol: Some(NetworkProtocol::Tcp),
                        port: Some(8080),
                        ..Default::default()
                    }]),
                }]),
                ..Default::default()
            }),
            ingress: Some(NetworkIngressPolicy {
                default: Some(local),
                host_loopback: Some(local),
            }),
            ..Default::default()
        }),
        environment: Some(
            ["HTTP_PROXY", "HTTPS_PROXY", "http_proxy", "https_proxy"]
                .into_iter()
                .map(|name| (name.to_string(), proxy.clone()))
                .collect(),
        ),
        inherit_default_environment: Some(true),
        timeout_ms: Some(30_000),
        ..ContainerRequest::new(
            "curl.exe --fail --show-error --silent --max-time 20 --noproxy \"\" https://www.example.com/",
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
