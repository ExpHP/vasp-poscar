// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ::math::{inv_f64, det_f64};
use ::std::borrow::{Cow};

/// Represents a POSCAR file.
///
/// The key parts of the API are currently:
///
/// * **Reading files** through [`Poscar::from_reader`].
/// * **Manipulation/inspection** of the data via [`raw`] and [`RawPoscar`].
///   *(this will be supplanted with cleaner solutions over time)*
/// * **Writing files** through `std::fmt::Display`. (e.g. `print!` and `write!`)
///
/// [`Poscar::from_reader`]: #method.from_reader
/// [`RawPoscar`]: struct.RawPoscar.html
/// [`raw`]: #method.raw
#[derive(Debug, Clone)]
pub struct Poscar(pub(crate) RawPoscar);

impl Poscar {
    /// Convert into a form with public data members that you can freely match
    /// against and unpack.
    ///
    /// When you are done modifying the object, you may call [`validate`]
    /// to turn it back into a `Poscar`.
    /// (or you can just keep all the data to yourself. We don't mind!)
    ///
    /// Currently, this is the most versatile way of manipulating a Poscar object,
    /// though it may not be the most stable or convenient. **Be prepared for breaking
    /// changes to affect code using this method.** In the future, stabler alternatives
    /// for common operations may be provided on `Poscar` itself.
    ///
    /// [`validate`]: struct.RawPoscar.html#method.validate
    pub fn raw(self) -> RawPoscar { self.0 }
}

/// # Accessing simple properties
impl Poscar {
    pub fn comment(&self) -> &str
    { &self.0.comment }

    /// Get the symbols for each atom type, if provided.
    ///
    /// The returned object is an `Iterator` of `&str`.
    pub fn group_symbols(&self) -> Option<GroupSymbols>
    {
        self.0.group_symbols.as_ref()
            .map(|syms| Box::new(syms.iter().map(|sym| &sym[..])) as Box<_>)
    }

    /// Get the counts of each atom type.
    ///
    /// The returned object is an `Iterator` of `usize`.
    pub fn group_counts(&self) -> GroupCounts
    { Box::new(self.0.group_counts.iter().map(|&c| c)) }
}

#[test]
fn test_group_iters() {
    let poscar = RawPoscar {
        comment: "".into(),
        dynamics: None,
        group_counts: vec![2, 5, 1],
        group_symbols: Some(vec!["C".into(), "B".into(), "C".into()]),
        scale: ScaleLine::Factor(1.0),
        lattice_vectors: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        positions: Coords::Cart(vec![[0.0; 3]; 8]),
        velocities: None,
    }.validate().unwrap();

    // verify that .rev() and .len() are usable
    assert_eq!(poscar.group_counts().len(), 3);
    assert_eq!(
        poscar.group_counts().rev().collect::<Vec<_>>(),
        vec![1, 5, 2],
    );
    assert_eq!(
        poscar.group_symbols().map(|v| v.collect::<Vec<_>>()),
        Some(vec!["C", "B", "C"]),
    );

    let mut poscar = poscar.raw();
    poscar.group_symbols = None;
    let poscar = poscar.validate().unwrap();

    assert!(poscar.group_symbols().is_none());
}

/// Combines useful standard library iterator traits into one.
///
/// This exists because a trait object may only involve one non-OIBIT trait.
pub trait VeclikeIterator: ExactSizeIterator + DoubleEndedIterator { }
impl<Xs: ExactSizeIterator + DoubleEndedIterator> VeclikeIterator for Xs { }

/// Returned by [`Poscar::group_symbols`].
///
/// [`Poscar::group_symbols`]: struct.Poscar.html#method.group_symbols
pub type GroupSymbols<'a> = Box<VeclikeIterator<Item=&'a str> + 'a>;

/// Returned by [`Poscar::group_counts`].
///
/// [`Poscar::group_counts`]: struct.Poscar.html#method.group_counts
pub type GroupCounts<'a> = Box<VeclikeIterator<Item=usize> + 'a>;

/// # Accessing computed properties
impl Poscar {
    /// Volume of a unit cell, taking the scale line into account.
    ///
    /// This quantity is non-negative.
    pub fn scaled_volume(&self) -> f64
    { match self.0.scale {
        ScaleLine::Volume(v) => v,
        ScaleLine::Factor(f) => self.unscaled_determinant().abs() * (f * f * f),
    }}

    fn unscaled_determinant(&self) -> f64
    { det_f64(&self.0.lattice_vectors) }

