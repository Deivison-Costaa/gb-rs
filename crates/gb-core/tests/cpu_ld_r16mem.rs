//! ROADMAP 1.4c — o indireto por par de registradores: `LD (r16mem),A` e
//! `LD A,(r16mem)`.
//!
//! Spec: `docs/reference/03-opcodes.md`, linhas `02 0A 12 1A 22 2A 32 3A`
//! (gbops `90b9bf296aed`), `docs/reference/02-cpu.md` § Block 0 (layouts de
//! `ld [r16mem], a` e `ld a, [r16mem]`) e § CPU Instruction Set (tabela de
//! placeholders), e `docs/reference/06-ppu.md` § OAM Corruption Bug (Pan Docs
//! `fe246067b695`).
//!
//! # Duas regras, oito opcodes, uma forma de M-cycle
//!
//! A § Block 0 dá as duas codificações, e elas diferem num bit só — o 3:
//!
//! ```text
//! Bits | Campo                 Bits | Campo
//!    7 | 0                        7 | 0
//!    6 | 0                        6 | 0
//!  5-4 | Dest (r16mem)          5-4 | Source (r16mem)
//!    3 | 0                        3 | 1
//!    2 | 0                        2 | 0
//!    1 | 1                        1 | 1
//!    0 | 0                        0 | 0
//! ```
//!
//! O outro operando não aparece no opcode porque não varia: é sempre `A`.
//!
//! O placeholder é `r16mem`, e **não** `r16` nem `r16stk`: os quatro valores são
//! `bc de hl+ hl-`. A tabela que os define é a que a nota 24 do `STATUS.md`
//! registra como corrompida na conversão — ela emenda `r8`, `r16`, `r16stk`,
//! `r16mem` e `cond` numa lista só, com os índices 0–3 repetidos quatro vezes e
//! sem cabeçalho que diga a qual grupo cada bloco pertence. O terceiro bloco é o
//! `r16mem`, e quem confirma a leitura é a tabela de gbops, que enumera os oito
//! opcodes um a um: `$22` é `LD (HL+),A` e `$32` é `LD (HL-),A`.
//!
//! As oito linhas de gbops têm a **mesma** forma, e é a primeira vez que isso
//! acontece num sub-item do 1.4:
//!
//! | Forma | Bytes | T-cycles | M-cycles (passo a passo) |
//! |---|---|---|---|
//! | `LD (rr),A` | 1 | 8 | `fetch → write(A->(rr))` |
//! | `LD A,(rr)` | 1 | 8 | `fetch → read((rr)->A)` |
//!
//! # O conceito novo é o efeito colateral, e ele tem endereço e M-cycle
//!
//! A coluna escreve o incremento **dentro** do passo do acesso —
//! `write(A->(HL++))`, e não `write(A->(HL))` seguido de um passo `internal`.
//! Então `HL` muda no M2, junto com o acesso, e não no M1: depois do fetch o par
//! ainda vale o que valia. São 8 T-cycles; não há terceiro M-cycle onde pôr o
//! incremento nem onde adiantá-lo.
//!
//! Que o endereço é o valor **anterior** ao incremento é o `++` postfixo da
//! coluna, e não é só isso: a § OAM Corruption Bug diz, de `ld a, [hli]` e
//! companhia, que o bug dispara conforme o conteúdo de 16 bits *"(before the
//! operation)"* esteja na faixa `$FE00`–`$FEFF`. A faixa é do valor de antes.
//!
//! Essa seção também descreve o incremento como evento de **barramento** — a
//! IDU põe o valor nas linhas de endereço mesmo sem leitura nem escrita
//! assertada, e é por isso que essas quatro instruções corrompem a OAM *duas*
//! vezes. Nada disso é implementado aqui (é o 7.2), mas desmente a leitura
//! natural de que o `HL++` seja aritmética de registrador sem contrapartida no
//! barramento.
//!
//! `unwrap`/`expect` são permitidos aqui: a R6 proíbe fora de teste.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

/// `$0100` — o endereço em que a boot ROM entrega o controle ao cartucho.
const ENTRY: usize = 0x0100;

