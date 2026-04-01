use std::rc::Rc;

use crate::expression::CompileTimeExpression;

pub struct Module {
    name: String,
    parameters: Vec<()>, // TODO
    logic: (),           // TODO
    interface: (),       // TODO
}

#[derive(Clone, Copy, Debug)]
pub enum PortDir {
    Input,
    Output,
    Inout,
}

#[derive(Clone, Copy, Debug)]
pub struct Frequency {
    frequency: u64,
}

#[derive(Clone, Copy, Debug)]
pub enum PortModifier {
    Clock(Frequency),
    Reset,
}

#[derive(Clone, Debug)]
pub struct Port {
    pub name: String,
    pub direction: PortDir,
    pub width: Rc<Box<dyn CompileTimeExpression>>,
    pub modifiers: Vec<PortModifier>,
}

pub struct Identifier {
    identifier: Vec<String>,
}

pub struct ModuleInstance {
    module: String,
    identifier: Identifier,
}

pub struct Wire {
    identifier: Identifier,
    // pin: Option(String),
}

pub enum Token {
    Module(),
}

struct Net {
    name: String,
}

struct Parameter {
    name: String,
    value: u32,
}

struct FlipFlop<'a> {
    clock: &'a Net,
    reset: &'a Net,
    input: &'a Net,
    output: &'a Net,
}

pub trait CombLogic {
    // fn eval()
}

// struct Module<'a> {
//     name: String,
//     inputs: std::vec::Vec<&'a Net>,
//     outputs: std::vec::Vec<&'a Net>,
//     inouts: std::vec::Vec<&'a Net>,

//     nets: std::vec::Vec<Net>,
//     flip_flops: std::vec::Vec<FlipFlop<'a>,
//     comb_logic: std::vec::Vec<Box<dyn CombLogic>>,
// }