    // The quantity that each cartesian component needs to be multiplied
    // by to properly account for the scale line.
    //
    // This quantity is non-negative, but may be infinite.
    fn effective_scale_factor(&self) -> f64
    { match self.0.scale {
        ScaleLine::Factor(f) => f,
        ScaleLine::Volume(v) => (v / self.unscaled_determinant().abs()).cbrt(),
    }}
}

/// # Accessing the lattice vectors
impl Poscar {
    /// Compute the true lattice vectors, taking the scale line into account.
    pub fn scaled_lattice_vectors(&self) -> [[f64; 3]; 3]
    { self.scaled_lattice() }

    /// Get the lattice vectors as they are written.
    pub fn unscaled_lattice_vectors(&self) -> [[f64; 3]; 3]
    { self.0.lattice_vectors }
}

/// # Accessing positions
impl Poscar {
    /// Compute the Cartesian positions, taking into account the scale factor.
    pub fn scaled_cart_positions(&self) -> Cow<[[f64; 3]]>
    {
        // TODO maybe later: a reference can be returned when
        //   carts are stored and the scale line is Factor(1.0)
        match self.0.positions.as_ref() {
            Coords::Cart(pos) => {
                let scale = self.effective_scale_factor();
                ::math::scale_n3(pos, scale).0.into()
            },
            Coords::Frac(x) => ::math::mul_n3_33(x, &self.scaled_lattice()).into(),
        }
    }

    /// Get the Cartesian positions, as they would be written in the file.
    pub fn unscaled_cart_positions(&self) -> Cow<[[f64; 3]]>
    { self.0.positions.to_tag(&self.unscaled_lattice(), CART) }

    /// Get the fractional positions, as they would be written in the file.
    pub fn frac_positions(&self) -> Cow<[[f64; 3]]>
    { self.0.positions.to_tag(&self.unscaled_lattice(), FRAC) }
}

/// # Accessing velocities
impl Poscar {
    /// Get the fractional-space velocities.
    pub fn frac_velocities(&self) -> Option<Cow<[[f64; 3]]>>
    { self.0.velocities.as_ref().map(|c| {
        c.to_tag(&self.unscaled_lattice(), FRAC)
    })}

    /// Get the cartesian velocities.
    ///
    /// Notice that the scale factor does not affect velocities.
    pub fn cart_velocities(&self) -> Option<Cow<[[f64; 3]]>>
    { self.0.velocities.as_ref().map(|c| {
        c.to_tag(&self.unscaled_lattice(), CART)
    })}
}

// Accessing the lattice matrix.
//
// NOTE: These are not exposed because the crate deliberately tries to
//       avoid mentioning matrices, because then it would have to clarify
//       a bunch of irrelevant garbage about formalism when users really
//       only need to know the data layout (which is AoS).
//
// For the record:
//
// * our formalism is row-based (nearly all vectors are formally row vectors)
// * our storage is row-major (`matrix[i]` conceptually yields a row vector)
impl Poscar {
    fn unscaled_lattice(&self) -> [[f64; 3]; 3]
    { self.0.lattice_vectors }

    fn scaled_lattice(&self) -> [[f64; 3]; 3]
    {
        let f = self.effective_scale_factor();
        ::math::scale_33(&self.0.lattice_vectors, f).0
    }
}

