//! ALU de 8 bits (ROADMAP 1.6a-1.6e). spec: `02-cpu.md` § The Carry Flag, § The BCD Flags.

use crate::cpu::{Flag, Registers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AluOp {
    Add,
    AddWithCarry,
    Subtract,
    SubtractWithCarry,
    Compare,
    And,
    Xor,
    Or,
}

// H/C invertem magnitude entre soma e subtração; o carry/empréstimo de entrada conta nos dois (ver docs/iterations/0023).
pub(super) fn apply(registers: &mut Registers, op: AluOp, operand: u8) {
    match op {
        AluOp::Add => add(registers, operand, 0),
        AluOp::AddWithCarry => {
            let carry_in = u8::from(registers.flag(Flag::C));
            add(registers, operand, carry_in);
        }
        AluOp::Subtract => subtract(registers, operand, 0, true),
        AluOp::SubtractWithCarry => {
            let carry_in = u8::from(registers.flag(Flag::C));
            subtract(registers, operand, carry_in, true);
        }
        AluOp::Compare => subtract(registers, operand, 0, false),
        AluOp::And => logic(registers, registers.a & operand, true),
        AluOp::Xor => logic(registers, registers.a ^ operand, false),
        AluOp::Or => logic(registers, registers.a | operand, false),
    }
}

fn add(registers: &mut Registers, operand: u8, carry_in: u8) {
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

// N é `1` literal no 1.6b; `writes_result` separa SUB/SBC (escrevem A) de CP (só produz flags).
fn subtract(registers: &mut Registers, operand: u8, carry_in: u8, writes_result: bool) {
    let accumulator = registers.a;
    let half_borrow = (accumulator & 0x0F) < (operand & 0x0F) + carry_in;
    let borrowed = u16::from(accumulator) < u16::from(operand) + u16::from(carry_in);
    let result = accumulator.wrapping_sub(operand).wrapping_sub(carry_in);

    if writes_result {
        registers.a = result;
    }
    registers.set_flag(Flag::Z, result == 0);
    registers.set_flag(Flag::N, true);
    registers.set_flag(Flag::H, half_borrow);
    registers.set_flag(Flag::C, borrowed);
}

// H/C do 1.6c são constantes na coluna, não conta: `half` chega pronto de `apply`, e `C` é sempre 0.
fn logic(registers: &mut Registers, result: u8, half: bool) {
    registers.a = result;
    registers.set_flag(Flag::Z, result == 0);
    registers.set_flag(Flag::N, false);
    registers.set_flag(Flag::H, half);
    registers.set_flag(Flag::C, false);
}

// 1.6e: primeira coluna de flag que fica como estava — `C` não entra aqui.
// O operando é `r8` ou `(HL)`, não só `A`; por isso devolve o resultado em vez
// de escrever em `registers.a` como `add`/`subtract`/`logic` fazem.
#[must_use]
pub(super) fn increment(registers: &mut Registers, value: u8) -> u8 {
    let result = value.wrapping_add(1);
    registers.set_flag(Flag::Z, result == 0);
    registers.set_flag(Flag::N, false);
    registers.set_flag(Flag::H, value & 0x0F == 0x0F);
    result
}

#[must_use]
pub(super) fn decrement(registers: &mut Registers, value: u8) -> u8 {
    let result = value.wrapping_sub(1);
    registers.set_flag(Flag::Z, result == 0);
    registers.set_flag(Flag::N, true);
    registers.set_flag(Flag::H, value & 0x0F == 0);
    result
}

// CB RLC: rotação à esquerda com Z calculado (ao contrário do RLCA).
// spec: docs/reference/03-opcodes.md § Opcodes com prefixo CB
pub(super) fn rlc(value: u8) -> (u8, bool) {
    let carry = (value & 0x80) != 0;
    let result = (value << 1) | u8::from(carry);
    (result, carry)
}

// CB RRC: rotação circular à direita. spec: docs/reference/03-opcodes.md
pub(super) fn rrc(value: u8) -> (u8, bool) {
    let carry = (value & 0x01) != 0;
    let result = (value >> 1) | (u8::from(carry) << 7);
    (result, carry)
}

// CB RL: rotação à esquerda via carry. `carry_in` é o C antigo.
pub(super) fn rl(value: u8, carry_in: bool) -> (u8, bool) {
    let carry = (value & 0x80) != 0;
    let result = (value << 1) | u8::from(carry_in);
    (result, carry)
}

// CB RR: rotação à direita via carry.
pub(super) fn rr(value: u8, carry_in: bool) -> (u8, bool) {
    let carry = (value & 0x01) != 0;
    let result = (value >> 1) | (u8::from(carry_in) << 7);
    (result, carry)
}

// CB SLA: shift left arithmetic, bit 0 ← 0.
// spec: docs/reference/03-opcodes.md § CB 20–CB 27
pub(super) fn sla(value: u8) -> (u8, bool) {
    let carry = (value & 0x80) != 0;
    let result = value << 1;
    (result, carry)
}

// CB SRA: shift right arithmetic, bit 7 preservado (sign extension).
pub(super) fn sra(value: u8) -> (u8, bool) {
    let carry = (value & 0x01) != 0;
    let result = (value >> 1) | (value & 0x80);
    (result, carry)
}

// CB SWAP: troca nibbles alto e baixo; C = 0 sempre.
// No Z80, CB 30-3F é SLL — divergência registrada em docs/reference/02-cpu.md:883.
pub(super) fn swap(value: u8) -> (u8, bool) {
    let result = value.rotate_right(4);
    (result, false)
}

// CB SRL: shift right logical, bit 7 ← 0.
pub(super) fn srl(value: u8) -> (u8, bool) {
    let carry = (value & 0x01) != 0;
    let result = value >> 1;
    (result, carry)
}

pub(super) fn rlca(registers: &mut Registers) {
    let a = registers.a;
    let carry_out = (a & 0x80) != 0;
    registers.a = (a << 1) | u8::from(carry_out);
    registers.set_flag(Flag::Z, false);
    registers.set_flag(Flag::N, false);
    registers.set_flag(Flag::H, false);
    registers.set_flag(Flag::C, carry_out);
}

pub(super) fn rrca(registers: &mut Registers) {
    let a = registers.a;
    let carry_out = (a & 0x01) != 0;
    registers.a = (a >> 1) | (u8::from(carry_out) << 7);
    registers.set_flag(Flag::Z, false);
    registers.set_flag(Flag::N, false);
    registers.set_flag(Flag::H, false);
    registers.set_flag(Flag::C, carry_out);
}

pub(super) fn rla(registers: &mut Registers) {
    let a = registers.a;
    let carry_out = (a & 0x80) != 0;
    let carry_in = u8::from(registers.flag(Flag::C));
    registers.a = (a << 1) | carry_in;
    registers.set_flag(Flag::Z, false);
    registers.set_flag(Flag::N, false);
    registers.set_flag(Flag::H, false);
    registers.set_flag(Flag::C, carry_out);
}

pub(super) fn daa(registers: &mut Registers) {
    let mut a = u16::from(registers.a);
    let n = registers.flag(Flag::N);
    let h = registers.flag(Flag::H);
    let c = registers.flag(Flag::C);

    if !n {
        if (a & 0x0F) > 9 || h {
            a = a.wrapping_add(0x06);
        }
        if a > 0x9F || c {
            a = a.wrapping_add(0x60);
            registers.set_flag(Flag::C, true);
        }
    } else {
        if h {
            a = a.wrapping_sub(0x06);
        }
        if c {
            a = a.wrapping_sub(0x60);
        }
    }

    registers.a = a as u8;
    registers.set_flag(Flag::Z, (a & 0xFF) == 0);
    registers.set_flag(Flag::H, false);
}

pub(super) fn rra(registers: &mut Registers) {
    let a = registers.a;
    let carry_out = (a & 0x01) != 0;
    let carry_in = u8::from(registers.flag(Flag::C)) << 7;
    registers.a = (a >> 1) | carry_in;
    registers.set_flag(Flag::Z, false);
    registers.set_flag(Flag::N, false);
    registers.set_flag(Flag::H, false);
    registers.set_flag(Flag::C, carry_out);
}
