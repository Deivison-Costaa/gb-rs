//! ROADMAP 1.5d — os dois avulsos do `x16/lsm`: `LD SP,HL` (`$F9`) e
//! `LD (u16),SP` (`$08`).
//!
//! As duas colunas são `fetch → internal` e
//! `fetch → read(u16:lower) → read(u16:upper) → write(SP:lower->(u16)) →
//! write(SP:upper->(u16+1))`.
//!
//! O `$F9` é o primeiro caso do projeto em que a spec local **não decide** o
//! instante: nenhum dos dois passos leva seta, e o par tem de ser escrito em
//! algum deles. O que este arquivo fixa ali é escolha, e o teste diz isso.
//! No `$08` a seta existe e mora nas duas escritas — a leitura do endereço é
//! latch, como no `$FA` do 1.4d. Ver `STATUS.md`, notas 21, 34, 36 e 40.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

const ENTRY: usize = 0x0100;

const LD_SP_HL: u8 = 0xF9;
const LD_IMM16_SP: u8 = 0x08;

// Endereço cujas duas metades trocadas caem na WRAM também: escrita no lugar
// errado fica legível em vez de sumir engolida pela ROM.
const TARGET: u16 = 0xC1D2;
const TARGET_ABOVE: u16 = TARGET.wrapping_add(1);
const TARGET_SWAPPED: u16 = TARGET.swap_bytes();

// Metades diferentes entre si e diferentes das do endereço: byte trocado de
// lugar e byte não escrito ficam ambos visíveis.
const STACK_POINTER: u16 = 0x9C57;
const SP_HIGH: u8 = STACK_POINTER.to_be_bytes()[0];
const SP_LOW: u8 = STACK_POINTER.to_be_bytes()[1];

const SENTINEL_LOW: u8 = 0xA5;
const SENTINEL_HIGH: u8 = 0x5A;

const SOURCE_PAIR: u16 = 0x8B3D;
const STALE_SP: u16 = 0x1234;

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);

    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

const fn store_program(address: u16) -> [u8; 3] {
    let [high, low] = address.to_be_bytes();
    [LD_IMM16_SP, low, high]
}

fn machine_storing(address: u16) -> (Cpu, Bus) {
    let (mut cpu, mut bus) = machine(&store_program(address));

    cpu.registers.sp = STACK_POINTER;
    bus.write(address, SENTINEL_LOW);
    bus.write(address.wrapping_add(1), SENTINEL_HIGH);

    (cpu, bus)
}

fn machine_copying() -> (Cpu, Bus) {
    let (mut cpu, bus) = machine(&[LD_SP_HL]);

    cpu.registers.set_hl(SOURCE_PAIR);
    cpu.registers.sp = STALE_SP;

    (cpu, bus)
}

#[test]
fn the_two_are_one_and_three_bytes_and_two_and_five_m_cycles() {
    let (mut cpu, mut bus) = machine_copying();

    cpu.step(&mut bus);
    assert!(
        !cpu.is_between_instructions(),
        "$F9 tem 8 T-cycles: dois M-cycles, e o fetch é o primeiro"
    );
    cpu.step(&mut bus);
    assert!(
        cpu.is_between_instructions(),
        "$F9: `fetch → internal`, e acabou"
    );
    assert_eq!(cpu.registers.pc, 0x0101, "$F9 tem 1 byte");

    let (mut cpu, mut bus) = machine_storing(TARGET);

    for step in 1..=4 {
        cpu.step(&mut bus);
        assert!(
            !cpu.is_between_instructions(),
            "$08 tem 20 T-cycles: cinco M-cycles, e o M{step} não é o último"
        );
    }
    cpu.step(&mut bus);
    assert!(
        cpu.is_between_instructions(),
        "$08: cinco M-cycles — a instrução mais longa do projeto até aqui"
    );
    assert_eq!(cpu.registers.pc, 0x0103, "$08 tem 3 bytes: opcode e o u16");
}

#[test]
fn ld_sp_hl_copies_the_whole_pair() {
    let (mut cpu, mut bus) = machine_copying();

    for _ in 0..2 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        cpu.registers.sp, SOURCE_PAIR,
        "$F9 é `LD SP,HL`: o par inteiro, não uma metade"
    );
    assert_eq!(
        cpu.registers.hl(),
        SOURCE_PAIR,
        "$F9 copia; a fonte fica onde estava"
    );
}

#[test]
fn the_write_to_sp_is_in_the_internal_and_the_column_does_not_say_so() {
    let (mut cpu, mut bus) = machine_copying();

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.sp, STALE_SP,
        "a coluna do `$F9` é `fetch → internal` e **não tem seta nem anotação \
         em passo nenhum**: onde o par é escrito é escolha deste projeto, não \
         spec. E a spec local não é só omissa — o pouco que ela diz aponta para \
         o outro lado. As únicas linhas que dizem quando o `SP` recebe são o \
         `$33` (`fetch(Probably writes to SP:lower here) → internal(Probably \
         writes to SP:upper here)`) e o `$E8`, e as duas partem o par em duas \
         metades, a baixa primeiro. São `x16/alu` e vêm com `Probably`; o `$F8`, \
         o vizinho de mnemônico, tem `internal` pelado e não sustenta nada. A \
         escolha aqui é o par inteiro no último passo, como o `JP u16` (0013) — \
         que é o único cujo `internal` a tabela anota (`branch decision?`). \
         Escrever no fetch dá o mesmo estado final e os mesmos 8 T-cycles, e \
         este assert é o único do projeto que separa os dois"
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.sp, SOURCE_PAIR, "e o M2 é onde ela acontece");
}

