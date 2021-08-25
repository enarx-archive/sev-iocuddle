// SPDX-License-Identifier: Apache-2.0

/// Helpful abstractions for issuing ioctls to the SEV platform.
use crate::error::{Error, Indeterminate};

use iocuddle::*;
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

/// The SEV iocuddle group.
pub const SEV: Group = Group::new(b'S');

/// An ID to be associated with an SEV ioctl.
pub trait Id {
    /// The value of the ID (defined in the linux kernel).
    const ID: u32;
}

/// The Rust-flavored, FFI-friendly version of `struct sev_issue_cmd` which is
/// used to pass arguments to the SEV ioctl implementation.
///
/// This struct is defined in the Linux kernel: include/uapi/linux/psp-sev.h
#[repr(C, packed)]
pub struct Command<'a, T: Id> {
    code: u32,
    data: u64,
    error: u32,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: Id> Command<'a, T> {
    /// Create an SEV-SNP command with the expectation that the host platform/kernel will write to
    /// the caller's address space either to the data held in the `Command.subcmd` field or some
    /// other region specified by the `Command.subcmd` field.
    pub fn from_mut(subcmd: &'a mut T) -> Self {
        Command {
            code: T::ID,
            data: subcmd as *mut T as u64,
            error: 0,
            phantom: PhantomData,
        }
    }

    /// Create an SEV-SNP command with the expectation that the host platform/kernel *WILL NOT* mutate
    /// the caller's address space in its response. Note: this does not actually prevent the host
    /// platform/kernel from writing to the caller's address space if it wants to. This is primarily
    /// a semantic tool for programming against the SEV-SNP ioctl API.
    pub fn from(subcmd: &'a T) -> Self {
        Command {
            code: T::ID,
            data: subcmd as *const T as u64,
            error: 0,
            phantom: PhantomData,
        }
    }

    /// Rather than relying on status codes from the Linux kernel, match the specific error code
    /// returned by the SNP firmware to output errors in more detail.
    pub fn encapsulate(&self, err: std::io::Error) -> Indeterminate<Error> {
        match self.error {
            0 => Indeterminate::<Error>::from(err),
            _ => Indeterminate::<Error>::from(self.error as u32),
        }
    }
}

/// Information about the SEV-SNP platform version.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    /// The major version number.
    pub major: u8,

    /// The minor version number.
    pub minor: u8,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
