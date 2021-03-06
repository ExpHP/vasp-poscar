// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::math::{inv_f64, det_f64};
use std::borrow::{Cow};

/// Represents a POSCAR file.
///
/// The key parts of the API are currently:
///
/// * **Reading files** through [`Poscar::from_reader`].
/// * **In-memory construction** via [`Builder`].
/// * **Manipulation/inspection** of the data via [`raw`] and [`RawPoscar`].
///   *(this will be supplanted with cleaner solutions over time)*
/// * **Writing files**, via `std::fmt::Display`.
///
/// Please follow the links above to learn about these APIs.  The remaining item
/// is documented below.
///
/// # Writing files
///
/// Printing of POSCAR files is implemented as a `std::fmt::Display`
/// impl on the [`Poscar`] type.  This means that you can use it in all of
/// the standard library macros like `print!`, `format!`, and `write!`.
///
/// By default, the crate prints to roundtrip precision, switching to
/// exponential for values with large or small magnitudes.  If you would prefer
/// a more tabular output format, you may specify format flags, which will
/// be used to control the formatting of all floats.
///
/// ```rust
/// # fn main() -> Result<(), failure::Error> {Ok({
/// use vasp_poscar::Poscar;
///
/// let poscar = Poscar::from_reader("\
/// POSCAR File
///    1.0
///      1.0   0.0   0.0
///      0.0   1.23456789012 -0.2
///      0.0   0.0   1.0
///   1
/// Direct
///  0.1  -1.2e-30  0.0
/// ".as_bytes())?;
///
/// // Default format: roundtrip
/// assert_eq!(format!("{}", poscar), "\
/// POSCAR File
///   1.0
///     1.0 0.0 0.0
///     0.0 1.23456789012 -0.2
///     0.0 0.0 1.0
///    1
/// Direct
///   0.1 -1.2e-30 0.0
/// ");
///
/// // Custom formats
/// assert_eq!(format!("{:>9.6}", poscar), "\
/// POSCAR File
///    1.000000
///      1.000000  0.000000  0.000000
///      0.000000  1.234568 -0.200000
///      0.000000  0.000000  1.000000
///    1
/// Direct
///    0.100000 -0.000000  0.000000
/// ");
/// # })}
/// ```
///
/// [`Poscar::from_reader`]: #method.from_reader
/// [`RawPoscar`]: struct.RawPoscar.html
/// [`into_raw`]: #method.into_raw
/// [`Builder`]: builder/struct.Builder.html
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
    pub fn into_raw(self) -> RawPoscar { self.0 }
}

/// # Accessing simple properties
impl Poscar {
    pub fn comment(&self) -> &str
    { &self.0.comment }

    /// Get the symbols for each atom type, if provided.
    pub fn group_symbols(&self) -> Option<impl VeclikeIterator<Item=&str> + '_>
    {
        self.0.group_symbols.as_ref()
            .map(|syms| syms.iter().map(|sym| &sym[..]))
    }

    /// Get the counts of each atom type.
    pub fn group_counts(&self) -> impl VeclikeIterator<Item=usize> + '_
    { self.0.group_counts.iter().map(|&c| c) }

    /// Get the number of sites in the unit cell.
    pub fn num_sites(&self) -> usize
    { self.0.positions.as_ref().raw().len() }

    /// Get the symbols for each site in the unit cell.
    pub fn site_symbols(&self) -> Option<impl VeclikeIterator<Item=&str> + '_>
    {
        self.group_symbols().map(|group_symbols| {
            assert_eq!(
                self.0.group_counts.len(), group_symbols.len(),
                "(BUG) length invariant violated!",
            );

            WithKnownLen {
                iter: {
                    self.0.group_counts.iter().zip(group_symbols)
                        .flat_map(|(&count, symbol)| RepeatN { value: symbol, n: count })
                },
                len: self.num_sites(),
            }
        })
    }
}

