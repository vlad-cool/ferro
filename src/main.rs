mod expression;
mod parser;
mod syntax;
mod tokens;

fn main() {
    println!("{:?}", crate::parser::parse_str(include_str!("../examples/some_module.fr")).unwrap());
}
