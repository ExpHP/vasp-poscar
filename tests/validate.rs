// Tests of `RawPoscar::validate`, which is one of the two
// major ways to construct a `Poscar`.
#![deny(unused)]

extern crate vasp_poscar;
use vasp_poscar::{RawPoscar, Coords, ScaleLine, ValidationError};

#[macro_use]
mod common;

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


#[test]
fn comment_newline() {
    let mut poscar = boring_poscar();

    poscar.comment = "lol\rrite".into();
    assert_matches!(
        Err(ValidationError::NewlineInComment),
        poscar.clone().validate(),
    );

    poscar.comment = "lol\nrite".into();
    assert_matches!(
        Err(ValidationError::NewlineInComment),
        poscar.clone().validate(),
    );
}


#[test]
fn bad_scale() {
    let mut poscars = vec![
        boring_poscar(), boring_poscar(),
        boring_poscar(), boring_poscar(),
    ];
    poscars[0].scale = ScaleLine::Factor(0.0);
    poscars[1].scale = ScaleLine::Volume(0.0);
    poscars[2].scale = ScaleLine::Factor(-1.0);
    poscars[3].scale = ScaleLine::Volume(-1.0);

    for poscar in poscars {
        assert_matches!(
            Err(ValidationError::BadScaleLine),
            poscar.validate(),
        );
    }
}

#[test]
fn no_atoms() {
    let mut poscar = boring_poscar();
    poscar.coords = Coords::Frac(vec![]);

    poscar.group_counts = vec![];
    assert_matches!(
        Err(ValidationError::NoAtoms),
        poscar.clone().validate(),
    );

    poscar.group_counts = vec![0, 0];
    assert_matches!(
        Err(ValidationError::NoAtoms),
        poscar.clone().validate(),
    );
}

#[test]
fn inconsistent_num_groups() {
    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.coords = Coords::Frac(vec![[0.0; 3]; 3]);

    poscar.group_symbols = Some(vec!["C".into()]);
    assert_matches!(
        Err(ValidationError::InconsistentNumGroups),
        poscar.clone().validate(),
    );

    poscar.group_symbols = Some(vec!["C".into(); 3]);
    assert_matches!(
        Err(ValidationError::InconsistentNumGroups),
        poscar.clone().validate(),
    );
}

#[test]
fn inconsistent_num_atoms() {
    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.coords = Coords::Frac(vec![[0.0; 3]; 2]);
    assert_matches!(
        Err(ValidationError::WrongLength(..)),
        poscar.validate(),
    );

    let mut poscar = boring_poscar();
    poscar.group_counts = vec![2, 1];
    poscar.coords = Coords::Frac(vec![[0.0; 3]; 3]);
    assert_matches!(
        Ok(_),
        poscar.clone().validate(),
    );

    {
        let mut poscar = poscar.clone();
        poscar.dynamics = Some(vec![[true; 3]; 2]);
        assert_matches!(
            Err(ValidationError::WrongLength(..)),
            poscar.validate(),
        );
    }

    {
        let mut poscar = poscar.clone();
        poscar.velocities = Some(Coords::Frac(vec![[0.0; 3]; 2]));
        assert_matches!(
            Err(ValidationError::WrongLength(..)),
            poscar.validate(),
        );
    }
}
