
// (Invariant: There is at least one atom.)
#[derive(Debug, Clone)]
pub struct Poscar(pub(crate) RawPoscar);

impl Poscar {
    /// Convert into a form with public data members that you can freely match
    /// against and unpack.
    ///
    /// When you are done modifying the object, you may call `.validate()`
    /// to turn it back into a Poscar. (or you can keep all the data for yourself
    /// if you want!)
    ///
    /// Currently, this is the most versatile way of manipulating a Poscar object,
    /// though it may not be the most stable or convenient. In the future, more
    /// helper operations may be provided on `Poscar`.
    pub fn raw(self) -> RawPoscar { self.0 }

    // pub fn comment(&self) -> &str { &self.comment }

    // /// Get the three lattice vectors, taking into account the scale line.
    // pub fn lattice_vectors(&self) -> [[f64; 3]; 3] { TODO }

    // /// Determinant of the lattice matrix.
    // pub fn determinant(&self) -> f64 { TODO }

    // /// Unit cell volume. This quantity is always positive.
    // pub fn volume(&self) -> f64 { self.determinant().abs() }

    // fn effective_scale_factor(&self) -> f64 {
    //     match self.scale {
    //         ScaleLine::Factor(f) => f,
    //         ScaleLine::Volume(v) => (v / self.volume()).powf(1f64/3f64),
    //     }
    // }

    // // NOTE: These are dangerous because Carts does not incorporate the scale factor
    // // pub fn coords(&self) -> Coords<&[[f64; 3]]> { TODO }
    // // pub fn coords_mut(&self) -> Coords<&mut [[f64; 3]]> { TODO }
    // pub fn cart_coords(&self) -> Vec<[f64; 3]> { TODO }
    // pub fn frac_coords(&self) -> Vec<[f64; 3]> { TODO }

    // pub fn velocities(&self) -> Option<Coords<Vec<[f64; 3]>>> { TODO }
    // pub fn symbols(&self) -> Option<Symbols> { TODO }
    // pub fn symbol_groups(&self) -> Option<SymbolGroups> { TODO }

}

/// Basic representation of a POSCAR with public data members.
///
/// The mapping between its fields and those of the the POSCAR file
/// should be braindead obvious.  Note in particular that the scale
/// line is preserved rather than incorporated into the structure
///
/// All members are public to allow you to construct it.
/// Be prepared for breakage as more fields are added;
/// you are advised to limit your usage of this type to small,
/// self-contained functions.
#[derive(Debug, Clone)]
pub struct RawPoscar {
    pub comment: String,
    pub scale: ScaleLine,
    pub lattice_vectors: [[f64; 3]; 3],
    pub group_symbols: Option<Vec<String>>,
    pub group_counts: Vec<usize>,
    pub coords: Coords,
    pub velocities: Option<Coords>,
    pub dynamics: Option<Vec<[bool; 3]>>,
    // pub predictor_corrector: Option<PredictorCorrector>,
}

// TODO move into Github issue
// FIXME my attempts to get VASP to parse data in this format have failed.
//       I don't know what's wrong, but VASP seems to be expecting more than
//         4 + 3*N lines (this is including the blank line) to be present.
// /// This is a large section of data that may appear at the end of a CONTCAR.
// struct PredictorCorrector {
//     /// The first line of the predictor corrector.
//     ///
//     /// It appears from the VASP source code that this should always be 1.
//     /// But if you know better, you can do whatever.
//     ///
//     /// Vasp considers a value of 0 to mean that no predictor corrector is present
//     /// (it skips the lines that parse the predictor corrector). The purpose is not
//     /// exactly clear, but it may be keep open the possibility of having another
//     /// optional field after the predictor corrector.
//     ///
//     /// Setting this to zero is forbidden (`RawPoscar::validate` will fail).
//     /// You should set `predictor_corrector` to `None` instead.
//     pub init: i64,

//     /// The second line of the predictor corrector.
//     ///
//     /// It seems that VASP writes the value of POTIM here for validation purposes,
//     /// so that it may warn you if the value has changed. Good on them!
//     pub potim: f64,

//     /// The third line of the predictor corrector.
//     ///
//     /// We believe it does sciency science things.
//     pub nose: f64,

//     /// Three Nx3 arrays.
//     ///
//     /// The author of this crate has no idea what they are for, but almost assuredly
//     /// they make dreams come true.
//     pub data: (Vec<[f64; 3]>, Vec<[f64; 3]>, Vec<[f64; 3]>),
// }

/// Covers all the reasons why `RawPoscar::validate` might get mad at you.
///
/// The variants are public so that by looking at the docs you can see all the possible errors.
/// You have no good reason to write code that matches on this.
///
/// ...right?
#[derive(Debug, Fail)]
pub enum ValidationError {
    /// The comment line is more than one line.
    #[fail(display = "the comment may not contain a newline")]
    NewlineInComment,
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

