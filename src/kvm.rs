// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error, Indeterminate};
use crate::sev::Id;

use iocuddle::*;

use std::marker::PhantomData;
use std::os::raw::c_ulong;
use std::os::unix::io::AsRawFd;

/// The KVM iocuddle group.
pub const KVM: Group = Group::new(0xAE);
pub const ENC_OP: Ioctl<WriteRead, &c_ulong> = unsafe { KVM.write_read(0xBA) };

// These two ioctls are specified as read, although they write.
// To compensate for the reference `Group::read` returns, specify
// the write with a reference, too.

/// Corresponds to the `KVM_MEMORY_ENCRYPT_REG_REGION` ioctl
pub const ENC_REG_REGION: Ioctl<Write, &KvmEncRegion> =
    unsafe { KVM.read::<KvmEncRegion>(0xBB).lie() };

/// Corresponds to the `KVM_MEMORY_ENCRYPT_UNREG_REGION` ioctl
pub const ENC_UNREG_REGION: Ioctl<Write, &KvmEncRegion> =
    unsafe { KVM.read::<KvmEncRegion>(0xBC).lie() };

/// The Rust-flavored, FFI-friendly version of `struct sev_issue_cmd` which is
/// used to pass arguments to the SEV ioctl implementation.
///
/// This struct is defined in the Linux kernel: include/uapi/linux/psp-sev.h
#[repr(C)]
pub struct Command<'a, T: Id> {
    code: u32,
    data: u64,
    error: u32,
    sev_fd: u32,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: Id> Command<'a, T> {
    /// Create an SEV-SNP command with the expectation that the host platform/kernel will write to
    /// the caller's address space either to the data held in the `Command.subcmd` field or some
    /// other region specified by the `Command.subcmd` field.
    pub fn from_mut(sev: &'a mut impl AsRawFd, subcmd: &'a mut T) -> Self {
        Command {
            code: T::ID,
            data: subcmd as *mut T as u64,
            error: 0,
            sev_fd: sev.as_raw_fd() as _,
            _phantom: PhantomData,
        }
    }

    /// Create an SEV-SNP command with the expectation that the host platform/kernel *WILL NOT* mutate
    /// the caller's address space in its response. Note: this does not actually prevent the host
    /// platform/kernel from writing to the caller's address space if it wants to. This is primarily
    /// a semantic tool for programming against the SEV-SNP ioctl API.
    pub fn from(sev: &'a mut impl AsRawFd, subcmd: &'a T) -> Self {
        Command {
            code: T::ID,
            data: subcmd as *const T as u64,
            error: 0,
            sev_fd: sev.as_raw_fd() as _,
            _phantom: PhantomData,
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

/// Corresponds to the kernel struct `kvm_enc_region`
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct KvmEncRegion<'a> {
    addr: u64,
    size: u64,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> KvmEncRegion<'a> {
    /// Create a new `KvmEncRegion` referencing some memory assigned to the virtual machine.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            addr: data.as_ptr() as _,
            size: data.len() as _,
            phantom: PhantomData,
        }
    }
}
