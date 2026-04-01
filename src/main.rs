mod expression;
mod parser;
mod syntax;

fn main() {
    crate::parser::parse_str(include_str!("../examples/some_module.fr")).unwrap();
}
