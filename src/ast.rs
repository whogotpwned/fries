use crate::token::TokenKind;

pub type Block = Vec<Expr>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Ident(String),

    UnaryOp {
        op: TokenKind,
        right: Box<Expr>,
    },

    BinaryOp {
        left: Box<Expr>,
        op: TokenKind,
        right: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    Dot {
        object: Box<Expr>,
        field: String,
    },

    List(Vec<Expr>),

    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },

    RangeStep {
        start: Box<Expr>,
        step: Box<Expr>,
        end: Box<Expr>,
    },

    Comprehension {
        expr: Box<Expr>,
        name: String,
        iterable: Box<Expr>,
        filter: Option<Box<Expr>>,
    },

    If {
        condition: Box<Expr>,
        then_block: Block,
        elif_clauses: Vec<(Expr, Block)>,
        else_block: Option<Block>,
    },

    Let {
        name: String,
        value: Box<Expr>,
    },

    Var {
        name: String,
        value: Box<Expr>,
    },

    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },

    CompoundAssign {
        target: Box<Expr>,
        op: TokenKind,
        value: Box<Expr>,
    },

    Fn {
        name: Option<String>,
        params: Vec<String>,
        body: Block,
    },

    Return(Box<Expr>),

    Do(Block),

    While {
        condition: Box<Expr>,
        body: Block,
    },

    For {
        name: String,
        iterable: Box<Expr>,
        body: Block,
    },

    Match {
        subject: Box<Expr>,
        arms: Vec<(Pattern, Block)>,
    },

    TypeDecl {
        name: String,
        variants: Vec<TypeVariant>,
    },

    Load(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeVariant {
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Ident(String),
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Constructor {
        name: String,
        bindings: Vec<String>,
    },
    List(Vec<Pattern>),
    Rest,
}