use super::*;

#[test]
fn with_atom_subtrahend_errors_badarith() {
    with_subtrahend_errors_badarith(|_| Term::str_to_atom("minuend", DoNotCare).unwrap());
}

#[test]
fn with_local_reference_subtrahend_errors_badarith() {
    with_subtrahend_errors_badarith(|process| Term::next_local_reference(process));
}

#[test]
fn with_empty_list_subtrahend_errors_badarith() {
    with_subtrahend_errors_badarith(|_| Term::EMPTY_LIST);
}

#[test]
fn with_list_subtrahend_errors_badarith() {
    with_subtrahend_errors_badarith(|process| {
        Term::cons(0.into_process(&process), 1.into_process(&process), &process)
    });
}

#[test]
fn with_small_integer_subtrahend_returns_big_integer() {
    with(|minuend, process| {
        let subtrahend = crate::integer::small::MIN.into_process(&process);

        assert_eq!(subtrahend.tag(), SmallInteger);

        let result = erlang::subtract_2(minuend, subtrahend, &process);

        assert!(result.is_ok());

        let difference = result.unwrap();

        assert_eq!(difference.tag(), Boxed);

        let unboxed_difference: &Term = difference.unbox_reference();

        assert_eq!(unboxed_difference.tag(), BigInteger);
    })
}

#[test]
fn with_big_integer_subtrahend_with_underflow_returns_small_integer() {
    with(|minuend, process| {
        let subtrahend = (crate::integer::small::MAX + 1).into_process(&process);

        assert_eq!(subtrahend.tag(), Boxed);

        let unboxed_subtrahend: &Term = subtrahend.unbox_reference();

        assert_eq!(unboxed_subtrahend.tag(), BigInteger);

        let result = erlang::subtract_2(minuend, subtrahend, &process);

        assert!(result.is_ok());

        let difference = result.unwrap();

        assert_eq!(difference.tag(), SmallInteger);
    })
}

#[test]
fn with_big_integer_subtrahend_returns_big_integer() {
    with(|minuend, process| {
        let subtrahend = (crate::integer::small::MIN - 1).into_process(&process);

        assert_eq!(subtrahend.tag(), Boxed);

        let unboxed_subtrahend: &Term = subtrahend.unbox_reference();

        assert_eq!(unboxed_subtrahend.tag(), BigInteger);

        let result = erlang::subtract_2(minuend, subtrahend, &process);

        assert!(result.is_ok());

        let difference = result.unwrap();

        assert_eq!(difference.tag(), Boxed);

        let unboxed_difference: &Term = difference.unbox_reference();

        assert_eq!(unboxed_difference.tag(), BigInteger);
    })
}

#[test]
fn with_float_subtrahend_without_underflow_or_overflow_returns_float() {
    with(|minuend, process| {
        let subtrahend = 3.0.into_process(&process);

        let result = erlang::subtract_2(minuend, subtrahend, &process);

        assert!(result.is_ok());

        let difference = result.unwrap();

        assert_eq!(difference.tag(), Boxed);

        let unboxed_difference: &Term = difference.unbox_reference();

        assert_eq!(unboxed_difference.tag(), Float);
    })
}

#[test]
fn with_float_subtrahend_with_underflow_returns_min_float() {
    with(|minuend, process| {
        let subtrahend = std::f64::MAX.into_process(&process);

        assert_eq!(
            erlang::subtract_2(minuend, subtrahend, &process),
            Ok(std::f64::MIN.into_process(&process))
        );
    })
}

#[test]
fn with_float_subtrahend_with_overflow_returns_max_float() {
    with(|minuend, process| {
        let subtrahend = std::f64::MIN.into_process(&process);

        assert_eq!(
            erlang::subtract_2(minuend, subtrahend, &process),
            Ok(std::f64::MAX.into_process(&process))
        );
    })
}

#[test]
fn with_local_pid_subtrahend_errors_badarith() {
    with_subtrahend_errors_badarith(|_| Term::local_pid(0, 1).unwrap());
}

#[test]
fn with_external_pid_subtrahend_errors_badarith() {
    with_subtrahend_errors_badarith(|process| Term::external_pid(1, 2, 3, &process).unwrap());
}

#[test]
fn with_tuple_subtrahend_errors_badarith() {
    with_subtrahend_errors_badarith(|process| Term::slice_to_tuple(&[], &process));
}

#[test]
fn with_map_is_subtrahend_errors_badarith() {
    with_subtrahend_errors_badarith(|process| Term::slice_to_map(&[], &process));
}

#[test]
fn with_heap_binary_subtrahend_errors_badarith() {
    with_subtrahend_errors_badarith(|process| Term::slice_to_binary(&[], &process));
}

#[test]
fn with_subbinary_subtrahend_errors_badarith() {
    with_subtrahend_errors_badarith(|process| {
        let original = Term::slice_to_binary(&[0b0000_00001, 0b1111_1110, 0b1010_1011], &process);
        Term::subbinary(original, 0, 7, 2, 1, &process)
    });
}

fn with<F>(f: F)
where
    F: FnOnce(Term, &Process) -> (),
{
    with_process(|process| {
        let minuend: Term = (crate::integer::small::MAX + 1).into_process(&process);

        assert_eq!(minuend.tag(), Boxed);

        let unboxed_minuend: &Term = minuend.unbox_reference();

        assert_eq!(unboxed_minuend.tag(), BigInteger);

        f(minuend, &process)
    })
}

fn with_subtrahend_errors_badarith<M>(subtrahend: M)
where
    M: FnOnce(&Process) -> Term,
{
    super::errors_badarith(|process| {
        let minuend: Term = (crate::integer::small::MAX + 1).into_process(&process);

        assert_eq!(minuend.tag(), Boxed);

        let unboxed_minuend: &Term = minuend.unbox_reference();

        assert_eq!(unboxed_minuend.tag(), BigInteger);

        let subtrahend = subtrahend(&process);

        erlang::subtract_2(minuend, subtrahend, &process)
    });
}