/// Três endereços de WRAM, um por par, para que escrever pelo par errado nunca
/// caia no endereço certo. `BC`, `DE` e `HL` no hand-off apontam para ROM, que
/// engole escrita.
///
/// Os bytes baixos são todos diferentes de zero pelo motivo da nota 27 do
/// `STATUS.md`: com `$C000` o byte baixo é `$00` e a WRAM começa zerada, e um
/// teste que compare contra `$00` passa sem que nada tenha sido escrito. Os
/// vizinhos (`±1`) também são WRAM, que é o que os testes de `HL+`/`HL-`
/// precisam.
const VIA_BC: u16 = 0xC0A7;
/// Ver [`VIA_BC`].
const VIA_DE: u16 = 0xC1B3;
/// Ver [`VIA_BC`].
const VIA_HL: u16 = 0xC2C9;

/// O byte em `A`, que as quatro formas de escrita levam para a memória.
const STORED: u8 = 0x5A;

/// O byte que fica no endereço apontado antes de uma escrita, e que tem de
/// continuar lá durante o M1.
///
/// Não é `$00` de propósito: contra um zero, "a escrita ainda não aconteceu" e
/// "a escrita aconteceu e gravou zero" são a mesma leitura.
const UNTOUCHED: u8 = 0x6D;

/// O byte no endereço apontado, que as quatro formas de leitura levam para `A`.
const AT_TARGET: u8 = 0xE7;

/// O byte no endereço **seguinte**. Uma implementação que incremente `HL` antes
/// do acesso lê este em vez de [`AT_TARGET`].
const AT_NEXT: u8 = 0x3C;

/// O byte no endereço **anterior**. Uma implementação que decremente `HL` antes
/// do acesso lê este em vez de [`AT_TARGET`].
const AT_PREVIOUS: u8 = 0x91;

/// Os quatro valores de `r16mem`, na ordem dos índices 0 a 3.
const R16MEM: [R16Mem; 4] = [
    R16Mem::Bc,
    R16Mem::De,
    R16Mem::HlIncrement,
    R16Mem::HlDecrement,
];

/// Um operando `r16mem`: o par que dá o endereço, e o que ele faz com o par
/// depois de usá-lo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum R16Mem {
    /// Índice 0 — `bc`. Nenhum efeito colateral.
    Bc,
    /// Índice 1 — `de`. Nenhum efeito colateral.
    De,
    /// Índice 2 — `hl+`.
    HlIncrement,
    /// Índice 3 — `hl-`.
    HlDecrement,
}

impl R16Mem {
    /// O nome que a tabela de placeholders lhe dá.
    const fn name(self) -> &'static str {
        match self {
            Self::Bc => "BC",
            Self::De => "DE",
            Self::HlIncrement => "HL+",
            Self::HlDecrement => "HL-",
        }
    }

    /// O endereço que o par aponta no estado montado por [`seed_registers`].
    const fn address(self) -> u16 {
        match self {
            Self::Bc => VIA_BC,
            Self::De => VIA_DE,
            Self::HlIncrement | Self::HlDecrement => VIA_HL,
        }
    }

    /// O que sobra em `HL` depois da instrução. Só os índices 2 e 3 o movem.
    const fn hl_afterwards(self) -> u16 {
        match self {
            Self::Bc | Self::De => VIA_HL,
            Self::HlIncrement => VIA_HL.wrapping_add(1),
            Self::HlDecrement => VIA_HL.wrapping_sub(1),
        }
    }
}

/// O opcode de `LD (r16mem),A`, montado pelo layout da § Block 0:
/// `0b00_mm_0010`.
const fn store_through(pair: u8) -> u8 {
    0b0000_0010 | (pair << 4)
}

