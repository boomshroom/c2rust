//! Analysis passes used to drive various transformations.

use std::collections::HashSet;

use command::{Registry, DriverCommand};
use driver::Phase;
use c2rust_ast_builder::IntoSymbol;


pub mod labeled_ty;
pub mod ownership;
pub mod type_eq;


/// # `test_analysis_type_eq` Command
/// 
/// Test command - not intended for general use.
/// 
/// Usage: `test_analysis_type_eq`
/// 
/// Runs the `type_eq` analysis and logs the result (at level `info`).
fn register_test_analysis_type_eq(reg: &mut Registry) {
    reg.register("test_analysis_type_eq", |_args| {
        Box::new(DriverCommand::new(Phase::Phase3, move |st, cx| {
            let result = type_eq::analyze(cx.hir_map(), cx.ty_ctxt(), cx.ty_arena(), &st.krate());
            info!("{:?}", result);
        }))
    });
}

/// # `test_analysis_ownership` Command
/// 
/// Test command - not intended for general use.
/// 
/// Usage: `test_analysis_ownership`
/// 
/// Runs the `ownership` analysis and dumps the results to stderr.
fn register_test_analysis_ownership(reg: &mut Registry) {
    reg.register("test_analysis_ownership", |_args| {
        Box::new(DriverCommand::new(Phase::Phase3, move |st, cx| {
            let results = ownership::analyze(&st, &cx);
            ownership::dump_results(&cx, &results);
        }))
    });
}

/// # `mark_related_types` Command
/// 
/// Usage: `mark_related_types [MARK]`
/// 
/// Marks: `MARK`/`target`
/// 
/// For each type annotation bearing `MARK` (default: `target`),
/// apply `MARK` to all other type annotations that must be the same
/// type according to (a simplified version of) Rust's typing rules.
/// 
/// For example, in this code:
/// 
///     fn f(x: i32, y: i32) -> i32 {
///         x
///     }
/// 
/// The `i32` annotations on `x` and the return type of `f` are
/// related, because changing these annotations to two unequal types
/// would produce a type error.  But the `i32` annotation on `y` is
/// unrelated, and can be changed independently of the other two.
fn register_mark_related_types(reg: &mut Registry) {
    reg.register("mark_related_types", |args| {
        let label = args.get(0).map_or("target", |x| x).into_symbol();
        Box::new(DriverCommand::new(Phase::Phase3, move |st, cx| {
            let ty_class = type_eq::analyze(cx.hir_map(), cx.ty_ctxt(), cx.ty_arena(), &st.krate());

            let mut related_classes = HashSet::new();
            for &(id, l) in st.marks().iter() {
                if l == label {
                    if let Some(&cls) = ty_class.get(&id) {
                        related_classes.insert(cls);
                    }
                }
            }

            for (&id, &cls) in &ty_class {
                if related_classes.contains(&cls) {
                    st.add_mark(id, label);
                }
            }
        }))
    });
}

pub fn register_commands(reg: &mut Registry) {
    register_test_analysis_type_eq(reg);
    register_test_analysis_ownership(reg);
    register_mark_related_types(reg);
}
