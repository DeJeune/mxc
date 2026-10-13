//! Close-on-exec hygiene for descriptors inherited across `fork`.
//!
//! macOS has no atomic close-on-exec pipe: there is no `pipe2`, and
//! `socketpair` has no `SOCK_CLOEXEC`. Both ends are therefore inheritable
//! between `pipe(2)` and the `fcntl` that marks them, so a spawn running on
//! another thread during that window captures them. The child then holds a
//! live handle to another container's stdin or stdout.
//!
//! Only path-based `open(2)` is atomic here, which pipes cannot use. The
//! window is closed from the other side instead: between `fork` and `exec`,
//! drop every descriptor that is not marked close-on-exec.

use std::os::fd::RawFd;

/// Entries collected per enumeration pass before falling back to a full scan.
const MAX_TRACKED_DESCRIPTORS: usize = 1024;

/// Close inherited descriptors that are not marked close-on-exec.
///
/// Standard streams are left alone, as is every descriptor already marked
/// `FD_CLOEXEC` — including the pipe the Rust runtime uses to report a failed
/// `exec` back to the parent.
///
/// # Safety
///
/// Must be called only between `fork` and `exec`. It closes descriptors the
/// caller does not own, and is async-signal-safe only in that context.
pub unsafe fn close_unmarked_inherited_descriptors() {
    // SAFETY: the caller guarantees the post-fork, pre-exec context.
    if unsafe { close_enumerated() } {
        return;
    }
    // SAFETY: as above.
    unsafe { close_scanned() };
}

/// Close unmarked descriptors found by enumerating `/dev/fd`.
///
/// Returns `false` if the directory could not be enumerated, or if it held
/// more entries than [`MAX_TRACKED_DESCRIPTORS`].
///
/// # Safety
///
/// Must be called only between `fork` and `exec`.
unsafe fn close_enumerated() -> bool {
    let mut found = [0 as RawFd; MAX_TRACKED_DESCRIPTORS];
    let Some(count) = collect_unmarked(&mut found) else {
        return false;
    };
    for fd in &found[..count] {
        // SAFETY: `fd` was reported open and is not a standard stream.
        unsafe { libc::close(*fd) };
    }
    true
}

/// Record the open descriptors that lack `FD_CLOEXEC` into `found`, returning
/// how many were stored.
///
/// Standard streams are skipped, as is the handle used for the walk. Returns
/// `None` if `/dev/fd` could not be read or `found` was too small, which tells
/// the caller to fall back to a full scan.
///
/// The directory is read to completion before anything is closed: removing
/// entries mid-walk would shift the directory offset and skip descriptors.
fn collect_unmarked(found: &mut [RawFd]) -> Option<usize> {
    // `getdirentries64`. The libc wrapper is unavailable once 64-bit inodes
    // are in effect, so the syscall is issued directly.
    const SYS_GETDIRENTRIES64: libc::c_int = 344;

    // SAFETY: a NUL-terminated literal path with no output parameters.
    let dir = unsafe { libc::open(c"/dev/fd".as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC) };
    if dir < 0 {
        return None;
    }

    let mut count = 0usize;
    let mut buffer = [0u8; 4096];
    let mut offset = 0i64;
    let mut complete = true;

    'walk: loop {
        // SAFETY: `buffer` and `offset` are valid writable locals sized as
        // passed, and `dir` is an open directory descriptor.
        let read = unsafe {
            libc::syscall(
                SYS_GETDIRENTRIES64,
                dir,
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut offset,
            )
        };
        if read < 0 {
            complete = false;
            break;
        }
        if read == 0 {
            break;
        }

        let mut cursor = 0usize;
        let limit = read as usize;
        while cursor + DIRENT_NAME_OFFSET < limit {
            let entry = &buffer[cursor..];
            let length = dirent_record_length(entry);
            if length == 0 || cursor + length > limit {
                complete = false;
                break 'walk;
            }
            if let Some(fd) = dirent_descriptor(entry, length) {
                // Skip the standard streams and the enumeration handle.
                if fd > libc::STDERR_FILENO && fd != dir && is_unmarked(fd) {
                    if count == found.len() {
                        complete = false;
                        break 'walk;
                    }
                    found[count] = fd;
                    count += 1;
                }
            }
            cursor += length;
        }
    }

    // SAFETY: `dir` is open and owned by this function.
    unsafe { libc::close(dir) };
    complete.then_some(count)
}

/// Close unmarked descriptors by scanning the whole descriptor table.
///
/// Used when `/dev/fd` is unavailable. Correct but slow, since the table is
/// sized by `RLIMIT_NOFILE` rather than by the number of open descriptors.
///
/// # Safety
///
/// Must be called only between `fork` and `exec`.
unsafe fn close_scanned() {
    // SAFETY: no arguments and no output parameters.
    let limit = unsafe { libc::getdtablesize() };
    for fd in (libc::STDERR_FILENO + 1)..limit {
        if is_unmarked(fd) {
            // SAFETY: `fd` is open, since `F_GETFD` succeeded for it.
            unsafe { libc::close(fd) };
        }
    }
}

/// Whether `fd` is open and lacks `FD_CLOEXEC`.
fn is_unmarked(fd: RawFd) -> bool {
    // SAFETY: `F_GETFD` only reads descriptor flags and reports `EBADF` for
    // descriptors that are not open.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    flags >= 0 && flags & libc::FD_CLOEXEC == 0
}

