use super::*;

#[test]
fn with_number_atom_reference_function_port_pid_or_tuple_second_returns_second() {
    run!(
        |arc_process| {
            (
                strategy::term::map(arc_process.clone()),
                strategy::term::number_atom_reference_function_port_pid_or_tuple(arc_process),
            )
        },
        |(first, second)| {
            prop_assert_eq!(native(first, second), second);

            Ok(())
        },
    );
}

#[test]
fn with_smaller_map_second_returns_second() {
    min(
        |_, process| {
            process
                .map_from_slice(&[(Atom::str_to_term("a"), process.integer(1).unwrap())])
                .unwrap()
        },
        Second,
    );
}

#[test]
fn with_same_size_map_with_lesser_keys_returns_second() {
    min(
        |_, process| {
            process
                .map_from_slice(&[
                    (Atom::str_to_term("a"), process.integer(2).unwrap()),
                    (Atom::str_to_term("b"), process.integer(3).unwrap()),
                ])
                .unwrap()
        },
        Second,
    );
}

#[test]
fn with_same_size_map_with_same_keys_with_lesser_values_returns_second() {
    min(
        |_, process| {
            process
                .map_from_slice(&[
                    (Atom::str_to_term("b"), process.integer(2).unwrap()),
                    (Atom::str_to_term("c"), process.integer(2).unwrap()),
                ])
                .unwrap()
        },
        Second,
    );
}

#[test]
fn with_same_map_returns_first() {
    min(|first, _| first, First);
}

#[test]
fn with_same_value_map_returns_first() {
    min(
        |_, process| {
            process
                .map_from_slice(&[
                    (Atom::str_to_term("b"), process.integer(2).unwrap()),
                    (Atom::str_to_term("c"), process.integer(3).unwrap()),
                ])
                .unwrap()
        },
        First,
    );
}

#[test]
fn with_same_size_map_with_same_keys_with_greater_values_returns_first() {
    min(
        |_, process| {
            process
                .map_from_slice(&[
                    (Atom::str_to_term("b"), process.integer(3).unwrap()),
                    (Atom::str_to_term("c"), process.integer(4).unwrap()),
                ])
                .unwrap()
        },
        First,
    );
}

#[test]
fn with_same_size_map_with_greater_keys_returns_first() {
    min(
        |_, process| {
            process
                .map_from_slice(&[
                    (Atom::str_to_term("c"), process.integer(2).unwrap()),
                    (Atom::str_to_term("d"), process.integer(3).unwrap()),
                ])
                .unwrap()
        },
        First,
    );
}

#[test]
fn with_greater_size_map_returns_first() {
    min(
        |_, process| {
            process
                .map_from_slice(&[
                    (Atom::str_to_term("a"), process.integer(1).unwrap()),
                    (Atom::str_to_term("b"), process.integer(2).unwrap()),
                    (Atom::str_to_term("c"), process.integer(3).unwrap()),
                ])
                .unwrap()
        },
        First,
    );
}

#[test]
fn with_list_or_bitstring_second_returns_first() {
    run!(
        |arc_process| {
            (
                strategy::term::map(arc_process.clone()),
                strategy::term::list_or_bitstring(arc_process.clone()),
            )
        },
        |(first, second)| {
            prop_assert_eq!(native(first, second), first);

            Ok(())
        },
    );
}

fn min<R>(second: R, which: FirstSecond)
where
    R: FnOnce(Term, &Process) -> Term,
{
    super::min(
        |process| {
            process
                .map_from_slice(&[
                    (Atom::str_to_term("b"), process.integer(2).unwrap()),
                    (Atom::str_to_term("c"), process.integer(3).unwrap()),
                ])
                .unwrap()
        },
        second,
        which,
    );
}