        if self.coords.as_ref().raw().len() != n {
            g_bail!(ValidationError::WrongLength("coords", n));
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

// /// Modification of the scale line while preserving the overall structure.
// ///
// /// Preserves:
// /// * `lattice()` (up to numerical precision)
// /// * `frac_coords()` (identically if they're stored; up to numerical precision otherwise)
// /// * `cart_coords()` (up to numerical precision)
// /// * `raw_lattice_vectors()` (except in `normalize_scale`)
// /// * `raw_velocities()`
// ///
// /// May affect:
// /// * scale line (duh)
// /// * raw lattice vectors
// /// * raw cartesian coordinates
// /// * `cart_velocities()` (if the stored velocities are direct)
// /// * `frac_velocities()` (if the stored velocities are cartesian)
// ///
// /// (the velocities are not directly touched, but conversions are affected
// ///  by the fact that cartesian velocities ignore the scale factor.
// ///  Call `use_frac_velocities()` or `use_cart_velocities()` prior to
// ///  the method if you want to select which one should be preserved)
// impl Poscar {
//     /// If the scale line is a volume, turn it into a factor.
//     pub fn use_factor_scale(&mut self) -> &mut Self { TODO }

//     /// If the scale line is a factor, turn it into a volume.
//     pub fn use_volume_scale(&mut self) -> &mut Self { TODO }

//     /// Incorporate the scale into the lattice and cartesian data, so that the scale line is now "1.0".
//     pub fn normalize_scale(&mut self) -> &mut Self {
//         if let &mut Coords::Carts(ref mut c) = &mut self.coords {
//             TODO
//         };
//         self.lattice = self.lattice_vectors();
//         self.scale = ScaleLine::Factor(1.0);
//         self
//     }
// }

// /// Conversion between cartesian and direct.
// impl Poscar {
//     /// Ensure that the coords are cartesian.
//     pub fn use_cart_coords(&mut self) -> &mut Self { TODO } // scale by scaled lattice
//     /// Ensure that the coords are direct.
//     pub fn use_frac_coords(&mut self) -> &mut Self { TODO }

//     /// Ensure that the velocities, if present, are cartesian.
//     pub fn use_cart_velocities(&mut self) -> &mut Self { TODO } // scale by raw lattice
//     /// Ensure that the velocities, if present, are direct.
//     pub fn use_frac_velocities(&mut self) -> &mut Self { TODO }
// }

// impl Poscar {
//     /// Get the three lattice vector lines as written, without applying the scale factor.
//     #[inline] pub fn raw_lattice_vectors(&self) -> [[f64; 3]; 3] { self.lattice }

//     /// Set the three lattice vector lines without touching the scale line.
//     ///
//     /// You are allowed to set lattice vectors that produce a determinant that
//     /// is zero or even negative; they will be left as is. Be aware of the fact
//     /// that VASP prohibits vectors with a negative determinant, and that other
//     /// poorly-written software may do things to your structure (such as changing
//     /// its chirality) in an attempt to get rid of negative determinants.
//     #[inline] pub fn set_raw_lattice_vectors(&self, vectors: &[[f64; 3]; 3]) { self.lattice = *vectors; }

//     /// Get the scale line as written (either a scale or the overall volume).
//     #[inline] pub fn scale_line(&self) -> ScaleLine { self.scale }

//     /// Set the scale line.
//     ///
//     /// # Panics
//     ///
//     /// Panics if the value (be it volume or scale) is not strictly positive.
//     #[inline] pub fn set_scale_line(&mut self, scale: ScaleLine) {
//         match scale {
//             ScaleLine::Factor(raw) |
//             ScaleLine::Volume(raw) => assert!(raw > 0.0, "scale value is not positive"),
//         };
//         self.scale = scale;
//     }
// }

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
    pub(crate) fn map<B, F>(self, f: F) -> Coords<B>
    where F: FnOnce(A) -> B,
    { match self {
        Coords::Cart(x) => Coords::Cart(f(x)),
        Coords::Frac(x) => Coords::Frac(f(x)),
    }}

    pub(crate) fn as_ref(&self) -> Coords<&A>
    { match *self {
        Coords::Cart(ref x) => Coords::Cart(x),
        Coords::Frac(ref x) => Coords::Frac(x),
    }}

    pub(crate) fn as_mut(&mut self) -> Coords<&mut A>
    { match *self {
        Coords::Cart(ref mut x) => Coords::Cart(x),
        Coords::Frac(ref mut x) => Coords::Frac(x),
    }}

    pub(crate) fn raw(self) -> A
    { match self {
        Coords::Cart(x) => x,
        Coords::Frac(x) => x,
    }}
}
