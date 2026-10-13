// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![cfg(target_os = "linux")]

#[path = "support/wxc_e2e_tests.rs"]
mod wxc_e2e_tests;
use serde_json::json;
use wxc_e2e_tests::{has_lxc_host, has_platform_exec, run_platform_config_value};

const CAP_NET_ADMIN: u64 = 1 << 12;
const CAP_NET_RAW: u64 = 1 << 13;

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

/// Ask the kernel for every socket the filter is meant to refuse, for two it is
/// meant to leave alone, and print one tagged line per answer.  ctypes calls
/// `socket(2)` and `io_uring_setup(2)` by number, so nothing between the test
/// and the kernel can substitute one route for another.  The two permitted
/// sockets are the control: they come back from the same mechanism in the same
/// run, so a refusal above them is the filter choosing, not the probe failing.
const PACKET_SOCKET_PROBE: &str = r#"python3 - <<'MXC_PROBE'
import ctypes, errno

libc = ctypes.CDLL(None, use_errno=True)
libc.syscall.restype = ctypes.c_long

def report(tag, fd):
    if fd >= 0:
        libc.close(fd)
        print(tag + " ALLOWED")
    else:
        code = ctypes.get_errno()
        print(tag + " REFUSED " + errno.errorcode.get(code, str(code)))

def sock(tag, domain, kind, protocol):
    ctypes.set_errno(0)
    report(tag, libc.socket(domain, kind, protocol))

CLOEXEC = 0o2000000

sock("PACKET_RAW", 17, 3, 0)
sock("PACKET_DGRAM", 17, 2, 0)
sock("PACKET_RAW_CLOEXEC", 17, 3 | CLOEXEC, 0)
sock("LEGACY_SOCK_PACKET", 2, 10, 0)
sock("LEGACY_SOCK_PACKET_CLOEXEC", 2, 10 | CLOEXEC, 0)
sock("XDP", 44, 3, 0)
sock("ICMP_RAW", 2, 3, 1)
sock("TCP", 2, 1, 0)

params = (ctypes.c_ubyte * 120)()
ctypes.set_errno(0)
report("IO_URING", int(libc.syscall(425, 1, ctypes.byref(params))))
MXC_PROBE
"#;

/// Pick one tagged line out of the probe's output.
fn probe_line<'a>(output: &'a str, tag: &str) -> &'a str {
    output
        .lines()
        .map(str::trim)
        .find(|line| line.split_whitespace().next() == Some(tag))
        .unwrap_or_else(|| panic!("the probe printed no {tag} line\n{output}"))
}

/// Read one decimal field out of the container's `/proc/self/status`.
fn status_number(status: &str, field: &str) -> u64 {
    status
        .lines()
        .find(|line| line.starts_with(field))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("the container reported no readable {field} line\n{status}"))
}

#[test]
fn workload_cannot_reconfigure_or_bypass_the_network() {
    if !ready() {
        return;
    }

    // The confinement only happens when chains exist.  This policy asks for a
    // firewall to bring it on.
    let config = json!({
        "version": "0.9.0-alpha",
        "containerId": "lxc-network-capability",
        "containment": "lxc",
        "process": { "commandLine": "sh -c \"cat /proc/self/status\"" },
        "lifecycle": { "destroyOnExit": true },
        "lxc": { "distribution": "alpine", "release": "3.23" },
        "network": {
            "egress": {
                "default": "deny",
                "allow": [
                    {
                        "to": [{ "cidr": "140.82.112.0/20" }],
                        "ports": [{ "protocol": "tcp", "port": 443 }]
                    }
                ]
            }
        }
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
            "{field} still carries CAP_NET_ADMIN; the workload can rewrite the firewall confining it\n{status}"
        );

        assert_ne!(
            capability_mask(&status, field) & CAP_NET_RAW,
            0,
            "{field} lost CAP_NET_RAW; an explicit `protocol: \"icmp\"` allow needs a raw socket\n{status}"
        );
    }

    // 2 is SECCOMP_MODE_FILTER.  The kernel writes this line too, and a filter
    // cannot be lifted once it is attached.  This asserts only that one is
    // attached; that it refuses AF_PACKET with EPERM is proven by the
    // lxc_bindings unit test the_filter_refuses_a_packet_socket_with_eperm.
    assert_eq!(
        status_number(&status, "Seccomp:"),
        2,
        "the workload runs with no seccomp filter; AF_PACKET reaches the interface below the chains\n{status}"
    );
}

#[test]
fn workload_cannot_open_a_packet_capable_socket_by_any_route() {
    if !ready() {
        return;
    }

    // Same firewall as above, because the confinement only happens when chains
    // exist.  Ubuntu rather than Alpine: the probe needs an interpreter that
    // can call socket(2) by number, and Alpine's image is busybox alone.
    let config = json!({
        "version": "0.9.0-alpha",
        "containerId": "lxc-packet-socket-refusal",
        "containment": "lxc",
        "process": { "commandLine": PACKET_SOCKET_PROBE, "timeout": 120000 },
        "lifecycle": { "destroyOnExit": true },
        "lxc": { "distribution": "ubuntu", "release": "noble" },
        "network": {
            "egress": {
                "default": "deny",
                "allow": [
                    {
                        "to": [{ "cidr": "140.82.112.0/20" }],
                        "ports": [{ "protocol": "tcp", "port": 443 }]
                    }
                ]
            }
        }
    });

    let result = run_platform_config_value("lxc packet socket refusal", &config, &[], None);
    let output = result.combined_output();

    assert_eq!(
        result.code,
        Some(0),
        "lxc-exec did not run the probe\n--- stderr ---\n{}",
        result.stderr
    );

    // AF_PACKET by its own family, the legacy SOCK_PACKET spelling that the
    // kernel rewrites into that family after seccomp has read the argument, and
    // AF_XDP, each with and without a socket-type flag riding along.
    for tag in [
        "PACKET_RAW",
        "PACKET_DGRAM",
        "PACKET_RAW_CLOEXEC",
        "LEGACY_SOCK_PACKET",
        "LEGACY_SOCK_PACKET_CLOEXEC",
        "XDP",
    ] {
        assert_eq!(
            probe_line(&output, tag),
            format!("{tag} REFUSED EPERM"),
            "the workload opened a packet-capable socket below the egress chains\n{output}"
        );
    }

    // io_uring creates sockets without the socket(2) syscall the filter reads,
    // so the ring itself is refused.  ENOSYS rather than EPERM because callers
    // probe for io_uring and fall back when it reports unsupported.
    assert_eq!(
        probe_line(&output, "IO_URING"),
        "IO_URING REFUSED ENOSYS",
        "io_uring is reachable, and it creates sockets the filter never sees\n{output}"
    );

    // The control.  These two come back from the same ctypes call in the same
    // container, so the refusals above are the filter and not a broken probe.
    // Raw ICMP is also the socket an explicit `protocol: "icmp"` allow needs.
    for tag in ["ICMP_RAW", "TCP"] {
        assert_eq!(
            probe_line(&output, tag),
            format!("{tag} ALLOWED"),
            "the filter took a socket it is supposed to leave alone\n{output}"
        );
    }
}
