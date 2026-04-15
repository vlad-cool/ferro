use std::rc::Rc;

use crate::expression::CompileTimeExpression;

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub parameters: Vec<ParameterDeclaration>,
    pub logic: (),     // TODO
    pub interface: (), // TODO
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
    LogicType(LogicType),
}

#[derive(Clone, Debug)]
pub struct Port {
    pub name: String,
    pub direction: PortDir,
    pub width: Rc<Box<dyn CompileTimeExpression>>,
    pub modifiers: Vec<PortModifier>,
}

#[derive(Clone, Copy, Debug)]
pub enum LogicType {
    Reg,
    Wire,
    Wor,
    Wand,
    Unknown,
}

pub struct Logic {
    pub name: String,
    pub width: Rc<Box<dyn CompileTimeExpression>>,
    pub src: Option<Rc<Box<dyn CombLogic>>>,
    pub logic_type: LogicType,
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

#[derive(Clone, Copy, Debug)]
pub enum ParameterType {
    Unsigned,
}

#[derive(Clone, Debug)]
pub struct ParameterDeclaration {
    pub name: String,
    pub default: Option<usize>,
    pub parameter_type: ParameterType,
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