/// O opcode de `LD A,(r16mem)`, montado pelo layout da § Block 0:
/// `0b00_mm_1010`. É [`store_through`] com o bit 3 ligado.
const fn load_through(pair: u8) -> u8 {
    0b0000_1010 | (pair << 4)
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

/// Põe os três pares apontando para os três endereços de rascunho, e uma
/// sentinela em `A`.
fn seed_registers(cpu: &mut Cpu) {
    cpu.registers.a = STORED;
    cpu.registers.set_bc(VIA_BC);
    cpu.registers.set_de(VIA_DE);
    cpu.registers.set_hl(VIA_HL);
}

/// Deixa [`AT_TARGET`] no endereço apontado e vizinhos distintos dos dois lados,
/// para que ler do endereço errado nunca devolva o byte certo.
fn seed_memory_around(bus: &mut Bus, address: u16) {
    bus.write(address.wrapping_sub(1), AT_PREVIOUS);
    bus.write(address, AT_TARGET);
    bus.write(address.wrapping_add(1), AT_NEXT);
}

// ---------------------------------------------------------------------------
// A codificação: oito opcodes, e são estes oito
// ---------------------------------------------------------------------------

#[test]
fn the_two_layouts_of_block_0_give_the_eight_opcodes_gbops_lists() {
    // As duas fórmulas contra as oito linhas de `03-opcodes.md`, uma a uma.
    // Isto é o que amarra a leitura da tabela de placeholders corrompida
    // (nota 24 do `STATUS.md`) a uma fonte que não se perdeu na conversão: se
    // `r16mem` fosse `bc de hl sp`, ou se os índices 2 e 3 estivessem
    // trocados, a lista abaixo não fecharia.
    assert_eq!(
        [
            store_through(0),
            store_through(1),
            store_through(2),
            store_through(3)
        ],
        [0x02, 0x12, 0x22, 0x32],
        "`LD (BC),A` `LD (DE),A` `LD (HL+),A` `LD (HL-),A`"
    );
    assert_eq!(
        [
            load_through(0),
            load_through(1),
            load_through(2),
            load_through(3)
        ],
        [0x0A, 0x1A, 0x2A, 0x3A],
        "`LD A,(BC)` `LD A,(DE)` `LD A,(HL+)` `LD A,(HL-)`"
    );
}

// ---------------------------------------------------------------------------
// `LD (r16mem),A` — 4 opcodes, 2 M-cycles
// ---------------------------------------------------------------------------

#[test]
fn every_store_through_a_pair_writes_a_at_the_address_the_pair_holds() {
    for (index, pair) in R16MEM.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let opcode = store_through(index as u8);

        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        bus.write(pair.address(), UNTOUCHED);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            bus.read(pair.address()),
            STORED,
            "${opcode:02X} é `LD ({}),A`",
            pair.name()
        );
        assert_eq!(
            cpu.registers.a, STORED,
            "${opcode:02X} lê `A`, não escreve nele"
        );
        assert_eq!(cpu.registers.pc, 0x0101, "${opcode:02X} tem 1 byte");
    }
}

#[test]
fn a_store_through_a_pair_is_two_m_cycles_and_the_write_is_the_second() {
    // `LD (BC),A` (`$02`): `fetch → write(A->(BC))`. A escrita é o M2, e o M1 é
    // o fetch do opcode e nada mais.
    let (mut cpu, mut bus) = machine(&[0x02]);
    seed_registers(&mut cpu);
    bus.write(VIA_BC, UNTOUCHED);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "o M1 é o fetch do opcode");
    assert_eq!(
        bus.read(VIA_BC),
        UNTOUCHED,
        "no M1 só o opcode chegou; nada foi escrito"
    );
    assert!(
        !cpu.is_between_instructions(),
        "`LD (BC),A` tem 8 T-cycles: uma implementação que já terminou no fetch \
         fez o acesso um M-cycle adiantado"
    );

    cpu.step(&mut bus);
    assert_eq!(bus.read(VIA_BC), STORED, "o M2 é o `write(A->(BC))`");
    assert!(
        cpu.is_between_instructions(),
        "dois M-cycles e acabou: não há terceiro"
    );
}

// ---------------------------------------------------------------------------
// `LD A,(r16mem)` — 4 opcodes, 2 M-cycles
// ---------------------------------------------------------------------------

#[test]
fn every_load_through_a_pair_reads_into_a_from_the_address_the_pair_holds() {
    for (index, pair) in R16MEM.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let opcode = load_through(index as u8);

        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        seed_memory_around(&mut bus, pair.address());

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.a,
            AT_TARGET,
            "${opcode:02X} é `LD A,({})`",
            pair.name()
        );
        assert_eq!(cpu.registers.pc, 0x0101, "${opcode:02X} tem 1 byte");
    }
}

