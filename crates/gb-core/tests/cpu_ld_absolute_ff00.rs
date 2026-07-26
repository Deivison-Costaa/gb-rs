//! ROADMAP 1.4d — endereço absoluto e a página `$FF00`: `LD (u16),A`,
//! `LD A,(u16)`, `LD (FF00+u8),A`, `LD A,(FF00+u8)`, `LD (FF00+C),A` e
//! `LD A,(FF00+C)`.
//!
//! Spec: `docs/reference/03-opcodes.md`, linhas `E0 E2 EA F0 F2 FA` (gbops
//! `90b9bf296aed`), e `docs/reference/02-cpu.md` § Block 3 e § Moved, Removed,
//! and Added Opcodes (Pan Docs `fe246067b695`).
//!
//! # Seis opcodes, três pares, e nenhuma máscara
//!
//! Os três sub-itens anteriores do 1.4 eram blocos de bits: `01 ddd sss`,
//! `00 ddd 110`, `00 mm x010`. Este **não é**. Os seis são três pares, e o que
//! separa cada par é o bit 4 — `$Ex` escreve, `$Fx` lê:
//!
//! | Opcode | Instrução | Bytes | T-cycles | M-cycles (passo a passo) |
//! |---|---|---|---|---|
//! | `$E2` | `LD (FF00+C),A` | 1 | 8 | `fetch → write(A->(FF00+C))` |
//! | `$F2` | `LD A,(FF00+C)` | 1 | 8 | `fetch → read((FF00+C)->A)` |
//! | `$E0` | `LD (FF00+u8),A` | 2 | 12 | `fetch → read(u8) → write(A->(FF00+u8))` |
//! | `$F0` | `LD A,(FF00+u8)` | 2 | 12 | `fetch → read(u8) → read((FF00+u8)->A)` |
//! | `$EA` | `LD (u16),A` | 3 | 16 | `fetch → read(u16:lower) → read(u16:upper) → write(A->(u16))` |
//! | `$FA` | `LD A,(u16)` | 3 | 16 | `fetch → read(u16:lower) → read(u16:upper) → read((u16)->A)` |
//!
//! Não há uma máscara que reconheça os seis e mais ninguém: os bits constantes
//! entre eles são só `111` no topo e o bit 0 zerado, o que deixaria passar
//! dezesseis opcodes. Em particular `$E8` (`ADD SP,i8`) e `$F8` (`LD HL,SP+i8`),
//! que são o 1.7, entram em qualquer máscara frouxa o bastante para pegar
//! `$E0`, `$E2` e `$EA` de uma vez. O reconhecimento é opcode a opcode, e quem
//! guarda isso é a varredura dos 256 no fim do arquivo.
//!
//! # Onde a § Block 3 está corrompida, e como três fontes fecham a conta
//!
//! A nota 24 do `STATUS.md` registra que a conversão HTML→Markdown do
//! `02-cpu.md` perdeu a prosa e emendou tabelas. Aqui a perda é a maior do
//! arquivo: os **seis** layouts de bits deste sub-item aparecem empilhados sob
//! **um** cabeçalho só, `ldh [c], a`, sem nada que diga onde um acaba e o outro
//! começa (linhas 627–678). São seis blocos de oito linhas `| bit | valor |`:
//!
//! ```text
//! 1110 0010   ldh [c], a          $E2
//! 1110 0000   ldh [imm8], a       $E0
//! 1110 1010   ld [imm16], a       $EA
//! 1111 0010   ldh a, [c]          $F2
//! 1111 0000   ldh a, [imm8]       $F0
//! 1111 1010   ld a, [imm16]       $FA
//! ```
//!
//! Os mnemônicos acima **não** estão no arquivo — só o primeiro está, e ele
//! virou o cabeçalho dos seis. O que amarra cada layout a um opcode são as
//! outras duas fontes, que concordam entre si e com os bits:
//!
//! - a tabela de gbops (`03-opcodes.md`), que dá as seis linhas da tabela acima;
//! - a § Moved, Removed, and Added Opcodes, que lista `E0 → LD (FF00+n),A`,
//!   `E2 → LD (FF00+C),A`, `EA → LD (nn),A`, `F0 → LD A,(FF00+n)`,
//!   `F2 → LD A,(FF00+C)` e `FA → LD A,(nn)` na coluna `GB CPU`.
//!
//! A mesma seção diz por que estes opcodes existem: *"Unlike the 8080 and Z80,
//! the Game Boy has no dedicated I/O bus and no IN/OUT opcodes. Instead, I/O
//! ports are accessed directly by normal LD instructions, or by new LD (FF00+n)
//! opcodes."* A página `$FF00` não é um espaço de endereçamento à parte — é o
//! fim do mapa de memória, e estes seis opcodes só encurtam o caminho até lá.
//!
//! # `$E2` e `$F2` têm **um** byte
//!
//! A coluna `Bytes` de gbops diz `1`, e é o ponto em que a memória deste agente
//! está errada: tabelas de opcode que circulam há décadas listam `LDH (C),A`
//! como instrução de dois bytes, com um operando que não existe. O registrador
//! `C` **é** o operando, e ele já está na CPU.
//!
//! O erro é barato de cometer e caro de achar: o efeito na memória fica certo,
//! os flags ficam certos, os 8 T-cycles ficam certos, e o `PC` para um byte
//! adiante do que devia — o que desalinha o fluxo de instruções a partir dali.
//! Nenhuma asserção sobre o alvo o pega; só uma sobre o `PC`, ou sobre a
//! instrução seguinte.
//!
//! `unwrap`/`expect` são permitidos aqui: a R6 proíbe fora de teste.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