#[test]
fn ld_sp_hl_changes_only_the_stack_pointer_and_the_program_counter() {
    let (mut cpu, mut bus) = machine_copying();

    let mut expected = cpu.registers;
    expected.pc = 0x0101;
    expected.sp = SOURCE_PAIR;

    for _ in 0..2 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        cpu.registers, expected,
        "$F9 não faz acesso ao barramento no M2 e não tem coluna de flags"
    );
}

#[test]
fn ld_imm16_sp_puts_the_low_half_at_the_address_and_the_high_half_above_it() {
    let (mut cpu, mut bus) = machine_storing(TARGET);

    for _ in 0..5 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        [bus.read(TARGET), bus.read(TARGET_ABOVE)],
        [SP_LOW, SP_HIGH],
        "`write(SP:lower->(u16))` antes de `write(SP:upper->(u16+1))`: little-\
         endian, metade baixa no endereço baixo. O `PUSH` do 1.5b guarda o mesmo \
         layout final escrevendo a metade **alta** primeiro, porque lá o \
         endereço desce a cada escrita e aqui sobe — copiar a ordem de lá \
         inverteria o par na memória"
    );
    assert_eq!(
        cpu.registers.sp, STACK_POINTER,
        "$08 lê o `SP`; quem o move é o `PUSH`"
    );
}

#[test]
fn the_two_writes_are_the_last_two_m_cycles_one_byte_each() {
    let (mut cpu, mut bus) = machine_storing(TARGET);

    for step in 1..=3 {
        cpu.step(&mut bus);
        assert_eq!(
            [bus.read(TARGET), bus.read(TARGET_ABOVE)],
            [SENTINEL_LOW, SENTINEL_HIGH],
            "o M{step} do `$08` é fetch ou leitura do operando: nada foi escrito \
             ainda. Os dois bytes do endereço vão para um latch — `read(u16:lower)` \
             está **sem seta**, como o `$FA` do 1.4d e ao contrário do \
             `read(u16:lower->C)` do 1.5a"
        );
    }

    cpu.step(&mut bus);
    assert_eq!(
        [bus.read(TARGET), bus.read(TARGET_ABOVE)],
        [SP_LOW, SENTINEL_HIGH],
        "depois do M4 só a metade baixa está gravada. Juntar as duas escritas \
         num M-cycle e gastar o M5 num `internal` dá a mesma memória no fim, os \
         mesmos 20 T-cycles e o mesmo `SP` — é o erro #1 da 0015 na instrução \
         que escreve mais bytes do projeto, e este é o único assert que o vê \
         (nota 40: do lado que escreve, o erro de instante é silencioso)"
    );

    cpu.step(&mut bus);
    assert_eq!(
        [bus.read(TARGET), bus.read(TARGET_ABOVE)],
        [SP_LOW, SP_HIGH],
        "e o M5 fecha em `(u16+1)`"
    );
}

#[test]
fn the_two_operand_bytes_are_read_one_per_m_cycle() {
    const AFTER: [u16; 5] = [0x0101, 0x0102, 0x0103, 0x0103, 0x0103];

    let (mut cpu, mut bus) = machine_storing(TARGET);

    for (step, expected) in (1..=5).zip(AFTER) {
        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.pc, expected,
            "o `PC` anda um byte por M-cycle até o M3 e para. Ler os dois bytes \
             do endereço no M2 e gastar o M3 num `internal` dá o mesmo endereço, \
             a mesma memória, o mesmo `PC` no fim e os mesmos 20 T-cycles — e \
             põe dois acessos ao barramento num M-cycle só, contra a invariante \
             do 1.3. O `PC` observado **entre** os passos (aqui, o M{step}) é o \
             único lugar em que isso aparece"
        );
    }
}

#[test]
fn the_operand_of_the_store_is_little_endian() {
    let (mut cpu, mut bus) = machine_storing(TARGET);

    for _ in 0..5 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        [
            bus.read(TARGET_SWAPPED),
            bus.read(TARGET_SWAPPED.wrapping_add(1))
        ],
        [0x00, 0x00],
        "`read(u16:lower)` é o **primeiro** dos dois: o byte que segue o opcode é \
         a metade baixa. Lidos na ordem trocada o destino seria \
         {TARGET_SWAPPED:#06X}, que também é WRAM — por isso a escrita errada \
         apareceria aqui em vez de sumir engolida pela ROM"
    );
}

