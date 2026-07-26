//! ROADMAP 1.4b — os imediatos de 8 bits: `LD r8,u8` e `LD (HL),u8`.
//!
//! Spec: `docs/reference/03-opcodes.md`, linhas `06 0E 16 1E 26 2E 36 3E`
//! (gbops `90b9bf296aed`), e `docs/reference/02-cpu.md` § Block 0, layout de
//! bits de `ld r8, imm8` (Pan Docs `fe246067b695`).
//!
//! # Uma regra, oito opcodes, duas formas de M-cycle
//!
//! A § Block 0 dá a codificação em quatro linhas:
//!
//! ```text
//! Bits | Campo
//!    7 | 0
//!    6 | 0
//!  5-3 | Dest (r8)
//!    2 | 1
//!    1 | 1
//!    0 | 0
//! ```
//!
//! O campo `Dest` é o mesmo `r8` do 1.4a — `b c d e h l [hl] a` nos índices 0 a
//! 7 —, então o bloco tem **oito** opcodes e o índice 6 dá `LD (HL),u8`. E
//! aqui, ao contrário da § Block 1, **não há exceção**: os oito são load. Um
//! decodificador escrito direto dos bits acerta os oito.
//!
//! O que muda entre eles é o timing, e a tabela de gbops o separa em dois:
//!
//! | Forma | Bytes | T-cycles | M-cycles (passo a passo) |
//! |---|---|---|---|
//! | `LD r,u8` | 2 | 8 | `fetch → read(u8->r)` |
//! | `LD (HL),u8` | 2 | 12 | `fetch → read(u8) → write((HL))` |
//!
//! **A linha do `$36` é o teste mais importante do arquivo.** Ela tem três
//! passos, e o terceiro é a escrita: o operando chega no M2 e vai para a
//! memória no M3. As duas metades da instrução são M-cycles diferentes, e
//! juntá-las no M2 (com um `internal` no M3 para completar os 12 T-cycles) dá o
//! mesmo total e desloca a escrita em um — invisível num teste de instrução
//! isolada, visível para o timer e a PPU.
//!
//! É a nota 26 do `STATUS.md` correndo ao contrário: o 1.4a aprendeu que
//! `LD r,(HL)` **não** tem um terceiro M-cycle onde o efeito aconteça, e essa
//! lição, generalizada, produz exatamente o `$36` errado. **A coluna é por
//! instrução.**
//!
//! # Estes são os primeiros opcodes com operando no fluxo de instruções desde
//! o `JP u16`
//!
//! O bloco do 1.4a tem 1 byte e não lê nada depois do opcode. Aqui o `PC`
//! volta a andar duas vezes por instrução, e com ele volta a pergunta que a
//! nota 25 do `STATUS.md` deixou aberta e datada: a volta do `PC` em `$FFFF` é
//! comportamento real e não estava coberta por nenhum teste — a bateria de
//! mutação da 0013 mediu que trocar `wrapping_add` por `saturating_add` ficava
//! verde. `an_immediate_load_reads_its_operand_across_the_program_counter_wrap`
//! fecha isso.
//!
//! `unwrap`/`expect` são permitidos aqui: a R6 proíbe fora de teste.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

/// `$0100` — o endereço em que a boot ROM entrega o controle ao cartucho.
const ENTRY: usize = 0x0100;

/// Um endereço de WRAM, para o `$36`, que precisa de memória que aceite
/// escrita. `HL` no hand-off vale `$014D`, que é **ROM**.
///
/// Os dois bytes são diferentes de zero pelo motivo da nota 27 do `STATUS.md`:
/// com `$C000` o byte baixo é `$00` e a WRAM começa zerada, e um teste que
/// compare contra `$00` passa sem que nada tenha sido escrito.
const SCRATCH: u16 = 0xC0A7;

/// O byte que fica em [`SCRATCH`] antes de o `$36` escrever, e que tem de
/// continuar lá até o M3.
///
/// Não é `$00` de propósito: contra um zero, "a escrita ainda não aconteceu"
/// e "a escrita aconteceu e gravou zero" são a mesma leitura.
const UNTOUCHED: u8 = 0xE7;

/// O operando imediato dos testes. Distinto de [`UNTOUCHED`] e de todas as
/// sentinelas de [`seed_registers`].
const IMMEDIATE: u8 = 0x5A;

/// Os oito valores de `r8`, na ordem dos índices 0 a 7. O índice 6 é `[hl]` e
/// não é registrador — é ele que dá a segunda forma de M-cycle deste bloco.
const R8: [Operand; 8] = [
    Operand::Register("B"),
    Operand::Register("C"),
    Operand::Register("D"),
    Operand::Register("E"),
    Operand::Register("H"),
    Operand::Register("L"),
    Operand::Memory,
    Operand::Register("A"),
];