/// `$0100` — o endereço em que a boot ROM entrega o controle ao cartucho.
const ENTRY: usize = 0x0100;

// ---------------------------------------------------------------------------
// Os seis opcodes, montados dos layouts de bits da § Block 3
// ---------------------------------------------------------------------------

/// `ldh [c], a` — o primeiro dos seis layouts, e o único cujo mnemônico
/// sobreviveu à conversão.
const LDH_C_A: u8 = 0b1110_0010;
/// `ldh [imm8], a` — o segundo layout.
const LDH_IMM8_A: u8 = 0b1110_0000;
/// `ld [imm16], a` — o terceiro layout.
const LD_IMM16_A: u8 = 0b1110_1010;
/// `ldh a, [c]` — o quarto layout.
const LDH_A_C: u8 = 0b1111_0010;
/// `ldh a, [imm8]` — o quinto layout.
const LDH_A_IMM8: u8 = 0b1111_0000;
/// `ld a, [imm16]` — o sexto layout.
const LD_A_IMM16: u8 = 0b1111_1010;

/// Os seis, para a varredura dos 256 e para os laços que percorrem todos.
const THIS_ITEM: [u8; 6] = [
    LDH_C_A, LDH_IMM8_A, LD_IMM16_A, LDH_A_C, LDH_A_IMM8, LD_A_IMM16,
];

// ---------------------------------------------------------------------------
// Endereços e valores
// ---------------------------------------------------------------------------

/// O deslocamento imediato de `$E0`/`$F0`. `$FF00 + $85` é HRAM, que tem célula
/// de verdade — ao contrário dos 87 endereços de `$FF00`–`$FF7F` que a 0012
/// deixou sem dono, e que leem `$FF` e engolem escrita.
const OFFSET_IMM8: u8 = 0x85;
/// `$FF85`. Ver [`OFFSET_IMM8`].
const VIA_IMM8: u16 = 0xFF00 | OFFSET_IMM8 as u16;

/// O deslocamento que `$E2`/`$F2` leem de `C`. Diferente de [`OFFSET_IMM8`] para
/// que os dois testes não possam se cobrir por acidente.
const OFFSET_IN_C: u8 = 0x8A;
/// `$FF8A`. Ver [`OFFSET_IN_C`].
const VIA_C: u16 = 0xFF00 | OFFSET_IN_C as u16;

/// O que fica em `B`: um valor que não é [`OFFSET_IN_C`], para que usar `B` no
/// lugar de `C` erre o endereço. `$FF12` é `NR12`, registrador nomeado com
/// célula e valor inicial `$F3` — endereço válido, byte diferente.
const DECOY_IN_B: u8 = 0x12;

