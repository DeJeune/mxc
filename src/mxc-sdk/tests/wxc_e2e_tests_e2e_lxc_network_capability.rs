// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![cfg(target_os = "linux")]

#[path = "support/wxc_e2e_tests.rs"]
mod wxc_e2e_tests;
use serde_json::json;
use wxc_e2e_tests::{has_lxc_host, has_platform_exec, run_platform_config_value};

const CAP_NET_ADMIN: u64 = 1 << 12;

/// Whether the LXC capability prerequisites are present.
fn ready() -> bool {
    has_platform_exec() && has_lxc_host()
}

/// Read one capability mask out of the container's `/proc/self/status`.
fn capability_mask(status: &str, field: &str) -> u64 {
    status
        .lines()
        .find(|line| line.starts_with(field))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|mask| u64::from_str_radix(mask, 16).ok())
        .unwrap_or_else(|| panic!("the container reported no readable {field} line\n{status}"))
}

#[test]
fn workload_cannot_reconfigure_the_network() {
    if !ready() {
        return;
    }

    let config = json!({
        "version": "0.9.0-alpha",
        "containerId": "lxc-network-capability",
        "containment": "lxc",
        "process": { "commandLine": "sh -c \"cat /proc/self/status\"" },
        "lifecycle": { "destroyOnExit": true },
        "lxc": { "distribution": "alpine", "release": "3.23" }
    });

    let result = run_platform_config_value("lxc network capability", &config, &[], None);
    let status = result.combined_output();

    assert_eq!(
        result.code,
        Some(0),
        "lxc-exec did not run the container\n--- stderr ---\n{}",
        result.stderr
    );

    // The kernel writes these masks; the workload cannot forge them.  Effective
    // alone would not settle it: a process raises a permitted capability into
    // its effective set whenever it likes, and only a bounding-set drop
    // survives execve.
    for field in ["CapEff:", "CapPrm:", "CapBnd:"] {
        assert_eq!(
            capability_mask(&status, field) & CAP_NET_ADMIN,
            0,
            "{field} still carries CAP_NET_ADMIN; the workload could reconfigure the network it asked to go without\n{status}"
        );
    }
}

#[test]
fn workload_can_still_ping() {
    if !ready() {
        return;
    }

    let config = json!({
        "version": "0.9.0-alpha",
        "containerId": "lxc-ping-allowed",
        "containment": "lxc",
        "process": { "commandLine": "ping -c 1 127.0.0.1" },
        "lifecycle": { "destroyOnExit": true },
        "lxc": { "distribution": "ubuntu", "release": "noble" }
    });

    let result = run_platform_config_value("lxc ping allowed", &config, &[], None);

    assert_eq!(
        result.code,
        Some(0),
        "the workload could not ping after the capability drop\n{}",
        result.combined_output()
    );
}

#[test]
fn workload_cannot_open_a_raw_or_packet_socket() {
    if !ready() {
        return;
    }

    // Bash does not have the ability to make RAW and PACKET sockets.  Use perl instead.
    const RAW_AND_PACKET_PROBE: &str = r#"perl -e 'use Socket; my $raw = socket(RAW, PF_INET, SOCK_RAW, 1) ? "RAW_OPENED" : "RAW_REFUSED: $!"; my $pkt = socket(PKT, 17, SOCK_RAW, 0) ? "PACKET_OPENED" : "PACKET_REFUSED: $!"; print "$raw\n$pkt\n";'"#;

    // Use Ubuntu for pearl.
    let config = json!({
        "version": "0.9.0-alpha",
        "containerId": "lxc-raw-socket-refusal",
        "containment": "lxc",
        "process": { "commandLine": RAW_AND_PACKET_PROBE },
        "lifecycle": { "destroyOnExit": true },
        "lxc": { "distribution": "ubuntu", "release": "noble" }
    });

    let result = run_platform_config_value("lxc raw socket refusal", &config, &[], None);
    let output = result.combined_output();

    assert_eq!(
        result.code,
        Some(0),
        "lxc-exec did not run the probe\n--- stderr ---\n{}",
        result.stderr
    );

    assert!(
        output.contains("RAW_REFUSED"),
        "the workload opened a raw socket the capability drop must refuse\n{output}"
    );

    assert!(
        output.contains("PACKET_REFUSED"),
        "the workload opened a packet socket the capability drop must refuse\n{output}"
    );
}
