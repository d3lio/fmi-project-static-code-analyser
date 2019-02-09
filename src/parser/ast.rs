use lexpar::lexer::Span;

pub trait AstVisitor {
    fn observe(&mut self, span: &Span, ast: &Ast);
    fn backtrack(&mut self, from: &Ast, to: &Ast);
    fn done(&mut self);
}

#[derive(Clone, Debug)]
pub struct AstNode {
    pub span: Span,
    pub expr: Box<Ast>,
}

impl AstNode {
    pub fn new(span: Span, expr: Ast) -> Self {
        Self {
            span,
            expr: Box::new(expr),
        }
    }

    pub fn traverse<T: AstVisitor>(&self, visitor: &mut T) {
        self.internal_traverse(visitor);

        visitor.done();
    }

    fn internal_traverse<T: AstVisitor>(&self, visitor: &mut T) {
        let expr = &*self.expr;
        visitor.observe(&self.span, expr);

        match expr {
            Ast::FunctionCall { args, .. } => for arg in args {
                arg.internal_traverse(visitor);
                visitor.backtrack(&*arg.expr, expr);
            },
            Ast::FunctionExpr { body: Block(exprs), .. } => for ex in exprs {
                ex.internal_traverse(visitor);
                visitor.backtrack(&*ex.expr, expr);
            },
            Ast::VariableDef { value, .. } => {
                if let Some(value) = value {
                    value.internal_traverse(visitor);
                    visitor.backtrack(&*value.expr, expr);
                }
            },
            Ast::Block(Block(exprs)) => for ex in exprs {
                ex.internal_traverse(visitor);
                visitor.backtrack(&*ex.expr, expr);
            },
            Ast::BinOp { lhs, rhs, .. } => {
                lhs.internal_traverse(visitor);
                visitor.backtrack(&*lhs.expr, expr);

                rhs.internal_traverse(visitor);
                visitor.backtrack(&*rhs.expr, expr);
            },
            _ => (),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Block(pub Vec<AstNode>);

#[derive(Clone, Debug)]
pub enum BinOp {
    Assign,
    Or,
    And,
    Eq,
    NotEq,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug)]
pub enum Ast {
    String(String),
    Number(f64),
    ReferenceExpr(String),
    FunctionExpr {
        name: String,
        params: Vec<String>,
        body: Block,
    },
    FunctionCall {
        name: String,
        args: Vec<AstNode>,
    },
    VariableDef {
        name: String,
        value: Option<AstNode>,
    },
    Block(Block),
    BinOp {
        op: BinOp,
        lhs: AstNode,
        rhs: AstNode,
    },
}
