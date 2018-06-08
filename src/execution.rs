use std::collections::{HashMap, HashSet};

use ast::{Expr, OpCode};

pub fn resolve(expr: &Expr, vars: &HashMap<String, Expr>) -> Result<i32, ExecError> {
    let mut resolving = HashSet::new();
    resolve_inner(expr, vars, &mut resolving)
}

fn resolve_inner<'this>(
    expr: &'this Expr,
    vars: &'this HashMap<String, Expr>,
    resolving: &mut HashSet<&'this str>,
) -> Result<i32, ExecError> {
    match expr {
        Expr::Num(n) => Ok(*n),
        Expr::Ident(name) => {
            let existed = !resolving.insert(&name);
            if existed {
                Err(ExecError::Recursion { identifier: name.clone() })
            } else {
                let res = vars
                    .get(*&name)
                    .ok_or_else(|| ExecError::Name(name.clone()))
                    .and_then(|e| resolve_inner(*&e, vars, resolving));
                resolving.remove(name.as_str());
                res
            }
        }
        Expr::Op(a, op, b) => resolve_binop(vars, resolving, a, *op, b),
        Expr::Resolve(e) => resolve_inner(*&e, vars, resolving),
    }
}

fn resolve_binop<'this>(
    vars: &'this HashMap<String, Expr>,
    resolving: &mut HashSet<&'this str>,
    a: &'this Expr,
    op: OpCode,
    b: &'this Expr,
) -> Result<i32, ExecError> {
    use std::ops::{Add, Div, Mul, Sub};

    let a = resolve_inner(a, vars, resolving)?;
    let b = resolve_inner(b, vars, resolving)?;

    let func = match op {
        OpCode::Add => Add::add,
        OpCode::Sub => Sub::sub,
        OpCode::Mul => Mul::mul,
        OpCode::Div => {
            if b == 0 {
                return Err(ExecError::DivisionByZero { dividend: a });
            }
            Div::div
        }
    };

    Ok(func(a, b))
}

#[derive(Debug, Fail)]
pub enum ExecError {
    #[fail(display = "Identifier `{}` not defined", _0)]
    Name(String),

    #[fail(display = "Tried to divide {} by zero", dividend)]
    DivisionByZero { dividend: i32 },

    #[fail(display = "Identifier `{}` is defined recursively", identifier)]
    Recursion { identifier: String },
}