/// O endereço absoluto de `$EA`/`$FA`, em WRAM. Os dois bytes são distintos e
/// nenhum é zero, pelo motivo da nota 27 do `STATUS.md`.
const ABSOLUTE: u16 = 0xC3D7;

/// `ABSOLUTE` com os bytes trocados — e **também** WRAM, de propósito.
///
/// Uma implementação que leia o operando em big-endian escreve num endereço que
/// existe e aceita a escrita, sem estourar nada. O teste é que separa os dois.
const ABSOLUTE_SWAPPED: u16 = 0xD7C3;

/// O byte em `A`, que as três formas de escrita levam para a memória.
const STORED: u8 = 0x5A;

/// O byte que já está no endereço alvo antes de uma escrita, e que tem de
/// continuar lá até o M-cycle do acesso.
///
/// Não é `$00` de propósito: contra um zero, "a escrita ainda não aconteceu" e
/// "a escrita aconteceu e gravou zero" são a mesma leitura.
const UNTOUCHED: u8 = 0x6D;

/// O byte no endereço alvo, que as três formas de leitura levam para `A`.
const AT_TARGET: u8 = 0xE7;

/// O byte nos vizinhos do alvo. Ler do endereço errado por um nunca devolve
/// [`AT_TARGET`].
const AT_NEIGHBOUR: u8 = 0x3C;

// ---------------------------------------------------------------------------
// Montagem da máquina
// ---------------------------------------------------------------------------

/// Uma ROM de 32 KiB com `program` em `$0100`. O resto é `$00` (`NOP`).
fn rom_with(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);
    rom
}

/// Uma CPU no hand-off da boot ROM e um barramento com `program` em `$0100`.
fn machine(program: &[u8]) -> (Cpu, Bus) {
    let rom = rom_with(program);
    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

/// Sentinela em `A`, o deslocamento em `C`, e o chamariz em `B`.
fn seed_registers(cpu: &mut Cpu) {
    cpu.registers.a = STORED;
    cpu.registers.b = DECOY_IN_B;
    cpu.registers.c = OFFSET_IN_C;
}

/// Deixa [`AT_TARGET`] no alvo e [`AT_NEIGHBOUR`] nos dois vizinhos.
fn seed_memory_around(bus: &mut Bus, address: u16) {
    bus.write(address.wrapping_sub(1), AT_NEIGHBOUR);
    bus.write(address, AT_TARGET);
    bus.write(address.wrapping_add(1), AT_NEIGHBOUR);
}

// ---------------------------------------------------------------------------
// A codificação: seis opcodes, e são estes seis
// ---------------------------------------------------------------------------

#[test]
fn the_six_bit_layouts_of_block_3_give_the_six_opcodes_gbops_lists() {
    // A § Block 3 empilha os seis layouts sob o cabeçalho `ldh [c], a` e não diz
    // qual é qual. Esta asserção é o que amarra a leitura dos bits às duas
    // fontes que não se perderam na conversão: a tabela de gbops e a coluna
    // `GB CPU` da § Moved, Removed, and Added Opcodes.
    assert_eq!(
        THIS_ITEM,
        [0xE2, 0xE0, 0xEA, 0xF2, 0xF0, 0xFA],
        "os seis layouts, na ordem em que a § Block 3 os empilha"
    );
}

#[test]
fn each_pair_differs_only_in_bit_4() {
    // Os três sub-itens anteriores eram blocos de bits com um campo variável.
    // Aqui não há campo: há três pares, e dentro de cada par o bit 4 escolhe a
    // direção. É o que substitui a máscara — e é pouco: fora do par, o bit 4 não
    // significa nada.
    for (store, load) in [
        (LDH_C_A, LDH_A_C),
        (LDH_IMM8_A, LDH_A_IMM8),
        (LD_IMM16_A, LD_A_IMM16),
    ] {
        assert_eq!(
            store ^ load,
            0b0001_0000,
            "${store:02X} e ${load:02X} diferem no bit 4 e em mais nada"
        );
        assert_eq!(store & 0b0001_0000, 0, "${store:02X} é o `$Ex`: escreve");
        assert_eq!(load & 0b0001_0000, 0b0001_0000, "${load:02X} é o `$Fx`: lê");
    }
}

// ---------------------------------------------------------------------------
// `$E2` / `$F2` — a página `$FF00` indexada por `C`: 1 byte, 2 M-cycles
// ---------------------------------------------------------------------------

#[test]
fn store_through_c_writes_a_at_ff00_plus_c() {
    let (mut cpu, mut bus) = machine(&[LDH_C_A]);
    seed_registers(&mut cpu);
    bus.write(VIA_C, UNTOUCHED);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.read(VIA_C), STORED, "`$E2` é `LD (FF00+C),A`");
    assert_eq!(
        cpu.registers.c, OFFSET_IN_C,
        "`C` é o índice, e a instrução não mexe nele"
    );
}

