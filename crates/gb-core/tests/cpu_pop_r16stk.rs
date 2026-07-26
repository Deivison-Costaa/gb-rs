//! ROADMAP 1.5c — `POP r16stk`: o bloco `11 rr 0001`.
//!
//! A coluna é `fetch → read((SP++)->C) → read((SP++)->B)`. Duas coisas moram
//! nela: o `++` é **pós**-incremento (lê em `SP`, depois anda), espelho de papel
//! e não de notação do `(--SP)` do 1.5b; e a seta põe **meia metade do par por
//! M-cycle**, como o 1.5a. Ver `STATUS.md`, notas 32, 34 e 36.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup, Registers};

const ENTRY: usize = 0x0100;

const STACK_BOTTOM: u16 = 0xCF00;
const LOW_SLOT: u16 = STACK_BOTTOM;
const HIGH_SLOT: u16 = STACK_BOTTOM.wrapping_add(1);
const STACK_AFTER: u16 = STACK_BOTTOM.wrapping_add(2);

const POPPED: u16 = 0x9C57;
const POPPED_BYTES: [u8; 2] = POPPED.to_be_bytes();

// Par pré-carregado com as duas metades diferentes das de POPPED: metade escrita
// por acidente e metade não escrita ficam ambas visíveis.
const PREVIOUS: u16 = 0x1234;
const AFTER_LOW_HALF: u16 = (PREVIOUS & 0xFF00) | (POPPED & 0x00FF);

const R16STK: [Pair; 4] = [Pair::Bc, Pair::De, Pair::Hl, Pair::Af];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pair {
    Bc,
    De,
    Hl,
    Af,
}

impl Pair {
    const fn name(self) -> &'static str {
        match self {
            Self::Bc => "BC",
            Self::De => "DE",
            Self::Hl => "HL",
            Self::Af => "AF",
        }
    }

    const fn set(self, registers: &mut Registers, value: u16) {
        match self {
            Self::Bc => registers.set_bc(value),
            Self::De => registers.set_de(value),
            Self::Hl => registers.set_hl(value),
            Self::Af => registers.set_af(value),
        }
    }

    const fn get(self, cpu: &Cpu) -> u16 {
        match self {
            Self::Bc => cpu.registers.bc(),
            Self::De => cpu.registers.de(),
            Self::Hl => cpu.registers.hl(),
            Self::Af => cpu.registers.af(),
        }
    }
}

const fn pop_pair(rr: u8) -> u8 {
    0b1100_0001 | (rr << 4)
}

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);

    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

fn machine_popping(pair: Pair, rr: u8) -> (Cpu, Bus) {
    let (mut cpu, mut bus) = machine(&[pop_pair(rr)]);

    pair.set(&mut cpu.registers, PREVIOUS);
    cpu.registers.sp = STACK_BOTTOM;
    bus.write(LOW_SLOT, POPPED_BYTES[1]);
    bus.write(HIGH_SLOT, POPPED_BYTES[0]);

    (cpu, bus)
}

fn indexed_pairs() -> impl Iterator<Item = (u8, Pair)> {
    #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
    R16STK
        .iter()
        .enumerate()
        .map(|(index, pair)| (index as u8, *pair))
}

#[test]
fn the_block_3_layout_gives_the_four_opcodes_gbops_lists() {
    assert_eq!(
        [pop_pair(0), pop_pair(1), pop_pair(2), pop_pair(3)],
        [0xC1, 0xD1, 0xE1, 0xF1],
        "`POP BC` `POP DE` `POP HL` `POP AF`. O bloco vizinho, `11 rr 0101`, é o \
         `PUSH` do 1.5b — os dois moram sob o mesmo cabeçalho do `02-cpu.md` \
         (nota 38) e o que os separa é o bit 2"
    );
}

#[test]
fn every_pair_is_popped_low_byte_first_from_the_lower_address() {
    for (rr, pair) in indexed_pairs() {
        let opcode = pop_pair(rr);
        let (mut cpu, mut bus) = machine_popping(pair, rr);

        for _ in 0..3 {
            cpu.step(&mut bus);
        }

        assert_eq!(
            pair.get(&cpu),
            POPPED,
            "${opcode:02X} é `POP {}`: a metade **baixa** vem do endereço mais \
             baixo e é a primeira lida. Invertidas as duas, o par fica {:#06X} — \
             e o `PUSH` do 1.5b, que escreve a alta primeiro, deixaria de fechar",
            pair.name(),
            POPPED.swap_bytes()
        );
        assert_eq!(
            cpu.registers.sp, STACK_AFTER,
            "${opcode:02X}: dois incrementos, um por leitura"
        );
        assert_eq!(cpu.registers.pc, 0x0101, "${opcode:02X} tem 1 byte");
    }
}

