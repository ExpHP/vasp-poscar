// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Smoke test of Poscar::write.
//!
//! This sets a pretty low bar, and only the most
//! phenomenally broken implementation can fail here.
//!
//! Is incidentally sensitive to output format.

use vasp_poscar::{Builder, RawPoscar, ScaleLine, Coords};

fn boring_poscar() -> RawPoscar {
    Builder::new()
        .dummy_lattice_vectors()
        .positions(Coords::Frac(vec![[0.0; 3]; 1]))
        .build_raw()
}

// Stringify a poscar and grab a few select lines into an array.
macro_rules! poscar_lines {
    ($poscar:expr, [$($i:expr),+ $(,)*]) => {{
        let poscar: vasp_poscar::Poscar = $poscar;
        let s = format!("{}", poscar);
        [
            // (this is only for use in assert_eq, where a default value
            //  for the string is a bit more helpful than an early panic)
            $( s.lines().nth($i).unwrap_or_else(|| "(LINE NOT PRESENT)").to_string() ,)+
        ]
    }}
}

#[test]
fn comment() {
    let mut poscar = boring_poscar();
    poscar.comment = "  this is a # boring ! comment".into();

    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [0]),
        ["  this is a # boring ! comment"],
    );
}

#[test]
fn scale() {
    let mut poscar = boring_poscar();
    poscar.scale = ScaleLine::Factor(2.75);

    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [1]),
        ["  2.75"],
    );

    let mut poscar = boring_poscar();
    poscar.scale = ScaleLine::Volume(2.75);

    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [1]),
        ["  -2.75"],
    );
}

#[test]
fn lattice() {
    let mut poscar = boring_poscar();
    poscar.lattice_vectors = [
        [1.25, 2.5, 3.0],
        [-1.25, 2.5, 3.0],
        [1.25, -2.5, 3.0],
    ];
    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [2, 3, 4]),
        [
            "    1.25 2.5 3.0",
            "    -1.25 2.5 3.0",
            "    1.25 -2.5 3.0",
        ],
    );
}

#[test]
fn atom_types() {
    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.positions = Coords::Frac(vec![[0.0; 3]; 3]);
    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [5]),
        [
            "   2  1",
        ],
    );

    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.group_symbols = Some(vec!["C".into(), "N".into()]);
    poscar.positions = Coords::Frac(vec![[0.0; 3]; 3]);
    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [5, 6]),
        [
            "   C  N",
            "   2  1",
        ],
    );
}


#[test]
fn positions() {
    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.positions = Coords::Frac(vec![
        [0.0, 0.25, 0.5],
        [1.0, 1.25, 1.5],
        [2.0, 2.25, 2.5],
    ]);
    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [6, 7, 8, 9]),
        [
            "Direct",
            "  0.0 0.25 0.5",
            "  1.0 1.25 1.5",
            "  2.0 2.25 2.5",
        ],
    );

    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.positions = Coords::Cart(vec![
        [0.0, 0.25, 0.5],
        [1.0, 1.25, 1.5],
        [2.0, 2.25, 2.5],
    ]);
    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [6, 7, 8, 9]),
        [
            "Cartesian",
            "  0.0 0.25 0.5",
            "  1.0 1.25 1.5",
            "  2.0 2.25 2.5",
        ],
    );
}

#[test]
fn dynamics() {
    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.positions = Coords::Frac(vec![[0.0; 3]; 3]);
    poscar.dynamics = Some(vec![
        [ true, false,  true],
        [false, false,  true],
        [ true,  true, false],
    ]);
    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [6, 7, 8, 9, 10]),
        [
            "Selective Dynamics",
            "Direct",
            "  0.0 0.0 0.0 T F T",
            "  0.0 0.0 0.0 F F T",
            "  0.0 0.0 0.0 T T F",
        ],
    );
}

#[test]
fn velocities() {
    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.positions = Coords::Frac(vec![[0.0; 3]; 3]);

    poscar.velocities = None;
    {
        let poscar = poscar.clone().validate().unwrap();
        assert_eq!(10, format!("{}", poscar).lines().count());
    }

    poscar.velocities = Some(Coords::Frac(vec![[0.1; 3]; 3]));
    {
        let poscar = poscar.clone().validate().unwrap();
        assert_eq!(14, format!("{}", poscar).lines().count());
        assert_eq!(
            poscar_lines!(poscar, [10, 11, 12, 13]),
            [
                "",
                "  0.1 0.1 0.1",
                "  0.1 0.1 0.1",
                "  0.1 0.1 0.1",
            ],
        );
    }

    poscar.velocities = Some(Coords::Cart(vec![[0.1; 3]; 3]));
    {
        let poscar = poscar.clone().validate().unwrap();
        assert_eq!(14, format!("{}", poscar).lines().count());
        assert_eq!(
            poscar_lines!(poscar, [10, 11, 12, 13]),
            [
                "Cartesian",
                "  0.1 0.1 0.1",
                "  0.1 0.1 0.1",
                "  0.1 0.1 0.1",
            ],
        );
    }
}