#[test]
fn test_group_iters() -> Result<(), failure::Error> {
    use crate::{Builder, Zeroed};

    let poscar =
        Builder::new()
        .group_counts(vec![2, 5, 1])
        .group_symbols(vec!["C", "B", "C"])
        .dummy_lattice_vectors()
        .positions(Coords::Cart(Zeroed))
        .build()?;

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

    let poscar = {
        let mut poscar = poscar.into_raw();
        poscar.group_symbols = None;
        poscar.validate()?
    };

    assert!(poscar.group_symbols().is_none());
    Ok(())
}

#[test]
fn test_site_symbols() -> Result<(), failure::Error> {
    use crate::{Builder, Zeroed};

    let builder = {
        Builder::new()
            .positions(Coords::Frac(Zeroed))
            .dummy_lattice_vectors()
            .group_counts([2, 3, 1].iter().cloned())
            .clone()
    };

    fn strings<S: ToString>(strs: impl IntoIterator<Item=S>) -> Vec<String>
    { strs.into_iter().map(|s| s.to_string()).collect() }

    let get_group_symbols = |poscar: &Poscar| poscar.group_symbols().map(strings);
    let get_site_symbols = |poscar: &Poscar| poscar.site_symbols().map(strings);

    let poscar = builder.clone().build()?;
    assert_eq!(poscar.group_counts().collect::<Vec<_>>(), vec![2, 3, 1]);
    assert_eq!(get_group_symbols(&poscar), None);
    assert_eq!(get_site_symbols(&poscar), None);

    let poscar = builder.clone().group_symbols(vec!["Xe", "C", "Xe"]).build()?;
    assert_eq!(poscar.group_counts().collect::<Vec<_>>(), vec![2, 3, 1]);
    assert_eq!(get_group_symbols(&poscar), Some(strings(vec!["Xe", "C", "Xe"])));
    assert_eq!(get_site_symbols(&poscar), Some(strings(vec!["Xe", "Xe", "C", "C", "C", "Xe"])));

    // test DoubleEndedIterator and ExactSizeIterator impls
    let poscar = builder.clone().group_symbols(vec!["Xe", "C", "B"]).build()?;
    let mut iter = poscar.site_symbols().unwrap();
    assert_eq!(iter.len(), 6);
    assert_eq!(iter.next(), Some("Xe"));
    assert_eq!(iter.len(), 5);
    assert_eq!(iter.next_back(), Some("B"));
    assert_eq!(iter.len(), 4);
    assert_eq!(strings(iter.rev()), strings(vec!["C", "C", "C", "Xe"]));

    Ok(())
}

/// Combines useful standard library iterator traits into one.
///
/// Used in `impl Trait` return types to abbreviate an otherwise long type.
pub trait VeclikeIterator: ExactSizeIterator + DoubleEndedIterator { }
impl<Xs: ExactSizeIterator + DoubleEndedIterator> VeclikeIterator for Xs { }

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
    /// Get either `scaled_cart_positions` or `frac_positions`, depending on
    /// which is stored.
    pub fn scaled_positions(&self) -> Coords<Cow<'_, [[f64; 3]]>>
    {
        match self.0.positions {
            Coords::Cart(_) => Coords::Cart(self.scaled_cart_positions()),
            Coords::Frac(_) => Coords::Frac(self.frac_positions()),
        }
    }

    /// Compute the Cartesian positions, taking into account the scale factor.
    pub fn scaled_cart_positions(&self) -> Cow<'_, [[f64; 3]]>
    {
        // TODO maybe later: a reference can be returned when
        //   carts are stored and the scale line is Factor(1.0)
        match self.0.positions.as_ref() {
            Coords::Cart(pos) => {
                let scale = self.effective_scale_factor();
                crate::math::scale_n3(pos, scale).0.into()
            },
            Coords::Frac(x) => crate::math::mul_n3_33(x, &self.scaled_lattice()).into(),
        }
    }

    /// Get the Cartesian positions, as they would be written in the file.
    pub fn unscaled_cart_positions(&self) -> Cow<'_, [[f64; 3]]>
    { self.0.positions.to_tag(&self.unscaled_lattice(), CART) }

    /// Get the fractional positions, as they would be written in the file.
    pub fn frac_positions(&self) -> Cow<'_, [[f64; 3]]>
    { self.0.positions.to_tag(&self.unscaled_lattice(), FRAC) }
}