#[test]
fn load_through_c_reads_into_a_from_ff00_plus_c() {
    let (mut cpu, mut bus) = machine(&[LDH_A_C]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, VIA_C);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.a, AT_TARGET, "`$F2` é `LD A,(FF00+C)`");
    assert_eq!(cpu.registers.c, OFFSET_IN_C, "e `C` continua o índice");
}

#[test]
fn the_c_indexed_pair_is_one_byte_and_the_next_opcode_follows_immediately() {
    // A coluna `Bytes` de gbops diz `1`. O erro de memória deste agente era `2`
    // — tabelas antigas listam `LDH (C),A` com um operando que não existe.
    //
    // Nenhuma asserção sobre a memória, sobre `A` ou sobre os flags pega isso: o
    // efeito sai certo e os 8 T-cycles saem certos. O que muda é o `PC`, e o
    // preço é o fluxo de instruções desalinhado a partir dali — por isso o
    // programa abaixo tem uma segunda instrução, e o teste exige que ela seja a
    // que roda.
    //
    // A segunda instrução é `LD B,(HL)` (`$46`, 1.4a), escolhida por não tocar
    // em `A` nem em `C`. Se o `PC` andar dois bytes, o M3 busca o `$00` que vem
    // depois — um `NOP`, que deixa `B` como estava.
    for opcode in [LDH_C_A, LDH_A_C] {
        let (mut cpu, mut bus) = machine(&[opcode, 0x46, 0x00]);
        seed_registers(&mut cpu);
        seed_memory_around(&mut bus, VIA_C);
        cpu.registers.set_hl(ABSOLUTE);
        bus.write(ABSOLUTE, AT_TARGET);

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.pc, 0x0101,
            "${opcode:02X} tem 1 byte: o fetch anda um, e `C` já é o operando"
        );

        cpu.step(&mut bus);
        assert!(
            cpu.is_between_instructions(),
            "${opcode:02X} tem 8 T-cycles: acabou no M2"
        );

        // O `LD B,(HL)` que vem em `$0101`, em dois M-cycles.
        cpu.step(&mut bus);
        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.b, AT_TARGET,
            "${opcode:02X}: a instrução seguinte é a de `$0101`, e não a de \
             `$0102` — um byte a mais no `PC` desalinha o fluxo inteiro"
        );
    }
}

#[test]
fn the_c_indexed_store_writes_on_the_second_m_cycle_and_not_on_the_fetch() {
    // Nota 32 do `STATUS.md`: asserção **depois de cada** M-cycle, e não só no
    // fim. A coluna é `fetch → write(A->(FF00+C))`, então no M1 nada foi escrito.
    let (mut cpu, mut bus) = machine(&[LDH_C_A]);
    seed_registers(&mut cpu);
    bus.write(VIA_C, UNTOUCHED);

    cpu.step(&mut bus);
    assert_eq!(
        bus.read(VIA_C),
        UNTOUCHED,
        "M1: só o opcode chegou; o alvo está intacto"
    );
    assert!(!cpu.is_between_instructions(), "M1: falta o M2");

    cpu.step(&mut bus);
    assert_eq!(bus.read(VIA_C), STORED, "M2: `write(A->(FF00+C))`");
    assert!(cpu.is_between_instructions(), "M2: e acabou");
}

