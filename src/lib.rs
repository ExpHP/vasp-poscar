//!
//!
//!
//! TODO TODO
//!
//!  * `vasp-poscar` is **round-trippable.** TODO TODO
//!  * `vasp-poscar` is **unopinionated.** (Or at least, it tries to be.)
//!    It does not wrap your positions into the unit cell.
//!    It does not TODO TODO
//!
//! ## What `vasp-poscar` is **not:**
//!
//! `vasp-poscar` is **not** a general-use data type for manipulating
//! crystallographic structures.  You can *try,* but for most operations
//! it will be uncomfortable.  At the end of the day, `poscar`'s sole
//! purpose is to serve as an interface to a *file format.*
//!
//! It is intended to be used as a stepping stone between POSCAR files
//! and a more user-friendly `Structure` type (which might depend on
//! this crate privately as a parsing backend).
//!
//! ## POSCAR format
//!
//! The POSCAR format is phenomenally underspecified, and there is a
//! lot of false information about it on the web.  This section aims to
//! clarify the crate author's interpretation of the format, based on
//! reviewing the behavior of other libraries for VASP interop, and
//! checking against the VASP 5.4.1 source code.
//!
//! ### Structural elements
//!
//! This is an attempt to describe the key building blocks of the POSCAR format.
//!
//! #### Primitives
//!
//! The VASP documentation does not document what it expects most primitives
//! to look like, leaving every implementation of the format to fend for itself.
//! Needless to say, no two implementations are alike.
//!
//! In actuality, the implementation of VASP uses FORTRAN's `read(*)` for almost
//! all of its parsing. As a result, the *actual* set of inputs accepted by VASP
//! is far, far greater than what most might expect, allowing things such as
//! optional commas between fields, "0.0*3" for repetitions,
//! or ".tiddlyWinks" as a selective dynamics flag.
//!
//! But there is no compelling reason for this crate to support all of these
//! intricacies when nobody will use them.  Therefore, this crate defines the
//! format of each primitive as follows:
//!
//! * All primitives are understood to be **separated by spaces or tabs**.
//!   The rest of `read(*)`'s wild syntax is not supported.
//! * An **integer** is whatever can be parsed using `std::str::FromStr`.
//! * A **real** is whatever can be parsed using `std::str::FromStr`.
//! * A **logical** is parsed [like `read(*)` does](https://docs.oracle.com/cd/E19957-01/805-4939/6j4m0vnc5/index.html),
//!   which amounts to the regex `\.?[tTfF].*`.
//!
//! For best compatibility with other low-quality implementations, you would
//! be wise to follow the following limitations:
//!
//! * Please do not prefix integers with a leading zero.
//! * Please only use "T" and "F" for logicals.
//!
//! #### "Comments"
//!
//! Let us define a comment as *any arbitrary freeform text at the end of
//! a line after the parts that VASP actually cares about.*
//! If that definition terrifies you, it *should!*
//!
//! If you look at some of the examples in VASP's own documentation,
//! you'll find plain english text right next to actual meaningful data.
//! (such as in the example input for everybody's favorite structure,
//! "cubic&nbsp;diamond&nbsp;&nbsp;&nbsp;&nbsp;comment&nbsp;line")
//! Needless to say, the `poscar` crate makes large sacrifices in terms
//! of diagnostic quality in order to be able to parse these files.
//!
//! But that shouldn't surprise you. This describes virtually every
//! VASP compatibility library ever. All this is merely justification
//! for why this crate is so tolerant of seemingly malformed input, in
//! stark contrast with the ideology of rust.
//!
//! #### Flag lines
//!
//! A "flag line" is one whose **very first character**
//! (deemed the control character) is significant.
//! **Spaces count. Do not indent these lines!**
//!
//! A flag line can be empty. One could say the control character is some
//! out-of-band value like `None` in this case.
//!
//! *The rest of the line after the control character regarded is a comment.*
//!
//! ### Structure
//!
//! #### The comment
//!
//! The first line is known as **the comment**. (distinct from *a* comment)
//! It can contain anything.
//!
//! #### Scale line
//!
//! The scale line is a single real.
//! *Anything after this is a comment.*
//!
//! If it is positive, it is interpreted as a global scaling factor.
//! If it is negative, it is interpreted as a target unit cell volume.
//! It may not be zero.
//!
//! #### Lattice lines
//!
//! Three lines, each with three reals.
//! *The rest of each line is a comment.*
//!
//! #### Symbols and counts
//!
//! * Symbols line (optional)
//! * Counts line
//!
//! ```text
//!   Si O
//!   24 8  freeform comment
//! ```
//!
//! The optional symbols line before the counts line is detected by
//! checking if the first non-whitespace character is a digit.
//!
//! Every whitespace-separated token on the symbols line is regarded as a symbol.
//!
//! Each whitespace-separacted word on the counts line up *until the first word
//! which does not parse as an integer* is regarded as a count.  *The rest of the
//! line is a freeform comment.*
//!
//! If symbols are provided, the number of counts and symbols must match.
//!
//! Symbols are not validated by this crate, which will accept any arbitrary
//! string without whitespace. Knowing the periodic table is considered "out of scope"
//! for this crate.
//!
//! * It is forbidden for the total atom count to be zero.
//!   (this is in anticipation of eventual support for pymatgen-style symbols
//!    embedded in the coordinate data comments)
//! * It follows that the number of counts also must not be zero.
//! * It is *discouraged* for any of the individual counts to be zero.
//!
//! ### Position data and selective dynamics
//!
//! * Selective dynamics line (optional)
//! * Coordinate system line
//! * Data lines
//!
//! ```text
//! Selective dynamics
//! Cartesian
//!   0.0 0.0 0.0 T T F
//!   0.2 0.2 0.2 T T T
//! ```
//!
//! The first is an optional [flag line] whose control character is `S` or `s`.
//! The second flag line is interpreted as follows:
//! * A control character in the string `"cCkK"` means cartesian coords.
//! * Anything else means direct coordinates.
//!   * This includes an empty line. (the CONTCAR file produced by VASP actually does this!!)
//!   * This even includes a line like `"   Cartesian"`.  I am so, so sorry.
//!     The most this crate can do in such cases is to produce a warning via `log!`.
//!
//! (FIXME make implementation match)
//! (FIXME use log)
//!
//! Each data line begins with **three reals**.
//! If Selective dynamics is enabled, then these are followed by **three logicals**.
//! *The rest of the line is a comment.*
//!
//! As stated earlier, this crate parses logicals using the grammar of Fortran's `read(*)`.
//! It will accept input such as `"T"`, `"f"`, `".TRUE."` or `".T"`.
//! When writing files it will always print `"T"` or `"F"`.
//!
//! It is strongly recommended that you always **use** `T` and `F` as well, for
//! greatest compatibility with other libraries. In a brief review of other implementations,
//! it was found that both ASE and pymatgen parse these flags in dangerous ways that make
//! absolutely no attempt to validate their assumptions about the input.
//!
//! ### Velocities
//!
//! * Coordinate system line
//! * Data lines
//!
//! These may optionally appear after the coordinates. (without a separating blank line)
//! The first line is just like the one for positions.
//! Each data line has three reals. *The rest is a freeform comment.*
//!
//! When using velocities, your chances of interop with other libraries looks pretty slim.
//! In a review of other implementations, it was found that `pymatgen` expects a blank line
//! instead of the coordinate system control line. (N.B. this means that its velocities are
//! always in direct coordinates!). ASE does not even support velocities.