/// Índice de `[hl]` dentro de `r8`. É o valor que dá `$36`.
const HL_INDEX: u8 = 6;

/// Um operando `r8`: ou um dos sete registradores, ou a memória em `HL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operand {
    Register(&'static str),
    Memory,
}

/// O opcode de `LD dest,u8`, montado pela fórmula da § Block 0:
/// `0b00_ddd_110`.
const fn ld_imm(dest: u8) -> u8 {
    0b0000_0110 | (dest << 3)
}

/// Uma ROM de 32 KiB com `program` em `$0100`. O resto é `$00` (`NOP`).
fn rom_with(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);
    rom
}

/// Uma CPU no hand-off da boot ROM e um barramento com esta ROM dentro.
fn machine_with_rom(rom: Vec<u8>) -> (Cpu, Bus) {
    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

/// Uma CPU no hand-off da boot ROM e um barramento com `program` em `$0100`.
fn machine(program: &[u8]) -> (Cpu, Bus) {
    machine_with_rom(rom_with(program))
}

/// Sentinelas distintas para os sete registradores, para que escrever no errado
/// nunca produza o valor certo por acaso. `HL` fica valendo [`SCRATCH`], que é
/// o endereço que o `$36` usa.
fn seed_registers(cpu: &mut Cpu) {
    cpu.registers.b = 0xB1;
    cpu.registers.c = 0xC2;
    cpu.registers.d = 0xD3;
    cpu.registers.e = 0xE4;
    cpu.registers.a = 0xA5;
    cpu.registers.set_hl(SCRATCH);
}

/// Lê um dos sete registradores pelo nome que a tabela `r8` lhe dá.
fn read_register(cpu: &Cpu, name: &str) -> u8 {
    match name {
        "B" => cpu.registers.b,
        "C" => cpu.registers.c,
        "D" => cpu.registers.d,
        "E" => cpu.registers.e,
        "H" => cpu.registers.h,
        "L" => cpu.registers.l,
        "A" => cpu.registers.a,
        other => unreachable!("{other} não é um registrador da lista r8"),
    }
}

// ---------------------------------------------------------------------------
// `LD r,u8` — 7 opcodes, 2 M-cycles
// ---------------------------------------------------------------------------

#[test]
fn every_immediate_load_into_a_register_takes_the_byte_after_the_opcode() {
    for (dest_index, dest) in R8.iter().enumerate() {
        let Operand::Register(dest) = dest else {
            continue;
        };
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 7")]
        let opcode = ld_imm(dest_index as u8);

        let (mut cpu, mut bus) = machine(&[opcode, IMMEDIATE]);
        seed_registers(&mut cpu);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            read_register(&cpu, dest),
            IMMEDIATE,
            "${opcode:02X} é `LD {dest},u8`"
        );
    }
}

#[test]
fn an_immediate_load_into_a_register_is_two_m_cycles_and_the_register_changes_on_the_second() {
    // `LD B,u8` (`$06`): `fetch → read(u8->B)`. O destino recebe o byte no
    // **segundo** M-cycle, junto com a leitura do operando — a mesma forma do
    // `LD r,(HL)` do 1.4a, e pelo mesmo motivo: os 8 T-cycles não deixam onde
    // pôr um terceiro M-cycle.
    let (mut cpu, mut bus) = machine(&[0x06, IMMEDIATE]);
    seed_registers(&mut cpu);
    let before = cpu.registers.b;

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.b, before,
        "no M1 só o opcode chegou; o operando ainda está na ROM"
    );
    assert_eq!(cpu.registers.pc, 0x0101, "o M1 passou pelo opcode, e só");
    assert!(
        !cpu.is_between_instructions(),
        "`LD B,u8` tem 8 T-cycles: uma implementação que já terminou no fetch \
         leu dois bytes num M-cycle"
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.b, IMMEDIATE, "o M2 é o `read(u8->B)`");
    assert_eq!(cpu.registers.pc, 0x0102, "e é ele que passa pelo operando");
    assert!(
        cpu.is_between_instructions(),
        "dois M-cycles e acabou: não há terceiro"
    );
}

