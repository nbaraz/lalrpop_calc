use std::collections::{HashMap, HashSet};

use ast::{Expr, NameError, OpCode};

pub fn resolve(expr: &Expr, vars: &HashMap<String, Expr>) -> Result<i32, NameError> {
    let mut resolving = HashSet::new();
    resolve_inner(expr, vars, &mut resolving)
}

fn resolve_inner<'this>(
    expr: &'this Expr,
    vars: &'this HashMap<String, Expr>,
    resolving: &mut HashSet<&'this str>,
) -> Result<i32, NameError> {
    match expr {
        Expr::Num(n) => Ok(*n),
        Expr::Ident(name) => {
            let existed = !resolving.insert(&name);
            if existed {
                Err(NameError(format!("~~{}", name)))
            } else {
                let res = vars
                    .get(*&name)
                    .ok_or_else(|| NameError(name.clone()))
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
) -> Result<i32, NameError> {
    use std::ops::{Add, Div, Mul, Sub};

    let a = resolve_inner(a, vars, resolving)?;
    let b = resolve_inner(b, vars, resolving)?;

    let func = match op {
        OpCode::Add => Add::add,
        OpCode::Sub => Sub::sub,
        OpCode::Mul => Mul::mul,
        OpCode::Div => {
            if b == 0 {
                return Err(NameError("/0".to_owned()));
            }
            Div::div
        }
    };

    Ok(func(a, b))
}