#[test]
fn the_c_indexed_load_writes_a_on_the_second_m_cycle_and_not_on_the_fetch() {
    let (mut cpu, mut bus) = machine(&[LDH_A_C]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, VIA_C);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, STORED, "M1: `A` ainda é o de antes");
    assert!(!cpu.is_between_instructions(), "M1: falta o M2");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, AT_TARGET, "M2: `read((FF00+C)->A)`");
    assert!(cpu.is_between_instructions(), "M2: e acabou");
}

// ---------------------------------------------------------------------------
// `$E0` / `$F0` — a página `$FF00` indexada por imediato: 2 bytes, 3 M-cycles
// ---------------------------------------------------------------------------

#[test]
fn store_through_an_immediate_offset_writes_a_at_ff00_plus_u8() {
    let (mut cpu, mut bus) = machine(&[LDH_IMM8_A, OFFSET_IMM8]);
    seed_registers(&mut cpu);
    bus.write(VIA_IMM8, UNTOUCHED);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.read(VIA_IMM8), STORED, "`$E0` é `LD (FF00+u8),A`");
    assert_eq!(cpu.registers.pc, 0x0102, "`$E0` tem 2 bytes");
}

#[test]
fn load_through_an_immediate_offset_reads_into_a_from_ff00_plus_u8() {
    let (mut cpu, mut bus) = machine(&[LDH_A_IMM8, OFFSET_IMM8]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, VIA_IMM8);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.a, AT_TARGET, "`$F0` é `LD A,(FF00+u8)`");
    assert_eq!(cpu.registers.pc, 0x0102, "`$F0` tem 2 bytes");
}

#[test]
fn the_immediate_offset_store_is_three_m_cycles_with_the_write_last() {
    // A coluna é `fetch → read(u8) → write(A->(FF00+u8))`: dois acessos ao
    // barramento, em M-cycles diferentes. Juntá-los no M2 e gastar o M3 num
    // `internal` dá os mesmos 12 T-cycles e o mesmo estado final, e adianta a
    // escrita em um — é o erro que a 0015 cometeu no `LD (HL),u8`, que tem
    // exatamente esta forma.
    //
    // A asserção que o pega é a do M2, e ela é a única: a do M1 passa contra as
    // duas versões, e a do M3 também.
    let (mut cpu, mut bus) = machine(&[LDH_IMM8_A, OFFSET_IMM8]);
    seed_registers(&mut cpu);
    bus.write(VIA_IMM8, UNTOUCHED);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "M1: o fetch do opcode");
    assert_eq!(bus.read(VIA_IMM8), UNTOUCHED, "M1: nada escrito");

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0102,
        "M2: `read(u8)`, e o `PC` passa por ele"
    );
    assert_eq!(
        bus.read(VIA_IMM8),
        UNTOUCHED,
        "M2: o deslocamento chegou, mas a escrita é do M3"
    );
    assert!(!cpu.is_between_instructions(), "M2: falta o M3");

    cpu.step(&mut bus);
    assert_eq!(bus.read(VIA_IMM8), STORED, "M3: `write(A->(FF00+u8))`");
    assert!(cpu.is_between_instructions(), "M3: e acabou");
}

#[test]
fn the_immediate_offset_load_is_three_m_cycles_with_a_changing_last() {
    // Mesma forma, outra direção: `fetch → read(u8) → read((FF00+u8)->A)`. O
    // erro simétrico é ler o alvo já no M2 — mesmo `A` final, mesmos 12
    // T-cycles, um M-cycle adiantado.
    let (mut cpu, mut bus) = machine(&[LDH_A_IMM8, OFFSET_IMM8]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, VIA_IMM8);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "M1: o fetch do opcode");
    assert_eq!(cpu.registers.a, STORED, "M1: `A` intacto");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0102, "M2: `read(u8)`");
    assert_eq!(
        cpu.registers.a, STORED,
        "M2: o deslocamento chegou, mas `A` só muda no M3"
    );
    assert!(!cpu.is_between_instructions(), "M2: falta o M3");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, AT_TARGET, "M3: `read((FF00+u8)->A)`");
    assert!(cpu.is_between_instructions(), "M3: e acabou");
}