#[test]
fn the_stack_pointer_moves_between_the_two_reads() {
    for (rr, pair) in indexed_pairs() {
        let opcode = pop_pair(rr);
        let name = pair.name();
        let (mut cpu, mut bus) = machine_popping(pair, rr);

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.sp, STACK_BOTTOM,
            "${opcode:02X}: o M1 é o fetch e não mexe no `SP`"
        );
        assert_eq!(
            pair.get(&cpu),
            PREVIOUS,
            "${opcode:02X}: e não escreve em `{name}`"
        );
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X} tem 12 T-cycles: três M-cycles"
        );

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.sp,
            STACK_AFTER.wrapping_sub(1),
            "${opcode:02X}: `read((SP++)->{name}:lower)` é **pós**-incremento — o \
             byte vem de {LOW_SLOT:#06X} e o `SP` anda depois. Incrementar antes \
             (o espelho literal do `(--SP)` do `PUSH`) leria {HIGH_SLOT:#06X} aqui \
             e passaria do topo no M3"
        );
        assert_eq!(
            pair.get(&cpu),
            AFTER_LOW_HALF,
            "${opcode:02X}: a seta da coluna diz `->{name}:lower`, então a metade \
             baixa **já está** no registrador ao fim do M2. Latchar os dois bytes \
             e escrever o par no fim do M3 dá o mesmo estado final e os mesmos 12 \
             T-cycles — é o erro #1 da 0018, e este é o único assert que o vê"
        );
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X}: ainda falta o M3"
        );

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.sp, STACK_AFTER,
            "${opcode:02X}: o M3 lê em {HIGH_SLOT:#06X} e anda de novo"
        );
        assert_eq!(
            pair.get(&cpu),
            POPPED,
            "${opcode:02X}: o M3 é `read((SP++)->{name}:upper)`"
        );
        assert!(
            cpu.is_between_instructions(),
            "${opcode:02X}: três M-cycles e acabou"
        );
    }
}

#[test]
fn the_fourth_pair_of_r16stk_is_af_and_not_sp() {
    let (mut cpu, mut bus) = machine_popping(Pair::Af, 3);
    cpu.registers.set_bc(0x1111);
    cpu.registers.set_de(0x2222);
    cpu.registers.set_hl(0x3333);

    for _ in 0..3 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        cpu.registers.af(),
        POPPED,
        "$F1 é `POP AF`: o índice 3 do placeholder `r16stk` é `af`. O `r16` do \
         1.5a é a tabela vizinha e tem `sp` nesse índice"
    );
    assert_eq!(
        cpu.registers.sp, STACK_AFTER,
        "$F1: e o `SP` é o ponteiro da pilha, não o destino — desempilhar para \
         dentro dele daria {POPPED:#06X}"
    );
}

#[test]
fn pop_af_loads_the_whole_f_byte_including_the_low_nibble() {
    const DIRTY_F: u8 = 0b0101_0111;

    let (mut cpu, mut bus) = machine(&[pop_pair(3)]);
    cpu.registers.sp = STACK_BOTTOM;
    bus.write(LOW_SLOT, DIRTY_F);
    bus.write(HIGH_SLOT, 0x9C);

    for _ in 0..3 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        cpu.registers.f,
        DIRTY_F,
        "o 1.1 decidiu **não** mascarar os bits 3–0 de `F`, e o `POP` é a metade \
         da decisão que **lê**: {:#010b} seria o folclore (`POP AF` descarta o \
         nibble baixo) entrando por hábito. A spec no commit fixado não descreve \
         esses bits; quem cobra a máscara, se ela for necessária, é a blargg \
         `cpu_instrs/01-special` no 1.13, e não este teste",
        DIRTY_F & 0xF0
    );
}

