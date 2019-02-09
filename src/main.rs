mod analyser;
mod lexer;
mod parser;

use self::analyser::StaticAnalyser;

const SRC: &str = r#"
function f(a, b, c, d, e, f) {
    g() + 2 * f(a) + 3
    123
}

function g() {}
g()

function main(argv) {
    // var k = y;
    var a = 1;
    var b = 2;
    // var c;
    var c = function h() {};
    f()
}

f(((x)), g(), z, 1.1, -5, c)
"#;

pub fn run(source: &str) {
    let lexer = lexer::lexer();
    let iter = lexer.src_iter(source);
    let ast = parser::Parser::parse(iter);

    match ast {
        Ok(exprs) => {
            println!("{:?}\n", exprs);

            let mut analyser = StaticAnalyser::new();
            exprs.traverse(&mut analyser);

            println!("{:?}\n", analyser);

            for error in analyser.errors() {
                println!("{:?}", error);
            }
        },
        Err(err) => println!("{:?}", err),
    }
}

fn main() {
    run(SRC);
}
