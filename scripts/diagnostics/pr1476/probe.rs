// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Identical diagnostic fixture injected into both pinned revisions by run.py.
#![cfg(target_os = "macos")]

use std::fs;
use std::io::{Read, Write};
use std::process::{Child, Command, ExitStatus};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use mxc_sdk::mxc_pty::{LivePty, PtySize};
use mxc_sdk::v1::{self, ContainerRequest, MxcPtyProcess, WaitResult};

const BULK: usize = 1_048_576;
const MARKER: &[u8] = b"PR1476_DONE";
const LIMIT: Duration = Duration::from_secs(8);
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn command_text(size: usize) -> String {
    format!("head -c {size} /dev/zero; printf PR1476_DONE; exit 23")
}

fn register(pid: u32) -> TestResult {
    fs::write(std::env::var("MXC_PR1476_PID_FILE")?, pid.to_string())?;
    Ok(())
}

fn reaped() -> TestResult {
    fs::remove_file(std::env::var("MXC_PR1476_PID_FILE")?)?;
    Ok(())
}

fn collect(mut reader: Box<dyn Read + Send>) -> mpsc::Receiver<std::io::Result<Vec<u8>>> {
    let (send, receive) = mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = reader.read_to_end(&mut bytes).map(|_| bytes);
        let _ = send.send(result);
    });
    receive
}

fn check_bytes(bytes: &[u8], size: usize) {
    assert_eq!(bytes.len(), size + MARKER.len(), "output length changed");
    assert!(bytes[..size].iter().all(|byte| *byte == 0));
    assert_eq!(&bytes[size..], MARKER);
    println!(
        "PR1476_OBSERVATION bytes={} exact=true exit=23",
        bytes.len()
    );
}

fn poll_child(child: &mut Child, duration: Duration) -> std::io::Result<Option<ExitStatus>> {
    let deadline = Instant::now() + duration;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn direct(size: usize, delayed: bool) -> TestResult {
    let mut command = Command::new("/bin/sh");
    command.arg("-c").arg(command_text(size));
    let terminal = LivePty::attach(&mut command, PtySize::default(), &[])?;
    let mut child = command.spawn()?;
    drop(command);
    register(child.id())?;
    let reader = terminal.try_clone_reader()?;
    if delayed {
        let before = poll_child(&mut child, Duration::from_secs(1))?;
        println!("PR1476_OBSERVATION exit_before_read={before:?}");
    }
    let output = collect(reader);
    let status = poll_child(&mut child, LIMIT)?.ok_or("child did not exit after draining")?;
    reaped()?;
    let bytes = output.recv_timeout(LIMIT)??;
    assert_eq!(status.code(), Some(23));
    check_bytes(&bytes, size);
    Ok(())
}

#[test]
fn direct_small_delayed_reader() -> TestResult {
    direct(10, true)
}

#[test]
fn direct_bulk_delayed_reader() -> TestResult {
    direct(BULK, true)
}

#[test]
fn direct_bulk_concurrent_reader() -> TestResult {
    direct(BULK, false)
}

fn native(delayed: bool) -> TestResult {
    let mut command = Command::new("/bin/sh");
    command.arg("-c").arg(command_text(BULK));
    let terminal = LivePty::attach(&mut command, PtySize::default(), &[])?;
    let mut child = command.spawn()?;
    drop(command);
    register(child.id())?;
    let stdio = terminal.take_native_stdio()?;
    drop(stdio.stdin);
    if delayed {
        let before = poll_child(&mut child, Duration::from_secs(1))?;
        println!("PR1476_OBSERVATION exit_before_read={before:?}");
    }
    let output = collect(Box::new(fs::File::from(stdio.stdout)));
    let status =
        poll_child(&mut child, LIMIT)?.ok_or("native child did not exit after draining")?;
    reaped()?;
    terminal.finish_native_bridge();
    let bytes = output.recv_timeout(LIMIT)??;
    assert_eq!(status.code(), Some(23));
    check_bytes(&bytes, BULK);
    Ok(())
}

#[test]
fn native_bulk_delayed_reader() -> TestResult {
    native(true)
}

#[test]
fn native_bulk_concurrent_reader() -> TestResult {
    native(false)
}

#[test]
fn native_closed_consumer() -> TestResult {
    let mut command = Command::new("/bin/sh");
    command
        .arg("-c")
        .arg("read go; head -c 1048576 /dev/zero; exit 23");
    let terminal = LivePty::attach(&mut command, PtySize::default(), &[])?;
    let mut child = command.spawn()?;
    drop(command);
    register(child.id())?;
    let stdio = terminal.take_native_stdio()?;
    drop(stdio.stdout);
    let mut input = fs::File::from(stdio.stdin);
    input.write_all(b"go\n")?;
    drop(input);
    let status = poll_child(&mut child, LIMIT)?;
    println!("PR1476_OBSERVATION exit_with_closed_consumer={status:?}");
    // Drop the primary before killing/reaping: Darwin can be stuck in exit.
    drop(terminal);
    if status.is_none() {
        child.kill()?;
        let cleanup = poll_child(&mut child, LIMIT)?;
        assert!(
            cleanup.is_some(),
            "child was not reaped after primary close"
        );
    }
    reaped()?;
    assert_eq!(status.and_then(|status| status.code()), Some(23));
    Ok(())
}

fn sdk_process(size: usize) -> Result<MxcPtyProcess, Box<dyn std::error::Error>> {
    let terminal = v1::spawn_with_pty(
        ContainerRequest::new(command_text(size)),
        Default::default(),
    )?;
    register(terminal.id())?;
    Ok(terminal)
}

#[test]
fn sdk_unclaimed_bulk_wait() -> TestResult {
    let terminal = sdk_process(BULK)?;
    assert_eq!(terminal.wait()?, WaitResult::Exited(23));
    reaped()?;
    println!("PR1476_OBSERVATION unclaimed_wait=completed exit=23 output=discarded");
    Ok(())
}

fn sdk_claimed(size: usize, delayed: bool) -> TestResult {
    let terminal = Arc::new(sdk_process(size)?);
    let reader = terminal.try_clone_reader()?;
    let (send, receive) = mpsc::channel();
    let waiting = Arc::clone(&terminal);
    let waiter = std::thread::spawn(move || {
        let _ = send.send(waiting.wait());
    });
    let before = if delayed {
        match receive.recv_timeout(Duration::from_secs(1)) {
            Ok(result) => Some(result?),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(error) => return Err(error.into()),
        }
    } else {
        None
    };
    if delayed {
        println!("PR1476_OBSERVATION wait_before_read={before:?}");
    } else {
        println!("PR1476_OBSERVATION drain_mode=concurrent");
    }
    let output = collect(reader);
    let outcome = match before {
        Some(outcome) => outcome,
        None => receive.recv_timeout(LIMIT)??,
    };
    assert_eq!(outcome, WaitResult::Exited(23));
    reaped()?;
    waiter.join().map_err(|_| "waiter panicked")?;
    let bytes = output.recv_timeout(LIMIT)??;
    check_bytes(&bytes, size);
    Ok(())
}

#[test]
fn sdk_claimed_small_delayed_read() -> TestResult {
    sdk_claimed(10, true)
}

#[test]
fn sdk_claimed_bulk_concurrent_read() -> TestResult {
    sdk_claimed(BULK, false)
}
