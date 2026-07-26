//! `gb-core` — a máquina de estados do Game Boy DMG.
//!
//! Este crate não faz I/O (R3): nem `std::fs`, nem janela, nem áudio, nem
//! `println!`. Ele expõe framebuffer, buffer de áudio e porta serial como
//! dados, e quem os leva ao mundo é `gb-cli` ou `gb-desktop`. É isso que
//! permite rodar as ROMs de teste headless na CI.
//!
//! Hoje o crate lê o cabeçalho do cartucho (ROADMAP 0.3a), monta o cartucho
//! sem mapeador (0.4), tem o banco de registradores da CPU (1.1), o barramento
//! que decide quem responde a cada endereço (1.2a) e o estado da CPU no
//! hand-off da boot ROM (1.2b-i). Faltam os valores iniciais dos registradores
//! de hardware (1.2b-ii) e o laço de M-cycles (1.3), que é quem vai ligar a
//! CPU ao `Bus`.

#![forbid(unsafe_code)]

pub mod bus;
pub mod cart;
pub mod cpu;

/// Modelo de Game Boy que este core emula.
///
/// DMG = *Dot Matrix Game*, o Game Boy original monocromático. Não há plano
/// de suportar CGB — por isso `blargg/cgb_sound` fica fora do scoreboard.
pub const MODEL: &str = "DMG";
