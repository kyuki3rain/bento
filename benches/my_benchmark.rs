use criterion::{criterion_group, criterion_main, Criterion};

use mylib::evaluator::Evaluator;
use mylib::{lexer, parser};

fn bm1(c: &mut Criterion) {
    c.bench_function("v0.1.5", |b| {
        b.iter(|| {
            let input = "let a = 10;
            let b = 11;
            let add = fn(x, y){
                a + b;
            };
            let reduce = fn(x, y) {
                a - b;
            }
            add(a, b);
            add(a, add(a, add(a, b)));
            if(a == 10) {
                add(a, b);
            } else {
                reduce(a, b);
            }
            
            let i = 0;
            while (i < 10) {
                let i = i + 1;
            }
            ";

            let mut evaluator = Evaluator::new();
            let l = lexer::Lexer::new(&input);
            let mut p = parser::Parser::new(l);
            let program = p.parse_program();

            if p.errors.len() != 0 {
                let mut s = "".to_string();
                for err in p.errors {
                    s += &format!("\t{}\r\n", err);
                }
                panic!("parser errors:\r\n{}", s);
            }

            match evaluator.eval_program(program) {
                Some(obj) => println!("{}", &*obj.string()),
                None => panic!(),
            }
        })
    });
}

criterion_group!(benches, bm1);
criterion_main!(benches);
