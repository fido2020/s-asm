enum AluOperation {
    Add,
    Subtract,
    And,
    Or,
    Xor,
    Not,
    ShiftLeft,
    ShiftRight,
}

enum Condition {
    Always,
    Zero,
    NotZero,
    LessThan,
    GreaterThanEqual,
}

enum Instruction {
    Alu(AluOperation, Operand, Operand),
}
