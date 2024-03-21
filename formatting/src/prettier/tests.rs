use alloc::{boxed::Box, rc::Rc, string::ToString, vec::Vec};

use pretty_assertions::assert_str_eq;

use super::*;

/// FUN      ::= 'fn' ID '(' (PARAM ',')* PARAM? ')' RET_TYPE? '=' BLOCK
/// PARAM    ::= TYPED_ID
/// RET_TYPE ::= '->' TYPE
pub struct Function {
    pub name: Ident,
    pub args: Vec<TypedIdent>,
    pub ret: Option<Type>,
    pub body: Block,
}
impl PrettyPrint for Function {
    fn render(&self) -> Document {
        let singleline_params = self.args.iter().fold(Document::Empty, |acc, ti| match acc {
            Document::Empty => ti.render(),
            acc => acc + ", " + ti.render(),
        });
        let multiline_params = self.args.iter().fold(Document::Empty, |acc, ti| match acc {
            Document::Empty => ti.render(),
            acc => acc + ',' + nl() + ti.render(),
        });
        let params = ('(' + singleline_params + ')')
            | (indent(4, '(' + nl() + multiline_params) + nl() + ')');
        let return_ty = if let Some(ty) = self.ret {
            " -> " + display(ty)
        } else {
            Document::Empty
        };
        "fn " + text(self.name.as_str()) + params + return_ty + " = " + self.body.render()
    }
}
impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.pretty_print(f)
    }
}

/// BLOCK ::= EXPR
///         | '{' EXPR '}'
pub struct Block {
    pub body: Expr,
}
impl PrettyPrint for Block {
    fn render(&self) -> Document {
        if self.body.is_block_like() {
            indent(4, '{' + nl() + self.body.render()) + nl() + '}'
        } else {
            let body = self.body.render();
            let single_line = body.clone();
            let multi_line = indent(4, '{' + nl() + body) + nl() + '}';
            single_line | multi_line
        }
    }
}

/// EXPR ::= VAR
///        | NUM
///        | LET
///        | BINARY_EXPR
pub enum Expr {
    /// VAR ::= ID
    Var(Ident),
    /// NUM ::= [0-9]+
    Num(i64),
    Let(Box<LetExpr>),
    Binary(BinaryExpr),
}
impl Expr {
    pub fn is_block_like(&self) -> bool {
        match self {
            Self::Let(_) => true,
            Self::Binary(expr) => expr.lhs.is_block_like() || expr.rhs.is_block_like(),
            Self::Var(_) | Self::Num(_) => false,
        }
    }
}
impl PrettyPrint for Expr {
    fn render(&self) -> Document {
        match self {
            Self::Var(id) => id.render(),
            Self::Num(n) => display(*n),
            Self::Let(expr) => expr.render(),
            Self::Binary(expr) => expr.render(),
        }
    }
}

/// LET ::= 'let' ID '=' EXPR 'in' BLOCK
pub struct LetExpr {
    pub bound: Ident,
    pub expr: Expr,
    pub body: Block,
}
impl PrettyPrint for LetExpr {
    fn render(&self) -> Document {
        let decl = flatten("let " + self.bound.render() + ' ' + '=');
        let expr = self.expr.render();
        let sl_expr = ' ' + expr.clone();
        let ml_expr = indent(4, nl() + expr.clone()) + nl();
        let expr = sl_expr | ml_expr;
        let body = self.body.render();
        let sl_body = " in " + body.clone();
        let ml_body = indent(4, "in " + body);
        let body = sl_body | ml_body;
        decl + expr + body
    }
}

/// BINARY_EXPR ::= EXPR BINOP EXPR
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}
impl PrettyPrint for BinaryExpr {
    fn render(&self) -> Document {
        self.lhs.render() + ' ' + self.op.render() + ' ' + self.rhs.render()
    }
}

/// BINOP ::= '+'
///         | '-'
///         | '*'
#[derive(Debug, Copy, Clone)]
pub enum BinaryOp {
    Add,
    #[allow(unused)]
    Sub,
    Mul,
}
impl PrettyPrint for BinaryOp {
    fn render(&self) -> Document {
        match self {
            Self::Add => '+'.into(),
            Self::Sub => '-'.into(),
            Self::Mul => '*'.into(),
        }
    }
}
impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.pretty_print(f)
    }
}

/// TYPE ::= 'number'
///        | 'string'
#[derive(Debug, Copy, Clone)]
pub enum Type {
    Number,
    String,
}
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number => f.write_str("number"),
            Self::String => f.write_str("string"),
        }
    }
}
impl core::str::FromStr for Type {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "number" => Ok(Self::Number),
            "string" => Ok(Self::String),
            _ => Err(()),
        }
    }
}

/// ID ::= [a-z_][a-zA-Z0-9_-?!]+
#[derive(Debug, Clone)]
pub struct Ident(Rc<str>);
impl Ident {
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(Rc::from(s.as_ref().to_string().into_boxed_str()))
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}
impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl PrettyPrint for Ident {
    fn render(&self) -> Document {
        display(self.clone())
    }
}

/// TYPED_ID ::= ID ':' TYPE
#[derive(Debug, Clone)]
pub struct TypedIdent {
    pub id: Ident,
    pub ty: Type,
}
impl TypedIdent {
    pub fn new(id: Ident, ty: Type) -> Self {
        Self { id, ty }
    }
}
impl PrettyPrint for TypedIdent {
    fn render(&self) -> Document {
        self.id.render() + ": " + display(self.ty)
    }
}

