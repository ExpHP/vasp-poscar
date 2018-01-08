# POSCAR format for Rust

<!-- TODO badges, link to documentation -->

A parser and printer for the POSCAR file format for representing crystallographic compounds.  This is primarily an input file format used by the [Vienna Ab initio Simulation Package (VASP)](https://www.vasp.at/), which has become fairly well-supported by a wide variety of software related to crystallography and molecular dynamics.

```text
cubic diamond
  3.7
    0.5 0.5 0.0
    0.0 0.5 0.5
    0.5 0.0 0.5
   C
   2
Direct
  0.0 0.0 0.0
  0.25 0.25 0.25
```

## Usage

**Cargo.toml**

```toml
[dependencies]
vasp-poscar = "0.1.0"
```

```rust
extern crate vasp_poscar;

use vasp_poscar::ScaleLine;

// read from a BufRead instance, such as &[u8] or BufReader<File>
let file = io::BufReader::new(File::open("POSCAR")?);
let poscar = vasp_poscar::from_reader(file)?;

// get a RawPoscar object with public fields you can freely match on and manipulate
let mut poscar = poscar.raw();

assert_eq!(poscar.scale, ScaleLine::Factor(3.7));

poscar.comment = "[Subject Name Here] was here".into();
poscar.scale = ScaleLine::Volume(10.0);

// Turn back into a Poscar for writing
let poscar = poscar.validate()?;

let mut buffer = vec![];
vasp_poscar::to_writer(&mut buffer, &poscar)?;

assert_eq!(String::from_utf8(buffer)?, "\
[Subject Name Here] was here
  -10.0
    0.5 0.5 0.0
    0.0 0.5 0.5
    0.5 0.0 0.5
   C
   2
Direct
  0.0 0.0 0.0
  0.25 0.25 0.25
");
```

## Notes about the format

The POSCAR format is primarily equipped with two to three key pieces of information:

* A **periodic lattice**.
  * The structure in a POSCAR is *always* periodic in 3 dimensions.
    Lower-dimensional structures are represented approximately by assigning a long periodic length *(a "vacuum separation")* to the aperiodic directions.
* A set of **coordinates** for sites in a unit cell.
  * These can be cartesian (like an XYZ file) or "direct" (in units of the lattice vectors).
* (From Vasp 5 onwards) The **elemental symbols** associated with the sites.

One can contrast with XYZ files, which are only capable of representing aperiodic structures.

POSCAR has some optional sections that are probably mostly really only used by VASP itself.

* Velocities
* Selective dynamics, which allows sites to be constrained to movement along a subset of the lattice vectors.
* The... um... *"predictor corrector."*

## Scope of this crate

`vasp-poscar` is primarily a backend-level crate for *reading and writing a file format.*  It aims to provide:

* **reasonable diagnostics** on malformed files, with special consideration given to errors that are easy to make
* **round-trippable precision**; when a POSCAR is read in, it does not automatically absorb the scale into the lattice matrix, or convert everything into its favorite representation (direct vs cartesian). *Those are __your__ decisions to make!* In the meanwhile, the in-memory representation of the poscar leaves everything exactly as is (modulo nits like case and whitespace), so that nary even a negative zero is lost when it is written back to a file.

`vasp-poscar` is secondarily a crate for *managing the redundant forms of data that exist within the file.*  While not currently implemented, it is eventually planned for this crate to provide basic facilities for:

* Obtaining the scaled lattice and *true* cartesian coordinates
* Converting between direct and cartesian representations
* Manipulating the scale and lattice with respect to each other (e.g. switching between scale and volume, or absorbing the scale into the lattice)

`vasp-poscar` is **not really a crate for doing science.**  It will *never* provide things like symmetry analysis, primitive structure search, supercell construction, perturbation of positions, or cutting across a plane, etc. It won't tell you what the masses of your ions are, and it *most certainly won't __ever__ attempt to automatically search for a file called `POTCAR` in the same directory to figure out things like atomic numbers.* (you know who you are...).

These things are simply not its job.  The expectation is that the data read by `vasp-poscar` may be used to construct an instance of a more versatile---and more opinionated---`Structure` type implemented in another crate.  (of course, if you are designing such a type, you are invited to depend on this crate as a parsing backend!)
