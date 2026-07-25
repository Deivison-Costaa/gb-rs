//! `gb-core` — a máquina de estados do Game Boy DMG.
//!
//! Este crate não faz I/O (R3): nem `std::fs`, nem janela, nem áudio, nem
//! `println!`. Ele expõe framebuffer, buffer de áudio e porta serial como
//! dados, e quem os leva ao mundo é `gb-cli` ou `gb-desktop`. É isso que
//! permite rodar as ROMs de teste headless na CI.
//!
//! No momento o crate está vazio: o ROADMAP 0.1 entrega só o esqueleto do
//! workspace. A CPU chega em 1.1.

#![forbid(unsafe_code)]

/// Modelo de Game Boy que este core emula.
///
/// DMG = *Dot Matrix Game*, o Game Boy original monocromático. Não há plano
/// de suportar CGB — por isso `blargg/cgb_sound` fica fora do scoreboard.
pub const MODEL: &str = "DMG";