/// # Accessing velocities
impl Poscar {
    /// Get the fractional-space velocities.
    pub fn frac_velocities(&self) -> Option<Cow<'_, [[f64; 3]]>>
    { self.0.velocities.as_ref().map(|c| {
        c.to_tag(&self.unscaled_lattice(), FRAC)
    })}

    /// Get the cartesian velocities.
    ///
    /// Notice that the scale factor does not affect velocities.
    pub fn cart_velocities(&self) -> Option<Cow<'_, [[f64; 3]]>>
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
        crate::math::scale_33(&self.0.lattice_vectors, f).0
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
/// # Construction
///
/// ## From data
///
/// A `RawPoscar` can be constructed using the [`Builder`] API.
///
/// ```rust
/// use vasp_poscar::{Builder, ScaleLine, Coords};
///
/// # #[allow(unused)]
/// let poscar =
///     Builder::new()
///     .comment("Cubic BN")
///     .scale(ScaleLine::Factor(3.57))
///     .lattice_vectors(&[
///         [0.0, 0.5, 0.5],
///         [0.5, 0.0, 0.5],
///         [0.5, 0.5, 0.0],
///     ])
///     .group_symbols(vec!["B", "N"])
///     .group_counts(vec![1, 1])
///     .positions(Coords::Frac(vec![
///         [0.00, 0.00, 0.00],
///         [0.25, 0.25, 0.25],
///     ]))
///     .build_raw();
/// ```
///
/// ## From a file
///
/// You may parse the file into a Poscar first.
///
/// ```rust,no_run
/// # fn main() -> Result<(), failure::Error> {Ok({
/// # use vasp_poscar::Poscar;
/// #
/// # #[allow(unused)]
/// let poscar = Poscar::from_path("tests/POSCAR")?.into_raw();
/// #
/// # })}
/// ```
///
/// # Manipulation
///
/// All fields are public, barring a single trivial private field
/// used to prevent construction. You can freely access and manipulate
/// the data fields as you see fit.
///
/// # Display
///
/// Because it may contain invalid data, a `RawPoscar` object
/// **cannot be printed.** To write a `RawPoscar` to a file,
/// use the [`validate`] method to obtain a [`Poscar`] first.
///
/// ```rust,no_run
/// # fn get_raw_poscar() -> vasp_poscar::RawPoscar { unimplemented!() }
/// # fn main() -> Result<(), failure::Error> {Ok({
/// // suppose you have a RawPoscar
/// let raw = get_raw_poscar();
///
/// // validate() will "upgrade" it into a Poscar...
/// let poscar = raw.validate()?;
/// // ...which can be printed.
/// print!("{}", poscar);
/// # })}
/// ```
///
/// [VASP documentation]: https://cms.mpi.univie.ac.at/vasp/vasp/POSCAR_file.html
/// [`validate`]: struct.Poscar.html#method.validate
/// [`Poscar`]: struct.Poscar.html
/// [`Builder`]: builder/struct.Builder.html
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

    pub(crate) _cant_touch_this: (),
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
    let _: failure::Error = e.into();
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
                    crate::parse::is_valid_symbol_for_symbol_line(sym.as_str()),
                    ValidationError::InvalidSymbol(Some(sym.as_str().into())),
                )
            }

            // *Just in case:* Use the same logic as the parser to retokenize the entire
            // symbols line, thereby absolutely guaranteeing that it roundtrips.
            // This check is guaranteed to remain sufficient even if we were to change
            // the rules of tokenization.
            use crate::parse::Spanned;
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