#[test]
fn a_load_through_a_pair_is_two_m_cycles_and_a_changes_on_the_second() {
    // `LD A,(BC)` (`$0A`): `fetch → read((BC)->A)`. A leitura no barramento e a
    // escrita em `A` são o **mesmo** M2 — a mesma forma do `LD r,(HL)` do 1.4a,
    // e pelo mesmo motivo: os 8 T-cycles não deixam onde pôr um terceiro
    // M-cycle.
    let (mut cpu, mut bus) = machine(&[0x0A]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, VIA_BC);

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.a, STORED,
        "no M1 só o opcode chegou; `A` ainda é o de antes"
    );
    assert_eq!(cpu.registers.pc, 0x0101, "o M1 é o fetch do opcode");
    assert!(
        !cpu.is_between_instructions(),
        "`LD A,(BC)` tem 8 T-cycles: dois M-cycles"
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, AT_TARGET, "o M2 é o `read((BC)->A)`");
    assert!(
        cpu.is_between_instructions(),
        "dois M-cycles e acabou: não há terceiro"
    );
}

// ---------------------------------------------------------------------------
// O conceito novo: o efeito colateral em `HL`
// ---------------------------------------------------------------------------

#[test]
fn the_hl_variants_access_the_address_from_before_the_side_effect() {
    // O `++`/`--` da coluna é postfixo, e a § OAM Corruption Bug confirma pelo
    // outro lado: o bug dispara conforme o conteúdo do par *"(before the
    // operation)"*. Então o acesso é em `HL`, não em `HL±1`.
    //
    // Os vizinhos dos dois lados do alvo têm bytes distintos ([`AT_NEXT`] e
    // [`AT_PREVIOUS`]), então uma implementação que incremente ou decremente
    // antes do acesso lê o byte errado em vez de ler o certo por sorte.
    for (index, pair, expected_neighbour) in [
        (2u8, R16Mem::HlIncrement, AT_NEXT),
        (3u8, R16Mem::HlDecrement, AT_PREVIOUS),
    ] {
        let opcode = load_through(index);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        seed_memory_around(&mut bus, VIA_HL);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.a,
            AT_TARGET,
            "${opcode:02X} é `LD A,({})`: o endereço é o `HL` de antes, e não \
             o vizinho ({expected_neighbour:#04X})",
            pair.name()
        );
        assert_eq!(
            cpu.registers.hl(),
            pair.hl_afterwards(),
            "${opcode:02X} deixa `HL` movido em um"
        );
    }
}

#[test]
fn the_hl_side_effect_lands_on_the_second_m_cycle_and_not_on_the_fetch() {
    // Este é o teste que a nota 30 do `STATUS.md` pede: instrução com dois
    // acessos ao barramento (o fetch e o dado) precisa de asserção **entre**
    // eles, e não só no fim.
    //
    // A coluna escreve o incremento dentro do passo do acesso —
    // `write(A->(HL++))` —, então depois do M1 o par ainda vale o que valia. O
    // erro que isto pega é resolver o endereço e aplicar o `HL++` no
    // decodificador, guardando o endereço para o M2: mesmo estado final,
    // mesmos 8 T-cycles, e o `HL` muda um M-cycle adiantado. Duas iterações
    // seguidas erraram *em qual* M-cycle o efeito cai (notas 26 e 30), uma em
    // cada direção.
    for (opcode, expected) in [
        (0x22u8, VIA_HL.wrapping_add(1)),
        (0x32u8, VIA_HL.wrapping_sub(1)),
    ] {
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        bus.write(VIA_HL, UNTOUCHED);

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.hl(),
            VIA_HL,
            "${opcode:02X}: no M1 o `HL` ainda não se moveu — o `HL±` é do M2, \
             junto com o acesso"
        );
        assert_eq!(
            bus.read(VIA_HL),
            UNTOUCHED,
            "${opcode:02X}: e nada foi escrito ainda"
        );

        cpu.step(&mut bus);
        assert_eq!(
            bus.read(VIA_HL),
            STORED,
            "${opcode:02X}: o M2 escreve no endereço de antes do efeito"
        );
        assert_eq!(
            cpu.registers.hl(),
            expected,
            "${opcode:02X}: e é o mesmo M2 que move o `HL`"
        );
    }
}

