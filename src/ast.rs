use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;

pub enum Statement {
    Assign(String, Expr),
    Print(Expr),
    Repr(IdentReprMode, Expr),
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

pub trait ExprVisitor {
    fn should_continue(&mut self) -> bool {
        true
    }

    fn super_expr(&mut self, expr: &mut Expr) {
        if !self.should_continue() {
            return;
        }

        match expr {
            Expr::Num(num) => self.visit_num(num),
            Expr::Ident(ident) => self.visit_ident(ident),
            Expr::Op(a, op, b) => self.visit_op(a, op, b),
            resolve @ Expr::Resolve(_) => self.visit_resolve(resolve),
        }
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        self.super_expr(expr);
    }

    fn visit_num(&mut self, _num: &mut i32) {}
    fn visit_ident(&mut self, _ident: &mut String) {}
    fn visit_opcode(&mut self, _op: &mut OpCode) {}

    fn super_op(&mut self, a: &mut Expr, op: &mut OpCode, b: &mut Expr) {
        self.visit_expr(a);

        if !self.should_continue() {
            return;
        }

        self.visit_opcode(op);

        self.visit_expr(b);
    }

    fn visit_op(&mut self, a: &mut Expr, op: &mut OpCode, b: &mut Expr) {
        self.super_op(a, op, b);
    }

    fn super_resolve(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Resolve(inner) => self.visit_expr(inner),
            _ => panic!("Expected `Expr::Resolve`"),
        }
    }

    fn visit_resolve(&mut self, expr: &mut Expr) {
        self.super_resolve(expr)
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

#[derive(Clone, Copy)]
pub enum IdentReprMode {
    Eager,
    Lazy,
    LazyInner,
    Mixed,
}

pub struct ReprExpr<'a, 'b> {
    expr: &'a Expr,
    vars: &'b HashMap<String, Expr>,
    display_mode: IdentReprMode,
}

thread_local! {
    static OBJECTS_BEING_PRINTED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

fn fmt_ident_mixed(
    name: &str,
    vars: &HashMap<String, Expr>,
    existed: bool,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    if existed {
        write!(f, "{}[<recursive>]", name)
    } else {
        match vars.get(*&name) {
            Some(e) => write!(
                f,
                "{}[{}]",
                name,
                ReprExpr {
                    expr: e,
                    vars: vars,
                    display_mode: IdentReprMode::Mixed,
                }
            ),
            None => write!(f, "{}[?]", name),
        }
    }
}

fn fmt_ident_eager(
    name: &str,
    vars: &HashMap<String, Expr>,
    existed: bool,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    if existed {
        write!(f, "{}", name)
    } else {
        match vars.get(*&name) {
            Some(e) => write!(
                f,
                "{}",
                ReprExpr {
                    expr: e,
                    vars: vars,
                    display_mode: IdentReprMode::Eager,
                }
            ),
            None => write!(f, "{}", name),
        }
    }
}

fn fmt_ident_lazy(name: &str, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", name)
}

impl<'a, 'b> fmt::Display for ReprExpr<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        OBJECTS_BEING_PRINTED.with(|resolving| match self.expr {
            Expr::Num(n) => write!(f, "{}", n),
            Expr::Ident(name) => {
                let existed = !{ resolving.borrow_mut().insert(name.clone()) };
                let res = match self.display_mode {
                    IdentReprMode::Eager => fmt_ident_eager(name, self.vars, existed, f),
                    IdentReprMode::Lazy => match self.vars.get(name) {
                        Some(e) => fmt::Display::fmt(
                            &ReprExpr {
                                expr: e,
                                vars: self.vars,
                                display_mode: IdentReprMode::LazyInner,
                            },
                            f,
                        ),
                        None => fmt_ident_lazy(name, f),
                    },
                    IdentReprMode::LazyInner => fmt_ident_lazy(name, f),
                    IdentReprMode::Mixed => fmt_ident_mixed(name, self.vars, existed, f),
                };
                if !existed {
                    resolving.borrow_mut().remove(&*name);
                }

                res
            }
            Expr::Op(a, op, b) => write!(
                f,
                "({} {} {})",
                ReprExpr {
                    expr: a,
                    vars: self.vars,
                    display_mode: self.display_mode,
                },
                op,
                ReprExpr {
                    expr: b,
                    vars: self.vars,
                    display_mode: self.display_mode,
                }
            ),
            Expr::Resolve(e) => write!(
                f,
                "resolve {}",
                ReprExpr {
                    expr: e,
                    vars: self.vars,
                    display_mode: self.display_mode,
                }
            ),
        })
    }
}

pub fn repr_expr<'a, 'b>(
    expr: &'a Expr,
    vars: &'b HashMap<String, Expr>,
    display_mode: IdentReprMode,
) -> ReprExpr<'a, 'b> {
    ReprExpr {
        expr,
        vars,
        display_mode,
    }
}
