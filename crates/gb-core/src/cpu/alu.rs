//! ALU de 8 bits (ROADMAP 1.6a). spec: `02-cpu.md` § The Carry Flag, § The BCD Flags.

use crate::cpu::{Flag, Registers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AluOp {
    Add,
    AddWithCarry,
}

// H é o carry do nibble baixo e C o do byte; o carry de entrada faz parte do
// resultado, então conta para os dois (ver docs/iterations/0022).
pub(super) fn apply(registers: &mut Registers, op: AluOp, operand: u8) {
    let carry_in = match op {
        AluOp::Add => 0,
        AluOp::AddWithCarry => u8::from(registers.flag(Flag::C)),
    };

    let accumulator = registers.a;
    let total = u16::from(accumulator) + u16::from(operand) + u16::from(carry_in);
    let half = (accumulator & 0x0F) + (operand & 0x0F) + carry_in;

    #[expect(
        clippy::cast_possible_truncation,
        reason = "o byte baixo do total é o resultado; o que sobra é o C"
    )]
    let result = total as u8;

    registers.a = result;
    registers.set_flag(Flag::Z, result == 0);
    registers.set_flag(Flag::N, false);
    registers.set_flag(Flag::H, half > 0x0F);
    registers.set_flag(Flag::C, total > 0xFF);
}
