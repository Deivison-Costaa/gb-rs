//! A CPU SM83 — o processador do Game Boy, que não é um Z80.
//!
//! Spec: `docs/reference/02-cpu.md` (Pan Docs `fe246067b695`) e a tabela de
//! opcodes em `docs/reference/03-opcodes.md` (gbops `90b9bf296aed`).
//!
//! Dois módulos, duas coisas:
//!
//! - [`Registers`] — o banco de registradores (ROADMAP 1.1) e o estado com que
//!   ele começa a rodar, do hand-off da boot ROM ([`Registers::after_boot_rom`],
//!   1.2b-i).
//! - [`Cpu`] — o laço de M-cycles (1.3), que junta aquilo ao
//!   [`crate::bus::Bus`] e o faz andar. É onde a R2 vive: [`Cpu::step`] avança
//!   **um** M-cycle e retorna.
//!
//! O decodificador ainda é de duas instruções (`NOP` e `JP u16`); os outros 245
//! opcodes chegam nos itens 1.4 a 1.11 e, até lá, param a CPU com
//! [`Lockup::UndecodedOpcode`].

mod mcycle;
mod registers;

pub use mcycle::{Cpu, Lockup};
pub use registers::{Flag, Registers};
