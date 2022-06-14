#[cfg(test)]
mod tests {
    use rematch::rematch;

    #[test]
    fn test_enum() {
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
    }

    #[test]
    fn test_struct() {
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
    }
}