#[test]
fn an_immediate_load_does_not_execute_its_operand() {
    // O operando é `$D3`, um dos onze opcodes que não existem. Se a instrução
    // tivesse 1 byte, o M-cycle seguinte o buscaria como opcode e travaria — e
    // o lockup diz *qual* byte foi lido a mais, o que um `assert_eq!` no `PC`
    // sozinho não diz.
    let (mut cpu, mut bus) = machine(&[0x06, 0xD3]);
    seed_registers(&mut cpu);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.b, 0xD3,
        "`LD B,$D3` carrega o byte: aqui ele é dado, não instrução"
    );
    assert_eq!(cpu.lockup(), None, "e ninguém tentou decodificá-lo");
    assert_eq!(cpu.registers.pc, 0x0102, "`LD B,u8` tem 2 bytes");
}

#[test]
fn an_immediate_load_touches_nothing_but_the_destination() {
    // `LD D,u8` (`$16`). Um `assert_eq!` na struct inteira pega o que uma lista
    // de asserções por registrador não pega: o decodificador que acerta o
    // destino e escreve em mais alguém de lambuja.
    let (mut cpu, mut bus) = machine(&[0x16, IMMEDIATE]);
    seed_registers(&mut cpu);

    let mut expected = cpu.registers;
    expected.d = IMMEDIATE;
    expected.pc = 0x0102;

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers, expected,
        "`LD D,u8` tem `-` nas quatro colunas de flag e mexe em um registrador só"
    );
}

#[test]
fn an_immediate_load_reads_its_operand_across_the_program_counter_wrap() {
    // Nota 25 do `STATUS.md`: a volta do `PC` em `$FFFF` é comportamento real
    // e estava **medida como não coberta** — a bateria de mutação da 0013
    // trocou `wrapping_add` por `saturating_add` no `PC` e a suíte ficou verde.
    // Este é o primeiro item desde então com operando lido do fluxo de
    // instruções, e portanto o primeiro que pode fechar o buraco.
    //
    // O opcode vem de `$FFFF`, que é o `IE` — o único byte gravável naquele
    // endereço. O operando vem de `$0000`, que é ROM. Com `saturating_add` o
    // `PC` empaca em `$FFFF` e o M2 lê o próprio opcode de volta.
    let mut rom = rom_with(&[]);
    rom[0x0000] = IMMEDIATE;
    let (mut cpu, mut bus) = machine_with_rom(rom);
    seed_registers(&mut cpu);

    bus.write(0xFFFF, 0x06);
    cpu.registers.pc = 0xFFFF;

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0000,
        "o `PC` é de 16 bits e dá a volta: depois de `$FFFF` vem `$0000`"
    );

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.b, IMMEDIATE,
        "o operando de `LD B,u8` é o byte seguinte, e `$0000` é o byte \
         seguinte a `$FFFF`"
    );
    assert_eq!(cpu.registers.pc, 0x0001, "e o `PC` continua andando dali");
}

// ---------------------------------------------------------------------------
// `LD (HL),u8` — 1 opcode, 3 M-cycles
// ---------------------------------------------------------------------------

#[test]
fn storing_an_immediate_at_hl_writes_the_byte_after_the_opcode() {
    assert_eq!(
        ld_imm(HL_INDEX),
        0x36,
        "a fórmula `00 ddd 110` com o destino em `[hl]` dá $36"
    );

    let (mut cpu, mut bus) = machine(&[0x36, IMMEDIATE]);
    seed_registers(&mut cpu);
    bus.write(SCRATCH, UNTOUCHED);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(SCRATCH),
        IMMEDIATE,
        "`LD (HL),u8` grava o imediato onde `HL` aponta"
    );
    assert_eq!(cpu.registers.pc, 0x0102, "`LD (HL),u8` tem 2 bytes");
    assert_eq!(
        cpu.registers.hl(),
        SCRATCH,
        "e `HL` não se move: quem mexe em `HL` é o 1.4c"
    );
}

#[test]
fn storing_an_immediate_at_hl_is_three_m_cycles_and_the_write_is_the_third() {
    // `$36`: `fetch → read(u8) → write((HL))`. O operando chega no M2 e vai
    // para a memória no M3 — são M-cycles diferentes.
    //
    // O erro que este teste existe para pegar não é "esqueci um M-cycle": é
    // ler **e** escrever no M2 e completar os 12 T-cycles com um `internal` no
    // M3. Mesmo total, escrita um M-cycle adiantada. Vem de generalizar a
    // lição do 1.4a (`LD r,(HL)` não tem terceiro M-cycle) para uma instrução
    // cuja coluna tem três passos — a nota 26 do `STATUS.md` ao contrário.
    let (mut cpu, mut bus) = machine(&[0x36, IMMEDIATE]);
    seed_registers(&mut cpu);
    bus.write(SCRATCH, UNTOUCHED);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "o M1 é o fetch do opcode");
    assert_eq!(
        bus.read(SCRATCH),
        UNTOUCHED,
        "no M1 só o opcode chegou; nada foi escrito"
    );
    assert!(
        !cpu.is_between_instructions(),
        "`LD (HL),u8` tem 12 T-cycles: três M-cycles"
    );

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0102,
        "o M2 é o `read(u8)`: o `PC` passa pelo operando"
    );
    assert_eq!(
        bus.read(SCRATCH),
        UNTOUCHED,
        "e a escrita ainda **não** aconteceu — ela é o M3, não este M-cycle \
         com um `internal` depois"
    );
    assert!(
        !cpu.is_between_instructions(),
        "ainda falta o `write((HL))`"
    );

    cpu.step(&mut bus);
    assert_eq!(bus.read(SCRATCH), IMMEDIATE, "o M3 é o `write((HL))`");
    assert!(
        cpu.is_between_instructions(),
        "três M-cycles e acabou: não há quarto"
    );
}

