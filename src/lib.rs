// SPDX-License-Identifier: Apache-2.0

//! `sev-iocuddle` provides a set of helpful abstractions used for issuing ioctls across the AMD
//! SEV platform. It is mainly used by Enarx's `sev` and `snp` crates.

#![deny(clippy::all)]
#![deny(missing_docs)]
#![allow(unknown_lints)]
#![allow(clippy::identity_op)]
#![allow(clippy::unreadable_literal)]

/// Error module: Errors that can be returned by the OS when issuing ioctls for the SEV/KVM
/// platform.
pub mod error;

/// KVM module: Abstractions/tools for issuing ioctls for the KVM platform.
pub mod kvm;

/// SEV module: Abstractions/tools for issuing ioctls for the SEV platform.
pub mod sev;

/// Utility module: Helpful primitives for developing the crate.
pub mod util;