/// Unencumbered `struct` form of a Poscar with public data members.
///
/// This is basically the [`Poscar`] type, minus all the type-protected
/// invariants which ensure that it can be printed.
///
/// # General notes
///
/// **Working with this type requires you to be familiar with the POSCAR
/// format.** Its fields map one-to-one with the sections of a POSCAR file.
/// Please see the [VASP documentation] for help regarding its semantics.
///
/// **Important:** not mentioned on that page is the **symbols line**, which
/// may appear right after the lattice vectors, before the counts. The number
/// of symbols must match the number of counts.
/// Example with a symbols line:
///
/// <!-- FIXME this example sucks because the number of atoms also matches
///            the number of groups -->
///
/// ```text
/// Cubic BN
///    3.57
///  0.0 0.5 0.5
///  0.5 0.0 0.5
///  0.5 0.5 0.0
///    B N
///    1 1
/// Direct
///  0.00 0.00 0.00
///  0.25 0.25 0.25
/// ```
///
/// # Buyer beware
///
/// All fields are public, allow you to **construct it using basic
/// struct syntax:**
///
/// ```rust
/// use vasp_poscar::{RawPoscar, ScaleLine, Coords};
///
/// # #[allow(unused)]
/// let poscar = RawPoscar {
///     comment: "Cubic BN".into(),
///     scale: ScaleLine::Factor(3.57),
///     lattice_vectors: [
///         [0.0, 0.5, 0.5],
///         [0.5, 0.0, 0.5],
///         [0.5, 0.5, 0.0],
///     ],
///     group_symbols: Some(vec!["B".into(), "N".into()]),
///     group_counts: vec![1, 1],
///     positions: Coords::Frac(vec![
///         [0.00, 0.00, 0.00],
///         [0.25, 0.25, 0.25],
///     ]),
///     velocities: None,
///     dynamics: None,
/// };
/// ```
///
/// This is, of course, is a double-edged sword.
/// For better or worse,
/// this type brings **simplicity at the cost of stability.**
/// Be prepared for breakage as more fields are added;
/// for now, you are advised to limit your usage of this type to
/// a small number of self-contained functions. (e.g. conversions
/// to and from a datatype of your own)
///
/// # Display
///
/// `RawPoscar` itself **cannot be printed.** To write out your modified
/// file, use the [`validate`] method to obtain a [`Poscar`] first.
///
/// [VASP documentation]: https://cms.mpi.univie.ac.at/vasp/vasp/POSCAR_file.html
/// [`validate`]: #method.validate
/// [`Poscar`]: struct.Poscar.html
#[derive(Debug, Clone)]
pub struct RawPoscar {
    pub comment: String,
    pub scale: ScaleLine,
    pub lattice_vectors: [[f64; 3]; 3],
    pub group_symbols: Option<Vec<String>>,
    pub group_counts: Vec<usize>,
    pub positions: Coords,
    pub velocities: Option<Coords>,
    pub dynamics: Option<Vec<[bool; 3]>>,
    // pub predictor_corrector: Option<PredictorCorrector>,
}

// --------------------------------
// validation

/// Covers all the reasons why [`RawPoscar::validate`] might get mad at you.
///
/// Beyond checking obvious problems like mismatched lengths, these
/// limitations also exist to ensure that a [`Poscar`] can be roundtripped
/// through its file representation.
///
/// The variants are public so that by looking at the docs you can see all the possible errors.
/// That said, you have no good reason to write code that matches on this.
///
/// ...right?
///
/// [`Poscar`]: struct.Poscar.html
/// [`RawPoscar::validate`]: struct.RawPoscar.html#method.validate
#[derive(Debug, Fail)]
pub enum ValidationError {
    /// The comment line is more than one line.
    #[fail(display = "the comment may not contain a newline")]
    NewlineInComment,

    /// A requirement on `group_symbols` was violated.
    ///
    /// There are a few more restrictions in addition to the no-leading-digit
    /// restriction mentioned in format.md, in order to ensure roundtripping:
    ///
    /// * A symbol may not be the empty string
    /// * A symbol may not contain whitespace
    // (NOTE: `None` when the specific problematic symbol could not be identified.)
    #[fail(display = "invalid symbol in group_symbols: {:?}", _0)]
    InvalidSymbol(Option<String>),

    /// Poscar is required to have at least one atom.
    #[fail(display = "at least one atom is required")]
    NoAtoms,

    /// The inner value in the scale line must be positive.
    #[fail(display = "the value inside Factor(x) or Volume(x) must be positive")]
    BadScaleLine,

    /// Mismatch between `group_counts` and `group_symbols` lengths.
    #[fail(display = "inconsistent number of atom types")]
    InconsistentNumGroups,

