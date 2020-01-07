use super::*;

#[test]
fn with_positive_start_and_positive_length_returns_subbinary() {
    crate::test::with_positive_start_and_positive_length_returns_subbinary(
        file!(),
        returns_subbinary,
    );
}

#[test]
fn with_size_start_and_negative_size_length_returns_binary() {
    run!(
        |arc_process| {
            (
                Just(arc_process.clone()),
                strategy::term::is_binary::with_byte_len_range(1..=4, arc_process.clone()),
            )
                .prop_map(|(arc_process, binary)| {
                    let byte_len = total_byte_len(binary);

                    (
                        arc_process.clone(),
                        binary,
                        arc_process.integer(byte_len).unwrap(),
                        arc_process.integer(-(byte_len as isize)).unwrap(),
                    )
                })
        },
        returns_binary,
    );
}

#[test]
fn with_zero_start_and_size_length_returns_binary() {
    crate::test::with_zero_start_and_size_length_returns_binary(file!(), returns_binary);
}

fn returns_binary(
    (arc_process, binary, start, length): (Arc<Process>, Term, Term, Term),
) -> TestCaseResult {
    prop_assert_eq!(native(&arc_process, binary, start, length), Ok(binary));

    let returned_binary = native(&arc_process, binary, start, length).unwrap();

    prop_assert_eq!(
        returned_binary.is_boxed_subbinary(),
        binary.is_boxed_subbinary()
    );

    Ok(())
}
