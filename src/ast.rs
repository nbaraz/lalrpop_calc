use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

pub enum Statement {
    Assign(String, Expr),
    Print(Expr),
    Repr(Expr),
}

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Add,
    Sub,
    Mul,
    Div,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let st = match self {
            OpCode::Add => "+",
            OpCode::Sub => "-",
            OpCode::Mul => "*",
            OpCode::Div => "/",
        };
        write!(f, "{}", st)
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Num(i32),
    Ident(String),
    Op(Box<Expr>, OpCode, Box<Expr>),
    Resolve(Box<Expr>),
}

impl Expr {
    pub fn new_op(a: Expr, op: OpCode, b: Expr) -> Expr {
        Expr::Op(Box::new(a), op, Box::new(b))
    }
}

trait ExprVisitor {
    fn should_continue(&mut self) -> bool {
        true
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        if !self.should_continue() {
            return;
        }

        match expr {
            Expr::Num(num) => self.visit_num(num),
            Expr::Ident(ident) => self.visit_ident(ident),
            Expr::Op(a, op, b) => self.visit_op(a, op, b),
            Expr::Resolve(expr) => self.visit_resolve(expr),
        }
    }

    fn visit_num(&mut self, _num: &mut i32) {}
    fn visit_ident(&mut self, _ident: &mut String) {}
    fn visit_opcode(&mut self, _op: &mut OpCode) {}

    fn visit_op(&mut self, a: &mut Expr, op: &mut OpCode, b: &mut Expr) {
        self.visit_expr(a);

        if !self.should_continue() {
            return;
        }

        self.visit_opcode(op);

        self.visit_expr(b);
    }

    fn visit_resolve(&mut self, expr: &mut Expr) {
        self.visit_expr(expr)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Num(n) => write!(f, "{}", n),
            Expr::Ident(name) => write!(f, "{}", name),
            Expr::Op(a, op, b) => write!(f, "({} {} {})", a, op, b),
            Expr::Resolve(e) => write!(f, "resolve {}", e),
        }
    }
}

pub struct ReprExpr<'a, 'b> {
    expr: &'a Expr,
    vars: &'b HashMap<String, Expr>,
}

thread_local! {
    static OBJECTS_BEING_PRINTED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

impl<'a, 'b> fmt::Display for ReprExpr<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        OBJECTS_BEING_PRINTED.with(|resolving| match self.expr {
            Expr::Num(n) => write!(f, "{}", n),
            Expr::Ident(name) => {
                let existed = !{ resolving.borrow_mut().insert(name.clone()) };
                if existed {
                    write!(f, "{}[<recursive>]", name)
                } else {
                    let res = match self.vars.get(*&name) {
                        Some(e) => write!(
                            f,
                            "{}[{}]",
                            name,
                            ReprExpr {
                                expr: e,
                                vars: self.vars
                            }
                        ),
                        None => write!(f, "{}[?]", name),
                    };
                    resolving.borrow_mut().remove(&*name);
                    res
                }
            }
            Expr::Op(a, op, b) => write!(
                f,
                "({} {} {})",
                ReprExpr {
                    expr: a,
                    vars: self.vars
                },
                op,
                ReprExpr {
                    expr: b,
                    vars: self.vars
                }
            ),
            Expr::Resolve(e) => write!(
                f,
                "resolve {}",
                ReprExpr {
                    expr: e,
                    vars: self.vars
                }
            ),
        })
    }
}

pub fn repr_expr<'a, 'b>(expr: &'a Expr, vars: &'b HashMap<String, Expr>) -> ReprExpr<'a, 'b> {
    ReprExpr { expr, vars }
}

#[derive(Debug)]
pub struct NameError(pub String);

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not resolve `{}`", self.0)
    }
}

impl Error for NameError {
    fn description(&self) -> &str {
        "identifier not defined"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
