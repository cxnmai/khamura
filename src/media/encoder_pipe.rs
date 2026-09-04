//! Wait for pipe readiness instead of imposing a delay on every partial frame.
use std::os::fd::RawFd;

pub(super) fn make_nonblocking(fd: RawFd) -> Result<(), String> {
    // SAFETY: the caller retains the live pipe; fcntl does not transfer ownership.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(format!(
            "Cannot configure recording pipe: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

pub(super) fn wait_writable(fd: RawFd) -> Result<(), String> {
    let mut descriptor = libc::pollfd {
        fd,
        events: libc::POLLOUT,
        revents: 0,
    };
    // SAFETY: descriptor points to one initialized pollfd; poll only borrows fd.
    // A bounded wait keeps stop requests and encoder-exit checks responsive.
    let ready = unsafe { libc::poll(&mut descriptor, 1, 100) };
    if ready < 0 {
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            return Err(format!("Cannot wait for recording pipe: {error}"));
        }
    }
    if descriptor.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
        return Err("Video encoder closed its input".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::{fd::AsRawFd, unix::net::UnixStream};
    #[test]
    fn recognizes_ready_and_closed_writers() {
        let (writer, reader) = UnixStream::pair().unwrap();
        make_nonblocking(writer.as_raw_fd()).unwrap();
        assert!(wait_writable(writer.as_raw_fd()).is_ok());
        drop(reader);
        assert!(wait_writable(writer.as_raw_fd()).is_err());
    }
}
