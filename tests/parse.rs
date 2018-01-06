

extern crate poscar;

use ::std::io;
use ::std::io::prelude::*;
use ::std::fs::{self, DirEntry};
use ::std::path::{Path, PathBuf};


fn main() { _main().unwrap(); }
fn _main() -> io::Result<()> {
    let tests = collect_tests("tests/parse".as_ref())?;

    println!("running {} tests", tests.len());

    let mut failures = vec![];
    for test in &tests {
        print!("file {}...", test.in_path.display());
        match test.run_opt() {
            None => println!(" ok"),
            Some(e) => {
                failures.push(e);
                println!(" BOOM!");
            },
        }
    }

    for failure in &failures {
        println!();
        println!(" ------ file {} FAILED! ------", failure.0.in_path.display());
        println!("Err: {:#?}", failure.1);
    }

    match failures.len() {
        0 => Ok(()),
        n => panic!("{} test(s) failed!", n),
    }
}

struct TestSpec {
    in_path: PathBuf,
    out_path: PathBuf,
}

fn collect_tests(dir: &Path) -> io::Result<Vec<TestSpec>> {
    let mut out = vec![];
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let entry = entry.path();
        if entry.extension() == Some("in".as_ref()) {
            out.push(TestSpec {
                in_path: entry.to_owned(),
                out_path: entry.with_extension("out"),
            });
        }
    }
    Ok(out)
}

struct Failure<'a>(&'a TestSpec, Error);

#[derive(Debug)]
enum Error {
    Io(io::Error), // u prolly typod a file
    Error(::poscar::failure::Error), // parse error
    Mismatch {
        bonafide: String, // a.k.a. "actual", but 8 letters long
        expected: String,
    },
}

impl TestSpec {
    fn run_opt(&self) -> Option<Failure> {
        match self.run() {
            Ok(()) => None,
            Err(e) => Some(Failure(self, e)),
        }
    }

    fn run(&self) -> Result<(), Error> {
        let in_file = fs::File::open(&self.in_path).map_err(Error::Io)?;
        let mut in_file = io::BufReader::new(in_file);
        let poscar = ::poscar::from_reader(&mut in_file).map_err(Error::Error)?;

        // We serialize back into text before comparing against the expected.
        // This has the advantage that a parser bug cannot inadvertently
        //   affect 'bonafide' and 'expected' in the same way.
        // It has the disadvantage that our "parse" tests are sensitive
        //   to changes in the output format.
        //
        // I suspect that an automatic outfile-generating script and careful
        // review of git diffs should be good enough to work around that disadvantage.
        let mut out_file = fs::File::open(&self.out_path).map_err(Error::Io)?;

        let mut expected = vec![];
        out_file.read_to_end(&mut expected);

        let mut bonafide = vec![];
        ::poscar::to_writer(&mut bonafide, &poscar);

        if expected != bonafide {
            return Err(Error::Mismatch {
                bonafide: String::from_utf8(bonafide).unwrap(),
                expected: String::from_utf8(expected).unwrap(),
            });
        }

        Ok(())
    }
}
