// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// An example of a unimodular matrix.
///
/// Its determinant is +1.
///
/// These are nice for unit tests because their inverse
/// can be exactly represented with floats.
/// You can also scale this by a power of 2 or a factor of -1
/// to create a test where the determinant is not equal to 1.
pub(crate) const EXAMPLE_UNIMODULAR: [[f64; 3]; 3] = [
    [ 2.0, -1.0,  2.0],
    [-1.0,  3.0, -3.0],
    [ 1.0,  1.0,  0.0],
];

/// The (exact) inverse of EXAMPLE_UNIMODULAR.
pub(crate) const EXAMPLE_UNIMODULAR_INV: [[f64; 3]; 3] = [
    [ 3.0,  2.0, -3.0],
    [-3.0, -2.0,  4.0],
    [-4.0, -3.0,  5.0],
];