#[test]
fn storing_an_immediate_at_hl_does_not_execute_its_operand() {
    let (mut cpu, mut bus) = machine(&[0x36, 0xD3]);
    seed_registers(&mut cpu);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(SCRATCH),
        0xD3,
        "`LD (HL),$D3` grava o byte: aqui ele é dado, não instrução"
    );
    assert_eq!(cpu.lockup(), None, "e ninguém tentou decodificá-lo");
}

// ---------------------------------------------------------------------------
// Flags: as quatro colunas são `-` nas oito linhas
// ---------------------------------------------------------------------------

#[test]
fn no_immediate_load_touches_the_flags() {
    // Um `F` com os quatro flags ligados e o nibble baixo sujo, pelo motivo do
    // 1.4a: o 1.1 decidiu **não** mascarar os bits 3–0, e um `LD` que os
    // limpasse de passagem seria a máscara entrando pela porta dos fundos.
    const DIRTY_F: u8 = 0b1111_1010;

    for dest_index in 0..8u8 {
        let opcode = ld_imm(dest_index);

        let (mut cpu, mut bus) = machine(&[opcode, IMMEDIATE]);
        seed_registers(&mut cpu);
        cpu.registers.f = DIRTY_F;

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.f, DIRTY_F,
            "${opcode:02X} tem `-` nas quatro colunas de flag da tabela"
        );
    }
}

// ---------------------------------------------------------------------------
// Controle negativo: exatamente os oito de `00 ddd 110`
// ---------------------------------------------------------------------------

#[test]
fn the_block_this_item_decodes_is_exactly_the_eight_opcodes_of_00_ddd_110() {
    // Nota 25 do `STATUS.md`: teste que afirma *pertinência* ("estes oito são
    // loads") não pega *excesso* ("e mais um"). O bloco `00 ddd 110` é vizinho
    // de duas famílias que compartilham prefixo — `INC r8` (`00 ddd 100`) e
    // `DEC r8` (`00 ddd 101`) — e uma máscara frouxa nos bits 2-0 as engoliria
    // sem que nenhum teste acima reclamasse. Por isso a varredura é dos 256.
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00]);
        seed_registers(&mut cpu);

        // Três M-cycles: o bastante para a forma mais longa deste sub-item
        // acabar, e de menos para `JP u16`, que só desvia no M4.
        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        // Os blocos já decodificados: `NOP` e `JP u16` (1.3), o `01 ddd sss`
        // sem o `$76` (1.4a), o `00 ddd 110` inteiro (este item), o
        // `00 mm x010` (`$02 $0A $12 $1A $22 $2A $32 $3A`, o 1.4c) e os seis
        // do 1.4d (`$E0 $E2 $EA $F0 $F2 $FA`) — estes um a um, porque não há
        // máscara que os pegue sem levar `$E8`/`$F8` (o 1.7) junto.
        let previously_decoded = opcode == 0x00
            || opcode == 0xC3
            || ((0x40..=0x7F).contains(&opcode) && opcode != 0x76)
            || opcode & 0b1100_0111 == 0b0000_0010
            || matches!(opcode, 0xE0 | 0xE2 | 0xEA | 0xF0 | 0xF2 | 0xFA);
        let in_block = opcode & 0b1100_0111 == 0b0000_0110;

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} está no bloco `00 ddd 110` e o 1.4b o decodifica"
            );
        } else if ILLEGAL.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::IllegalOpcode(opcode)),
                "${opcode:02X} continua sendo um dos onze que não existem"
            );
        } else if !previously_decoded {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::UndecodedOpcode(opcode)),
                "${opcode:02X} é opcode legítimo fora deste sub-item: o rótulo \
                 tem de continuar dizendo `falta implementar`"
            );
        }
    }
}