// FIXME Despite my best efforts I cannot seem to actually get VASP to parse this.
//       It keeps triggering segmentation faults on unexpected EOF.
//       I've looked the VASP source up and down and simply cannot see how
//        this is happening.
// //! #### Predictor corrector (only if velocities are present)
// //!
// //! * Blank line
// //! * "Init" line
// //! * Timestep line
// //! * Nose parameter line
// //! * Data lines (3 * N of them)
// //!
// //! This entire section is optional.
// //!
// //! *The "blank line" may be a freeform comment.*
// //!
// //! The next three lines each have one primitive. *The rest of each line is a comment.*
// //!
// //! * The init line is an integer.  If it is 0, then the rest of the predictor corrector
// //!   is taken to be not present.  It appears that this value should always be 1.
// //! * The timestep line is a real.  It stores the value of `POTIM`.
// //! * The nose parameter line is a real.
// //!
// //! Each data line has 3 reals. Freeform comments are **not allowed** here.

#[macro_use]
pub extern crate failure;
extern crate dtoa;

// general bail! and ensure! macros that don't constrain the type to failure::Error
macro_rules! g_bail { ($e:expr $(,)*) => { return Err($e.into()); }; }
macro_rules! g_ensure { ($cond:expr, $e:expr $(,)*) => { if !$cond { g_bail!($e); } }; }


mod parse;
mod types;
mod write;

pub use types::*;
pub use parse::{from_path, from_reader};
pub use write::{to_writer};
