use ::std::io;
use ::std::fmt;
use ::std::io::prelude::*;
use ::{Poscar, RawPoscar, ScaleLine, Coords};

/// Writes a POSCAR to an open file.
pub fn to_writer<W>(
    mut w: W,
    poscar: &Poscar,
) -> io::Result<()>
where W: Write
{
    let &Poscar(RawPoscar {
        scale, ref lattice_vectors, ref velocities, ref dynamics,
        ref comment, ref coords, ref group_counts, ref group_symbols,
    }) = poscar;

    assert!(!comment.contains("\n"), "BUG");
    assert!(!comment.contains("\r"), "BUG");

    writeln!(&mut w, "{}", comment)?;
    match scale {
        ScaleLine::Factor(x) => writeln!(&mut w, " {}", Dtoa(x))?,
        ScaleLine::Volume(x) => writeln!(&mut w, " -{}", Dtoa(x))?,
    }

    for row in lattice_vectors {
        writeln!(&mut w, "  {}", By3(*row, Dtoa))?;
    }

    if let Some(group_symbols) = group_symbols.as_ref() {
        for sym in group_symbols { write!(&mut w, " {}", sym)?; }
        writeln!(&mut w)?;
    }

    assert!(!group_counts.is_empty(), "BUG");
    for count in group_counts { write!(&mut w, " {}", count)?; }
    writeln!(&mut w)?;

    if let &Some(_) = dynamics {
        writeln!(&mut w, "Selective Dynamics")?;
    }

    match coords {
        &Coords::Cart(_) => writeln!(&mut w, "Cartesian")?,
        &Coords::Frac(_) => writeln!(&mut w, "Direct")?,
    }

    let coords = coords.as_ref().raw();
    for (i, c) in coords.iter().enumerate() {
        write!(&mut w, "  {}", By3(*c, Dtoa))?;
        if let &Some(ref dynamics) = dynamics {
            let fmt = |b| match b { true => 'T', false => 'F' };
            write!(&mut w, " {}", By3(dynamics[i], fmt))?;
        }
        writeln!(&mut w)?;
    }

    if let &Some(ref velocities) = velocities {
        match velocities {
            &Coords::Cart(_) => writeln!(&mut w, "Cartesian")?,
            // (NOTE: typical appearance in CONTCAR; pymatgen expects this form)
            &Coords::Frac(_) => writeln!(&mut w, "")?,
        }

        let velocities = velocities.as_ref().raw();
        for v in velocities {
            writeln!(&mut w, "  {}", By3(*v, Dtoa))?;
        }
    }

    Ok(())
}

struct Dtoa(f64);
impl fmt::Display for Dtoa {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // not the most efficient thing in the world...
        let mut bytes = vec![];
        ::dtoa::write(&mut bytes, self.0).map_err(|_| fmt::Error)?;
        f.write_str(&String::from_utf8(bytes).unwrap())
    }
}

// Formats three space-separated tokens after applying a conversion function to each.
// Merely having this around makes it easier to remember to use Dtoa.
struct By3<A, F>([A; 3], F);
impl<A, B, F> fmt::Display for By3<A, F>
where A: Clone,
      F: Fn(A) -> B,
      B: fmt::Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let By3(ref arr, ref f) = *self;
        write!(fmt, "{} {} {}", f(arr[0].clone()), f(arr[1].clone()), f(arr[2].clone()))
    }
}
