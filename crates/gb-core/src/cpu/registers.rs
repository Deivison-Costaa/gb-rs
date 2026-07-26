//! O banco de registradores do SM83 (ROADMAP 1.1).
//!
//! Spec: `docs/reference/02-cpu.md` § CPU registers and flags (Pan Docs
//! `fe246067b695`).
//!
//! ```text
//! 16-bit |Hi |Lo | Name/Function
//! -------|---|---|--------------
//!    AF  | A | - | Accumulator & Flags
//!    BC  | B | C | BC
//!    DE  | D | E | DE
//!    HL  | H | L | HL
//!    SP  | - | - | Stack Pointer
//!    PC  | - | - | Program Counter/Pointer
//! ```
//!
//! Os `-` da coluna `Lo` não são omissão: `SP` e `PC` não se dividem em
//! registradores de 8 bits endereçáveis, e `F` não é alcançável como operando
//! de 8 bits — a lista `r8` do § CPU Instruction Set é `b c d e h l [hl] a`,
//! sem `f`. `F` só se lê e se escreve inteiro, via `AF` (o `r16stk` de
//! `PUSH`/`POP`), ou um bit de cada vez, via [`Flag`].
//!
//! O estado com que a máquina começa a rodar está em
//! [`Registers::after_boot_rom`] (ROADMAP 1.2b-i), e vem de outra spec:
//! `docs/reference/01-memory-map.md` § Console state after boot ROM hand-off.

use crate::cart::HeaderChecksum;

/// Um dos quatro flags do SM83, com o bit que a spec lhe dá.
///
/// A spec (§ The Flags Register) lista **quatro**, e só:
///
/// ```text
/// Bit | Name | Explanation
/// ----|------|-------
///   7 |   z  | Zero flag
///   6 |   n  | Subtraction flag (BCD)
///   5 |   h  | Half Carry flag (BCD)
///   4 |   c  | Carry flag
/// ```
///
/// O Z80 tem `S` (sinal) no bit 7 e `P/V` (paridade/overflow) no bit 2; o
/// § CPU Comparison with Z80 diz que os dois **foram removidos**, junto com as
/// 12 instruções condicionadas a eles. Quem portar tabela de flags de Z80 sem
/// ler aquela seção põe `S` onde mora `Z`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flag {
    /// Bit 7 — resultado igual a zero.
    Z,
    /// Bit 6 — a operação anterior foi subtração. Só o `DAA` lê.
    N,
    /// Bit 5 — carry dos 4 bits baixos. Só o `DAA` lê.
    H,
    /// Bit 4 — carry/borrow dos 8 (ou 16) bits.
    C,
}

impl Flag {
    /// O bit deste flag dentro de `F`, já como máscara.
    const fn mask(self) -> u8 {
        match self {
            Self::Z => 0b1000_0000,
            Self::N => 0b0100_0000,
            Self::H => 0b0010_0000,
            Self::C => 0b0001_0000,
        }
    }
}

/// Compõe um par de 16 bits a partir das duas metades.
const fn pair(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}

/// Divide um par de 16 bits nas duas metades, na ordem `(alto, baixo)`.
const fn split(value: u16) -> (u8, u8) {
    ((value >> 8) as u8, value as u8)
}

/// Os registradores do SM83.
///
/// Campos públicos de propósito: isto é um banco de registradores, não um
/// invariante a proteger. `regs.a = value` é o que a instrução faz, e um
/// acessor por registrador só acrescentaria ruído ao decodificador que chega
/// no 1.4. Os pares de 16 bits são métodos porque aí **há** cálculo.
///
/// O estado inicial é tudo zero. Isso **não** é o estado pós-boot do DMG
/// (`A=$01`, `SP=$FFFE`, `PC=$0100`, …): pular a boot ROM é o ROADMAP 1.2, e a
/// tabela está em `docs/reference/01-memory-map.md` § Console state after boot
/// ROM hand-off. Zero aqui é ausência de decisão, não uma decisão errada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Registers {
    /// Acumulador — o destino implícito de quase toda a ALU de 8 bits.
    pub a: u8,
    /// Flags. Os bits 7–4 são [`Flag`]; sobre os bits 3–0 ver [`Registers::set_af`].
    pub f: u8,
    /// `B` — byte alto de `BC`.
    pub b: u8,
    /// `C` — byte baixo de `BC`.
    pub c: u8,
    /// `D` — byte alto de `DE`.
    pub d: u8,
    /// `E` — byte baixo de `DE`.
    pub e: u8,
    /// `H` — byte alto de `HL`.
    pub h: u8,
    /// `L` — byte baixo de `HL`.
    pub l: u8,
    /// Stack Pointer. Não tem metades endereçáveis.
    pub sp: u16,
    /// Program Counter. Não tem metades endereçáveis.
    pub pc: u16,
}

