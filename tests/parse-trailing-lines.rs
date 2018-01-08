// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Exhaustively tests support for trailing lines, in an attempt to trip up
//! the code that detects whether velocities are present.
//!
//! This test is similar to the yaml-based parse tests, but iterates over a
//! large number of cases that would be repetitive to write out explicitly.

#![deny(unused)]

#[macro_use] extern crate indoc;
extern crate vasp_poscar;

use vasp_poscar::Poscar;

#[test]
fn parse_trailing_lines() {
    // Inputs that differ in where they end
    const BODIES: &'static [&'static [u8]] = &[
        // (note: these should be written in the default output format)
        // File that ends after positions
        indoc!(b"
            comment
              1.0
                1.0 0.0 0.0
                0.0 1.0 0.0
                0.0 0.0 1.0
               1
            Direct
              0.0 0.0 0.0
        "),
        // File that ends after velocities
        indoc!(b"
            comment
              1.0
                1.0 0.0 0.0
                0.0 1.0 0.0
                0.0 0.0 1.0
               1
            Direct
              0.0 0.0 0.0

              0.0 0.0 0.0
        "),
    ];

    // Things allowed to show up as trailing blank lines (which may
    //  be handled by different logic in some cases)
    const BLANK_LINES: &'static [&'static str] = &[
        "",
        "  \t \t ",
    ];

    // match a body with a sequence of trailing blank lines
    // (we try all possible sequences of blank lines up to a given length,
    //  in an attempt to find a case that trips up velocity detection)
    for &body in BODIES {
        for blanks in permutations_with_replacement_upto(3, BLANK_LINES) {

            // construct file with these parts
            let mut input = body.to_owned();
            for blank in blanks {
                input.extend("\n".bytes());
                input.extend(blank.bytes());
            }

            let input_s = String::from_utf8(input.clone()).unwrap();

            let poscar = Poscar::from_reader(&input[..]).expect(&input_s);
            let actual = format!("{}", poscar);

            // it should have stripped the trailing lines, leaving the original body.
            let expected = String::from_utf8(body.into()).unwrap();

            assert_eq!(expected, actual, "input was: {:?}", input_s);
        }
    }
}

fn permutations_with_replacement<T>(n: usize, items: &[T]) -> Vec<Vec<T>>
where T: Clone,
{
    match n {
        0 => vec![vec![]],
        _ => {
            permutations_with_replacement(n - 1, items)
                .into_iter()
                .flat_map(|v| items.iter().cloned().map(move |x| {
                    let mut v = v.clone();
                    v.push(x);
                    v
                }))
                .collect()
        }
    }
}

fn permutations_with_replacement_upto<T>(n_max: usize, items: &[T]) -> Vec<Vec<T>>
where T: Clone,
{ (0..n_max+1).flat_map(|n| permutations_with_replacement(n, items)).collect() }

#[test]
fn test_permutations_upto() {
    let mut actual = permutations_with_replacement_upto(2, &[1u32, 2u32]);
    actual.sort();

    let mut expected = vec![
        vec![], vec![1], vec![2],
        vec![1, 1], vec![1, 2],
        vec![2, 1], vec![2, 2],
    ];
    expected.sort();
    assert_eq!(actual, expected);
}