#[test]
fn only_the_hl_variants_move_their_pair() {
    // `BC` e `DE` não têm forma com efeito colateral: os índices 0 e 1 de
    // `r16mem` são `bc` e `de` pelados. Um `assert_eq!` na struct inteira pega
    // o que uma asserção por par não pega — o decodificador que acerta o
    // endereço e mexe em mais alguém de lambuja.
    for (index, pair) in R16MEM.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let index = index as u8;

        // A escrita só mexe na memória; a leitura também carrega `A`. Nos dois
        // casos, `HL` se move se — e só se — o par for `hl+` ou `hl-`.
        for (opcode, loaded) in [
            (store_through(index), None),
            (load_through(index), Some(AT_TARGET)),
        ] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed_registers(&mut cpu);
            seed_memory_around(&mut bus, pair.address());

            let mut expected = cpu.registers;
            expected.pc = 0x0101;
            expected.set_hl(pair.hl_afterwards());
            if let Some(value) = loaded {
                expected.a = value;
            }

            cpu.step(&mut bus);
            cpu.step(&mut bus);

            assert_eq!(
                cpu.registers,
                expected,
                "${opcode:02X} usa ({}) e mexe só no que a coluna diz",
                pair.name()
            );
        }
    }
}

#[test]
fn the_side_effect_wraps_and_does_not_saturate() {
    // `HL` é de 16 bits e dá a volta, como o `PC` da nota 25 do `STATUS.md`.
    // Aqui há duas voltas a cobrir, uma por sentido, e a de `HL+` é observável
    // pelo endereço: `$FFFF` é o `IE`, o único byte gravável ali.
    let (mut cpu, mut bus) = machine(&[0x22]);
    seed_registers(&mut cpu);
    cpu.registers.set_hl(0xFFFF);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(0xFFFF),
        STORED,
        "`LD (HL+),A` com `HL = $FFFF` escreve em `$FFFF`"
    );
    assert_eq!(
        cpu.registers.hl(),
        0x0000,
        "e `HL` dá a volta: depois de `$FFFF` vem `$0000`, não `$FFFF` outra vez"
    );

    let (mut cpu, mut bus) = machine(&[0x32]);
    seed_registers(&mut cpu);
    cpu.registers.set_hl(0x0000);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.hl(),
        0xFFFF,
        "e no outro sentido: antes de `$0000` vem `$FFFF`, não `$0000` outra vez"
    );
}

// ---------------------------------------------------------------------------
// Flags: as quatro colunas são `-` nas oito linhas
// ---------------------------------------------------------------------------

#[test]
fn no_load_or_store_through_a_pair_touches_the_flags() {
    // Um `F` com os quatro flags ligados e o nibble baixo sujo, pelo motivo do
    // 1.4a: o 1.1 decidiu **não** mascarar os bits 3–0, e um `LD` que os
    // limpasse de passagem seria a máscara entrando pela porta dos fundos.
    const DIRTY_F: u8 = 0b1111_1010;

    for index in 0..4u8 {
        for opcode in [store_through(index), load_through(index)] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed_registers(&mut cpu);
            cpu.registers.f = DIRTY_F;

            cpu.step(&mut bus);
            cpu.step(&mut bus);

            assert_eq!(
                cpu.registers.f, DIRTY_F,
                "${opcode:02X} tem `-` nas quatro colunas de flag da tabela"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Controle negativo: exatamente os oito de `00 mm 0010` e `00 mm 1010`
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
fn decoded_elsewhere(opcode: u8) -> bool {
    opcode == 0x00
        || opcode == 0xC3
        || ((0x40..=0x7F).contains(&opcode) && opcode != 0x76)
        || opcode & 0b1100_0111 == 0b0000_0110
}

#[test]
fn the_block_this_item_decodes_is_exactly_the_eight_opcodes_of_00_mm_x010() {
    // Nota 25 do `STATUS.md`: teste que afirma *pertinência* ("estes oito são
    // loads") não pega *excesso* ("e mais um"). Os oito andam de 8 em 8 e o
    // reconhecimento é por máscara, então os vizinhos que uma máscara frouxa
    // engoliria estão a um bit de distância: `INC r16`/`DEC r16` (`00 mm 0011`
    // e `00 mm 1011`, o 1.7), `INC r8`/`DEC r8` (`00 ddd 100`/`101`, o 1.6) e
    // `LD r16,u16` (`00 mm 0001`, o 1.5). **Nenhum teste de comportamento
    // acima os menciona** — quem os protege é esta varredura dos 256.
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00]);
        seed_registers(&mut cpu);

        // Três M-cycles: o bastante para a forma mais longa já decodificada
        // acabar, e de menos para `JP u16`, que só desvia no M4.
        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let in_block = opcode & 0b1100_0111 == 0b0000_0010;

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos oito `r16mem` e o 1.4c o decodifica"
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