    /// Length of a member is incorrect.
    #[fail(display = "member '{}' is wrong length (should be {})", _0, _1)]
    WrongLength(&'static str, usize),

    /// INIT in predictor corrector is zero. (you should use `None` instead)
    #[allow(unused)] // FIXME
    #[doc(hidden)]
    #[fail(display = "predictor corrector has an init value of 0")]
    PredictorCorrectorInitIsZero,

    #[doc(hidden)]
    #[fail(display = "something absurd happened and you're not supposed to see this")]
    AndManyMooooooooore,
}

// Compile-time test for a From impl.
fn _check_conv() {
    fn panic<T>() -> T { panic!() }
    let e: ValidationError = panic();
    let _: ::failure::Error = e.into();
}

impl RawPoscar {
    /// Convert into a [`Poscar`] object after checking its invariants.
    ///
    /// To see what those invariants are, check the docs for [`ValidationError`].
    ///
    /// [`Poscar`]: struct.Poscar.html
    /// [`ValidationError`]: enum.ValidationError.html
    pub fn validate(self) -> Result<Poscar, ValidationError> {
        if let Some(ref group_symbols) = self.group_symbols {
            if self.group_counts.len() != group_symbols.len() {
                g_bail!(ValidationError::InconsistentNumGroups);
            }
        }

        g_ensure!(!self.comment.contains("\n"), ValidationError::NewlineInComment);
        g_ensure!(!self.comment.contains("\r"), ValidationError::NewlineInComment);

        match self.scale {
            ScaleLine::Factor(x) |
            ScaleLine::Volume(x) => {
                g_ensure!(x > 0.0, ValidationError::BadScaleLine);
            },
        }

        let n = self.group_counts.iter().sum::<usize>();

        g_ensure!(n > 0, ValidationError::NoAtoms);

        if let Some(group_symbols) = self.group_symbols.as_ref() {
            // Check for conditions that we know are problematic.
            for sym in group_symbols {
                g_ensure!(
                    ::parse::is_valid_symbol_for_symbol_line(sym.as_str()),
                    ValidationError::InvalidSymbol(Some(sym.as_str().into())),
                )
            }

            // *Just in case:* Use the same logic as the parser to retokenize the entire
            // symbols line, thereby absolutely guaranteeing that it roundtrips.
            // This check is guaranteed to remain sufficient even if we were to change
            // the rules of tokenization.
            use ::parse::Spanned;
            let symbol_str = group_symbols.join(" ");
            let spanned = Spanned::wrap_arbitrary(symbol_str);

            let words = spanned.words().map(|x| x.as_str().to_string());
            g_ensure!(
                words.eq(group_symbols.iter().cloned()),
                {
                    // FIXME we should log a non-fatal internal error here since
                    //       currently it is not expected for this branch to get
                    //       entered (the individual checks per-symbol ought to
                    //       be enough)
                    ValidationError::InvalidSymbol(None)
                },
            );
        }

        if self.positions.as_ref().raw().len() != n {
            g_bail!(ValidationError::WrongLength("positions", n));
        }

        if let Some(ref velocities) = self.velocities {
            if velocities.as_ref().raw().len() != n {
                g_bail!(ValidationError::WrongLength("velocities", n));
            }
        }

        if let Some(ref dynamics) = self.dynamics {
            if dynamics.len() != n {
                g_bail!(ValidationError::WrongLength("dynamics", n));
            }
        }

        Ok(Poscar(self))
    }
}

// --------------------------------
// More public API data types

/// Represents the second line in a POSCAR file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleLine {
    Factor(f64),
    Volume(f64),
}

/// Represents data that can either be in direct units or cartesian.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coords<T=Vec<[f64; 3]>> {
    Cart(T),
    Frac(T),
}

// --------------------------------
// Meat of the coordinate conversion logic

type CoordsTag = Coords<()>;
const CART: CoordsTag = Coords::Cart(());
const FRAC: CoordsTag = Coords::Frac(());

impl Coords {
    /// Convert into a specific Coord representations on demand.
    ///
    /// May return a borrow if that data is immediately available.
    ///
    /// This may compute a lattice inverse; don't use it in a tight loop.
    #[inline(always)]
    fn to_tag(&self, lattice: &[[f64; 3]; 3], tag: Coords<()>) -> Cow<[[f64; 3]]>
    {
        use self::Coords::{Cart, Frac};
        match (self.as_ref(), tag) {
            // borrow if possible
            (Cart(v), CART) |
            (Frac(v), FRAC) => (&v[..]).into(),

            // compute
            (Frac(v), CART) => ::math::mul_n3_33(v, lattice).into(),
            (Cart(v), FRAC) => ::math::mul_n3_33(v, &inv_f64(lattice)).into(),
        }
    }
}

// --------------------------------
// Helpers

impl<A> Coords<A> {
    #[allow(unused)]
    pub(crate) fn map<B, F>(self, f: F) -> Coords<B>
    where F: FnOnce(A) -> B,
    { match self {
        Coords::Cart(x) => Coords::Cart(f(x)),
        Coords::Frac(x) => Coords::Frac(f(x)),
    }}

    #[allow(unused)]
    pub(crate) fn as_ref(&self) -> Coords<&A>
    { match *self {
        Coords::Cart(ref x) => Coords::Cart(x),
        Coords::Frac(ref x) => Coords::Frac(x),
    }}

    #[allow(unused)]
    pub(crate) fn as_mut(&mut self) -> Coords<&mut A>
    { match *self {
        Coords::Cart(ref mut x) => Coords::Cart(x),
        Coords::Frac(ref mut x) => Coords::Frac(x),
    }}

    #[allow(unused)]
    pub(crate) fn raw(self) -> A
    { match self {
        Coords::Cart(x) => x,
        Coords::Frac(x) => x,
    }}
}

// --------------------------------

