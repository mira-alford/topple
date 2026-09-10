//! An abstract interpreter over denotational semantics (see 3.5.2 of Antoine Mine's tutorial)

// TODO: This solver will give an abstract state for running the complete program, but will not give
// abstract states for intermediate parts of the program. To fix this, some sort of way to put
// annotations between statements would be needed I think... Or a CFG solver could be used instead

use super::domain::AbstractDomain;
use super::lattice::Lattice;
use crate::ast::{Expr, Stmt, UnOp};

fn lfp<F, L: Lattice>(f: F) -> L
where
    F: Fn(L) -> L,
{
    let mut cur = L::BOTTOM;
    let mut other = f(cur.clone());
    while cur != other {
        cur = other;
        other = f(cur.clone());
    }
    cur
}

fn not(e: Expr) -> Expr {
    Expr::UnExp(UnOp::Not, Box::new(e))
}

pub fn solve<D: AbstractDomain>(procedure: Stmt, init: D::State) -> D::State {
    match procedure {
        Stmt::Var(id) => D::transfer_decl(id, init),
        Stmt::Assign(lvar, expr) => D::transfer_assign(lvar, expr, init),
        Stmt::Input(lvar) => D::transfer_input(lvar, init),
        Stmt::Output(_) => init,
        Stmt::While(expr, stmt) => D::transfer_condition(
            not(expr.clone()),
            lfp(|s: D::State| {
                D::widen(
                    s.clone(),
                    D::State::join(
                        init.clone(),
                        solve::<D>(*stmt.clone(), D::transfer_condition(expr.clone(), s)),
                    ),
                )
            }),
        ),
        Stmt::IfThenElse(expr, stmt, stmt1) => D::State::meet(
            solve::<D>(*stmt, D::transfer_condition(expr.clone(), init.clone())),
            solve::<D>(*stmt1, D::transfer_condition(not(expr), init)),
        ),
        Stmt::StmtList(stmts) => stmts
            .into_iter()
            .fold(init, |acc, stmt| solve::<D>(stmt, acc)),
        Stmt::Return(_) => init, // ?
    }
}
