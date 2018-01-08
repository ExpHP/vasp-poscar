// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Library for reading and writing [VASP POSCAR] files.
//!
//! See the [`Poscar`] type for more details.
//!
//! ```rust
//! # #[derive(Debug)] enum Never {}
//! #
//! # impl<T: ::std::fmt::Display> From<T> for Never {
//! #     fn from(x: T) -> Never { panic!("{}", x); }
//! # }
//! #
//! # fn _main() -> Result<(), Never> {Ok({
//!
//! use vasp_poscar::{Poscar, ScaleLine};
//!
//! const EXAMPLE: &'static str = "\
//! cubic diamond
//!   3.7
//!     0.5 0.5 0.0
//!     0.0 0.5 0.5
//!     0.5 0.0 0.5
//!    C
//!    2
//! Direct
//!   0.0 0.0 0.0
//!   0.25 0.25 0.25
//! ";
//!
//! // read from a BufRead instance, such as &[u8] or BufReader<File>
//! let poscar = Poscar::from_reader(EXAMPLE.as_bytes())?;
//!
//! // get a RawPoscar object with public fields you can freely match on and manipulate
//! let mut poscar = poscar.raw();
//! assert_eq!(poscar.scale, ScaleLine::Factor(3.7));
//!
//! poscar.comment = "[Subject Name Here] was here".into();
//! poscar.scale = ScaleLine::Volume(10.0);
//!
//! // Turn the RawPoscar back into a Poscar
//! let poscar = poscar.validate()?;
//!
//! // Poscar implements Display
//! assert_eq!(
//!     format!("{}", poscar),
//!     "\
//! [Subject Name Here] was here
//!   -10.0
//!     0.5 0.5 0.0
//!     0.0 0.5 0.5
//!     0.5 0.0 0.5
//!    C
//!    2
//! Direct
//!   0.0 0.0 0.0
//!   0.25 0.25 0.25
//! ");
//! # })}
//! # fn main() { _main().unwrap() }
//! #
//! ```
//!
//! [VASP POSCAR]: http://cms.mpi.univie.ac.at/vasp/guide/node59.html
//! [`Poscar`]: struct.Poscar.html

#[macro_use]
pub extern crate failure;
extern crate dtoa;

// general bail! and ensure! macros that don't constrain the type to failure::Error
macro_rules! g_bail { ($e:expr $(,)*) => { return Err($e.into()); }; }
macro_rules! g_ensure { ($cond:expr, $e:expr $(,)*) => { if !$cond { g_bail!($e); } }; }

mod parse;
mod types;
mod write;

pub use types::{Coords, ScaleLine, RawPoscar, Poscar};
pub use types::ValidationError;
