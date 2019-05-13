use std::vec::Vec;
use num_bigint::BigInt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // Specials characters
    LeftParens, // '('
    RightParens, // ')'
    LeftCurlyBracket, // '{'
    RightCurlyBracket, // '}'
    LeftSquareBracket, // '['
    RightSquareBracket, // ']'
    Semicolon, // ';'
    Colon, // ':'
    DoubleColons, // '::'
    Dot, // '.'
    Assignment, // '='
    Hyphen, // '-'
    RightArrow, // '>'
    Comma, // ','

    // Literal values
    True,
    False,
    Null,

    Symbol { name: String }, 
    CustomType { name: String },

    // Builtin types
    IntType,
    IntValue { value: BigInt },
    FloatType,
    FloatValue { value: f64 },
    StringType,
    StringValue { value: String },
    CharType,
    CharValue { value: char },
    BoolType,
    BoolValue { value: bool },
    BytesType,
    BytesValue { value: Vec<u8> },
    TupleType,
    TupleValue,
    EnumType,
    EnumValue,
    ListType,
    ListValue,
    DictType,
    DictValue,

    // Keywords
    If,
    Elif,
    Else,
    For,
    While,
    Class,
    Function,
    Private,
    Public,
    NewInstance,
    SelfInstance,
    Constructor,
    Destructor,
    Super,
    Return,

    // Operators
    LogicalOr,
    LogicalAnd,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
    BitwiseAnd,
    BitwiseRightShift,
    BitwiseLeftShift,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}