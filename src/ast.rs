#[derive(Debug)]
pub struct Id(pub String);

// Associate a type with the inner T
#[derive(Debug)]
pub struct Typed<T> {
    inner: T,
    typ: Type,
}

#[derive(Debug)]
pub enum Literal {
    Unit,
    Integer(i64),
    Boolean(bool),
    Record {
        fields: Vec<(Typed<Id>, Box<Literal>)>,
    },
    Null,
}

#[derive(Debug)]
pub enum LVar {
    Var(Id),
    Deref(Id),
    Field(Id, Id),
}

#[derive(Debug)]
pub enum RVar {
    Id(Id),
    Deref(Box<RVar>),
    Field(Box<RVar>, Id),
    Ref(Box<RVar>),
}

#[derive(Debug)]
pub enum Expr {
    RVar(RVar),
    Call(RVar, Vec<Expr>),
    Alloc(Box<Expr>),
    UnExp(UnOp, Box<Expr>),
    BinExp(Box<Expr>, BinOp, Box<Expr>),
    Literal(Literal),
    Cast(Typed<Box<Expr>>),
}

#[derive(Debug)]
pub enum UnOp {
    Not,
    Neg,
}

#[derive(Debug)]
pub enum BinOp {
    // Integer math:
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    // Comparison Ops
    Lt,
    Gt,
    Leq,
    Geq,
    Eq,
    Neq,
    // Boolean Ops
    And,
    Or,
    XOr,
}

#[derive(Debug)]
pub enum Stmt {
    Var(Id, Option<Type>),
    Assign(LVar, Expr),
    Input(LVar),
    Output(Expr),
    While(Expr, Box<Stmt>),
    IfThenElse(Expr, Box<Stmt>, Box<Stmt>),
    StmtList(Vec<Stmt>),
    Return(Expr),
}

#[derive(Debug)]
pub enum Type {
    Unit,
    Integer,
    Boolean,
    Ref(Box<Type>),
    Fun { args: Vec<Type>, ret: Box<Type> },
    Array { typ: Box<Type>, len: usize },
    Record { fields: Vec<Typed<Id>> },
    Variable(Id),
}

#[derive(Debug)]
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
