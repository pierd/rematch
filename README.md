# rematch
A procedural macro to generate simple FromStr implementation based on Regular Expression matching

## Examples

Usage for enum:
```rs
use rematch::rematch;

#[derive(Debug, PartialEq, Eq)]
#[rematch]
enum Test {
    #[rematch(r"a")]
    A,
    #[rematch(r"b (\d+)")]
    B(usize),
    #[rematch(r"c = (\d+)")]
    C { x: usize },
}

assert_eq!("a".parse::<Test>().unwrap(), Test::A);
assert_eq!("b 42".parse::<Test>().unwrap(), Test::B(42));
assert_eq!("c = 123".parse::<Test>().unwrap(), Test::C { x: 123 });
```

Usage for struct:
```rs
use rematch::rematch;

#[derive(Debug, PartialEq, Eq)]
#[rematch(r"a number (\d+) with some string ([abc]+)")]
struct Test {
    a: usize,
    s: String,
}

assert_eq!(
    "a number 42 with some string cabab"
        .parse::<Test>()
        .unwrap(),
    Test {
        a: 42,
        s: "cabab".to_owned(),
    }
);
```