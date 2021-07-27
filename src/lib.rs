// SPDX-License-Identifier: Apache-2.0

//! `sev-iocuddle` provides a set of helpful abstractions used for issuing ioctls across the AMD
//! SEV platform. It is mainly used by Enarx's `sev` and `snp` crates.

#![deny(clippy::all)]
#![allow(unknown_lints)]
#![allow(clippy::identity_op)]
#![allow(clippy::unreadable_literal)]

/// Types of potential errors returned by the OS when issuing ioctls to the SEV platform.
pub mod error {
    use std::fmt::Debug;

    /// There are a number of error conditions that can occur between this
    /// layer all the way down to the SEV-SNP platform. Most of these cases have
    /// been enumerated; however, there is a possibility that some error
    /// conditions are not encapsulated here.
    #[derive(Debug)]
    pub enum Indeterminate<T: Debug> {
        /// The error condition is known.
        Known(T),

        /// The error condition is unknown.
        Unknown,
    }

    /// Error conditions returned by the SEV platform or by layers above it
    /// (i.e., the Linux kernel).
    ///
    /// These error conditions are documented in the AMD SEV API spec, but
    /// their documentation has been copied here for completeness.
    #[derive(Debug)]
    pub enum Error {
        /// Something went wrong when communicating with the "outside world"
        /// (kernel, SEV platform).
        IoError(std::io::Error),

        /// The platform state is invalid for this command.
        InvalidPlatformState,

        /// The guest state is invalid for this command.
        InvalidGuestState,

        /// The platform configuration is invalid.
        InvalidConfig,

        /// A memory buffer is too small.
        InvalidLen,

        /// The platform is already owned.
        AlreadyOwned,

        /// The certificate is invalid.
        InvalidCertificate,

        /// Request is not allowed by guest policy.
        PolicyFailure,

        /// The guest is inactive.
        Inactive,

        /// The address provided is invalid.
        InvalidAddress,

        /// The provided signature is invalid.
        BadSignature,

        /// The provided measurement is invalid.
        BadMeasurement,

        /// The ASID is already owned.
        AsidOwned,

        /// The ASID is invalid.
        InvalidAsid,

        /// WBINVD instruction required.
        WbinvdRequired,

        /// `DF_FLUSH` invocation required.
        DfFlushRequired,

        /// The guest handle is invalid.
        InvalidGuest,

        /// The command issued is invalid.
        InvalidCommand,

        /// The guest is active.
        Active,

        /// A hardware condition has occurred affecting the platform. It is safe
        /// to re-allocate parameter buffers.
        HardwarePlatform,

        /// A hardware condition has occurred affecting the platform. Re-allocating
        HardwareUnsafe,

        /// Feature is unsupported.
        Unsupported,

        /// A given parameter is invalid.
        InvalidParam,

        /// The SEV firmware has run out of a resource required to carry out the
        /// command.
        ResourceLimit,

        /// The SEV platform observed a failed integrity check.
        SecureDataInvalid,
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let err_description = match self {
                Error::IoError(_) => "I/O Error",
                Error::InvalidPlatformState => "Invalid platform state",
                Error::InvalidGuestState => "Invalid guest state",
                Error::InvalidConfig => "Platform configuration invalid",
                Error::InvalidLen => "Memory buffer too small",
                Error::AlreadyOwned => "Platform is already owned",
                Error::InvalidCertificate => "Invalid certificate",
                Error::PolicyFailure => "Policy failure",
                Error::Inactive => "Guest is inactive",
                Error::InvalidAddress => "Provided address is invalid",
                Error::BadSignature => "Provided signature is invalid",
                Error::BadMeasurement => "Provided measurement is invalid",
                Error::AsidOwned => "ASID is already owned",
                Error::InvalidAsid => "ASID is invalid",
                Error::WbinvdRequired => "WBINVD instruction required",
                Error::DfFlushRequired => "DF_FLUSH invocation required",
                Error::InvalidGuest => "Guest handle is invalid",
                Error::InvalidCommand => "Issued command is invalid",
                Error::Active => "Guest is active",
                Error::HardwarePlatform => {
                    "Hardware condition occured, safe to re-allocate parameter buffers"
                }
                Error::HardwareUnsafe => {
                    "Hardware condition occured, unsafe to re-allocate parameter buffers"
                }
                Error::Unsupported => "Feature is unsupported",
                Error::InvalidParam => "Given parameter is invalid",
                Error::ResourceLimit => {
                    "SEV firmware has run out of required resources to carry out command"
                }
                Error::SecureDataInvalid => "SEV platform observed a failed integrity check",
            };
            write!(f, "{}", err_description)
        }
    }

    impl From<std::io::Error> for Error {
        #[inline]
        fn from(error: std::io::Error) -> Error {
            Error::IoError(error)
        }
    }

    impl From<std::io::Error> for Indeterminate<Error> {
        #[inline]
        fn from(error: std::io::Error) -> Indeterminate<Error> {
            Indeterminate::Known(error.into())
        }
    }

    impl From<u32> for Indeterminate<Error> {
        #[inline]
        fn from(error: u32) -> Indeterminate<Error> {
            Indeterminate::Known(match error {
                0 => std::io::Error::last_os_error().into(),
                1 => Error::InvalidPlatformState,
                2 => Error::InvalidGuestState,
                3 => Error::InvalidConfig,
                4 => Error::InvalidLen,
                5 => Error::AlreadyOwned,
                6 => Error::InvalidCertificate,
                7 => Error::PolicyFailure,
                8 => Error::Inactive,
                9 => Error::InvalidAddress,
                10 => Error::BadSignature,
                11 => Error::BadMeasurement,
                12 => Error::AsidOwned,
                13 => Error::InvalidAsid,
                14 => Error::WbinvdRequired,
                15 => Error::DfFlushRequired,
                16 => Error::InvalidGuest,
                17 => Error::InvalidCommand,
                18 => Error::Active,
                19 => Error::HardwarePlatform,
                20 => Error::HardwareUnsafe,
                21 => Error::Unsupported,
                22 => Error::InvalidParam,
                23 => Error::ResourceLimit,
                24 => Error::SecureDataInvalid,
                _ => return Indeterminate::Unknown,
            })
        }
    }

    impl std::fmt::Display for Indeterminate<Error> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let err = match self {
                Indeterminate::Known(e) => format!("Known error: {}", e),
                Indeterminate::Unknown => "Unknown error".to_string(),
            };

            write!(f, "{}", err)
        }
    }
}

/// Helpful abstractions for issuing ioctls to the SEV platform.
pub mod sev {
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
        _phantom: PhantomData<&'a T>,
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
                _phantom: PhantomData,
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
                _phantom: PhantomData,
            }
        }
    }

    /// Information about the SEV-SNP platform version.
    #[repr(C)]
    #[derive(
        Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
    )]
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
}
