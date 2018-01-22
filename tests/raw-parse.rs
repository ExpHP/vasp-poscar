// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Smoke test of poscar parsing.
//!
//! This sets a pretty low bar, and only the most
//! phenomenally broken implementation can fail here.

#![deny(unused)]

#[macro_use]
extern crate indoc;
extern crate vasp_poscar;
use ::vasp_poscar::{Poscar, ScaleLine, Coords};

macro_rules! poscar {
    ($s:expr) => {{
        let doc: &[u8] = indoc!($s); // ensure arrayness is coerced away
        Poscar::from_reader(doc)
    }}
}

#[test]
fn comment() {
    assert_eq!(
        poscar!(b"
              this is a # boring ! comment
            1
            1 0 0
            0 1 0
            0 0 1
            1
            Direct
            0 0 0
        ").unwrap().into_raw().comment,
        "  this is a # boring ! comment",
    );
}

#[test]
fn scale() {
    assert_eq!(
        poscar!(b"
            comment
            2.45
            1 0 0
            0 1 0
            0 0 1
            1
            Direct
            0 0 0
        ").unwrap().into_raw().scale,
        ScaleLine::Factor(2.45),
    );

    assert_eq!(
        poscar!(b"
            comment
            -2.45
            1 0 0
            0 1 0
            0 0 1
            1
            Direct
            0 0 0
        ").unwrap().into_raw().scale,
        ScaleLine::Volume(2.45),
    );
}


#[test]
fn lattice() {
    // NOTE: uses nontrivial scale to check that it is
    //       not incorporated into the lattice
    assert_eq!(
        poscar!(b"
            comment
            2.45
            1.25 2.5 3.0
            -1.25 2.5 3.0
            1.25 -2.5 3.0
            1
            Direct
            0 0 0
        ").unwrap().into_raw().lattice_vectors,
        [
            [1.25, 2.5, 3.0],
            [-1.25, 2.5, 3.0],
            [1.25, -2.5, 3.0],
        ],
    );
}

#[test]
fn atom_types() {
    let p = poscar!(b"
        comment
        2.45
        1.25 2.5 3.0
        -1.25 2.5 3.0
        1.25 -2.5 3.0
        C N
        2 1
        Direct
        0 0 0
        0.25 0.25 0.25
        0.5 0.5 0.5
    ").unwrap().into_raw();

    assert_eq!(p.group_symbols, Some(vec!["C".to_string(), "N".to_string()]));
    assert_eq!(p.group_counts, vec![2, 1]);

    let p = poscar!(b"
        comment
        2.45
        1.25 2.5 3.0
        -1.25 2.5 3.0
        1.25 -2.5 3.0
        2 1
        Direct
        0 0 0
        0.25 0.25 0.25
        0.5 0.5 0.5
    ").unwrap().into_raw();

    assert_eq!(p.group_symbols, None);
    assert_eq!(p.group_counts, vec![2, 1]);
}

#[test]
fn positions() {
    assert_eq!(
        // NOTE: uses nontrivial scale and lattice to check
        //       that these are not incorporated into the positions
        poscar!(b"
            comment
            2.45
            1.25 2.5 3.0
            -1.25 2.5 3.0
            1.25 -2.5 3.0
            2 1
            Cartesian
            0 0.25 0.5
            1 1.25 1.5
            2 2.25 2.5
        ").unwrap().into_raw().positions,
        Coords::Cart(vec![
            [0.0, 0.25, 0.5],
            [1.0, 1.25, 1.5],
            [2.0, 2.25, 2.5],
        ]),
    );

    assert_eq!(
        poscar!(b"
            comment
            2.45
            1.25 2.5 3.0
            -1.25 2.5 3.0
            1.25 -2.5 3.0
            2 1
            Direct
            0 0.25 0.5
            1 1.25 1.5
            2 2.25 2.5
        ").unwrap().into_raw().positions,
        Coords::Frac(vec![
            [0.0, 0.25, 0.5],
            [1.0, 1.25, 1.5],
            [2.0, 2.25, 2.5],
        ]),
    );
}

#[test]
fn dynamics() {
    assert_eq!(
        poscar!(b"
            comment
            2.45
            1.25 2.5 3.0
            -1.25 2.5 3.0
            1.25 -2.5 3.0
            2 1
            Selective Dynamics
            Cartesian
            0 0.25 0.5 T F T
            1 1.25 1.5 F F T
            2 2.25 2.5 T T F
        ").unwrap().into_raw().dynamics,
        Some(vec![
            [ true, false,  true],
            [false, false,  true],
            [ true,  true, false],
        ]),
    );

    assert_eq!(
        poscar!(b"
            comment
            2.45
            1.25 2.5 3.0
            -1.25 2.5 3.0
            1.25 -2.5 3.0
            2 1
            Cartesian
            0 0.25 0.5
            1 1.25 1.5
            2 2.25 2.5
        ").unwrap().into_raw().dynamics,
        None,
    );
}

#[test]
fn velocities() {
    assert_eq!(
        // NOTE: uses nontrivial scale and lattice to check
        //       that these are not incorporated into the velocities
        poscar!(b"
            comment
            2.45
            1.25 2.5 3.0
            -1.25 2.5 3.0
            1.25 -2.5 3.0
            2
            Cartesian
            0 0 0
            0 0 0
        ").unwrap().into_raw().velocities,
        None,
    );

    assert_eq!(
        // NOTE: uses nontrivial scale and lattice to check
        //       that these are not incorporated into the velocities
        poscar!(b"
            comment
            2.45
            1.25 2.5 3.0
            -1.25 2.5 3.0
            1.25 -2.5 3.0
            2
            Cartesian
            0 0 0
            0 0 0
            Cartesian
            0 0.25 0.5
            1 1.25 1.5
        ").unwrap().into_raw().velocities,
        Some(Coords::Cart(vec![
            [0.0, 0.25, 0.5],
            [1.0, 1.25, 1.5],
        ])),
    );

    assert_eq!(
        poscar!(b"
            comment
            2.45
            1.25 2.5 3.0
            -1.25 2.5 3.0
            1.25 -2.5 3.0
            2
            Cartesian
            0 0 0
            0 0 0
            Direct
            0 0.25 0.5
            1 1.25 1.5
        ").unwrap().into_raw().velocities,
        Some(Coords::Frac(vec![
            [0.0, 0.25, 0.5],
            [1.0, 1.25, 1.5],
        ])),
    );

    // A blank line meaning "Direct".
    // This is actually what our crate does in its own output. (why? vaspdiddit)
    assert_eq!(
        poscar!(b"
            comment
            2.45
            1.25 2.5 3.0
            -1.25 2.5 3.0
            1.25 -2.5 3.0
            2
            Cartesian
            0 0 0
            0 0 0

            0 0.25 0.5
            1 1.25 1.5
        ").unwrap().into_raw().velocities,
        Some(Coords::Frac(vec![
            [0.0, 0.25, 0.5],
            [1.0, 1.25, 1.5],
        ])),
    );
}
