use ::{ScaleLine, Coords, RawPoscar, Poscar, ValidationError};
use ::types::{CoordsTag};

/// Allows construction of [`Poscar`]/[`RawPoscar`] via the builder pattern.
///
/// # General notes
///
/// **Working with this API requires you to be familiar with the POSCAR
/// format.** Its setters map almost one-to-one with the sections of a
/// POSCAR file (though there are a few additional conveniences).
///
/// There are two groups of methods:
///
/// * **setters**: e.g. [`positions`], [`comment`], ...
/// * **build methods**: [`build`] and [`build_raw`].
///
/// # Defaults
///
/// All optional fields of a Poscar are disabled by default.
/// Others may have default values or behavior that is detailed in the documentation
/// of the appropriate setter. Some fields, like [`positions`] and [`lattice_vectors`],
/// have *no default*, and failure to set them will result in a panic at runtime in
/// the build method.
///
/// # Panics
///
/// ## Contract violations
///
/// Generally speaking, invalid data provided to the Builder will at worst
/// produce a [`ValidationError`], and even then, it will only do so when
/// building a [`Poscar`]. (building a [`RawPoscar`] performs no validation)
///
/// However, egregious misuse of the Builder API may make it impossible to
/// construct even a [`RawPoscar`]. In this case, the build methods will panic.
/// In particular, the rules are:
///
/// **All required fields must be set:**
///
/// * [`positions`]
/// * [`lattice_vectors`]
///
/// If positions is set to [`Zeroed`], then **[`group_counts`]
/// also becomes required.**
///
/// ## Poisoning
///
/// Calling [`build_raw`] or [`build`] "consumes" the `Builder` in a manner
/// which causes **all future method calls** to panic at runtime.
/// If you wish to reuse a `Builder`, you must clone it before calling
/// one of these methods.
///
/// [`ValidationError`]: struct.ValidationError.html
/// [`Poscar`]: struct.Poscar.html
/// [`RawPoscar`]: struct.RawPoscar.html
/// [`Zeroed`]: struct.Zeroed.html
/// [`positions`]: #method.positions
/// [`comment`]: #method.comment
/// [`lattice_vectors`]: #method.lattice_vectors
/// [`group_counts`]: #method.group_counts
/// [`build_raw`]: #method.build_raw
/// [`build`]: #method.build
#[derive(Debug)]
pub struct Builder(Option<Data>);

#[derive(Debug, Clone)]
struct Data {
    comment: String,
    scale: ScaleLine,
    lattice_vectors: Lattice,
    group_symbols: Symbols,
    group_counts: Counts,
    positions: Positions,
    velocities: Velocities,
    dynamics: Dynamics,
}

/// Generates zero-filled coordinate data.
///
/// Accepted by [`Builder::positions`] and [`Builder::velocities`].
///
/// ```rust
/// use vasp_poscar::{Builder, Zeroed, Coords};
///
/// let raw =
///         Builder::new()
///         .dummy_lattice_vectors()
///         .group_counts(vec![1, 2, 2])
///         .positions(Coords::Frac(Zeroed))
///         .build_raw();
///
/// assert_eq!(
///     Coords::Frac(vec![[0.0; 3]; 5]),
///     raw.positions,
/// );
/// ```
///
/// [`Builder::positions`]: struct.Builder.html#method.positions
/// [`Builder::velocities`]: struct.Builder.html#method.velocities
#[derive(Debug, Copy, Clone)]
pub struct Zeroed;

// NOTE: Custom enums are used to let `None` variants have names more
//       evocative of their meaning:
//
// * A variant called `Missing` should cause a panic.
// * A variant called `None` omits an optional section from the Poscar.
// * Other variants may indicate that the field will be derived based
//   on other fields.

#[derive(Debug, Clone)]
enum Lattice {
    Missing,
    This(Box<[[f64; 3]; 3]>),
}

#[derive(Debug, Clone)]
enum Symbols {
    None,
    These(Vec<String>),
}