macro_rules! id {
    ($name:ident) => {
        Ident::new(stringify!($name))
    };

    ($name:literal) => {
        Ident::new($name)
    };
}

macro_rules! var {
    ($name:ident) => {
        Expr::Var(id!($name))
    };

    ($name:literal) => {
        Expr::Var(id!($name))
    };
}

macro_rules! typed_id {
    ($name:ident : $ty:ident) => {
        TypedIdent::new(id!($name), stringify!($ty).parse().expect("invalid type"))
    };

    ($name:literal : $ty:ident) => {
        TypedIdent::new(id!($name), stringify!($ty).parse().expect("invalid type"))
    };
}

macro_rules! let_expr {
    ($id:ident = $value:expr => $body:expr) => {
        Expr::Let(Box::new(LetExpr {
            bound: id!($id),
            expr: $value,
            body: Block { body: $body },
        }))
    };
}

macro_rules! bin {
    ($op:expr, $lhs:expr, $rhs:expr) => {
        Expr::Binary(BinaryExpr {
            op: $op,
            lhs: Box::new($lhs),
            rhs: Box::new($rhs),
        })
    };
}

macro_rules! add {
    ($lhs:ident, $rhs:ident) => {
        bin!(BinaryOp::Add, var!($lhs), var!($rhs))
    };

    ($lhs:ident, $rhs:literal) => {
        bin!(BinaryOp::Add, var!($lhs), Expr::Num($rhs))
    };

    ($lhs:ident, $rhs:expr) => {
        bin!(BinaryOp::Add, var!($lhs), $rhs)
    };

    ($lhs:expr, $rhs:ident) => {
        bin!(BinaryOp::Add, $lhs, var!($rhs))
    };

    ($lhs:expr, $rhs:expr) => {
        bin!(BinaryOp::Add, $lhs, $rhs)
    };

    ($lhs:expr, $rhs:literal) => {
        bin!(BinaryOp::Add, $lhs, Expr::Num($rhs))
    };

    ($lhs:expr, $rhs:ident) => {
        bin!(BinaryOp::Add, $lhs, var!($rhs))
    };
}

#[allow(unused)]
macro_rules! sub {
    ($lhs:ident, $rhs:ident) => {
        bin!(BinaryOp::Sub, var!($lhs), var!($rhs))
    };

    ($lhs:ident, $rhs:literal) => {
        bin!(BinaryOp::Sub, var!($lhs), Expr::Num($rhs))
    };

    ($lhs:ident, $rhs:expr) => {
        bin!(BinaryOp::Sub, var!($lhs), $rhs)
    };

    ($lhs:expr, $rhs:ident) => {
        bin!(BinaryOp::Sub, $lhs, var!($rhs))
    };

    ($lhs:expr, $rhs:expr) => {
        bin!(BinaryOp::Sub, $lhs, $rhs)
    };

    ($lhs:expr, $rhs:literal) => {
        bin!(BinaryOp::Sub, $lhs, Expr::Num($rhs))
    };

    ($lhs:expr, $rhs:ident) => {
        bin!(BinaryOp::Sub, $lhs, var!($rhs))
    };
}

macro_rules! mul {
    ($lhs:ident, $rhs:ident) => {
        bin!(BinaryOp::Mul, var!($lhs), var!($rhs))
    };

    ($lhs:ident, $rhs:literal) => {
        bin!(BinaryOp::Mul, var!($lhs), Expr::Num($rhs))
    };

    ($lhs:ident, $rhs:expr) => {
        bin!(BinaryOp::Mul, var!($lhs), $rhs)
    };

    ($lhs:expr, $rhs:ident) => {
        bin!(BinaryOp::Mul, $lhs, var!($rhs))
    };

    ($lhs:expr, $rhs:expr) => {
        bin!(BinaryOp::Mul, $lhs, $rhs)
    };

    ($lhs:expr, $rhs:literal) => {
        bin!(BinaryOp::Mul, $lhs, Expr::Num($rhs))
    };

    ($lhs:expr, $rhs:ident) => {
        bin!(BinaryOp::Mul, $lhs, var!($rhs))
    };
}

macro_rules! fun {
    ($name:ident ( $($id:ident : $id_ty:ident),* ) => $ret_ty:ident in $body:expr) => {
        Function {
            name: id!($name),
            args: vec![$(typed_id!($id : $id_ty)),*],
            ret: Some(stringify!($ret_ty).parse().unwrap()),
            body: Block { body: $body },
        }
    };

    ($name:ident ( $($id:ident : $id_ty:ident),* ) in $body:expr) => {
        Function {
            name: id!($name),
            args: vec![$(typed_id!($id : $id_ty)),*],
            ret: None,
            body: Block { body: $body },
        }
    }
}

#[test]
fn integration_test() {
    let ast = fun!(square_plus_1 (a : number, b : number) => number in let_expr!(c = mul!(a, b) => add!(c, 1)));
    let expected = "\
fn square_plus_1(a: number, b: number) -> number = {
    let c = a * b in c + 1
}";
    let actual = ast.to_pretty_string();
    assert_str_eq!(actual, expected);
}
