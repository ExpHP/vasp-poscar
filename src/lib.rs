// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc(html_root_url = "https://docs.rs/vasp-poscar/0.2.0")]

//! Library for reading and writing [VASP POSCAR] files.
//!
//! See the [`Poscar`] type for more details.
//!
//! ```rust
//! # #[derive(Debug)] enum Never {}
//! #
//! # impl<T: std::fmt::Display> From<T> for Never {
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
//! let mut poscar = poscar.into_raw();
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

#[macro_use]
mod util;
mod parse;
mod types;
mod write;
mod math;
pub mod builder;

pub use crate::types::{Coords, ScaleLine, RawPoscar, Poscar};
pub use crate::types::ValidationError;
pub use crate::builder::{Builder, Zeroed};

/// Types convertable into `Vec<[X; 3]>`.
///
/// Appears in generic bounds for some of the crate's public API methods
/// (such as [`Builder`]).
///
/// # Example implementors:
/// <!-- @@To3 -->
///
/// * `Vec<[X; 3]>`
/// * `&[[X; 3]]` (where `X: Clone`)
/// * Any iterable of `(X, X, X)`
///
/// [`Builder`]: builder/struct.Builder.html
pub trait ToN3<X>: IntoIterator {
    #[doc(hidden)]
    fn _to_enn_3(self) -> Vec<[X; 3]>;
}

impl<X, V, Vs> ToN3<X> for Vs
where
    Vs: IntoIterator<Item=V>,
    V: To3<X>,
{
    fn _to_enn_3(self) -> Vec<[X; 3]>
    { self.into_iter().map(To3::_to_array_3).collect() }
}

/// Types convertible into `[X; 3]`.
///
/// Appears in generic bounds for some of the crate's public API methods
/// (such as [`Builder`]).
///
/// [`Builder`]: builder/struct.Builder.html
pub trait To3<X> {
    #[doc(hidden)]
    fn _to_array_3(self) -> [X; 3];
}

// NOTE: When adding new impls, search for the string @@To3@@
//       and update those docs accordingly.

impl<X> To3<X> for [X; 3] {
    fn _to_array_3(self) -> [X; 3] { self }
}

impl<'a, X> To3<X> for &'a [X; 3] where X: Clone {
    fn _to_array_3(self) -> [X; 3] { self.clone() }
}

impl<X> To3<X> for (X, X, X) {
    fn _to_array_3(self) -> [X; 3] { [self.0, self.1, self.2] }
}