#[derive(Debug, Clone)]
enum Counts {
    Auto,
    These(Vec<usize>),
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum Positions {
    Missing,
    Zero(CoordsTag),
    These(Coords<Vec<[f64; 3]>>),
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum Velocities {
    None,
    Zero(CoordsTag),
    These(Coords<Vec<[f64; 3]>>),
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum Dynamics {
    None,
    These(Vec<[bool; 3]>),
}

impl Default for Builder {
    fn default() -> Builder
    { Builder(Some(Data {
        // NOTE: This is reproduced in the doc comment for `comment()`
        comment: "POSCAR File".into(),
        scale: ScaleLine::Factor(1.0),
        lattice_vectors: Lattice::Missing,
        group_symbols: Symbols::None,
        group_counts: Counts::Auto,
        positions: Positions::Missing,
        velocities: Velocities::None,
        dynamics: Dynamics::None,
    }))}
}

const EYE: [[f64; 3]; 3] = [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];

pub use self::positions::PositionsArgument;
mod positions {
    use super::*;

    pub trait PositionsArgument: Sealed { }

    pub(crate) use self::private::Sealed;
    mod private {
        use super::*;

        #[doc(hidden)]
        pub trait Sealed {
            fn _get(self) -> Positions;
        }

        impl<T: AsRef<[[f64; 3]]>> Sealed for Coords<T> {
            fn _get(self) -> Positions { Positions::These(self.map(|v| v.as_ref().to_vec())) }
        }

        impl Sealed for Coords<Zeroed> {
            fn _get(self) -> Positions { Positions::Zero(self.tag()) }
        }
    }

    impl<T: AsRef<[[f64; 3]]>> PositionsArgument for Coords<T> { }
    impl PositionsArgument for Coords<Zeroed> { }
}

pub use self::velocities::VelocitiesArgument;
mod velocities {
    use super::*;

    pub trait VelocitiesArgument: Sealed { }

    pub(crate) use self::private::Sealed;
    mod private {
        use super::*;

        #[doc(hidden)]
        pub trait Sealed {
            fn _get(self) -> Velocities;
        }

        impl<T: AsRef<[[f64; 3]]>> Sealed for Coords<T> {
            fn _get(self) -> Velocities { Velocities::These(self.map(|v| v.as_ref().to_owned())) }
        }

        impl Sealed for Coords<Zeroed> {
            fn _get(self) -> Velocities { Velocities::Zero(self.tag()) }
        }
    }

    impl<T: AsRef<[[f64; 3]]>> VelocitiesArgument for Coords<T> { }
    impl VelocitiesArgument for Coords<Zeroed> { }
}

pub use self::dynamics::DynamicsArgument;
mod dynamics {
    use super::*;

    pub trait DynamicsArgument: Sealed { }

    pub(crate) use self::private::Sealed;
    mod private {
        use super::*;

        #[doc(hidden)]
        pub trait Sealed {
            fn _get(self) -> Dynamics;
        }

        impl<T: AsRef<[[bool; 3]]>> Sealed for T {
            fn _get(self) -> Dynamics { Dynamics::These(self.as_ref().to_vec()) }
        }
    }

    impl<T: AsRef<[[bool; 3]]>> DynamicsArgument for T { }
}

const ALREADY_CONSUMED_MSG: &'static str = "\
    Attempted to use a Builder that has already been consumed! \
    You should clone it before calling the build method.";

impl Builder {
    // panics on poison
    fn as_mut(&mut self) -> &mut Data
    { self.0.as_mut().expect(ALREADY_CONSUMED_MSG) }

    // consume the builder, leaving behind a poison value
    fn take(&mut self) -> Data
    { self.0.take().expect(ALREADY_CONSUMED_MSG) }
}

impl Builder {
    /// Alias for `Default::default()`.
    pub fn new() -> Builder
    { Default::default() }

    // Sets even all required fields to dummy values. For unit tests.
    #[cfg(test)]
    fn new_dumdum() -> Builder
    {
        let mut b = Builder::new();
        b.dummy_lattice_vectors();
        b.positions(Coords::Frac(vec![[0.0; 3]]));
        b
    }
}

impl Builder {
    /// Set the comment line.
    ///
    /// Defaults to "POSCAR File", which you will no doubt agree
    /// is spectacularly exciting.
    pub fn comment<S: Into<String>>(&mut self, s: S) -> &mut Self
    { self.as_mut().comment = s.into(); self }

    /// Set the scale line.
    ///
    /// Defaults to `ScaleLine::Factor(1.0)`.
    pub fn scale(&mut self, s: ScaleLine) -> &mut Self
    { self.as_mut().scale = s; self }

    /// Set the unscaled lattice vectors, as they would be written in the file.
    ///
    /// **This field is required.** The [`build_raw`] and [`build`] methods will panic
    /// unless this method or [`dummy_lattice_vectors`] has been called.
    ///
    /// [`dummy_lattice_vectors`]: #method.dummy_lattice_vectors
    /// [`build_raw`]: #method.build_raw
    /// [`build`]: #method.build
    pub fn lattice_vectors(&mut self, vectors: &[[f64; 3]; 3]) -> &mut Self
    { self.as_mut().lattice_vectors = Lattice::This(Box::new(*vectors)); self }

    /// Set an identity matrix as the lattice.
    ///
    /// You may think of this as an "explicit default".
    /// This may be useful in applications where the lattice given
    /// to the builder will ultimately be discarded.
    pub fn dummy_lattice_vectors(&mut self) -> &mut Self
    { self.as_mut().lattice_vectors = Lattice::This(Box::new(EYE)); self }

    /// Set unscaled positions as they would be written in the Poscar.
    ///
    /// **This field is required.** The [`build_raw`] and [`build`] methods will
    /// panic unless this method has been called.
    ///
    /// The argument should be FIXME.
    /// You may also use `Coords::Cart(Zeroed)` or `Coords::Frac(Zeroed)`
    /// to set dummy values equal in length to the total atom count.
    ///
    /// # Panics
    ///
    /// If [`Zeroed`] is used, then you must also supply [`group_counts`],
    /// or else [`build_raw`] and [`build`] will panic.
    ///
    /// [`Zeroed`]: struct.Zeroed.html
    /// [`build_raw`]: #method.build_raw
    /// [`build`]: #method.build
    /// [`group_counts`]: #method.group_counts
    pub fn positions<V>(&mut self, vs: V) -> &mut Self
    where V: PositionsArgument,
    { self.as_mut().positions = vs._get(); self }

    /// Set velocities as they would be written in the file.
    ///
    /// The argument should be FIXME.
    /// You may also use `Coords::Cart(Zeroed)` or `Coords::Frac(Zeroed)`
    /// to set dummy values equal in length to the total atom count.
    pub fn velocities<V>(&mut self, vs: V) -> &mut Self
    where V: VelocitiesArgument,
    { self.as_mut().velocities = vs._get(); self }

    /// Restore the default behavior.  The poscar will not have velocities.
    pub fn no_velocities(&mut self) -> &mut Self
    { self.as_mut().velocities = Velocities::None; self }

    /// Set explicit counts for each atom type.
    pub fn group_counts<Cs>(&mut self, cs: Cs) -> &mut Self
    where Cs: IntoIterator<Item=usize>,
    { self.as_mut().group_counts = Counts::These(cs.into_iter().collect()); self }

    /// Unset explicit counts, restoring the default behavior.
    ///
    /// By default, it is assumed that all atoms are the same type,
    /// resulting in a single atom type of count `positions.len()`.
    pub fn auto_group_counts(&mut self) -> &mut Self
    { self.as_mut().group_counts = Counts::Auto; self }

    /// Set symbols for each atom type.
    pub fn group_symbols<Cs>(&mut self, syms: Cs) -> &mut Self
    where Cs: IntoIterator, Cs::Item: Into<String>,
    { self.as_mut().group_symbols = Symbols::These(syms.into_iter().map(Into::into).collect()); self }

    /// Default behavior. The poscar will not have group symbols.
    pub fn no_group_symbols(&mut self) -> &mut Self
    { self.as_mut().group_symbols = Symbols::None; self }

    /// Set selective dynamics flags.
    ///
    /// The argument should be TODO.
    pub fn dynamics<V>(&mut self, vs: V) -> &mut Self
    where V: DynamicsArgument,
    { self.as_mut().dynamics = vs._get(); self }

    /// Default behavior. The poscar will not have selective dynamics.
    pub fn no_dynamics(&mut self) -> &mut Self
    { self.as_mut().dynamics = Dynamics::None; self }
}

impl Builder {
    /// Creates a [`Poscar`].
    ///
    /// # Errors
    ///
    /// Conditions for error are documented on the page for
    /// [`ValidationError`].
    ///
    /// # Panics
    ///
    /// This method (and methods called after it) may panic.
    /// See the [toplevel documentation] on `Builder` for more information.
    ///
    /// [toplevel documentation]: #
    /// [`ValidationError`]: struct.ValidationError.html
    /// [`Poscar`]: struct.Poscar.html
    pub fn build(&mut self) -> Result<Poscar, ValidationError>
    { self.build_raw().validate() }

    /// Creates a [`RawPoscar`]
    ///
    /// # Panics
    ///
    /// This method (and methods called after it) may panic.
    /// See the [toplevel documentation] on `Builder` for more information.
    ///
    /// [toplevel documentation]: #
    /// [`RawPoscar`]: struct.RawPoscar.html
    pub fn build_raw(&mut self) -> RawPoscar
    {
        let Data {
            comment, scale, lattice_vectors,
            group_symbols, group_counts,
            positions, velocities, dynamics,
        } = self.take();

        let lattice_vectors = match lattice_vectors {
            Lattice::Missing => panic!("missing required field 'lattice_vectors'"),
            Lattice::This(x) => *x,
        };

        let (positions, group_counts) = match (positions, group_counts) {
            (Positions::Missing, _) => panic!("missing required field 'positions'"),
            (Positions::Zero(_), Counts::Auto) => panic!("cannot determine number of atoms"),
            (Positions::Zero(tag), Counts::These(counts)) => {
                let n = counts.iter().sum();
                let pos = Coords::of_tag(tag, vec![[0f64; 3]; n]);
                (pos, counts)
            },
            (Positions::These(pos), Counts::Auto) => {
                let n = pos.as_ref().raw().len();
                (pos, vec![n])
            },
            (Positions::These(pos), Counts::These(counts)) => (pos, counts),
        };

        // NOTE: We arbitrarily choose to prioritize the value of
        //       `group_counts` over `positions` when length is ambiguous.
        //
        //       We don't validate length mismatch here because
        //       that is the responsibility of `validate()`.
        //       After all, it is possible we are being fed data
        //       from a user file, and our only mode of failure here
        //       is to panic.
        let n_atom = group_counts.iter().sum();

        let group_symbols = match group_symbols {
            Symbols::None => None,
            Symbols::These(v) => Some(v),
        };

        let velocities = match velocities {
            Velocities::None => None,
            Velocities::Zero(tag) => {
                Some(Coords::of_tag(tag, vec![[0f64; 3]; n_atom]))
            },
            Velocities::These(v) => Some(v),
        };

        let dynamics = match dynamics {
            Dynamics::None => None,
            Dynamics::These(v) => Some(v),
        };

        RawPoscar {
            comment, scale, lattice_vectors,
            group_symbols, group_counts,
            positions, velocities, dynamics,
            _cant_touch_this: (),
        }
    }
}

#[cfg(test)]
#[deny(unused)]
mod tests {
    use super::*;

    #[test]
    fn test_comment() {
        assert_eq!(
            "POSCAR File",
            Builder::new_dumdum().build_raw().comment,
        );
        assert_eq!(
            "hello warld",
            Builder::new_dumdum().comment("hello warld").build_raw().comment,
        );
    }

    #[test]
    fn test_scale() {
        assert_eq!(
            ScaleLine::Factor(1.0),
            Builder::new_dumdum().build_raw().scale,
        );
        assert_eq!(
            ScaleLine::Volume(0.5),
            Builder::new_dumdum().scale(ScaleLine::Volume(0.5)).build_raw().scale,
        );
    }

    #[test]
    fn test_lattice() {
        assert_eq!(EYE, Builder::new_dumdum().build_raw().lattice_vectors);

        let m = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [9.0, 8.0, 7.0]];
        assert_eq!(m, Builder::new_dumdum().lattice_vectors(&m).build_raw().lattice_vectors);
    }


    #[test]
    fn test_positions() {
        for coords in vec![
            Coords::Frac(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
            Coords::Cart(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
        ] {
            assert_eq!(
                coords.clone(),
                Builder::new_dumdum().positions(coords).build_raw().positions,
            );
        }

        for (zero, expected) in vec![
            (Coords::Frac(Zeroed), Coords::Frac(vec![[0.0; 3]; 7])),
            (Coords::Cart(Zeroed), Coords::Cart(vec![[0.0; 3]; 7])),
        ] {
            assert_eq!(
                expected,
                Builder::new_dumdum()
                    .group_counts(vec![3, 1, 3])
                    .positions(zero)
                    .build_raw().positions,
            );
        }
    }


    #[test]
    fn test_velocities() {
        use Coords::{Frac, Cart};

        for coords in vec![
            Frac(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
            Cart(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
        ] {
            assert_eq!(
                Some(coords.clone()),
                Builder::new_dumdum().velocities(coords).build_raw().velocities,
            );
        }

        // pos is something that is "clearly not the argument to velocities()"
        // but has the right length
        for (zero, expected, pos) in vec![
            (Frac(Zeroed), Frac(vec![[0.0; 3]; 7]), Cart(vec![[0.5; 3]; 7])),
            (Cart(Zeroed), Cart(vec![[0.0; 3]; 7]), Frac(vec![[0.5; 3]; 7])),
        ] {
            // Zeroed with group_counts.
            // Positions are deliberately given wrong length
            //  to make sure that `build_raw()` doesn't care
            assert_eq!(
                Some(expected.clone()),
                Builder::new_dumdum()
                    .positions(Coords::Cart(vec![[0.0; 3]]))
                    .group_counts(vec![3, 1, 3])
                    .velocities(zero)
                    .build_raw().velocities,
            );

            // Zeroed with auto counts; length comes from positions
            assert_eq!(
                Some(expected.clone()),
                Builder::new_dumdum()
                    .positions(pos.clone())
                    .velocities(zero)
                    .build_raw().velocities,
            );
        }
    }

    #[test]
    fn test_group_counts() {
        assert_eq!(
            vec![4, 2, 5],
            Builder::new_dumdum()
                .dummy_lattice_vectors()
                .group_counts(vec![4, 2, 5])
                .build_raw().group_counts,
        );

        // Auto
        assert_eq!(
            vec![4],
            Builder::new()
                .dummy_lattice_vectors()
                .positions(Coords::Frac(vec![[0.0; 3]; 4]))
                .build_raw().group_counts,
        );
        assert_eq!(
            vec![4],
            Builder::new()
                .dummy_lattice_vectors()
                .group_counts(vec![3])
                .auto_group_counts()
                .positions(Coords::Frac(vec![[0.0; 3]; 4]))
                .build_raw().group_counts,
        );
    }

    #[test]
    fn test_group_symbols() {
        assert_eq!(
            None,
            Builder::new_dumdum()
                .group_counts(vec![4, 2, 5])
                .build_raw().group_symbols,
        );

        assert_eq!(
            Some(vec![format!("A"), format!("B"), format!("C")]),
            Builder::new_dumdum()
                .group_symbols(vec!["A", "B", "C"])
                .build_raw().group_symbols,
        );
    }

    #[test]
    fn test_dynamics() {
        assert_eq!(None, Builder::new_dumdum().build_raw().dynamics);

        assert_eq!(
            Some(vec![[true, true, false]; 6]),
            Builder::new_dumdum()
                .dynamics(vec![[true, true, false]; 6])
                .build_raw().dynamics,
        );
    }

    #[test]
    #[should_panic(expected = "required field 'lattice_vectors'")]
    fn panic_no_lattice_vectors() {
        let _ = Builder::new()
            .positions(Coords::Frac(vec![[1.0, 2.0, 3.0]]))
            .build_raw();
    }

    #[test]
    #[should_panic(expected = "required field 'positions'")]
    fn panic_no_positions() {
        let _ = Builder::new()
            .dummy_lattice_vectors()
            .build_raw();
    }

    #[test]
    #[should_panic(expected = "cannot determine number of atoms")]
    fn panic_no_num_atoms() {
        let _ = Builder::new()
            .dummy_lattice_vectors()
            .positions(Coords::Cart(Zeroed))
            .build_raw();
    }
}
