//! ROADMAP 1.5b — `PUSH r16stk`: o bloco `11 rr 0101`.
//!
//! A coluna é `fetch → internal → write(B->(--SP)) → write(C->(--SP))`: o
//! `--SP` mora **dentro** do passo da escrita, como o `HL++` do 1.4c. Decrementar
//! no `internal` do M2 dá o mesmo estado final e os mesmos 16 T-cycles — só o
//! `SP` observado entre os M-cycles muda. Ver `STATUS.md`, notas 32 e 34.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

const ENTRY: usize = 0x0100;

const STACK_TOP: u16 = 0xCF00;
const HIGH_SLOT: u16 = STACK_TOP.wrapping_sub(1);
const LOW_SLOT: u16 = STACK_TOP.wrapping_sub(2);

const PUSHED: u16 = 0x9C57;
const PUSHED_BYTES: [u8; 2] = PUSHED.to_be_bytes();
const UNTOUCHED: u8 = 0x3D;

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

    const fn set(self, cpu: &mut Cpu, value: u16) {
        match self {
            Self::Bc => cpu.registers.set_bc(value),
            Self::De => cpu.registers.set_de(value),
            Self::Hl => cpu.registers.set_hl(value),
            Self::Af => cpu.registers.set_af(value),
        }
    }
}

const fn push_pair(rr: u8) -> u8 {
    0b1100_0101 | (rr << 4)
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

// Pilha marcada: "continua zerado" acertaria por acidente, já que a WRAM nasce zerada.
fn machine_pushing(pair: Pair, rr: u8) -> (Cpu, Bus) {
    let (mut cpu, mut bus) = machine(&[push_pair(rr)]);

    pair.set(&mut cpu, PUSHED);
    cpu.registers.sp = STACK_TOP;
    bus.write(HIGH_SLOT, UNTOUCHED);
    bus.write(LOW_SLOT, UNTOUCHED);

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
        [push_pair(0), push_pair(1), push_pair(2), push_pair(3)],
        [0xC5, 0xD5, 0xE5, 0xF5],
        "`PUSH BC` `PUSH DE` `PUSH HL` `PUSH AF`"
    );
}

#[test]
fn every_pair_is_pushed_high_byte_first_to_the_higher_address() {
    for (rr, pair) in indexed_pairs() {
        let opcode = push_pair(rr);
        let (mut cpu, mut bus) = machine_pushing(pair, rr);

        for _ in 0..4 {
            cpu.step(&mut bus);
        }

        assert_eq!(
            [bus.read(HIGH_SLOT), bus.read(LOW_SLOT)],
            PUSHED_BYTES,
            "${opcode:02X} é `PUSH {}`: a metade **alta** vai para o endereço mais \
             alto, e é ela que a coluna escreve primeiro. Invertido, a pilha fica \
             {:?} — e a leitura little-endian de volta daria {:#06X}",
            pair.name(),
            [PUSHED_BYTES[1], PUSHED_BYTES[0]],
            PUSHED.swap_bytes()
        );
        assert_eq!(
            cpu.registers.sp, LOW_SLOT,
            "${opcode:02X}: dois decrementos, um por escrita"
        );
        assert_eq!(cpu.registers.pc, 0x0101, "${opcode:02X} tem 1 byte");
    }
}

#[test]
fn the_stack_pointer_moves_between_the_two_writes() {
    for (rr, pair) in indexed_pairs() {
        let opcode = push_pair(rr);
        let name = pair.name();
        let (mut cpu, mut bus) = machine_pushing(pair, rr);

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.sp, STACK_TOP,
            "${opcode:02X}: o M1 é o fetch e não mexe no `SP`"
        );
        assert_eq!(cpu.registers.pc, 0x0101, "${opcode:02X}: o M1 leu o opcode");
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X} tem 16 T-cycles: quatro M-cycles"
        );

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.sp, STACK_TOP,
            "${opcode:02X}: o M2 é `internal`, e o `--SP` da coluna está **dentro** \
             do passo da escrita — `write({name}:upper->(--SP))`. Decrementar aqui \
             dá o mesmo estado final e os mesmos 16 T-cycles"
        );
        assert_eq!(
            [bus.read(HIGH_SLOT), bus.read(LOW_SLOT)],
            [UNTOUCHED, UNTOUCHED],
            "${opcode:02X}: o M2 não acessa o barramento (R2: no máximo um acesso \
             por M-cycle, e este é o M-cycle sem nenhum)"
        );

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.sp, HIGH_SLOT,
            "${opcode:02X}: o M3 decrementa o `SP` e escreve `{name}:upper` no \
             endereço novo"
        );
        assert_eq!(
            bus.read(HIGH_SLOT),
            PUSHED_BYTES[0],
            "${opcode:02X}: a metade alta de `{name}` já está na pilha depois do M3"
        );
        assert_eq!(
            bus.read(LOW_SLOT),
            UNTOUCHED,
            "${opcode:02X}: e a metade baixa ainda não — são dois M-cycles, dois \
             acessos, e é entre eles que a diferença aparece"
        );
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X}: ainda falta o M4"
        );

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.sp, LOW_SLOT,
            "${opcode:02X}: o M4 decrementa de novo antes de escrever"
        );
        assert_eq!(
            bus.read(LOW_SLOT),
            PUSHED_BYTES[1],
            "${opcode:02X}: o M4 é `write({name}:lower->(--SP))`"
        );
        assert!(
            cpu.is_between_instructions(),
            "${opcode:02X}: quatro M-cycles e acabou"
        );
    }
}

