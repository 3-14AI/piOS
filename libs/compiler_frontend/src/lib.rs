#![no_std]
extern crate alloc;

pub mod ast;
pub mod mir;

use alloc::collections::BTreeMap;

use alloc::string::String;
use alloc::vec::Vec;

use ast::{Expr, Module, Stmt};
use mir::{MirFunction, MirInst, MirModule, MirOp};

#[derive(Debug, PartialEq)]
pub enum CompileError {
    UndefinedVariable(String),
}

pub struct CompilerContext {
    locals: BTreeMap<String, usize>,
    next_local: usize,
    insts: Vec<MirInst>,
}

impl CompilerContext {
    pub fn new() -> Self {
        Self {
            locals: BTreeMap::new(),
            next_local: 0,
            insts: Vec::new(),
        }
    }

    pub fn define_local(&mut self, name: String) -> usize {
        if let Some(&id) = self.locals.get(&name) {
            id
        } else {
            let id = self.next_local;
            self.locals.insert(name, id);
            self.next_local += 1;
            id
        }
    }

    pub fn get_local(&self, name: &str) -> Option<usize> {
        self.locals.get(name).copied()
    }

    pub fn emit(&mut self, op: MirOp) -> usize {
        let dest = self.next_local;
        self.next_local += 1;
        self.insts.push(MirInst::Assign { dest, op });
        dest
    }

    pub fn compile_expr(&mut self, expr: &Expr) -> Result<usize, CompileError> {
        match expr {
            Expr::IntLiteral(val) => Ok(self.emit(MirOp::Const(*val))),
            Expr::Var(name) => {
                let id = self
                    .get_local(name)
                    .ok_or_else(|| CompileError::UndefinedVariable(name.clone()))?;
                Ok(self.emit(MirOp::Load(id)))
            }
            Expr::Add(lhs, rhs) => {
                let lhs_id = self.compile_expr(lhs)?;
                let rhs_id = self.compile_expr(rhs)?;
                Ok(self.emit(MirOp::Add(lhs_id, rhs_id)))
            }
            Expr::Sub(lhs, rhs) => {
                let lhs_id = self.compile_expr(lhs)?;
                let rhs_id = self.compile_expr(rhs)?;
                Ok(self.emit(MirOp::Sub(lhs_id, rhs_id)))
            }
            Expr::Mul(lhs, rhs) => {
                let lhs_id = self.compile_expr(lhs)?;
                let rhs_id = self.compile_expr(rhs)?;
                Ok(self.emit(MirOp::Mul(lhs_id, rhs_id)))
            }
            Expr::Call(name, args) => {
                let mut arg_ids = Vec::new();
                for arg in args {
                    arg_ids.push(self.compile_expr(arg)?);
                }
                Ok(self.emit(MirOp::Call(name.clone(), arg_ids)))
            }
        }
    }

    pub fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
        match stmt {
            Stmt::Let(name, expr) => {
                let expr_id = self.compile_expr(expr)?;
                let local_id = self.define_local(name.clone());
                self.insts.push(MirInst::Assign {
                    dest: local_id,
                    op: MirOp::Load(expr_id),
                });
                Ok(())
            }
            Stmt::Assign(name, expr) => {
                let expr_id = self.compile_expr(expr)?;
                let local_id = self
                    .get_local(name)
                    .ok_or_else(|| CompileError::UndefinedVariable(name.clone()))?;
                self.insts.push(MirInst::Assign {
                    dest: local_id,
                    op: MirOp::Load(expr_id),
                });
                Ok(())
            }
            Stmt::Return(expr) => {
                let expr_id = self.compile_expr(expr)?;
                self.insts.push(MirInst::Return { src: expr_id });
                Ok(())
            }
            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
                Ok(())
            }
        }
    }
}

pub fn compile_to_mir(module: &Module) -> Result<MirModule, CompileError> {
    let mut mir_functions = Vec::new();

    for func in &module.functions {
        let mut ctx = CompilerContext::new();

        // Register params as locals
        for param in &func.params {
            ctx.define_local(param.clone());
        }

        for stmt in &func.body {
            ctx.compile_stmt(stmt)?;
        }

        // Implicit return if the last instruction wasn't a return
        if let Some(last) = ctx.insts.last() {
            if !matches!(last, MirInst::Return { .. }) {
                // If there's no return, return dummy value 0
                let dummy_id = ctx.emit(MirOp::Const(0));
                ctx.insts.push(MirInst::Return { src: dummy_id });
            }
        } else {
            // Empty body
            let dummy_id = ctx.emit(MirOp::Const(0));
            ctx.insts.push(MirInst::Return { src: dummy_id });
        }

        mir_functions.push(MirFunction {
            name: func.name.clone(),
            param_count: func.params.len(),
            locals_count: ctx.next_local,
            insts: ctx.insts,
        });
    }

    Ok(MirModule {
        functions: mir_functions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::boxed::Box;
    use crate::ast::Function;
    use alloc::vec;

    #[test]
    fn test_compile_simple() {
        let function = Function {
            name: "add".into(),
            params: vec!["a".into(), "b".into()],
            body: vec![Stmt::Return(Expr::Add(
                Box::new(Expr::Var("a".into())),
                Box::new(Expr::Var("b".into())),
            ))],
        };
        let module = Module {
            functions: vec![function],
        };

        let result = compile_to_mir(&module);
        assert!(result.is_ok());
        let mir = result.unwrap();
        assert_eq!(mir.functions.len(), 1);
        let func = &mir.functions[0];
        assert_eq!(func.name, "add");
        assert_eq!(func.param_count, 2);
    }

    #[test]
    fn test_compile_let() {
        let function = Function {
            name: "let_test".into(),
            params: vec![],
            body: vec![
                Stmt::Let("x".into(), Expr::IntLiteral(42)),
                Stmt::Return(Expr::Var("x".into())),
            ],
        };
        let module = Module {
            functions: vec![function],
        };

        let result = compile_to_mir(&module);
        assert!(result.is_ok());
        let mir = result.unwrap();
        assert_eq!(mir.functions.len(), 1);
    }
}
