mod tcp;

use std::io::{Read, Write};
use std::os::fd::AsRawFd;

/// Trait object responsible for handling reported I/O events.
/// The stream provided here must be in Non Blocking mode for this
/// Library to work properly
/// As such, when the operation is not ready to be completed an Error with
/// ErrorKind::WouldBlock should be returned
pub trait Stream : Read + Write + AsRawFd + Send { }