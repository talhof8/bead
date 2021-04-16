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
    StaticAccessor, // '::'
    MemberAccessor, // '.'
    FnReturnTypeDelim, // '->'
    Comma, // ','

    Symbol { name: String }, 

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
    EnumType,
    ListType,
    DictType,
    NullValue,

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
    DelObject,

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
    Not,
    Equals,
    NotEquals,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Assignment,
}