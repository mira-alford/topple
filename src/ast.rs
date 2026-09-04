pub struct Id {
    name: String,
}

pub struct Typed<T> {
    inner: T,
    typ: Type,
}

pub enum Literal {
    Unit,
    BitVec(Typed<usize>),
    Record {
        fields: Vec<(Typed<Id>, Box<Literal>)>,
    },
    Null,
}

pub enum LVar {
    Var(Id),
    Deref(Id),
    Field(Id, Id),
}

pub enum RVar {
    Id(Id),
    TypedId(Typed<Id>),
    Deref(Box<RVar>),
    Field(Box<RVar>, Id),
    Slice(Box<RVar>, Box<usize>, Box<usize>),
    Ref(Box<RVar>),
}

pub enum Expr {
    RVar(RVar),
    Call(RVar, Vec<Expr>),
    Alloc(Box<Expr>),
    UnExp(UnOp, Box<Expr>),
    BinExp(Box<Expr>, BinOp, Box<Expr>),
    Literal(Literal),
    Cast(Typed<Box<Expr>>),
}

// fn binexp(op: BinOp, e1: Expr, e2: Expr) -> Expr {
//     Expr::BinExp(op, Box::new(e1), Box::new(e2))
// }

pub enum UnOp {
    Not,
    Neg,
}

pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    And,
    Or,
    Shl,
    Shr,
    BvAnd,
    BvOr,
    BvXOr,
    Lt,
    Gt,
    Leq,
    Geq,
    Eq,
    Neq,
    Cat,
}

pub enum Stmt {
    Var(Id),
    Assign(LVar, Expr),
    Input(LVar),
    Output(Expr),
    While(Expr, Box<Stmt>),
    IfThenElse(Expr, Box<Stmt>, Box<Stmt>),
    StmtList(Vec<Stmt>),
    Return(Expr),
}

pub enum Type {
    Unit,
    BitVec { signed: bool, size: usize },
    Ref(Box<Type>),
    Fun { args: Vec<Type>, ret: Box<Type> },
    Array { typ: Box<Type>, len: usize },
    Record { fields: Vec<Typed<Id>> },
    Variable(Id),
}

pub enum Declaration {
    Fun {
        name: Id,
        args: Vec<Typed<Id>>,
        ret: Type,
        body: Stmt,
    },
    Type {
        name: Id,
        typ: Type,
    },
    Var {
        name: Typed<Id>,
        val: Expr,
    },
}