/// Offset of `d_name` within `struct dirent` on macOS: `d_ino` and
/// `d_seekoff` (8 bytes each), then `d_reclen` and `d_namlen` (2 each), then
/// `d_type`.
const DIRENT_NAME_OFFSET: usize = 21;
/// Offset of the `d_reclen` field.
const DIRENT_RECORD_LENGTH_OFFSET: usize = 16;
/// Offset of the `d_namlen` field.
const DIRENT_NAME_LENGTH_OFFSET: usize = 18;

/// Read `d_reclen` from a raw directory entry.
fn dirent_record_length(entry: &[u8]) -> usize {
    let Some(field) = entry.get(DIRENT_RECORD_LENGTH_OFFSET..DIRENT_RECORD_LENGTH_OFFSET + 2)
    else {
        return 0;
    };
    u16::from_ne_bytes([field[0], field[1]]) as usize
}

/// Parse a `/dev/fd` entry name as a descriptor number.
fn dirent_descriptor(entry: &[u8], record_length: usize) -> Option<RawFd> {
    let field = entry.get(DIRENT_NAME_LENGTH_OFFSET..DIRENT_NAME_LENGTH_OFFSET + 2)?;
    let name_length = u16::from_ne_bytes([field[0], field[1]]) as usize;
    if name_length == 0 || DIRENT_NAME_OFFSET + name_length > record_length {
        return None;
    }
    let name = entry.get(DIRENT_NAME_OFFSET..DIRENT_NAME_OFFSET + name_length)?;

    let mut value: RawFd = 0;
    for byte in name {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value
            .checked_mul(10)?
            .checked_add(RawFd::from(byte - b'0'))?;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::AsRawFd;

    /// Build a `dirent` image the way the kernel lays one out.
    fn dirent(name: &str) -> Vec<u8> {
        let mut entry = vec![0u8; DIRENT_NAME_OFFSET];
        entry.extend_from_slice(name.as_bytes());
        entry.push(0);
        let record_length = entry.len();
        entry[DIRENT_RECORD_LENGTH_OFFSET..DIRENT_RECORD_LENGTH_OFFSET + 2]
            .copy_from_slice(&(record_length as u16).to_ne_bytes());
        entry[DIRENT_NAME_LENGTH_OFFSET..DIRENT_NAME_LENGTH_OFFSET + 2]
            .copy_from_slice(&(name.len() as u16).to_ne_bytes());
        entry
    }

    #[test]
    fn dirent_descriptor_parses_a_numeric_name() {
        let entry = dirent("42");
        let length = dirent_record_length(&entry);
        assert_eq!(length, entry.len());
        assert_eq!(dirent_descriptor(&entry, length), Some(42));
    }

    #[test]
    fn dirent_descriptor_rejects_a_non_numeric_name() {
        let entry = dirent("stdin");
        let length = dirent_record_length(&entry);
        assert_eq!(dirent_descriptor(&entry, length), None);
    }

    #[test]
    fn dirent_descriptor_rejects_a_truncated_record() {
        let entry = dirent("7");
        // A record that claims to end before its own name.
        assert_eq!(dirent_descriptor(&entry, DIRENT_NAME_OFFSET), None);
    }

    #[test]
    fn is_unmarked_distinguishes_close_on_exec() {
        let marked = std::fs::File::open("/dev/null").expect("open /dev/null");
        assert!(!is_unmarked(marked.as_raw_fd()), "Rust opens files CLOEXEC");

        // SAFETY: `F_SETFD` only writes descriptor flags on an open fd.
        unsafe { libc::fcntl(marked.as_raw_fd(), libc::F_SETFD, 0) };
        assert!(is_unmarked(marked.as_raw_fd()));
    }

    #[test]
    fn is_unmarked_rejects_a_closed_descriptor() {
        let fd = {
            let file = std::fs::File::open("/dev/null").expect("open /dev/null");
            file.as_raw_fd()
        };
        assert!(!is_unmarked(fd));
    }

    #[test]
    fn collect_unmarked_finds_an_unmarked_descriptor_and_spares_the_rest() {
        let exposed = std::fs::File::open("/dev/null").expect("open /dev/null");
        let marked = std::fs::File::open("/dev/null").expect("open /dev/null");
        // SAFETY: `F_SETFD` only writes descriptor flags on an open fd.
        unsafe { libc::fcntl(exposed.as_raw_fd(), libc::F_SETFD, 0) };

        let mut found = [0 as RawFd; MAX_TRACKED_DESCRIPTORS];
        let count = collect_unmarked(&mut found).expect("/dev/fd walk succeeds");
        let found = &found[..count];

        assert!(
            found.contains(&exposed.as_raw_fd()),
            "walk missed fd {}: {found:?}",
            exposed.as_raw_fd()
        );
        assert!(!found.contains(&marked.as_raw_fd()));
        assert!(!found.iter().any(|fd| *fd <= libc::STDERR_FILENO));
    }

    #[test]
    fn collect_unmarked_reports_an_undersized_buffer() {
        let exposed = std::fs::File::open("/dev/null").expect("open /dev/null");
        // SAFETY: `F_SETFD` only writes descriptor flags on an open fd.
        unsafe { libc::fcntl(exposed.as_raw_fd(), libc::F_SETFD, 0) };

        // Too small to hold the descriptor just exposed, so the caller must be
        // told to fall back rather than silently skipping it.
        assert_eq!(collect_unmarked(&mut []), None);
    }
}
