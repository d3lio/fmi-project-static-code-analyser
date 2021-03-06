pub mod ast;

use lexpar::parse_rules;
use lexpar::lexer::{LexIter, Span};

use crate::lexer::token::Token::{self, *};

use self::ast::{
    Ast,
    AstNode,
    BinOp,
    Block,
};

type Term = (Span, Token);
type Result = ::lexpar::parser::Result<AstNode, Term>;

pub struct Parser;

impl Parser {
    pub fn parse(iter: LexIter<Term>) -> Result {
        let iter = iter.filter(|x| match *x {
            (_, Token::Whitespace(_)) => false,
            (_, Token::Comment(_)) => false,
            _ => true
        });

        top_level(&mut iter.into())
    }
}

macro_rules! span {
    ($astl: expr, $astr: expr) => {
        Span::new($astl.span.lo, $astr.span.hi, $astl.span.line)
    };
}

macro_rules! ast {
    ($span: expr, $ast_expr: expr) => {
        AstNode::new($span, $ast_expr)
    };
}

parse_rules! {
    term: Term;

    top_level: AstNode => {
        [node: _top_level] => {
            let default = Span::new(0, 0, 0);
            let exprs = &node.0;
            let first_span = exprs.first().map(|node| &node.span).unwrap_or(&default);
            let last_span = exprs.last().map(|node| &node.span).unwrap_or(&default);
            AstNode::new(first_span.clone().extend(last_span.hi), Ast::Block(node))
        }
    },

    #[fold(nodes)]
    _top_level: Block => {
        [node: __top_level] => {
            nodes.0.push(node);
            nodes
        },
        [@] => Block(Vec::new())
    },

    __top_level: AstNode => {
        [def: def] => def,
        [expr: expr, o: opt_semi] => expr,
    },

    opt_semi: bool => {
        [(_, Semicolon)] => true,
        [@] => false,
    },
}

// Definition and expression parsing
parse_rules! {
    term: Term;

    def: AstNode => {
        // Mutable variable definition
        [(lspan, KwVar), (_, Ident(name)), value: def_var_val, (rspan, Semicolon)] => {
            ast!(lspan.extend(rspan.hi), Ast::VariableDef { name, value })
        },
    },

    def_var_val: Option<AstNode> => {
        [(_, Assign), ex: expr] => {
            Some(ex)
        },
        [@] => None,
    },

    #[binop(infix)]
    expr: AstNode => _expr where u32 => |lhs, rhs| {
        &(_, Assign)     | 0 => ast!(span!(lhs, rhs), Ast::BinOp { op: BinOp::Assign, lhs, rhs }),
        &(_, Eq)         | 1 => ast!(span!(lhs, rhs), Ast::BinOp { op: BinOp::Eq, lhs, rhs }),
        &(_, NotEq)      | 1 => ast!(span!(lhs, rhs), Ast::BinOp { op: BinOp::NotEq, lhs, rhs }),
        &(_, Or)         | 2 => ast!(span!(lhs, rhs), Ast::BinOp { op: BinOp::Or, lhs, rhs }),
        &(_, And)        | 2 => ast!(span!(lhs, rhs), Ast::BinOp { op: BinOp::And, lhs, rhs }),
        &(_, Plus)       | 3 => ast!(span!(lhs, rhs), Ast::BinOp { op: BinOp::Add, lhs, rhs }),
        &(_, Minus)      | 3 => ast!(span!(lhs, rhs), Ast::BinOp { op: BinOp::Sub, lhs, rhs }),
        &(_, Asterisk)   | 4 => ast!(span!(lhs, rhs), Ast::BinOp { op: BinOp::Mul, lhs, rhs }),
        &(_, FSlash)     | 4 => ast!(span!(lhs, rhs), Ast::BinOp { op: BinOp::Div, lhs, rhs }),
    },

    _expr: AstNode => {
        // Number expression
        [(span, Number(num))] => ast!(span, Ast::Number(num)),

        // String expression
        [(span, String(string))] => ast!(span, Ast::String(string)),

        // Paren expression
        [(_, LParen), ex: expr, (_, RParen)] => ex,

        // Variable expression or function call
        [(mut span, Ident(name)), args: call] => {
            if let Some((call_span, args)) = args {
                span.hi = call_span.hi;
                ast!(span, Ast::FunctionCall { name, args })
            } else {
                ast!(span, Ast::ReferenceExpr(name))
            }
        },

        // Function definition
        [
            (span, KwFunction), (_, Ident(name)),
            (_, LParen), params: params, (_, RParen),
            (_, LBrace), body: _top_level, (rblock, RBrace)
        ] => {
            ast!(span.extend(rblock.hi), Ast::FunctionExpr {
                name,
                params: params,
                body,
            })
        },
    },
}

// Variable expression or function invocation
parse_rules! {
    term: Term;

    // Variable expression or function call
    call: Option<(Span, Vec<AstNode>)> => {
        [(span, LParen), args: args, (rspan, RParen)] => {
            Some((span.extend(rspan.hi), args))
        },
        [@] => None
    },

    // Function call arguments
    args: Vec<AstNode> => {
        [ex: expr, mut args: _args] => {
            use std::mem;
            mem::forget(mem::replace(&mut args[0], ex));
            args
        },
        [@] => Vec::new()
    },

    #[fold(args)]
    _args: Vec<AstNode> => {
        [(_, Comma), ex: expr] => {
            args.push(ex);
            args
        },
        [@] => {
            let mut acc = Vec::with_capacity(8);
            unsafe { acc.push(::std::mem::uninitialized()) }
            acc
        }
    },
}

// Functions
parse_rules! {
    term: Term;

    // Function prototype params
    params: Vec<std::string::String> => {
        [(_, Ident(name)), mut params: _params] => {
            params[0] = name;
            params
        },
        [@] => Vec::new()
    },

    #[fold(params)]
    _params: Vec<std::string::String> => {
        [(_, Comma), (_, Ident(name))] => {
            params.push(name);
            params
        },
        [@] => vec![std::string::String::new()]
    },
}
