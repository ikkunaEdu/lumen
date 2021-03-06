mod spawn_apply_3;

use std::sync::Arc;

use anyhow::*;

use liblumen_alloc::erts::exception::{self, AllocResult};
use liblumen_alloc::erts::process::code;
use liblumen_alloc::erts::process::code::result_from_exception;
use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::prelude::{Atom, Term};
use liblumen_alloc::{exit, ModuleFunctionArity};

use crate::scheduler::Scheduler;
use crate::test;

#[test]
fn scheduler_does_not_requeue_exiting_process() {
    let arc_process = test::process::default();

    exit_1_place_frame_with_arguments(
        &arc_process,
        Placement::Replace,
        Atom::str_to_term("normal"),
    )
    .unwrap();

    let scheduler = Scheduler::current();

    assert!(scheduler.is_run_queued(&arc_process));

    assert!(scheduler.run_through(&arc_process));

    assert!(!scheduler.is_run_queued(&arc_process));
}

#[test]
fn scheduler_does_run_exiting_process() {
    let arc_process = test::process::default();
    let scheduler = Scheduler::current();

    assert!(scheduler.is_run_queued(&arc_process));
    assert!(scheduler.run_through(&arc_process));
    assert!(scheduler.is_run_queued(&arc_process));

    arc_process.exit_normal(anyhow!("Test").into());

    assert!(scheduler.is_run_queued(&arc_process));
    assert!(!scheduler.run_through(&arc_process));
    assert!(!scheduler.is_run_queued(&arc_process));
}

fn exit_1_place_frame_with_arguments(
    process: &Process,
    placement: Placement,
    reason: Term,
) -> AllocResult<()> {
    process.stack_push(reason)?;
    let module_function_arity = Arc::new(ModuleFunctionArity {
        module: Atom::try_from_str("erlang").unwrap(),
        function: Atom::try_from_str("exit").unwrap(),
        arity: 1,
    });
    let frame = Frame::new(module_function_arity, exit_1_code);
    process.place_frame(frame, placement);

    Ok(())
}

fn exit_1_code(arc_process: &Arc<Process>) -> code::Result {
    arc_process.reduce();

    let reason = arc_process.stack_peek(1).unwrap();
    const STACK_USED: usize = 1;

    match exit_1_native(reason) {
        Ok(return_value) => {
            arc_process
                .return_from_call(STACK_USED, return_value)
                .unwrap();

            Process::call_code(arc_process)
        }
        Err(exception) => result_from_exception(arc_process, STACK_USED, exception),
    }
}

fn exit_1_native(reason: Term) -> exception::Result<Term> {
    Err(exit!(reason, anyhow!("explicit exit from Erlang").into()).into())
}
