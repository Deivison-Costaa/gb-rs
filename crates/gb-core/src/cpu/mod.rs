//! A CPU SM83 — o processador do Game Boy, que não é um Z80.
//!
//! Spec: `docs/reference/02-cpu.md` (Pan Docs `fe246067b695`) e a tabela de
//! opcodes em `docs/reference/03-opcodes.md` (gbops `90b9bf296aed`).
//!
//! Hoje este módulo tem só o banco de registradores (ROADMAP 1.1). O laço de
//! M-cycles (1.3) e o decodificador (1.4–1.11) chegam depois, e a R2 já vale
//! para eles: [`Registers`] é estado que o `step()` vai atravessar **no meio**
//! de uma instrução, não um retrato tirado entre uma instrução e outra.

mod registers;

pub use registers::{Flag, Registers};
