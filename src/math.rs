// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Prevents accidental usage of a 'fn(&T) -> T' as a 'fn(&mut T)'
/// by wrapping the output type.
#[must_use = "This function does not perform an in-place update! Use the output or seek a _mut version."]
pub(crate) struct MustUse<T>(pub(crate) T);

pub(crate) fn cross_f64(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3]
{ [
    a[1]*b[2] - a[2]*b[1],
    a[2]*b[0] - a[0]*b[2],
    a[0]*b[1] - a[1]*b[0],
]}

pub(crate) fn dot_f64(a: &[f64; 3], b: &[f64; 3]) -> f64
{ a[0]*b[0] + a[1]*b[1] + a[2]*b[2] }

pub(crate) fn det_f64(m: &[[f64; 3]; 3]) -> f64
{ dot_f64(&cross_f64(&m[0], &m[1]), &m[2]) }

pub(crate) fn inv_f64(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3]
{
    let cofactors = mat_3!((r, c) => {
        0.0
        + m[(r+1) % 3][(c+1) % 3] * m[(r+2) % 3][(c+2) % 3]
        - m[(r+1) % 3][(c+2) % 3] * m[(r+2) % 3][(c+1) % 3]
    });
    let det = dot_f64(&m[0], &cofactors[0]);
    mat_3!((r, c) => det.recip() * cofactors[c][r])
}

pub(crate) fn mul_3_33(v: &[f64; 3], m: &[[f64; 3]; 3]) -> [f64; 3]
{
    // I suspect this is *vaguely* more amenable to vector instructions
    // than an implementation in terms of `dot_3`.
    let mut out = [0.0; 3];
    for (coord, row) in zip!(v, m) {
        for (dest, mat_elem) in zip!(&mut out, row) {
            *dest += coord * mat_elem;
        }
    }
    out
}

pub(crate) fn mul_n3_33(vs: &[[f64; 3]], m: &[[f64; 3]; 3]) -> Vec<[f64; 3]>
{
    // Not the most vectorizable implementation ever, but it's doubtful we
    // can do much better given our SoA representation of vs.
    vs.iter().map(|v| mul_3_33(v, m)).collect()
}

pub(crate) fn scale_n3_mut(m: &mut [[f64; 3]], scale: f64) {
    for row in m {
        for x in row {
            *x *= scale;
        }
    }
}

pub(crate) fn scale_33(m: &[[f64; 3]; 3], scale: f64) -> MustUse<[[f64; 3]; 3]> {
    let mut out = *m;
    scale_n3_mut(&mut out, scale);
    MustUse(out)
}

pub(crate) fn scale_n3(vs: &[[f64; 3]], scale: f64) -> MustUse<Vec<[f64; 3]>> {
    let mut out = vs.to_vec();
    scale_n3_mut(&mut out, scale);
    MustUse(out)
}

// /// Returned when a mutation to a Poscar is aborted because it
// /// would have produced a non-finite float.
// #[derive(Debug, Fail)]
// #[fail(display = "a floating point number was not finite")]
// pub struct NonFiniteError;

// pub(crate) trait FloatHelperExt: Sized {
//     fn check_finite(self) -> Result<Self, NonFiniteError>;
//     fn cbrt(self) -> Self;
// }

// impl FloatHelperExt for f64 {
//     fn check_finite(self) -> Result<Self, NonFiniteError>
//     { match self.is_finite() {
//         true => Ok(self),
//         false => Err(NonFiniteError),
//     }}

//     fn cbrt(self) -> Self
//     { self.abs().powf(1./3.) * self.signum() }
// }

#[cfg(test)]
#[deny(unused)]
mod tests {
    use super::*;
    use ::util::test_matrices::*;

    #[test]
    fn test_mul_n3_33() {
        let vs = [
            [1.0, 1.0, 5.0],
            [0.0, 1.0, 5.0],
            [1.0, 5.0, 2.0],
            [2.0, 2.0, 0.0],
        ];
        let m = [
            [5.0, 5.0, 5.0],
            [2.0, 1.0, 5.0],
            [6.0, 4.0, 5.0],
        ];
        let prod = [
            [37.0, 26.0, 35.0],
            [32.0, 21.0, 30.0],
            [27.0, 18.0, 40.0],
            [14.0, 12.0, 20.0],
        ];
        assert_eq!(mul_n3_33(&vs, &m), prod);
    }

    #[test]
    fn test_inv_f64() {
        // test an inverse that can be computed exactly.
        assert_eq!(
            inv_f64(&EXAMPLE_UNIMODULAR),
            EXAMPLE_UNIMODULAR_INV,
        );
        // try a determinant not equal to 1.
        assert_eq!(
            inv_f64(&scale_33(&EXAMPLE_UNIMODULAR, -2.0).0),
            scale_33(&EXAMPLE_UNIMODULAR_INV, -0.5).0,
        );
    }
}