#[cfg(test)]
#[deny(unused)]
mod accessor_tests {
    use super::*;

    // This test aims to maximize bang-for-the-buck by trying
    // to break as many broken implementations as possible.
    #[test]
    fn smoke_test() {
        // * A lattice that:
        //   - is asymmetric
        //   - has a determinant != +/- 1
        //   - has a negative determinant
        // * A nontrivial scale factor
        //   (i.e. scaled and unscaled values can differentiated)
        // * Positions with:
        //   - a point that isn't the origin
        //   - a point that is outside the unit cell
        // * All quantities have exact floating point representations.

        // This is a unimodular matrix scaled by a factor of 2.
        // It has determinant -8.
        const UNSCALED_LATTICE: [[f64; 3]; 3] = [
            [-4.0,  2.0, -4.0],
            [ 2.0, -6.0,  6.0],
            [-2.0, -2.0,  0.0],
        ];
        assert_eq!(::math::det_f64(&UNSCALED_LATTICE), -8.0);
        // We will use the scale line to scale it by an additional factor of 2.
        const SCALE: f64 = 2.0;
        // These all have exact representations in f64.
        const FRACS: &'static [[f64; 3]] = &[
            [ 0.0 ,  0.25, 0.75 ],
            [ 0.25, -2.25, 3.125],
        ];

        // this data is all derived from the above
        const SCALED_VOLUME: f64 = 64.0;
        const SCALED_LATTICE: [[f64; 3]; 3] = [
            [-8.0,   4.0, -8.0],
            [ 4.0, -12.0, 12.0],
            [-4.0,  -4.0,  0.0],
        ];
        const UNSCALED_CARTS: &'static [[f64; 3]] = &[
            [ -1.0 , -3.0 ,   1.5],
            [-11.75,  7.75, -14.5],
        ];
        const SCALED_CARTS: &'static [[f64; 3]] = &[
            [ -2.0,  -6.0,   3.0],
            [-23.5,  15.5, -29.0],
        ];

        // Check all possible representations of this structure
        // to ensure all code branches are tested.

        // Ways to write the scale line
        for &(dbg_scale, scale) in &[
            ("factor", ScaleLine::Factor(SCALE)),
            ("volume", ScaleLine::Volume(SCALED_VOLUME)),
        ] {
            // Ways to write the coordinate data
            for &(dbg_coords, coord_data) in &[
                ("frac", Coords::Frac(FRACS)),
                ("cart", Coords::Cart(UNSCALED_CARTS)),
            ] {
                let dbg = format!("{:?}", (dbg_scale, dbg_coords));

                let lattice_vectors = UNSCALED_LATTICE;
                let positions = coord_data.map(|v| v.to_vec());

                let poscar = RawPoscar {
                    scale, positions, lattice_vectors,
                    comment: "".into(),
                    group_counts: vec![2],
                    group_symbols: None,
                    velocities: None,
                    dynamics: None,
                }.validate().unwrap();

                // --------
                // check all accessors
                assert_eq!(poscar.scaled_volume(), SCALED_VOLUME, "{}", dbg);
                assert_eq!(
                    poscar.scaled_lattice_vectors(),
                    SCALED_LATTICE,
                    "{}", dbg,
                );
                assert_eq!(
                    poscar.unscaled_lattice_vectors(),
                    UNSCALED_LATTICE,
                    "{}", dbg,
                );
                assert_eq!(poscar.frac_positions(), Cow::from(FRACS), "{}", dbg);
                assert_eq!(
                    poscar.unscaled_cart_positions(),
                    Cow::from(UNSCALED_CARTS),
                    "{}", dbg,
                );
                assert_eq!(
                    poscar.scaled_cart_positions(),
                    Cow::from(SCALED_CARTS),
                    "{}", dbg,
                );
                assert_eq!(poscar.frac_velocities(), None, "{}", dbg);
                assert_eq!(poscar.cart_velocities(), None, "{}", dbg);

                // --------
                // Check velocity accessors.
                let mut poscar = poscar.raw();

                // Move the data into the velocities.
                // Set positions to something different to make sure
                //   velocities are being read from the right field.
                poscar.velocities = Some(poscar.positions);
                poscar.positions = Coords::Cart(vec![[0f64; 3]; 2]);

                let poscar = poscar.validate().unwrap();

                assert_eq!(
                    poscar.frac_velocities(),
                    Some(Cow::from(FRACS)),
                    "{}", dbg,
                );
                assert_eq!(
                    poscar.cart_velocities(),
                    Some(Cow::from(UNSCALED_CARTS)),
                    "{}", dbg,
                );
            }
        }
    }
}
