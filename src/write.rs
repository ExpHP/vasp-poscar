// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use crate::{Poscar, RawPoscar, ScaleLine, Coords};

impl fmt::Display for Poscar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    { crate::write::display(f, self) }
}

fn display(w: &mut fmt::Formatter<'_>, poscar: &Poscar) -> fmt::Result
{
    let &Poscar(RawPoscar {
        scale, ref lattice_vectors, ref velocities, ref dynamics,
        ref comment, ref positions, ref group_counts, ref group_symbols,
        _cant_touch_this: (),
    }) = poscar;

    assert!(!comment.contains("\n"), "BUG");
    assert!(!comment.contains("\r"), "BUG");

    let style = FloatStyle::new(w);

    writeln!(w, "{}", comment)?;
    write!(w, "  ")?;
    match scale {
        ScaleLine::Factor(x) => {
            style.write_f64(w, x)?;
        },
        ScaleLine::Volume(x) => {
            write!(w, "-")?;
            style.write_f64(w, x)?;
        },
    }
    writeln!(w)?;

    for row in lattice_vectors {
        write!(w, "    ")?;
        style.write_v3(w, *row)?;
        writeln!(w)?;
    }

    if let Some(group_symbols) = group_symbols.as_ref() {
        write!(w, "  ")?;
        write_sep(&mut *w, " ", group_symbols.iter().map(|s| format!("{:>2}", s)))?;
        writeln!(w)?;
    }

    assert!(!group_counts.is_empty(), "BUG");
    write!(w, "  ")?;
    write_sep(&mut *w, " ", group_counts.iter().map(|&c| format!("{:>2}", c)))?;
    writeln!(w)?;

    if let &Some(_) = dynamics {
        writeln!(w, "Selective Dynamics")?;
    }

    match positions {
        &Coords::Cart(_) => writeln!(w, "Cartesian")?,
        &Coords::Frac(_) => writeln!(w, "Direct")?,
    }

    let positions = positions.as_ref().raw();
    for (i, pos) in positions.iter().enumerate() {
        write!(w, "  ")?;
        style.write_v3(w, *pos)?;
        if let &Some(ref dynamics) = dynamics {
            let fmt = |b| match b { true => 'T', false => 'F' };
            write!(w, " {}", By3(dynamics[i], fmt))?;
        }
        writeln!(w)?;
    }

    if let &Some(ref velocities) = velocities {
        match velocities {
            &Coords::Cart(_) => writeln!(w, "Cartesian")?,
            // (NOTE: typical appearance in CONTCAR; pymatgen expects this form)
            &Coords::Frac(_) => writeln!(w, "")?,
        }

        let velocities = velocities.as_ref().raw();
        for v in velocities {
            write!(w, "  ")?;
            style.write_v3(w, *v)?;
            writeln!(w)?;
        }
    }

    Ok(())
}

fn write_sep<W, Xs>(mut w: W, sep: &str, xs: Xs) -> fmt::Result
where
    W: fmt::Write,
    Xs: IntoIterator,
    Xs::Item: fmt::Display,
{
    let mut xs = xs.into_iter();
    if let Some(x) = xs.next() {
        write!(&mut w, "{}", x)?;
    }
    for x in xs {
        write!(&mut w, "{}{}", sep, x)?;
    }
    Ok(())
}

#[derive(Copy, Clone)]
enum FloatStyle { Dtoa, ForwardDisplay }
impl FloatStyle {
    fn new(f: &fmt::Formatter) -> Self {
        // Use Dtoa only for {}.
        if f.width().is_some() || f.precision().is_some() || f.sign_plus() || f.alternate() {
            FloatStyle::ForwardDisplay
        } else {
            FloatStyle::Dtoa
        }
    }

    // (directly implemented as a fn() -> fmt::Result instead of a Display type
    //  to ensure that we never forget to forward the flags)
    fn write_f64(self, f: &mut fmt::Formatter<'_>, value: f64) -> fmt::Result {
        match self {
            FloatStyle::ForwardDisplay => {
                fmt::Display::fmt(&value, f)
            },
            FloatStyle::Dtoa => {
                // not the most efficient thing in the world...
                let mut bytes = vec![];
                dtoa::write(&mut bytes, value).map_err(|_| fmt::Error)?;
                f.write_str(&String::from_utf8(bytes).unwrap())
            },
        }
    }

    fn write_v3(self, f: &mut fmt::Formatter<'_>, value: [f64; 3]) -> fmt::Result {
        self.write_f64(f, value[0])?;
        write!(f, " ")?;
        self.write_f64(f, value[1])?;
        write!(f, " ")?;
        self.write_f64(f, value[2])?;
        Ok(())
    }
}

// Formats three space-separated tokens after applying a conversion function to each.
struct By3<A, F>([A; 3], F);
impl<A, B, F> fmt::Display for By3<A, F>
where A: Clone,
      F: Fn(A) -> B,
      B: fmt::Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let By3(ref arr, ref f) = *self;
        write!(fmt, "{} {} {}", f(arr[0].clone()), f(arr[1].clone()), f(arr[2].clone()))
    }
}
