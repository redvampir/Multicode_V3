use iced::Color;

pub mod arithmetic;
pub mod conditional;
pub mod functions;
pub mod loops;
pub mod variables;

pub use arithmetic::ArithmeticBlock;
pub use conditional::ConditionalBlock;
pub use functions::FunctionBlock;
pub use loops::LoopBlock;
pub use variables::VariableBlock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Arithmetic(ArithmeticBlock),
    Conditional(ConditionalBlock),
    Loop(LoopBlock),
    Variable(VariableBlock),
    Function(FunctionBlock),
}

#[derive(Debug, Clone, Copy)]
pub struct BlockColors {
    pub arithmetic: Color,
    pub conditional: Color,
    pub loops: Color,
    pub variables: Color,
    pub functions: Color,
}

impl Default for BlockColors {
    fn default() -> Self {
        Self {
            arithmetic: Color::from_rgb(0.9, 0.3, 0.3),
            conditional: Color::from_rgb(0.3, 0.9, 0.3),
            loops: Color::from_rgb(0.3, 0.3, 0.9),
            variables: Color::from_rgb(0.9, 0.9, 0.3),
            functions: Color::from_rgb(0.9, 0.3, 0.9),
        }
    }
}