#[test]
fn the_immediate_offset_reaches_the_whole_high_page_including_ie() {
    // `$FF00 + $FF` é `$FFFF`, o `IE` — o último byte do mapa, e um dos poucos
    // graváveis acima da HRAM. O deslocamento é de 8 bits e a base é `$FF00`, o
    // que faz a soma nunca estourar: a página tem exatamente 256 endereços e o
    // operando tem exatamente 256 valores.
    let (mut cpu, mut bus) = machine(&[LDH_IMM8_A, 0xFF]);
    seed_registers(&mut cpu);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(0xFFFF),
        STORED,
        "`LD (FF00+$FF),A` escreve no `IE`, e não dá a volta para `$00FF`"
    );
}

// ---------------------------------------------------------------------------
// `$EA` / `$FA` — endereço absoluto: 3 bytes, 4 M-cycles
// ---------------------------------------------------------------------------

#[test]
fn store_to_an_absolute_address_writes_a_there() {
    let (mut cpu, mut bus) = machine(&[LD_IMM16_A, 0xD7, 0xC3]);
    seed_registers(&mut cpu);
    bus.write(ABSOLUTE, UNTOUCHED);
    bus.write(ABSOLUTE_SWAPPED, UNTOUCHED);

    for _ in 0..4 {
        cpu.step(&mut bus);
    }

    assert_eq!(bus.read(ABSOLUTE), STORED, "`$EA` é `LD (u16),A`");
    assert_eq!(
        bus.read(ABSOLUTE_SWAPPED),
        UNTOUCHED,
        "o operando é little-endian: `$D7 $C3` é `$C3D7`, e não `$D7C3` — que \
         também é WRAM e aceitaria a escrita sem reclamar"
    );
    assert_eq!(cpu.registers.pc, 0x0103, "`$EA` tem 3 bytes");
}

#[test]
fn load_from_an_absolute_address_reads_into_a() {
    let (mut cpu, mut bus) = machine(&[LD_A_IMM16, 0xD7, 0xC3]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, ABSOLUTE);
    bus.write(ABSOLUTE_SWAPPED, AT_NEIGHBOUR);

    for _ in 0..4 {
        cpu.step(&mut bus);
    }

    assert_eq!(cpu.registers.a, AT_TARGET, "`$FA` é `LD A,(u16)`");
    assert_eq!(cpu.registers.pc, 0x0103, "`$FA` tem 3 bytes");
}

#[test]
fn the_absolute_store_is_four_m_cycles_with_the_write_last() {
    // A forma mais longa do sub-item, e a que a armadilha (b) do `STATUS.md`
    // aponta: é o primeiro operando de dois bytes desde o `JP u16`, e lá o M4 é
    // um `internal`. Aqui o M4 é o **acesso**. O erro é escrever junto com o
    // byte alto, no M3 — mesmos 16 T-cycles, mesmo estado final.
    //
    // Três iterações seguidas erraram em qual M-cycle o efeito cai (notas 26, 30
    // e 32), e nas três o que reprovou foi a asserção do meio. Aqui há duas.
    let (mut cpu, mut bus) = machine(&[LD_IMM16_A, 0xD7, 0xC3]);
    seed_registers(&mut cpu);
    bus.write(ABSOLUTE, UNTOUCHED);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "M1: o fetch do opcode");
    assert_eq!(bus.read(ABSOLUTE), UNTOUCHED, "M1: nada escrito");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0102, "M2: `read(u16:lower)`");
    assert_eq!(
        bus.read(ABSOLUTE),
        UNTOUCHED,
        "M2: metade do endereço, e só"
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0103, "M3: `read(u16:upper)`");
    assert_eq!(
        bus.read(ABSOLUTE),
        UNTOUCHED,
        "M3: o endereço está inteiro dentro da CPU e a escrita ainda não \
         aconteceu — ela é o M4"
    );
    assert!(!cpu.is_between_instructions(), "M3: falta o M4");

    cpu.step(&mut bus);
    assert_eq!(bus.read(ABSOLUTE), STORED, "M4: `write(A->(u16))`");
    assert!(cpu.is_between_instructions(), "M4: e acabou");
}