impl Registers {
    /// O estado em que a boot ROM do **DMG** entrega o controle ao cartucho.
    ///
    /// Spec: `docs/reference/01-memory-map.md` § Console state after boot ROM
    /// hand-off → *CPU registers* (ROADMAP 1.2b-i). Este emulador não roda a
    /// boot ROM — pula direto para `PC = $0100` com os registradores que ela
    /// teria deixado.
    ///
    /// A tabela da spec tem **cinco colunas de modelo** (DMG0, DMG, MGB, SGB,
    /// SGB2) e a daqui é a **DMG**. As outras não são variações de detalhe: o
    /// DMG0, que é a coluna vizinha, tem `B=$FF`, `E=$C1`, `HL=$8403` e `F=$00`
    /// — um estado inteiro, plausível, que boota, e que erra.
    ///
    /// `F` é a única linha que não é literal. A nota de rodapé da coluna diz:
    ///
    /// > If the header checksum is $00, then the carry and half-carry flags are
    /// > clear; otherwise, they are both set.
    ///
    /// Daí o parâmetro. Ele é [`HeaderChecksum`] e não `u8` de propósito: o
    /// tipo guarda os dois checksums — o gravado em `$014D` e o calculado a
    /// partir de `$0134`–`$014C` — e a nota de rodapé se refere ao **gravado**,
    /// ligando a palavra "header checksum" à § 014D, cuja primeira frase é
    /// *"This byte contains an 8-bit checksum"*. Num cartucho que o boot ROM
    /// deixaria rodar os dois coincidem e a distinção é invisível; ela só
    /// aparece em ROM corrompida, que [`crate::cart::load`] monta de propósito.
    /// Receber um `u8` deixaria essa escolha na mão de cada chamador, onde ela
    /// some.
    ///
    /// Os bits 3–0 de `F` saem zero, e isso é consequência e não máscara: a
    /// coluna descreve `F` por quatro flags, o corpo abaixo monta `F` a partir
    /// desses quatro, e não sobra nada para pôr no nibble baixo. Continua
    /// valendo o que a 0009 registrou — o Pan Docs do commit fixado não diz uma
    /// frase sobre aqueles bits, e nada aqui os mascara.
    ///
    /// **A spec avisa que esta seção é frágil:** *"Some of the information
    /// below is highly volatile […] some of it may contain errors."* Quem cobra
    /// de verdade é a Mooneye `acceptance/boot_regs-dmgABC`, no ROADMAP 7.1.
    #[must_use]
    pub const fn after_boot_rom(checksum: HeaderChecksum) -> Self {
        let carries = checksum.stored() != 0;

        let mut regs = Self {
            a: 0x01,
            f: 0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            // Topo da HRAM: a pilha cresce para baixo a partir daqui.
            sp: 0xFFFE,
            // O ponto de entrada do cartucho. É por isso que a documentação de
            // GB fala em "start address $0100" e não `$0000`.
            pc: 0x0100,
        };

        // A linha `F` da coluna, entrada por entrada — inclusive o `N=0`, que
        // é redundante contra o `f: 0` acima. Transcrição literal: quem for
        // corrigir uma das quatro não precisa reconstruir de onde veio o resto.
        regs.set_flag(Flag::Z, true);
        regs.set_flag(Flag::N, false);
        regs.set_flag(Flag::H, carries);
        regs.set_flag(Flag::C, carries);

        regs
    }

    /// `AF` — `A` no byte alto, `F` no baixo.
    #[must_use]
    pub const fn af(&self) -> u16 {
        pair(self.a, self.f)
    }

    /// `BC` — `B` no byte alto, `C` no baixo.
    #[must_use]
    pub const fn bc(&self) -> u16 {
        pair(self.b, self.c)
    }

    /// `DE` — `D` no byte alto, `E` no baixo.
    #[must_use]
    pub const fn de(&self) -> u16 {
        pair(self.d, self.e)
    }

    /// `HL` — `H` no byte alto, `L` no baixo.
    #[must_use]
    pub const fn hl(&self) -> u16 {
        pair(self.h, self.l)
    }

    /// Escreve `AF`, **sem mascarar `F`**.
    ///
    /// Existe um folclore firme de que o nibble baixo de `F` é sempre zero, e
    /// de que `POP AF` o descarta. Pode ser verdade no silício — mas **não
    /// está na spec deste projeto**: a tabela do § The Flags Register termina
    /// no bit 4 sem dizer nada dos bits 3–0, e a string `POP AF` não aparece
    /// em nenhum dos 75 arquivos do Pan Docs no commit fixado (`fe246067b695`);
    /// o `03-opcodes.md` descreve `F1` sem mencionar máscara. Pela R1, o que
    /// não está na spec não vira código, por mais certo que pareça.
    ///
    /// Então o 1.1 carrega os 8 bits fielmente e não inventa fiação. A
    /// previsão registrada, para ser conferida e não retroajustada: se a
    /// máscara for mesmo necessária, quem cobra é a blargg
    /// `cpu_instrs/01-special` no ROADMAP 1.13. Nesse dia haverá **evidência**
    /// para trazê-la — e a fonte entra em `docs/reference/` junto.
    pub const fn set_af(&mut self, value: u16) {
        (self.a, self.f) = split(value);
    }

    /// Escreve `BC`, dividindo nas duas metades.
    pub const fn set_bc(&mut self, value: u16) {
        (self.b, self.c) = split(value);
    }

    /// Escreve `DE`, dividindo nas duas metades.
    pub const fn set_de(&mut self, value: u16) {
        (self.d, self.e) = split(value);
    }

    /// Escreve `HL`, dividindo nas duas metades.
    pub const fn set_hl(&mut self, value: u16) {
        (self.h, self.l) = split(value);
    }

    /// Lê um flag de `F`.
    #[must_use]
    pub const fn flag(&self, flag: Flag) -> bool {
        self.f & flag.mask() != 0
    }

    /// Liga ou desliga um flag, sem tocar nos outros sete bits de `F`.
    pub const fn set_flag(&mut self, flag: Flag, on: bool) {
        let mask = flag.mask();
        if on {
            self.f |= mask;
        } else {
            self.f &= !mask;
        }
    }
}
