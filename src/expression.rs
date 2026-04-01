use std::{collections::HashMap, fmt::Debug, rc::Rc};

pub trait CompileTimeExpression: Debug {
    fn calculate(&self, parameters: &HashMap<String, usize>) -> Option<usize>;
}

// trait RunTimeExpression {
//     fn display(&self) -> String;
// }

#[derive(Clone, Debug)]
pub struct Const {
    pub value: usize,
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum UnaryOperation {
    Clog2,
}

#[derive(Clone, Debug)]
pub struct UnaryExpression {
    pub exp: Rc<Box<dyn CompileTimeExpression>>,
    pub op: UnaryOperation,
}

#[derive(Clone, Debug)]
pub enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Max,
    Min,
}

impl BinaryOperation {
    pub fn priority(&self) -> u32 {
        match self {
            BinaryOperation::Add => 1,
            BinaryOperation::Sub => 1,
            BinaryOperation::Mul => 0,
            BinaryOperation::Div => 0,
            BinaryOperation::Mod => 0,
            BinaryOperation::Max => 0,
            BinaryOperation::Min => 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BinaryExpression {
    pub lhs: Rc<Box<dyn CompileTimeExpression>>,
    pub rhs: Rc<Box<dyn CompileTimeExpression>>,
    pub op: BinaryOperation,
}

impl CompileTimeExpression for Const {
    fn calculate(&self, _parameters: &HashMap<String, usize>) -> Option<usize> {
        Some(self.value)
    }
}

impl CompileTimeExpression for Parameter {
    fn calculate(&self, parameters: &HashMap<String, usize>) -> Option<usize> {
        if let Some(value) = parameters.get(&self.name) {
            Some(*value)
        } else {
            panic!("No value in parameters list") // TODO proper error propagation
        }
    }
}

impl CompileTimeExpression for UnaryExpression {
    fn calculate(&self, parameters: &HashMap<String, usize>) -> Option<usize> {
        let value = self.exp.calculate(parameters);
        match self.op {
            UnaryOperation::Clog2 => {
                let mut res: usize = 0;
                let mut value: usize = value?;

                while value > 0 {
                    value /= 2;
                    res += 1;
                }
                Some(res)
            }
        }
    }
}

impl BinaryExpression {
    pub const MIN_PRIORITY: u32 = 1;
}

impl CompileTimeExpression for BinaryExpression {
    fn calculate(&self, parameters: &HashMap<String, usize>) -> Option<usize> {
        let lhs: usize = self.lhs.calculate(parameters)?;
        let rhs: usize = self.rhs.calculate(parameters)?;
        match self.op {
            BinaryOperation::Add => lhs.checked_add(rhs),
            BinaryOperation::Sub => lhs.checked_sub(rhs),
            BinaryOperation::Mul => lhs.checked_mul(rhs),
            BinaryOperation::Div => lhs.checked_div(rhs),
            BinaryOperation::Mod => lhs.checked_rem(rhs),
            BinaryOperation::Max => Some(lhs.max(rhs)),
            BinaryOperation::Min => Some(lhs.min(rhs)),
        }
    }
}
