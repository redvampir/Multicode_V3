use std::fmt;

use serde::{Deserialize, Serialize};

use super::blocks::{
    ArithmeticBlock, BlockType, ConditionalBlock, FunctionBlock, LoopBlock, VariableBlock,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Russian,
    Spanish,
    German,
}

impl Language {
    pub const ALL: [Language; 4] = [
        Language::English,
        Language::Russian,
        Language::Spanish,
        Language::German,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Russian => "ru",
            Language::Spanish => "es",
            Language::German => "de",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Language::English => Language::Russian,
            Language::Russian => Language::Spanish,
            Language::Spanish => Language::German,
            Language::German => Language::English,
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::English => write!(f, "English"),
            Language::Russian => write!(f, "Русский"),
            Language::Spanish => write!(f, "Español"),
            Language::German => write!(f, "Deutsch"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockTranslation {
    pub block: BlockType,
    pub en: &'static str,
    pub ru: &'static str,
}

impl BlockTranslation {
    pub const fn new(block: BlockType, en: &'static str, ru: &'static str) -> Self {
        Self { block, en, ru }
    }

    pub fn get(&self, lang: Language) -> &'static str {
        match lang {
            Language::Russian => self.ru,
            _ => self.en,
        }
    }
}

pub const BLOCK_TRANSLATIONS: &[BlockTranslation] = &[
    // Arithmetic
    BlockTranslation::new(
        BlockType::Arithmetic(ArithmeticBlock::Add),
        "Add",
        "Сложить",
    ),
    BlockTranslation::new(
        BlockType::Arithmetic(ArithmeticBlock::Subtract),
        "Subtract",
        "Вычесть",
    ),
    BlockTranslation::new(
        BlockType::Arithmetic(ArithmeticBlock::Multiply),
        "Multiply",
        "Умножить",
    ),
    BlockTranslation::new(
        BlockType::Arithmetic(ArithmeticBlock::Divide),
        "Divide",
        "Делить",
    ),
    // Conditional
    BlockTranslation::new(BlockType::Conditional(ConditionalBlock::If), "If", "Если"),
    BlockTranslation::new(
        BlockType::Conditional(ConditionalBlock::ElseIf),
        "Else If",
        "Иначе если",
    ),
    BlockTranslation::new(
        BlockType::Conditional(ConditionalBlock::Else),
        "Else",
        "Иначе",
    ),
    // Loops
    BlockTranslation::new(BlockType::Loop(LoopBlock::For), "For", "Для"),
    BlockTranslation::new(BlockType::Loop(LoopBlock::While), "While", "Пока"),
    BlockTranslation::new(BlockType::Loop(LoopBlock::Loop), "Loop", "Цикл"),
    // Variables
    BlockTranslation::new(BlockType::Variable(VariableBlock::Set), "Set", "Присвоить"),
    BlockTranslation::new(BlockType::Variable(VariableBlock::Get), "Get", "Получить"),
    // Functions
    BlockTranslation::new(
        BlockType::Function(FunctionBlock::Define),
        "Define",
        "Определить",
    ),
    BlockTranslation::new(BlockType::Function(FunctionBlock::Call), "Call", "Вызвать"),
    BlockTranslation::new(
        BlockType::Function(FunctionBlock::Return),
        "Return",
        "Возврат",
    ),
];

pub fn translate(block: BlockType, lang: Language) -> Option<&'static str> {
    BLOCK_TRANSLATIONS
        .iter()
        .find(|t| t.block == block)
        .map(|t| t.get(lang))
}

pub fn translate_kind(kind: &str, lang: Language) -> Option<&'static str> {
    let bt = match kind {
        "Add" => BlockType::Arithmetic(ArithmeticBlock::Add),
        "Subtract" => BlockType::Arithmetic(ArithmeticBlock::Subtract),
        "Multiply" => BlockType::Arithmetic(ArithmeticBlock::Multiply),
        "Divide" => BlockType::Arithmetic(ArithmeticBlock::Divide),
        "If" | "Condition" => BlockType::Conditional(ConditionalBlock::If),
        "ElseIf" => BlockType::Conditional(ConditionalBlock::ElseIf),
        "Else" => BlockType::Conditional(ConditionalBlock::Else),
        "For" => BlockType::Loop(LoopBlock::For),
        "While" => BlockType::Loop(LoopBlock::While),
        "Loop" => BlockType::Loop(LoopBlock::Loop),
        "Set" => BlockType::Variable(VariableBlock::Set),
        "Get" => BlockType::Variable(VariableBlock::Get),
        "Function" | "Define" => BlockType::Function(FunctionBlock::Define),
        "Call" => BlockType::Function(FunctionBlock::Call),
        "Return" => BlockType::Function(FunctionBlock::Return),
        _ => return None,
    };
    translate(bt, lang)
}

pub const BLOCK_SYNONYMS: &[(&str, &[&str])] = &[
    ("Add", &["plus", "sum", "сложить", "прибавить"]),
    ("Subtract", &["minus", "difference", "вычесть", "минус"]),
    ("Loop", &["repeat", "повтор", "цикл"]),
];

pub fn block_synonyms(kind: &str) -> Option<&'static [&'static str]> {
    BLOCK_SYNONYMS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, v)| *v)
}
