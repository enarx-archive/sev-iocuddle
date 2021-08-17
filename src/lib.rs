// SPDX-License-Identifier: Apache-2.0

//! `sev-iocuddle` provides a set of helpful abstractions used for issuing ioctls across the AMD
//! SEV platform. It is mainly used by Enarx's `sev` and `snp` crates.

#![deny(clippy::all)]
#![allow(unknown_lints)]
#![allow(clippy::identity_op)]
#![allow(clippy::unreadable_literal)]

pub mod error;
pub mod kvm;
pub mod sev;
pub mod util;