pub(crate) type CoordsTag = Coords<()>;
pub(crate) const CART: CoordsTag = Coords::Cart(());
pub(crate) const FRAC: CoordsTag = Coords::Frac(());

impl<V> Coords<V> {
    #[inline(always)]
    pub(crate) fn tag(&self) -> CoordsTag
    { self.as_ref().map(|_| ()) }

    #[inline(always)]
    pub(crate) fn of_tag(tag: CoordsTag, value: V) -> Coords<V>
    { tag.map(|()| value) }
}

impl Coords {
    /// Convert into a specific Coord representations on demand.
    ///
    /// May return a borrow if that data is immediately available.
    ///
    /// This may compute a lattice inverse; don't use it in a tight loop.
    #[inline(always)]
    pub(crate) fn to_tag(&self, lattice: &[[f64; 3]; 3], tag: Coords<()>) -> Cow<'_, [[f64; 3]]>
    {
        use self::Coords::{Cart, Frac};
        match (self.as_ref(), tag) {
            // borrow if possible
            (Cart(v), CART) |
            (Frac(v), FRAC) => (&v[..]).into(),

            // compute
            (Frac(v), CART) => crate::math::mul_n3_33(v, lattice).into(),
            (Cart(v), FRAC) => crate::math::mul_n3_33(v, &inv_f64(lattice)).into(),
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

/// Adds `ExactSizeIterator` to an arbitrary iterator.
struct WithKnownLen<Iter> {
    iter: Iter,
    len: usize,
}

impl<Iter: Iterator> Iterator for WithKnownLen<Iter> {
    type Item = Iter::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.len = usize::saturating_sub(self.len, 1);
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    { (self.len, Some(self.len)) }
}

impl<Iter: Iterator> ExactSizeIterator for WithKnownLen<Iter> { }

impl<Iter: DoubleEndedIterator> DoubleEndedIterator for WithKnownLen<Iter> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.len = usize::saturating_sub(self.len, 1);
        self.iter.next_back()
    }
}

/// `std::iter::repeat(x).take(n)` with a `DoubleEndedIterator` impl
struct RepeatN<X> {
    value: X,
    n: usize,
}

impl<X: Clone> Iterator for RepeatN<X> {
    type Item = X;

    fn next(&mut self) -> Option<Self::Item> {
        match self.n {
            0 => None,
            _ => {
                self.n -= 1;
                Some(self.value.clone())
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    { (self.n, Some(self.n)) }
}

impl<X: Clone> DoubleEndedIterator for RepeatN<X> {
    fn next_back(&mut self) -> Option<Self::Item>
    { self.next() }
}

impl<X: Clone> ExactSizeIterator for RepeatN<X> { }

// --------------------------------

#[cfg(test)]
#[deny(unused)]
mod accessor_tests {
    use super::*;
    use crate::Builder;

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
        assert_eq!(crate::math::det_f64(&UNSCALED_LATTICE), -8.0);
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

                let poscar =
                    Builder::new()
                    .scale(scale)
                    .positions(coord_data.map(|v| v.to_vec()))
                    .lattice_vectors(&UNSCALED_LATTICE)
                    .build().unwrap();

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
                assert_eq!(
                    poscar.scaled_positions(),
                    match coord_data {
                        Coords::Cart(_) => Coords::Cart(Cow::from(SCALED_CARTS)),
                        Coords::Frac(_) => Coords::Frac(Cow::from(FRACS)),
                    },
                    "{}", dbg,
                );
                assert_eq!(poscar.frac_velocities(), None, "{}", dbg);
                assert_eq!(poscar.cart_velocities(), None, "{}", dbg);

                // --------
                // Check velocity accessors.
                let mut poscar = poscar.into_raw();

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