#[test]
fn the_second_write_crosses_the_region_boundary_above_the_first() {
    const LAST_HRAM_BYTE: u16 = 0xFFFE;

    let (mut cpu, mut bus) = machine_storing(LAST_HRAM_BYTE);

    for _ in 0..5 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        [bus.read(LAST_HRAM_BYTE), bus.read(0xFFFF)],
        [SP_LOW, SP_HIGH],
        "`(u16+1)` é o endereço seguinte no mapa inteiro, não dentro da região: \
         $FFFE é o último byte da HRAM e $FFFF é o `IE`"
    );
}

#[test]
fn the_address_wraps_above_the_top_of_the_address_space() {
    let (mut cpu, mut bus) = machine_storing(0xFFFF);

    for _ in 0..5 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        bus.read(0xFFFF),
        SP_LOW,
        "`u16+1` de $FFFF dá a volta para $0000, que é ROM e engole a escrita. \
         Saturar em $FFFF poria as duas metades no mesmo endereço e o `IE` \
         acabaria valendo {SP_HIGH:#04X}"
    );
    assert_eq!(
        bus.read(0x0000),
        0x00,
        "e a metade alta se perde na ROM, sem pânico"
    );
}

#[test]
fn neither_of_the_two_touches_the_flags() {
    const DIRTY_F: u8 = 0b1010_0101;

    for (opcode, steps) in [(LD_SP_HL, 2), (LD_IMM16_SP, 5)] {
        let (mut cpu, mut bus) = if opcode == LD_SP_HL {
            machine_copying()
        } else {
            machine_storing(TARGET)
        };
        cpu.registers.f = DIRTY_F;

        for _ in 0..steps {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.f, DIRTY_F,
            "${opcode:02X} tem `-` nas quatro colunas de flag. O vizinho de \
             mnemônico é o `$F8` (`LD HL,SP+i8`), que tem `0 0 H C` — e é o 1.7, \
             não este item"
        );
    }
}

#[test]
fn the_store_changes_only_the_program_counter() {
    let (mut cpu, mut bus) = machine_storing(TARGET);

    let mut expected = cpu.registers;
    expected.pc = 0x0103;

    for _ in 0..5 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        cpu.registers, expected,
        "$08 é escrita pura: nenhum registrador recebe nada"
    );
}

fn decoded_elsewhere(opcode: u8) -> bool {
    opcode == 0x00
        || opcode == 0xC3
        || ((0x40..=0x7F).contains(&opcode) && opcode != 0x76)
        || opcode & 0b1100_0111 == 0b0000_0110
        || opcode & 0b1100_0111 == 0b0000_0010
        || opcode & 0b1100_1111 == 0b0000_0001
        || opcode & 0b1100_1111 == 0b1100_0101
        || opcode & 0b1100_1111 == 0b1100_0001
        || matches!(opcode, 0xE0 | 0xE2 | 0xEA | 0xF0 | 0xF2 | 0xFA)
        || (0x80..=0x8F).contains(&opcode)
        || (0x90..=0x9F).contains(&opcode)
        || (0xA0..=0xB7).contains(&opcode)
        || (0xB8..=0xBF).contains(&opcode)
}

#[test]
fn the_opcodes_this_item_decodes_are_exactly_08_and_f9() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00, 0x00, 0x00, 0x00]);
        cpu.registers.sp = STACK_POINTER;

        for _ in 0..6 {
            cpu.step(&mut bus);
        }

        if matches!(opcode, LD_SP_HL | LD_IMM16_SP) {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos dois avulsos do 1.5d"
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
                "${opcode:02X} é opcode legítimo fora deste sub-item. Os dois são \
                 reconhecidos um a um: `$08` é `00 001 000` e a máscara de bloco \
                 (`00 rrr 000`) levaria `NOP`, `STOP` e os cinco `JR`; `$F9` é \
                 `11 111 001` e soltar o bit 0 leva o `$F8` (`LD HL,SP+i8`), que é \
                 o 1.7 e tem flags"
            );
        }
    }
}

#[test]
fn the_bit_layouts_of_the_two_live_under_the_wrong_heading_in_02_cpu() {
    assert_eq!(
        [0b0000_1000, 0b1111_1001],
        [LD_IMM16_SP, LD_SP_HL],
        "nenhum dos dois tem campo de placeholder — os oito bits são constantes. \
         No `02-cpu.md` o layout do `$08` está sob o cabeçalho `ld r16, imm16` e \
         o do `$F9` sob `add sp, imm8`, porque a conversão para Markdown funde \
         instruções consecutivas sob o primeiro cabeçalho do grupo (nota 38). \
         `grep` pelo mnemônico devolve zero para os dois; quem confirma a \
         codificação é o `03-opcodes.md`"
    );
}

#[test]
fn the_store_into_rom_is_swallowed_and_does_not_lock_the_cpu() {
    let (mut cpu, mut bus) = machine_storing(0x0200);

    for _ in 0..5 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        [bus.read(0x0200), bus.read(0x0201)],
        [0x00, 0x00],
        "escrever em ROM não é erro nem exceção: o `NoMbc` engole (0.4). O que \
         não pode é a CPU travar ou o `SP` mudar por causa disso"
    );
    assert_eq!(cpu.registers.sp, STACK_POINTER, "e o `SP` segue o mesmo");
    assert_eq!(cpu.lockup(), None, "e a CPU não trava");
}