#[test]
fn the_absolute_load_is_four_m_cycles_with_a_changing_last() {
    let (mut cpu, mut bus) = machine(&[LD_A_IMM16, 0xD7, 0xC3]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, ABSOLUTE);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "M1: o fetch do opcode");
    assert_eq!(cpu.registers.a, STORED, "M1: `A` intacto");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0102, "M2: `read(u16:lower)`");
    assert_eq!(cpu.registers.a, STORED, "M2: `A` intacto");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0103, "M3: `read(u16:upper)`");
    assert_eq!(
        cpu.registers.a, STORED,
        "M3: o endereço está inteiro e `A` ainda não mudou — a leitura é o M4"
    );
    assert!(!cpu.is_between_instructions(), "M3: falta o M4");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, AT_TARGET, "M4: `read((u16)->A)`");
    assert!(cpu.is_between_instructions(), "M4: e acabou");
}

// ---------------------------------------------------------------------------
// O que os seis **não** fazem
// ---------------------------------------------------------------------------

#[test]
fn none_of_the_six_touches_a_register_the_column_does_not_name() {
    // Um `assert_eq!` na struct inteira pega o que uma asserção por registrador
    // não pega: o decodificador que acerta o endereço e mexe em mais alguém de
    // lambuja. `A` é o único registrador que qualquer uma das seis linhas
    // escreve, e só nas três de leitura.
    for (opcode, operand, target, loads) in [
        (LDH_C_A, [0x00, 0x00], VIA_C, false),
        (LDH_A_C, [0x00, 0x00], VIA_C, true),
        (LDH_IMM8_A, [OFFSET_IMM8, 0x00], VIA_IMM8, false),
        (LDH_A_IMM8, [OFFSET_IMM8, 0x00], VIA_IMM8, true),
        (LD_IMM16_A, [0xD7, 0xC3], ABSOLUTE, false),
        (LD_A_IMM16, [0xD7, 0xC3], ABSOLUTE, true),
    ] {
        let (mut cpu, mut bus) = machine(&[opcode, operand[0], operand[1]]);
        seed_registers(&mut cpu);
        seed_memory_around(&mut bus, target);

        let mut expected = cpu.registers;
        expected.pc = 0x0100 + u16::from(bytes_of(opcode));
        if loads {
            expected.a = AT_TARGET;
        }

        // Exatamente os M-cycles da instrução — ver [`m_cycles_of`]. Um passo a
        // mais executa o que vem depois dela e o `PC` deixa de ser o dela.
        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers, expected,
            "${opcode:02X} mexe só no que a coluna diz"
        );
    }
}

/// O comprimento em bytes de cada um dos seis, da coluna `Bytes` de gbops.
const fn bytes_of(opcode: u8) -> u8 {
    match opcode {
        LDH_C_A | LDH_A_C => 1,
        LDH_IMM8_A | LDH_A_IMM8 => 2,
        _ => 3,
    }
}

/// A contagem de M-cycles de cada um dos seis, dos `T-cycles` de gbops
/// (8/12/16, quatro T por M).
///
/// Ela existe porque os laços abaixo não podem avançar um número fixo de
/// passos: 4 passos executam `$EA`/`$FA` inteiros, mas passam **dois** M-cycles
/// do fim de `$E2`/`$F2` — e o excesso busca os `$00` seguintes como `NOP`,
/// andando o `PC` além do que a instrução medida anda. Foi o que derrubou
/// `none_of_the_six_touches_a_register_the_column_does_not_name` contra uma
/// implementação **correta** na sessão que escreveu esta suíte (ver o doc da
/// 0017): o vermelho era do arnês, não do emulador.
const fn m_cycles_of(opcode: u8) -> u8 {
    match opcode {
        LDH_C_A | LDH_A_C => 2,
        LDH_IMM8_A | LDH_A_IMM8 => 3,
        _ => 4,
    }
}

