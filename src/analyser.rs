use crate::parser::ast::{Ast, AstVisitor, BinOp};

use lexpar::lexer::Span;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub enum Error {
    WrongNumberOfArguments {
        name: String,
        span: Span,
    },
    UsingUndefinedVariable {
        name: String,
        span: Span,
    },
    CallingUndefinedFunction {
        name: String,
        span: Span,
    },
    UnusedVariable {
        name: String,
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub enum Def {
    Var {
        span: Span,
        name: String,
        defined: bool,
        used: bool,
        callable: bool,
    },
    Fn {
        span: Span,
        name: String,
        args: usize,
    },
}

#[derive(Debug)]
struct Context {
    name: String,
    parent: Option<Weak<RefCell<Context>>>,
    children: Vec<Rc<RefCell<Context>>>,
    defs: HashMap<String, Def>,
}

impl Context {
    fn get_mut<F: FnOnce(&mut Def)>(&mut self, name: &str, f: F) -> Option<()> {
        let mut def = self.defs.get_mut(name);
        if let Some(ref mut def) = def {
            f(def);
            Some(())
        } else {
            if let Some(ref mut parent) = self.parent {
                parent.upgrade()?.borrow_mut().get_mut(name, f)
            } else {
                None
            }
        }
    }

    fn all_unused(&self) -> Vec<Def> {
        let mut unused_defs = Vec::new();
        for (_, def) in &self.defs {
            match def {
                Def::Var { used, .. } => {
                    if !used {
                        unused_defs.push(def.clone());
                    }
                },
                _ => (),
            }
        }
        for child in &self.children {
            unused_defs.append(&mut child.borrow().all_unused());
        }
        unused_defs
    }
}

#[derive(Debug)]
pub struct StaticAnalyser {
    errors: Vec<Error>,
    context: Rc<RefCell<Context>>,
    current: Rc<RefCell<Context>>,
}

impl StaticAnalyser {
    pub fn new() -> Self {
        let root = Context {
            name: String::from(":root"),
            parent: None,
            children: Vec::new(),
            defs: HashMap::new(),
        };

        let wrapped = Rc::new(RefCell::new(root));

        Self {
            errors: Vec::new(),
            context: wrapped.clone(),
            current: wrapped,
        }
    }

    pub fn errors(&self) -> &Vec<Error> {
        &self.errors
    }
}

impl AstVisitor for StaticAnalyser {
    fn observe(&mut self, span: &Span, ast: &Ast) {
        match ast {
            Ast::VariableDef { name, value, .. } => {
                self.current.borrow_mut().defs.insert(name.clone(), Def::Var {
                    span: span.clone(),
                    name: name.clone(),
                    defined: value.is_some(),
                    used: false,
                    callable: match value {
                        Some(value) => match &*value.expr {
                            Ast::FunctionExpr { .. } => true,
                            _ => false,
                        },
                        _ => false,
                    },
                });
            },
            Ast::ReferenceExpr(name) => {
                let mut undefined = true;
                let res = self.current.borrow_mut().get_mut(name, |def| {
                    if let Def::Var { used, defined, .. } = def {
                        *used = true;
                        undefined = !*defined;
                    }
                });
                if res.is_none() || undefined {
                    self.errors.push(Error::UsingUndefinedVariable {
                        name: name.clone(),
                        span: span.clone(),
                    });
                }
            },
            Ast::FunctionExpr { name, params, .. } => {
                let mut ctx = Context {
                    name: name.clone(),
                    parent: Some(Rc::downgrade(&self.current)),
                    children: Vec::new(),
                    defs: HashMap::new(),
                };

                for param in params {
                    ctx.defs.insert(param.clone(), Def::Var {
                        span: span.clone(),
                        name: param.clone(),
                        defined: true,
                        used: false,
                        callable: true,
                    });
                }

                self.current.borrow_mut().defs.insert(name.clone(), Def::Fn {
                    name: name.clone(),
                    span: span.clone(),
                    args: params.len(),
                });

                let wrapped = Rc::new(RefCell::new(ctx));

                self.current.borrow_mut().children.push(wrapped.clone());
                self.current = wrapped;
            },
            Ast::FunctionCall { name, args } => {
                let mut a = None;
                let res = self.current.borrow_mut().get_mut(name, |def| {
                    if let Def::Fn { args, .. } = def {
                        a = Some(Some(*args));
                    }
                    if let Def::Var { callable, .. } = def {
                        if *callable {
                            a = Some(None);
                        }
                    }
                });
                if res.is_none() || a.is_none() {
                    self.errors.push(Error::CallingUndefinedFunction {
                        name: name.clone(),
                        span: span.clone(),
                    });
                    return;
                }
                if let Some(Some(a)) = a {
                    if args.len() != a {
                        self.errors.push(Error::WrongNumberOfArguments {
                            name: name.clone(),
                            span: span.clone(),
                        })
                    }
                }
            },
            Ast::BinOp { op: BinOp::Assign, lhs, .. } => {
                match &*lhs.expr {
                    Ast::ReferenceExpr(name) => {
                        let res = self.current.borrow_mut().get_mut(name, |def| {
                            if let Def::Var { defined, .. } = def {
                                *defined = true;
                            }
                        });
                        if res.is_none() {
                            self.errors.push(Error::UsingUndefinedVariable {
                                name: name.clone(),
                                span: span.clone(),
                            });
                        }
                    },
                    _ => (),
                }
            },
            _ => (),
        }
    }

    fn backtrack(&mut self, from: &Ast, _to: &Ast) {
        match from {
            Ast::FunctionExpr { .. } => {
                let p;
                if let Some(ref parent) = self.current.borrow().parent {
                    p = parent.upgrade().expect("can't upgrade parent");
                } else {
                    panic!("no parent");
                }
                self.current = p;
            },
            _ => (),
        }
    }

    fn done(&mut self) {
        let unused = self.current.borrow().all_unused();
        for unused in unused {
            if let Def::Var { name, span, .. } = unused {
                self.errors.push(Error::UnusedVariable {
                    name,
                    span,
                });
            }
        }
    }
}
