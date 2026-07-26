//! `gb-core` — a máquina de estados do Game Boy DMG.
//!
//! Este crate não faz I/O (R3): nem `std::fs`, nem janela, nem áudio, nem
//! `println!`. Ele expõe framebuffer, buffer de áudio e porta serial como
//! dados, e quem os leva ao mundo é `gb-cli` ou `gb-desktop`. É isso que
//! permite rodar as ROMs de teste headless na CI.
//!
//! Hoje o crate lê o cabeçalho do cartucho (ROADMAP 0.3a), monta o cartucho
//! sem mapeador (0.4), tem o banco de registradores da CPU (1.1), o barramento
//! que decide quem responde a cada endereço (1.2a), o estado com que a máquina
//! começa a rodar quando se pula a boot ROM — registradores da CPU (1.2b-i) e
//! de hardware (1.2b-ii) — e o laço que faz a CPU andar sobre o `Bus`, um
//! M-cycle por chamada ([`cpu::Cpu::step`], 1.3).
//!
//! O decodificador conhece duas instruções, `NOP` e `JP u16`. As outras 245
//! param a CPU com [`cpu::Lockup::UndecodedOpcode`] e chegam nos itens 1.4 a
//! 1.11.

#![forbid(unsafe_code)]

pub mod bus;
pub mod cart;
pub mod cpu;

/// Modelo de Game Boy que este core emula.
///
/// DMG = *Dot Matrix Game*, o Game Boy original monocromático. Não há plano
/// de suportar CGB — por isso `blargg/cgb_sound` fica fora do scoreboard.
pub const MODEL: &str = "DMG";