#[test]
fn pop_af_is_the_only_one_of_the_four_that_writes_the_flags() {
    const DIRTY_F: u8 = 0b1111_1010;

    for (rr, pair) in indexed_pairs() {
        let opcode = pop_pair(rr);
        let (mut cpu, mut bus) = machine_popping(pair, rr);
        cpu.registers.f = DIRTY_F;

        for _ in 0..3 {
            cpu.step(&mut bus);
        }

        let expected = if pair == Pair::Af {
            POPPED_BYTES[1]
        } else {
            DIRTY_F
        };

        assert_eq!(
            cpu.registers.f, expected,
            "${opcode:02X}: das quatro linhas do bloco, só o `$F1` tem `Z N H C` \
             nas colunas de flag — as outras três têm `-`. A simetria com o \
             `PUSH` acaba aqui: lá `push_af` **lê** `F` e nenhum dos quatro o \
             escreve, então um `no_pop_touches_the_flags` copiado de lá estaria \
             errado justamente no `POP AF`"
        );
    }
}

#[test]
fn a_pop_changes_only_the_pair_the_stack_pointer_and_the_program_counter() {
    for (rr, pair) in indexed_pairs() {
        let opcode = pop_pair(rr);
        let (mut cpu, mut bus) = machine_popping(pair, rr);

        let mut expected = cpu.registers;
        expected.pc = 0x0101;
        expected.sp = STACK_AFTER;
        pair.set(&mut expected, POPPED);

        for _ in 0..3 {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers,
            expected,
            "${opcode:02X} escreve `{}` e nada mais",
            pair.name()
        );
    }
}

#[test]
fn a_pop_does_not_write_back_to_the_stack() {
    for (rr, pair) in indexed_pairs() {
        let opcode = pop_pair(rr);
        let (mut cpu, mut bus) = machine_popping(pair, rr);

        for _ in 0..3 {
            cpu.step(&mut bus);
        }

        assert_eq!(
            [bus.read(LOW_SLOT), bus.read(HIGH_SLOT)],
            [POPPED_BYTES[1], POPPED_BYTES[0]],
            "${opcode:02X}: os dois M-cycles do `POP` são leitura. A pilha não é \
             consumida nem limpa — quem a solta é o `SP`"
        );
    }
}

#[test]
fn the_stack_pointer_wraps_above_the_top_of_the_address_space() {
    const FROM_IE: u8 = 0x5A;

    let (mut cpu, mut bus) = machine(&[pop_pair(0)]);
    cpu.registers.set_bc(0xFFFF);
    cpu.registers.sp = 0xFFFF;
    bus.write(0xFFFF, FROM_IE);

    for _ in 0..3 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        cpu.registers.bc(),
        u16::from(FROM_IE),
        "`SP` de 16 bits dá a volta: a metade baixa vem de $FFFF (o `IE`) e a \
         alta de $0000, o primeiro byte da ROM. Saturar em $FFFF leria o `IE` \
         duas vezes e daria {:#06X}",
        u16::from_be_bytes([FROM_IE, FROM_IE])
    );
    assert_eq!(cpu.registers.sp, 0x0001, "e o `SP` para depois da volta");
}

fn decoded_elsewhere(opcode: u8) -> bool {
    opcode == 0x00
        || opcode == 0xC3
        || ((0x40..=0x7F).contains(&opcode) && opcode != 0x76)
        || opcode & 0b1100_0111 == 0b0000_0110
        || opcode & 0b1100_0111 == 0b0000_0010
        || opcode & 0b1100_1111 == 0b0000_0001
        || opcode & 0b1100_1111 == 0b1100_0101
        || matches!(
            opcode,
            0x08 | 0xE0 | 0xE2 | 0xEA | 0xF0 | 0xF2 | 0xF9 | 0xFA
        )
        || (0x80..=0x8F).contains(&opcode)
        || (0x90..=0x9F).contains(&opcode)
        || (0xA0..=0xB7).contains(&opcode)
        || (0xB8..=0xBF).contains(&opcode)
        || matches!(
            opcode,
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE
        )
}

#[test]
fn the_block_this_item_decodes_is_exactly_the_four_opcodes_of_11_rr_0001() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00, 0x00]);
        cpu.registers.sp = STACK_BOTTOM;

        for _ in 0..4 {
            cpu.step(&mut bus);
        }

        let in_block = opcode & 0b1100_1111 == 0b1100_0001;

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos quatro `POP r16stk` e o 1.5c o decodifica"
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
                 tem de continuar dizendo `falta implementar`. Máscara que solte \
                 o bit 2 leva `PUSH r16stk` (`11 rr 0101`, o 1.5b); soltar o bit 0 \
                 leva `RET`/`RETI` e os quatro `RST` pares do 1.10"
            );
        }
    }
}
