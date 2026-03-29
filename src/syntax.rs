pub struct Module {
    name: String,
    parameters: Vec<()>, // TODO
    logic: (),           // TODO
    interface: (),       // TODO
}

#[derive(Clone, Copy, Debug)]
pub enum ConnectionDirection {
    Input,
    Output,
    Inout,
}

#[derive(Clone, Copy, Debug)]
pub enum ConnectionModifiers {
    Clock,
    Reset,
}

#[derive(Clone, Debug)]
pub struct Connection {
    pub name: String,
    pub direction: ConnectionDirection,
    // pub width: usize,
    pub modifiers: Vec<ConnectionModifiers>,
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
