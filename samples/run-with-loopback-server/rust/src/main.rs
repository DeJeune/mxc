// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Exposes a one-request HTTP listener through host loopback.

use std::error::Error;
use std::io;
use std::path::Path;
use std::thread;

use mxc_sdk::v1::policy::{
    FilesystemPolicy, NetworkAction, NetworkEgressPolicy, NetworkIngressPolicy, NetworkPolicy,
};
use mxc_sdk::v1::{self, ContainerRequest, Containment, WaitResult};

fn run() -> Result<i32, Box<dyn Error>> {
    let sample_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("missing sample directory"))?;
    let script = sample_dir.join("server.ps1");
    let request = ContainerRequest {
        containment: Containment::ProcessContainer(Default::default()),
        filesystem: Some(FilesystemPolicy {
            readonly_paths: vec![sample_dir.to_string_lossy().into_owned()],
            ..Default::default()
        }),
        network: Some(NetworkPolicy {
            egress: Some(NetworkEgressPolicy {
                default: Some(NetworkAction::Deny),
                ..Default::default()
            }),
            ingress: Some(NetworkIngressPolicy {
                default: Some(NetworkAction::Deny),
                host_loopback: Some(NetworkAction::Allow),
            }),
            ..Default::default()
        }),
        timeout_ms: Some(60_000),
        ..ContainerRequest::new(format!(
            "powershell.exe -NoLogo -NoProfile -NonInteractive -File \"{}\"",
            script.display()
        ))
    };
    let mut process = v1::spawn(request, Default::default())?;
    let mut stdout = process
        .take_stdout()
        .ok_or_else(|| io::Error::other("missing container stdout"))?;
    let mut stderr = process
        .take_stderr()
        .ok_or_else(|| io::Error::other("missing container stderr"))?;
    let outcome = thread::scope(|scope| -> Result<_, Box<dyn Error>> {
        let output = scope.spawn(|| io::copy(&mut stdout, &mut io::stdout()));
        let errors = scope.spawn(|| io::copy(&mut stderr, &mut io::stderr()));
        let outcome = process.wait()?;
        output
            .join()
            .map_err(|_| io::Error::other("stdout reader panicked"))??;
        errors
            .join()
            .map_err(|_| io::Error::other("stderr reader panicked"))??;
        Ok(outcome)
    })?;

    for warning in process.warnings() {
        eprintln!("warning: {warning}");
    }
    Ok(match outcome {
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