#[test]
fn the_internal_m_cycle_of_a_push_changes_nothing_but_the_program_counter() {
    for (rr, pair) in indexed_pairs() {
        let opcode = push_pair(rr);
        let (mut cpu, mut bus) = machine_pushing(pair, rr);

        cpu.step(&mut bus);
        let after_fetch = cpu.registers;

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers, after_fetch,
            "${opcode:02X}: o M2 do `PUSH` é o primeiro `internal` do projeto que \
             não é o último passo, e ele não tem efeito nenhum — nem `SP`, nem o \
             par, nem o `PC`, que só anda em acesso ao barramento"
        );
    }
}

#[test]
fn the_fourth_pair_of_r16stk_is_af_and_not_sp() {
    let (mut cpu, mut bus) = machine_pushing(Pair::Af, 3);
    cpu.registers.set_bc(0x1111);
    cpu.registers.set_de(0x2222);
    cpu.registers.set_hl(0x3333);

    for _ in 0..4 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        [bus.read(HIGH_SLOT), bus.read(LOW_SLOT)],
        PUSHED_BYTES,
        "$F5 é `PUSH AF`: o índice 3 do placeholder `r16stk` é `af`. O `r16` do \
         1.5a é a tabela vizinha e tem `sp` nesse índice — empilhar o `SP` daria \
         {:#06X}, que é o próprio ponteiro",
        STACK_TOP
    );
}

#[test]
fn push_af_writes_the_whole_f_byte_including_the_low_nibble() {
    const DIRTY_F: u8 = 0b0101_0111;

    let (mut cpu, mut bus) = machine_pushing(Pair::Af, 3);
    cpu.registers.f = DIRTY_F;

    for _ in 0..4 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        bus.read(LOW_SLOT),
        DIRTY_F,
        "o 1.1 decidiu **não** mascarar os bits 3–0 de `F` (a spec no commit \
         fixado não os descreve), e o `PUSH` é a metade da decisão que escreve: \
         {:#010b} seria a máscara entrando por hábito. Quem cobra a máscara, se \
         ela for necessária, é a blargg `cpu_instrs/01-special` no 1.13",
        DIRTY_F & 0xF0
    );
}

#[test]
fn a_push_changes_only_the_stack_pointer_and_the_program_counter() {
    for (rr, pair) in indexed_pairs() {
        let opcode = push_pair(rr);
        let (mut cpu, mut bus) = machine_pushing(pair, rr);

        let mut expected = cpu.registers;
        expected.pc = 0x0101;
        expected.sp = LOW_SLOT;

        for _ in 0..4 {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers,
            expected,
            "${opcode:02X} lê `{}` e não o altera",
            pair.name()
        );
    }
}

#[test]
fn no_push_touches_the_flags() {
    const DIRTY_F: u8 = 0b1111_1010;

    for (rr, pair) in indexed_pairs() {
        let opcode = push_pair(rr);
        let (mut cpu, mut bus) = machine_pushing(pair, rr);
        cpu.registers.f = DIRTY_F;

        for _ in 0..4 {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.f, DIRTY_F,
            "${opcode:02X} tem `-` nas quatro colunas de flag da tabela — inclusive \
             o `PUSH AF`, que **lê** `F` e não o escreve"
        );
    }
}

#[test]
fn the_stack_pointer_wraps_below_zero() {
    let (mut cpu, mut bus) = machine(&[push_pair(0)]);
    cpu.registers.set_bc(PUSHED);
    cpu.registers.sp = 0x0000;

    for _ in 0..4 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        [bus.read(0xFFFF), bus.read(0xFFFE)],
        PUSHED_BYTES,
        "`SP` de 16 bits dá a volta: abaixo de $0000 vêm $FFFF (o `IE`) e $FFFE \
         (o último byte da HRAM)"
    );
    assert_eq!(cpu.registers.sp, 0xFFFE, "e o `SP` para lá");
}

fn decoded_elsewhere(opcode: u8) -> bool {
    opcode == 0x00
        || opcode == 0xC3
        || ((0x40..=0x7F).contains(&opcode) && opcode != 0x76)
        || opcode & 0b1100_0111 == 0b0000_0110
        || opcode & 0b1100_0111 == 0b0000_0010
        || opcode & 0b1100_1111 == 0b0000_0001
        || opcode & 0b1100_1111 == 0b1100_0001
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
fn the_block_this_item_decodes_is_exactly_the_four_opcodes_of_11_rr_0101() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00, 0x00]);
        cpu.registers.sp = STACK_TOP;

        for _ in 0..4 {
            cpu.step(&mut bus);
        }

        let in_block = opcode & 0b1100_1111 == 0b1100_0101;

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos quatro `PUSH r16stk` e o 1.5b o decodifica"
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
                 o bit 2 leva `POP r16stk` (`11 rr 0001`, o 1.5c); soltar o bit 3 \
                 leva `CALL u16` (`$CD`) e três dos onze inexistentes"
            );
        }
    }
}
