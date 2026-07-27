//! ROADMAP 1.3 — o laço de M-cycles: `Cpu::step()` avança **um** M-cycle.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup, Registers};

const ENTRY: usize = 0x0100;

const ILLEGAL_OPCODES: [u8; 11] = [
    0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
];

fn rom_with(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);
    rom
}

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let rom = rom_with(program);
    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

#[test]
fn the_cpu_starts_where_the_boot_rom_left_off() {
    let rom = rom_with(&[]);
    let checksum = CartridgeHeader::parse(&rom).unwrap().checksum();

    let cpu = Cpu::after_boot_rom(checksum);

    assert_eq!(
        cpu.registers,
        Registers::after_boot_rom(checksum),
        "o 1.2b-i já decidiu o estado dos registradores no hand-off; \
         a CPU do 1.3 carrega aquilo, não uma segunda cópia da tabela"
    );
    assert!(
        cpu.is_between_instructions(),
        "a boot ROM entrega o controle entre instruções: o próximo M-cycle \
         é o fetch do opcode em $0100"
    );
    assert_eq!(
        cpu.lockup(),
        None,
        "uma CPU recém-entregue não está travada"
    );
}

#[test]
fn each_step_advances_one_m_cycle_not_one_instruction() {
    let (mut cpu, mut bus) = machine(&[0xC3, 0x34, 0x12]);

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0101,
        "o primeiro M-cycle busca só o opcode"
    );
    assert!(
        !cpu.is_between_instructions(),
        "faltam três M-cycles: uma implementação que já terminou aqui é \
         instruction-stepped, e é exatamente isso que a R2 proíbe"
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0102, "um byte de operando por M-cycle");
    assert!(!cpu.is_between_instructions(), "faltam dois M-cycles");

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0103,
        "os dois bytes do operando chegaram, mas o desvio ainda não aconteceu"
    );
    assert!(!cpu.is_between_instructions(), "falta um M-cycle");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x1234, "o quarto M-cycle é o que desvia");
    assert!(
        cpu.is_between_instructions(),
        "quatro M-cycles depois a instrução acabou"
    );
}

#[test]
fn a_run_of_nops_advances_the_pc_one_byte_per_step() {
    let (mut cpu, mut bus) = machine(&[0x00; 8]);

    for step in 1..=8u16 {
        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.pc,
            0x0100 + step,
            "depois de {step} M-cycles de NOP o PC andou {step} bytes"
        );
        assert!(
            cpu.is_between_instructions(),
            "NOP tem um M-cycle só: nunca se está no meio de um"
        );
    }
}

#[test]
fn the_next_instruction_is_fetched_at_the_jump_target() {
    let (mut cpu, mut bus) = machine(&[0xC3, 0x06, 0x01, 0xD3, 0xD3, 0xD3, 0x00]);

    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert_eq!(cpu.registers.pc, 0x0106, "o desvio levou ao alvo");

    cpu.step(&mut bus);
    assert_eq!(
        cpu.lockup(),
        None,
        "o byte em $0106 é NOP; travar aqui quer dizer que o fetch caiu \
         num dos $D3 que o desvio devia ter pulado"
    );
    assert_eq!(cpu.registers.pc, 0x0107, "e a execução seguiu do alvo");
}

#[test]
fn nop_touches_the_pc_and_nothing_else() {
    let (mut cpu, mut bus) = machine(&[0x00]);
    let mut expected = cpu.registers;
    expected.pc = 0x0101;

    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers, expected,
        "NOP não afeta flag nenhuma (`-` nas quatro colunas) nem registrador"
    );
}

#[test]
fn jp_u16_touches_the_pc_and_nothing_else() {
    let (mut cpu, mut bus) = machine(&[0xC3, 0x34, 0x12]);
    let mut expected = cpu.registers;
    expected.pc = 0x1234;

    for _ in 0..4 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        cpu.registers, expected,
        "`C3` tem `-` nas quatro colunas de flag, e o operando não passa por \
         registrador nenhum: o byte baixo e o alto são estado interno do laço"
    );
}

#[test]
fn the_opcodes_the_spec_calls_unused_lock_the_cpu() {
    for opcode in ILLEGAL_OPCODES {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00]);

        cpu.step(&mut bus);

        assert_eq!(
            cpu.lockup(),
            Some(Lockup::IllegalOpcode(opcode)),
            "${opcode:02X} é `-` na coluna GB CPU: trava a CPU, não é NOP"
        );
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X} travou: não há próxima instrução a buscar"
        );
    }
}

#[test]
fn a_locked_cpu_never_runs_again() {
    let (mut cpu, mut bus) = machine(&[0xD3, 0x00, 0x00, 0x00]);

    cpu.step(&mut bus);
    let frozen = cpu.registers;

    for _ in 0..16 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        cpu.registers, frozen,
        "travada é para sempre: o tempo passa, a CPU não anda"
    );
    assert_eq!(
        cpu.lockup(),
        Some(Lockup::IllegalOpcode(0xD3)),
        "e o motivo continua o mesmo"
    );
}

#[test]
fn the_unused_opcodes_are_exactly_the_eleven_the_spec_names() {
    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00]);

        cpu.step(&mut bus);

        let locked_up = matches!(cpu.lockup(), Some(Lockup::IllegalOpcode(_)));
        assert_eq!(
            locked_up,
            ILLEGAL_OPCODES.contains(&opcode),
            "${opcode:02X}: a spec lista onze opcodes inexistentes, e este \
             está do lado errado da lista"
        );
    }
}

#[test]
fn an_illegal_opcode_is_not_mistaken_for_undecoded_one() {
    let (mut cpu, mut bus) = machine(&[0xD3]);

    cpu.step(&mut bus);

    assert_eq!(
        cpu.lockup(),
        Some(Lockup::IllegalOpcode(0xD3)),
        "os 11 opcodes inexistentes usam IllegalOpcode, não UndecodedOpcode"
    );
}

#[test]
fn locking_up_happens_after_the_opcode_byte_was_fetched() {
    let (mut cpu, mut bus) = machine(&[0xD3]);

    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.pc, 0x0101,
        "o opcode foi buscado como qualquer outro; o que não houve foi a \
         instrução"
    );
}
