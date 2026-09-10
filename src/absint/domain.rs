use super::lattice::{Lattice, LatticeMap};
use crate::ast::{BinOp, Expr, Id, LVar, Literal, RVar, UnOp};

pub trait AbstractDomain {
    type State: Lattice;

    fn widen(a: Self::State, b: Self::State) -> Self::State;

    fn transfer_decl(var: Id, state: Self::State) -> Self::State;
    fn transfer_assign(lhs: LVar, rhs: Expr, state: Self::State) -> Self::State;
    fn transfer_input(var: LVar, state: Self::State) -> Self::State;

    fn transfer_condition(cond: Expr, state: Self::State) -> Self::State;
}

pub trait AbstractValueDomain {
    type Value: Lattice;

    fn of_literal(lit: Literal) -> Self::Value;
    fn of_unary(op: UnOp, arg: Self::Value) -> Self::Value;
    fn of_binary(op: BinOp, arg1: Self::Value, arg2: Self::Value) -> Self::Value;

    fn widen(a: Self::Value, b: Self::Value) -> Self::Value;
}

pub struct StateDomain<VD: AbstractValueDomain> {
    _phantom: core::marker::PhantomData<VD>,
}

impl<VD: AbstractValueDomain> StateDomain<VD> {
    pub fn eval_rvar(rvar: RVar, state: &LatticeMap<Id, VD::Value>) -> VD::Value {
        match rvar {
            RVar::Id(id) => state.read(&id),
            RVar::TypedId(typed) => state.read(&typed.inner),
            RVar::Deref(_) => VD::Value::TOP,
            RVar::Field(_, _) => VD::Value::TOP,
            RVar::Slice(_, _, _) => VD::Value::TOP,
            RVar::Ref(_) => VD::Value::TOP,
        }
    }

    pub fn eval_expr(expr: Expr, state: &LatticeMap<Id, VD::Value>) -> VD::Value {
        match expr {
            Expr::RVar(rvar) => Self::eval_rvar(rvar, state),
            Expr::Call(_, _) => VD::Value::TOP,
            Expr::Alloc(_) => VD::Value::TOP,
            Expr::UnExp(un_op, expr) => VD::of_unary(un_op, Self::eval_expr(*expr, state)),
            Expr::BinExp(expr, bin_op, expr1) => VD::of_binary(
                bin_op,
                Self::eval_expr(*expr, state),
                Self::eval_expr(*expr1, state),
            ),
            Expr::Literal(literal) => VD::of_literal(literal),
            Expr::Cast(_) => VD::Value::TOP,
        }
    }
}

impl<VD: AbstractValueDomain> AbstractDomain for StateDomain<VD> {
    type State = LatticeMap<Id, VD::Value>;

    fn widen(a: Self::State, b: Self::State) -> Self::State {
        match (a, b) {
            (LatticeMap::BottomMap(mut a), LatticeMap::BottomMap(mut b)) => {
                if a.len() > b.len() {
                    (a, b) = (b, a);
                }
                for (k, v) in a.into_iter() {
                    if let Some(v2) = b.remove(&k) {
                        b.insert(k, VD::widen(v, v2));
                    } else {
                        b.insert(k, v);
                    }
                }
                LatticeMap::BottomMap(b)
            }

            (LatticeMap::BottomMap(a), LatticeMap::TopMap(mut b))
            | (LatticeMap::TopMap(mut b), LatticeMap::BottomMap(a)) => {
                for (k, v) in a.into_iter() {
                    if let Some(v2) = b.remove(&k) {
                        b.insert(k, VD::widen(v, v2));
                    }
                }
                LatticeMap::TopMap(b)
            }

            (LatticeMap::TopMap(mut a), LatticeMap::TopMap(mut b)) => {
                if a.len() > b.len() {
                    (a, b) = (b, a);
                }
                for (k, v) in a.into_iter() {
                    if let Some(v2) = b.remove(&k) {
                        b.insert(k, VD::widen(v, v2));
                    }
                }
                LatticeMap::TopMap(b)
            }
        }
    }

    fn transfer_decl(_: Id, state: Self::State) -> Self::State {
        state
    }

    fn transfer_assign(lhs: LVar, rhs: Expr, state: Self::State) -> Self::State {
        match lhs {
            LVar::Var(id) => {
                let val = Self::eval_expr(rhs, &state);
                state.update(id, val)
            }
            LVar::Deref(_) => Self::State::TOP,
            LVar::Field(_, _) => Self::State::TOP,
        }
    }

    fn transfer_input(var: LVar, state: Self::State) -> Self::State {
        match var {
            LVar::Var(id) => state.update(id, VD::Value::TOP),
            LVar::Deref(_) => Self::State::TOP,
            LVar::Field(_, _) => Self::State::TOP,
        }
    }

    fn transfer_condition(_: Expr, state: Self::State) -> Self::State {
        // an algorithm for this exists, apparently! something about going backwards down the expr
        // tree... scary!
        state
    }
}
