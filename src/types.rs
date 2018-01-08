// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Represents a POSCAR file.
///
/// Currently, the only API provided on this type is the `raw` method, which
/// produces an object you can manipulate directly.
#[derive(Debug, Clone)]
pub struct Poscar(pub(crate) RawPoscar);

impl Poscar {
    /// Convert into a form with public data members that you can freely match
    /// against and unpack.
    ///
    /// When you are done modifying the object, you may call `.validate()`
    /// to turn it back into a Poscar. (or you can just keep all the data to yourself.
    /// We don't mind!)
    ///
    /// Currently, this is the most versatile way of manipulating a Poscar object,
    /// though it may not be the most stable or convenient. **Be prepared for breaking
    /// changes to affect code using this method.** In the future, stabler alternatives
    /// for common operations may be provided on `Poscar` itself.
    pub fn raw(self) -> RawPoscar { self.0 }
}

/// Basic representation of a POSCAR with public data members.
///
/// All members are public to allow you to construct it.
/// The mapping between its fields and those of the the POSCAR file
/// should be *dead obvious.*  Note in particular that the scale
/// line is preserved rather than incorporated into the structure.
///
/// This type brings simplicity at the cost of stability.
/// **Be prepared for breakage** as more fields are added;
/// for now, you are advised to limit your usage of this type to
/// self-contained functions. (e.g. conversions to and from a
/// datatype of your own)
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

/// Covers all the reasons why `RawPoscar::validate` might get mad at you.
///
/// The variants are public so that by looking at the docs you can see all the possible errors.
/// That said, you have no good reason to write code that matches on this.
///
/// ...right?
#[derive(Debug, Fail)]
pub enum ValidationError {
    /// The comment line is more than one line.
    #[fail(display = "the comment may not contain a newline")]
    NewlineInComment,
    /// A requirement on `group_symbols` was violated.
    ///
    /// There are a few more restrictions in addition to the no-leading-digit
    /// restriction mentioned in format.md, in order to ensure roundtripping:
    /// * A symbol may not be the empty string
    /// * A symbol may not contain whitespace
    // (NOTE: `None` when the specific problematic symbol could not be identified.)
    #[fail(display = "invalid symbol in group_symbols: {:?}", _0)]
    InvalidSymbol(Option<String>),
    /// Poscar is required to have at least one atom.
    /// (this is to enable possible support for pymatgen-style labels in the future)
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
    #[fail(display = "predictor corrector has an init value of 0")]
    PredictorCorrectorInitIsZero,

    #[doc(hidden)]
    #[fail(display = "something absurd happened and you're not supposed to see this")]
    AndManyMooooooooore,
}

fn _check_conv() {
    fn panic<T>() -> T { panic!() }
    let e: ValidationError = panic();
    let _: ::failure::Error = e.into();
}

impl RawPoscar {
    /// Convert into a `Poscar` object after checking its invariants.
    ///
    /// To see what those invariants are, check the docs for ValidationError.
    // TODO: Link
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

/// Represents the second line in a POSCAR file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleLine {
    Factor(f64),
    Volume(f64),
}

/// Represents data that can either be in direct units or cartesian.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Coords<T=Vec<[f64; 3]>> {
    Cart(T),
    Frac(T),
}

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
