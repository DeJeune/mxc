// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Runs an HTTPS client through an existing ProcessContainer runtime proxy.

use std::error::Error;
use std::io::{self, Write};

use mxc_sdk::v1::configs::{ProcessContainerConfig, ProcessContainerNetwork};
use mxc_sdk::v1::policy::{
    NetworkAction, NetworkEgressPolicy, NetworkIngressPolicy, NetworkPolicy, NetworkRuntimeConfig,
};
use mxc_sdk::v1::{self, ContainerRequest, Containment, WaitResult};

fn run() -> Result<i32, Box<dyn Error>> {
    let peer = match std::env::var("MXC_PROXY_PEER") {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(error) => return Err(error.into()),
    };
    let host_loopback = if peer.is_some() {
        NetworkAction::Deny
    } else {
        NetworkAction::Allow
    };
    let mut network = ProcessContainerNetwork::default();
    network.allowed_proxy_peer = peer;
    let mut config = ProcessContainerConfig::default();
    config.network = Some(network);
    let request = ContainerRequest {
        containment: Containment::ProcessContainer(config),
        network: Some(NetworkPolicy {
            egress: Some(NetworkEgressPolicy {
                default: Some(NetworkAction::Deny),
                ..Default::default()
            }),
            ingress: Some(NetworkIngressPolicy {
                default: Some(NetworkAction::Allow),
                host_loopback: Some(host_loopback),
            }),
            runtime_config: Some(NetworkRuntimeConfig {
                network_proxy: Some("http://127.0.0.1:8080".into()),
            }),
        }),
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
