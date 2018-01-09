// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate vasp_poscar;
#[macro_use]
extern crate serde;
extern crate serde_yaml;

use ::std::fs;
use ::std::path::Path;

use ::vasp_poscar::Poscar;
use ::vasp_poscar::failure::Error as FailError;
use ::vasp_poscar::failure::ResultExt as FailResultExt;

fn main() {
    let tests = collect_tests("tests/parse".as_ref()).unwrap();

    println!("running {} tests", tests.len());

    let name_pad = tests.iter().map(|test| test.basename.len()).max().unwrap().min(32);

    let mut failures = vec![];
    for test in tests {

        print!("  {:>width$}.yaml: ", &test.basename[..], width=name_pad);
        for (i, case) in test.cases.iter().enumerate() {
            match case.run() {
                Ok(()) => print!("."),
                Err(e) => {
                    print!("E");

                    // give the test a Rusty-looking path, just for display purposes
                    let meth = case.name.clone().unwrap_or_else(|| format!("case_{}", i));
                    let path = format!("{}::{}", test.basename, meth).replace("-", "_");
                    failures.push(Failure(path, e));
                },
            }
        }
        println!();
    }

    for failure in &failures {
        println!();
        println!(" ------ test {} FAILED! ------", failure.0);
        println!("Err: {:#?}", failure.1);
    }

    assert_eq!(failures.len(), 0, "a test has failed!");
}

struct TestSpec {
    basename: String,
    cases: Vec<Test>,
}

// Format of test in yaml
#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum RawTest {
    Success { name: Option<String>, input: Text, output: Text },
    Failure { name: Option<String>, input: Text, error: String },
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum Text {
    // usually one big "|"-style YAML string
    Blob(String),
    // form sometimes used so that comments can be embedded
    // and so that the editor can't strip trailing whitespace
    Lines(Vec<String>),
}

impl Text {
    fn into_string(self) -> String
    { match self {
        Text::Blob(s) => s,
        Text::Lines(lines) => lines.join("\n") + "\n",
    }}
}

// Nicer representation of Test
struct Test { name: Option<String>, input: String, kind: TestKind }
enum TestKind { Success(String), Failure(String) }

impl RawTest {
    fn unraw(self) -> Test
    {
        let (name, input) = match self.clone() {
            RawTest::Success { name, input, .. } |
            RawTest::Failure { name, input, .. } => (name, input.into_string()),
        };
        let kind = match self {
            RawTest::Success { output, .. } => TestKind::Success(output.into_string()),
            RawTest::Failure { error, .. } => TestKind::Failure(error),
        };
        Test { name, input, kind }
    }
}


fn collect_tests(dir: &Path) -> Result<Vec<TestSpec>, FailError> {
    let mut out = vec![];
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension() == Some("yaml".as_ref()) {
            let file = fs::File::open(path.as_path())?;
            let cases: Vec<RawTest> = ::serde_yaml::from_reader(file)
                                      .with_context(|_| {
                                          format!("error reading {}", path.as_path().display())
                                      })?;
            let cases = cases.into_iter().map(RawTest::unraw).collect();

            let basename = path.file_stem().unwrap().to_string_lossy().to_string();
            out.push(TestSpec { basename, cases });
        }
    }
    Ok(out)
}

struct Failure(String, Error);

#[derive(Debug)]
enum Error {
    /// Parse error in a parse-succeed test
    Error(FailError),
    /// Output mismatch in a parse-succeed test
    Mismatch {
        bonafide: String, // a.k.a. "actual", but 8 letters long
        expected: String,
    },
    /// Successful parse in a parse-fail test
    NoError,
    /// Could not match error in a parse-fail test
    ErrorMismatch {
        bonafide: String,
        expected: String,
    },
}

impl Test {
    fn run(&self) -> Result<(), Error> {
        let Test { ref input, ref kind, .. } = *self;
        match *kind {
            TestKind::Success(ref expected) => {
                match Poscar::from_reader(input.as_bytes()) {
                    Err(e) => { return Err(Error::Error(e)); },
                    Ok(poscar) => {
                        // We serialize back into text before comparing against the expected.
                        // This has the advantage that a parser bug cannot inadvertently
                        //   affect 'bonafide' and 'expected' in the same way.
                        // It has the disadvantage that our "parse" tests are sensitive
                        //   to changes in the output format.
                        //
                        // I suspect that an automatic outfile-generating script and careful
                        // review of git diffs should be good enough to work around that disadvantage.
                        let bonafide = format!("{}", poscar);
                        let expected = expected.clone();
                        if expected != bonafide {
                            return Err(Error::Mismatch { bonafide, expected });
                        }
                    },
                }
            },
            TestKind::Failure(ref expected) => {
                match Poscar::from_reader(input.as_bytes()) {
                    Ok(_) => { return Err(Error::NoError); },
                    Err(e) => {
                        // do a substring search
                        let bonafide = format!("{}", e);
                        let expected = expected.clone();
                        if !bonafide.contains(&expected[..]) {
                            return Err(Error::ErrorMismatch { bonafide, expected })
                        }
                    }
                }
            },
        }
        Ok(())
    }
}

