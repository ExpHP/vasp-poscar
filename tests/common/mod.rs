macro_rules! assert_matches {
    ($pat:pat, $expr:expr $(,)*) => {{
        let e = $expr;
        match e {
            $pat => {},
            _ => panic!("assert_matches failed!
Expected: {}
  Actual: {:#?}", stringify!($pat), e),
        }
    }}
}