#[test]
fn none_of_the_six_touches_the_flags() {
    // As quatro colunas de flag são `-` nas seis linhas. O `F` vai com o nibble
    // baixo sujo pelo motivo do 1.4a: o 1.1 decidiu **não** mascarar os bits
    // 3–0, e um `LD` que os limpasse de passagem seria a máscara entrando pela
    // porta dos fundos.
    const DIRTY_F: u8 = 0b1111_1010;

    for opcode in THIS_ITEM {
        let (mut cpu, mut bus) = machine(&[opcode, OFFSET_IMM8, 0xC3]);
        seed_registers(&mut cpu);
        cpu.registers.f = DIRTY_F;

        // Exatamente os M-cycles da instrução — ver [`m_cycles_of`]. Os bytes
        // que sobram no programa não são neutros por acaso: `$85` é `LD A,L` e
        // `$C3` é `JP u16`. Nenhum dos dois toca em flag, então o veredito não
        // muda — mas o que se quer medir é a instrução, não os vizinhos.
        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.f, DIRTY_F,
            "${opcode:02X} tem `-` nas quatro colunas de flag da tabela"
        );
    }
}

// ---------------------------------------------------------------------------
// Controle negativo: exatamente estes seis, e nenhum vizinho
// ---------------------------------------------------------------------------

/// Opcodes que **outros** itens do ROADMAP já decodificam, e que portanto não
/// são "falta implementar" para o controle negativo abaixo.
///
/// A lista é duplicada em cada arquivo de sub-item de propósito — ver a
/// invariante de `decoded_elsewhere` no `STATUS.md`. Um ponto de verdade
/// compartilhado faria a atualização acontecer sozinha, e o controle negativo
/// perderia a única propriedade que o justifica: obrigar quem acrescenta opcode
/// a vir declarar o que acrescentou.
///
/// - `$00` (`NOP`) e `$C3` (`JP u16`) — 1.3.
/// - `01 ddd sss` sem o `$76` (`$40`–`$7F`) — 1.4a.
/// - `00 ddd 110` (`$06 $0E $16 $1E $26 $2E $36 $3E`) — 1.4b.
/// - `00 mm x010` (`$02 $0A $12 $1A $22 $2A $32 $3A`) — 1.4c.
fn decoded_elsewhere(opcode: u8) -> bool {
    opcode == 0x00
        || opcode == 0xC3
        || ((0x40..=0x7F).contains(&opcode) && opcode != 0x76)
        || opcode & 0b1100_0111 == 0b0000_0110
        || opcode & 0b1100_0111 == 0b0000_0010
}

#[test]
fn the_opcodes_this_item_decodes_are_exactly_the_six_of_block_3() {
    // Nota 25 do `STATUS.md`: teste que afirma *pertinência* ("estes seis são
    // loads") não pega *excesso* ("e mais um"). E aqui o excesso é mais provável
    // que nos três sub-itens anteriores, porque **não há máscara certa** para
    // errar frouxo: qualquer tentativa de reconhecer os seis por bits engole
    // vizinhos. `0b1110_0101 == 0b1110_0000`, por exemplo, é a máscara natural —
    // e leva junto `$E8` (`ADD SP,i8`) e `$F8` (`LD HL,SP+i8`), que são o 1.7.
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00, 0x00]);
        seed_registers(&mut cpu);

        // Quatro M-cycles: o bastante para a forma mais longa já decodificada
        // acabar — que agora é de quatro, e não mais de três.
        for _ in 0..4 {
            cpu.step(&mut bus);
        }

        if THIS_ITEM.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos seis do 1.4d e este item o decodifica"
            );
        } else if ILLEGAL.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::IllegalOpcode(opcode)),
                "${opcode:02X} continua sendo um dos onze que não existem"
            );
        } else if !decoded_elsewhere(opcode) {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::UndecodedOpcode(opcode)),
                "${opcode:02X} é opcode legítimo fora deste sub-item: o rótulo \
                 tem de continuar dizendo `falta implementar`"
            );
        }
    }
}
