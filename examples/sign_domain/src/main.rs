#![feature(deref_patterns)]

use topple::ast::{BinOp, Expr, Id, LVar, Literal, RVar, Stmt, UnOp};
use topple::{
    absint::{
        denotational,
        domain::{AbstractDomain, AbstractValueDomain, StateDomain},
        lattice::{FlatLattice, Lattice, LatticeMap},
    },
    ast::{Type, Typed},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Sign {
    Neg,
    Zero,
    Pos,
}

struct SignValueDomain;

impl AbstractValueDomain for SignValueDomain {
    type Value = FlatLattice<Sign>;

    fn of_literal(lit: Literal) -> Self::Value {
        match lit {
            Literal::Unit => FlatLattice::Top,
            Literal::BitVec(typed) => {
                // this language doesn't have integers :sob: guess this analysis is unsound because
                // i don't CARE about handling bitvectors correctly!! grrrr
                match typed.inner as isize {
                    1.. => FlatLattice::Value(Sign::Pos),
                    0 => FlatLattice::Value(Sign::Zero),
                    ..=-1 => FlatLattice::Value(Sign::Pos),
                }
            }
            Literal::Record { fields: _ } => FlatLattice::Top,
            Literal::Null => FlatLattice::Top,
        }
    }

    fn of_unary(op: UnOp, arg: Self::Value) -> Self::Value {
        match op {
            UnOp::Not => FlatLattice::Top,
            UnOp::Neg => match arg {
                FlatLattice::Value(Sign::Neg) => FlatLattice::Value(Sign::Pos),
                FlatLattice::Value(Sign::Zero) => FlatLattice::Value(Sign::Zero),
                FlatLattice::Value(Sign::Pos) => FlatLattice::Value(Sign::Neg),
                FlatLattice::Bottom | FlatLattice::Top => arg,
            },
        }
    }

    fn of_binary(op: BinOp, arg1: Self::Value, arg2: Self::Value) -> Self::Value {
        // you can handle more ops but this is just an example
        use FlatLattice as FL;
        match op {
            BinOp::Add => match (arg1, arg2) {
                (_, FL::Bottom) | (FL::Bottom, _) => FL::Bottom,
                (_, FL::Top) | (FL::Top, _) => FL::Top,
                (other, FL::Value(Sign::Zero)) | (FL::Value(Sign::Zero), other) => other,
                (FL::Value(Sign::Pos), FL::Value(Sign::Pos)) => FL::Value(Sign::Pos),
                (FL::Value(Sign::Neg), FL::Value(Sign::Neg)) => FL::Value(Sign::Neg),
                (FL::Value(Sign::Pos), FL::Value(Sign::Neg))
                | (FL::Value(Sign::Neg), FL::Value(Sign::Pos)) => FL::Top,
            },
            BinOp::Mul => match (arg1, arg2) {
                (_, FL::Bottom) | (FL::Bottom, _) => FL::Bottom,
                (_, FL::Value(Sign::Zero)) | (FL::Value(Sign::Zero), _) => FL::Value(Sign::Zero),
                (_, FL::Top) | (FL::Top, _) => FL::Top,
                (FL::Value(Sign::Pos), FL::Value(Sign::Pos))
                | (FL::Value(Sign::Neg), FL::Value(Sign::Neg)) => FL::Value(Sign::Pos),
                (FL::Value(Sign::Pos), FL::Value(Sign::Neg))
                | (FL::Value(Sign::Neg), FL::Value(Sign::Pos)) => FL::Value(Sign::Neg),
            },
            BinOp::Div => match (arg1, arg2) {
                (_, FL::Value(Sign::Zero)) | (_, FL::Bottom) | (FL::Bottom, _) => FL::Bottom,
                (FL::Value(Sign::Zero), _) => FL::Value(Sign::Zero),
                (_, FL::Top) | (FL::Top, _) => FL::Top,
                (FL::Value(Sign::Pos), FL::Value(Sign::Pos))
                | (FL::Value(Sign::Neg), FL::Value(Sign::Neg)) => FL::Value(Sign::Pos),
                (FL::Value(Sign::Pos), FL::Value(Sign::Neg))
                | (FL::Value(Sign::Neg), FL::Value(Sign::Pos)) => FL::Value(Sign::Neg),
            },
            _ => FL::Top,
        }
    }

    fn widen(a: Self::Value, b: Self::Value) -> Self::Value {
        FlatLattice::join(a, b)
    }
}

type SignDomain = StateDomain<SignValueDomain>;

struct CondSignDomain;

impl AbstractDomain for CondSignDomain {
    type State = LatticeMap<Id, FlatLattice<Sign>>;

    fn widen(a: Self::State, b: Self::State) -> Self::State {
        SignDomain::widen(a, b)
    }

    fn transfer_decl(var: Id, state: Self::State) -> Self::State {
        SignDomain::transfer_decl(var, state)
    }

    fn transfer_assign(lhs: LVar, rhs: Expr, state: Self::State) -> Self::State {
        SignDomain::transfer_assign(lhs, rhs, state)
    }

    fn transfer_input(var: LVar, state: Self::State) -> Self::State {
        SignDomain::transfer_input(var, state)
    }

    fn transfer_condition(cond: Expr, state: Self::State) -> Self::State {
        match cond {
            Expr::BinExp(
                deref!(Expr::RVar(RVar::Id(id))),
                BinOp::Eq,
                deref!(Expr::Literal(Literal::BitVec(typed))),
            ) if typed.inner == 0 => state.update(id, FlatLattice::Value(Sign::Zero)),
            Expr::BinExp(
                deref!(Expr::RVar(RVar::Id(id))),
                BinOp::Lt,
                deref!(Expr::Literal(Literal::BitVec(typed))),
            ) if typed.inner == 0 => state.update(id, FlatLattice::Value(Sign::Neg)),
            Expr::BinExp(
                deref!(Expr::RVar(RVar::Id(id))),
                BinOp::Gt,
                deref!(Expr::Literal(Literal::BitVec(typed))),
            ) if typed.inner == 0 => state.update(id, FlatLattice::Value(Sign::Pos)),
            _ => state,
        }
    }
}

fn main() {
    // x;
    // y;
    // x = 100;
    // y = 1;
    // while x > 0 {
    //   y = x * y;
    //   x = x - 1;
    // }
    let x = Id {
        name: "x".to_string(),
    };
    let y = Id {
        name: "y".to_string(),
    };
    let prog = Stmt::StmtList(vec![
        Stmt::Var(x.clone()),
        Stmt::Var(y.clone()),
        Stmt::Assign(
            LVar::Var(x.clone()),
            Expr::Literal(Literal::BitVec(Typed {
                inner: 100,
                typ: Type::BitVec {
                    signed: true,
                    size: 64,
                },
            })),
        ),
        Stmt::Assign(
            LVar::Var(y.clone()),
            Expr::Literal(Literal::BitVec(Typed {
                inner: 1,
                typ: Type::BitVec {
                    signed: true,
                    size: 64,
                },
            })),
        ),
        Stmt::While(
            Expr::BinExp(
                Box::new(Expr::RVar(RVar::Id(x.clone()))),
                BinOp::Gt,
                Box::new(Expr::Literal(Literal::BitVec(Typed {
                    inner: 0,
                    typ: Type::BitVec {
                        signed: true,
                        size: 64,
                    },
                }))),
            ),
            Box::new(Stmt::StmtList(vec![
                Stmt::Assign(
                    LVar::Var(y.clone()),
                    Expr::BinExp(
                        Box::new(Expr::RVar(RVar::Id(x.clone()))),
                        BinOp::Mul,
                        Box::new(Expr::RVar(RVar::Id(y.clone()))),
                    ),
                ),
                Stmt::Assign(
                    LVar::Var(x.clone()),
                    Expr::BinExp(
                        Box::new(Expr::RVar(RVar::Id(x.clone()))),
                        BinOp::Sub,
                        Box::new(Expr::Literal(Literal::BitVec(Typed {
                            inner: -1isize as usize,
                            typ: Type::BitVec {
                                signed: true,
                                size: 64,
                            },
                        }))),
                    ),
                ),
            ])),
        ),
    ]);
    println!(
        "Analysis results: {:?}",
        denotational::solve::<CondSignDomain>(prog, LatticeMap::TOP)
    );
}
