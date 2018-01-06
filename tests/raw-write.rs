// These test the output forms of various fields on RawPoscar.
//
// These are sensitive to a few arbitrary choices in output format;
// but other than that, they set a pretty low bar, and only the most
// phenomenally broken implementation can fail here.

extern crate poscar;
use ::poscar::{RawPoscar, ScaleLine, Coords};

fn boring_poscar() -> RawPoscar {
    RawPoscar {
        comment: "comment".into(),
        scale: ScaleLine::Factor(1.0),
        lattice_vectors: [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ],
        group_symbols: None,
        group_counts: vec![1],
        coords: Coords::Frac(vec![[0.0, 0.0, 0.0]]),
        dynamics: None,
        velocities: None,
    }
}

// Stringify a poscar and grab a few select lines into an array.
macro_rules! poscar_lines {
    ($poscar:expr, [$($i:expr),+ $(,)*]) => {{
        let mut bytes = vec![];
        ::poscar::to_writer(&mut bytes, &$poscar).unwrap();

        let s = String::from_utf8(bytes).unwrap();

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
        [" 2.75"],
    );

    let mut poscar = boring_poscar();
    poscar.scale = ScaleLine::Volume(2.75);

    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [1]),
        [" -2.75"],
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
            "  1.25 2.5 3.0",
            "  -1.25 2.5 3.0",
            "  1.25 -2.5 3.0",
        ],
    );
}

#[test]
fn atom_types() {
    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.coords = Coords::Frac(vec![[0.0; 3]; 3]);
    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [5]),
        [
            " 2 1",
        ],
    );

    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.group_symbols = Some(vec!["C".into(), "N".into()]);
    poscar.coords = Coords::Frac(vec![[0.0; 3]; 3]);
    assert_eq!(
        poscar_lines!(poscar.validate().unwrap(), [5, 6]),
        [
            " C N",
            " 2 1",
        ],
    );
}


#[test]
fn coords() {
    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.coords = Coords::Frac(vec![
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
    poscar.coords = Coords::Cart(vec![
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
    poscar.coords = Coords::Frac(vec![[0.0; 3]; 3]);
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