//! ROADMAP 1.6a — `ADD a,r8` e `ADC a,r8`: os blocos `10 000 rrr` e `10 001 rrr`.
//!
//! **As primeiras flags calculadas do projeto.** Até a 0021 as 254 linhas
//! implementadas tinham `-` nas quatro colunas, então nenhum teste anterior
//! afirma nada sobre `Z`/`N`/`H`/`C` além do estado de hand-off. As definições
//! estão no `02-cpu.md`: `H` é *"carry for the lower 4 bits of the result"*
//! (§ The BCD Flags) e `C` é o resultado de 8 bits ficar *"higher than $FF"*
//! (§ The Carry Flag). São elas que os testes citam — não uma expressão
//! deduzida de um `ADD` que passou.
//!
//! A coluna do `$86` é `fetch → read((HL))`, **sem seta**, em 8 T-cycles. A nota
//! 34 diz que seta ausente é latch; aqui não há terceiro passo onde o latch
//! pudesse aterrissar, e a ausência significa outra coisa — que o byte não vai
//! para registrador nenhum, e sim para a ALU. Ver `STATUS.md`, notas 32 e 34.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Flag, Lockup, Registers};

const ENTRY: usize = 0x0100;
const ENTRY_ADDRESS: u16 = 0x0100;

const OPERAND_SLOT: u16 = 0xC7A0;

// Semente com os sete registradores distintos entre si e distintos de A: soma
// que pegue o operando errado dá resultado errado, e não coincidência.
const SEED_A: u8 = 0x21;
const SEED: [u8; 6] = [0x31, 0x42, 0x53, 0x64, 0xC7, 0xA0];

const R8_NAMES: [&str; 8] = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];

const fn add_a(r8: u8) -> u8 {
    0b1000_0000 | r8
}

const fn adc_a(r8: u8) -> u8 {
    0b1000_1000 | r8
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

// SEED[4]/SEED[5] são H/L, então `HL` já aponta para OPERAND_SLOT: o operando do
// `$86` mora onde os registradores H e L mandam, e não num endereço à parte.
fn seed(registers: &mut Registers) {
    registers.a = SEED_A;
    registers.b = SEED[0];
    registers.c = SEED[1];
    registers.d = SEED[2];
    registers.e = SEED[3];
    registers.h = SEED[4];
    registers.l = SEED[5];
}

fn operand_of(registers: &Registers, r8: u8, from_memory: u8) -> u8 {
    match r8 {
        0 => registers.b,
        1 => registers.c,
        2 => registers.d,
        3 => registers.e,
        4 => registers.h,
        5 => registers.l,
        6 => from_memory,
        _ => registers.a,
    }
}

fn m_cycles_of(opcode: u8) -> usize {
    if opcode & 0b111 == 6 { 2 } else { 1 }
}

#[test]
fn the_block_2_layout_gives_the_sixteen_opcodes_gbops_lists() {
    let opcodes: Vec<u8> = (0..8).map(add_a).chain((0..8).map(adc_a)).collect();

    assert_eq!(
        opcodes,
        (0x80..=0x8Fu8).collect::<Vec<u8>>(),
        "§ Block 2 do `02-cpu.md`: `10 ooo rrr`, com o operando em 2-0 e a \
         operação em 5-3. As oito tabelas de layout do bloco estão fundidas sob \
         o cabeçalho `add a, r8` (nota 38) e se leem pelos bits: `000` é `ADD` e \
         `001` é `ADC`. `010` já é `SUB` ($90), que é o 1.6b"
    );
}

#[test]
fn add_sums_the_operand_into_a_for_every_r8_register() {
    for r8 in 0..8u8 {
        let opcode = add_a(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        bus.write(OPERAND_SLOT, 0x0C);

        let expected = SEED_A.wrapping_add(operand_of(&cpu.registers, r8, 0x0C));

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.a,
            expected,
            "${opcode:02X} é `ADD A,{}`: o operando entra, o acumulador recebe. \
             `$87` (`ADD A,A`) dobra — o operando e o destino são o mesmo \
             registrador, e ler o destino depois de escrever daria {:#04X}",
            R8_NAMES[r8 as usize],
            SEED_A.wrapping_mul(2).wrapping_add(SEED_A)
        );
    }
}

#[test]
fn adc_sums_the_operand_and_the_incoming_carry_into_a() {
    for r8 in 0..8u8 {
        let opcode = adc_a(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        cpu.registers.set_flag(Flag::C, true);
        bus.write(OPERAND_SLOT, 0x0C);

        let expected = SEED_A
            .wrapping_add(operand_of(&cpu.registers, r8, 0x0C))
            .wrapping_add(1);

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.a,
            expected,
            "${opcode:02X} é `ADC A,{}`: a § The Carry Flag do `02-cpu.md` lista \
             `ADC` entre as instruções que **usam** o `C`. Sem o carry de entrada \
             o resultado seria {:#04X} — um a menos, e invisível em qualquer \
             teste que entre com `C` limpo",
            R8_NAMES[r8 as usize],
            expected.wrapping_sub(1)
        );
    }
}

#[test]
fn add_ignores_the_incoming_carry() {
    for r8 in 0..8u8 {
        let opcode = add_a(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        cpu.registers.set_flag(Flag::C, true);
        bus.write(OPERAND_SLOT, 0x0C);

        let expected = SEED_A.wrapping_add(operand_of(&cpu.registers, r8, 0x0C));

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.a, expected,
            "${opcode:02X} é `ADD`, não `ADC`: o que separa os dois blocos é o \
             bit 3, e é só o `10 001 rrr` que soma o carry de entrada. Este é o \
             controle do teste do `ADC` — sem ele, uma ALU que sempre soma o \
             carry passa nos dois"
        );
    }
}

#[test]
fn the_half_carry_is_the_one_out_of_bit_3_and_the_carry_is_the_one_out_of_bit_7() {
    // (A, operando, Z, H, C) — os quatro cantos da conta, lidos das definições.
    const CASES: [(u8, u8, bool, bool, bool); 5] = [
        (0x0F, 0x01, false, true, false),
        (0xF0, 0x10, true, false, true),
        (0xFF, 0x01, true, true, true),
        (0x08, 0x07, false, false, false),
        (0x80, 0x80, true, false, true),
    ];

    for (a, operand, zero, half, carry) in CASES {
        let (mut cpu, mut bus) = machine(&[add_a(0)]);
        cpu.registers.a = a;
        cpu.registers.b = operand;
        cpu.registers.f = 0x00;

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.flag(Flag::H),
            half,
            "`ADD A,B` com A={a:#04X} e B={operand:#04X}: `H` é *carry for the \
             lower 4 bits of the result* (§ The BCD Flags). Calculá-lo do bit 7 \
             — o mesmo bit do `C` — daria {}",
            carry
        );
        assert_eq!(
            cpu.registers.flag(Flag::C),
            carry,
            "A={a:#04X} + B={operand:#04X}: `C` é o resultado de 8 bits ficar \
             *higher than $FF* (§ The Carry Flag), e não o carry do nibble baixo"
        );
        assert_eq!(
            cpu.registers.flag(Flag::Z),
            zero,
            "A={a:#04X} + B={operand:#04X}: `Z` é sobre o resultado de 8 bits que \
             sobrou, não sobre a soma antes do truncamento — $FF + $01 dá $100, e \
             o que fica em `A` é $00"
        );
    }
}

#[test]
fn the_incoming_carry_of_adc_counts_for_the_half_carry_too() {
    for (carry_in, half, result) in [(false, false, 0x0Fu8), (true, true, 0x10)] {
        let (mut cpu, mut bus) = machine(&[adc_a(0)]);
        cpu.registers.a = 0x0F;
        cpu.registers.b = 0x00;
        cpu.registers.set_flag(Flag::C, carry_in);

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.a,
            result,
            "A=$0F + B=$00 + C={}",
            u8::from(carry_in)
        );
        assert_eq!(
            cpu.registers.flag(Flag::H),
            half,
            "`ADC A,B` com A=$0F, B=$00 e C={}: `H` é carry dos 4 bits baixos **do \
             resultado**, e o resultado do `ADC` inclui o carry de entrada. Uma \
             ALU que some o carry só no total de 8 bits acerta `A` e erra `H` — \
             $0F+$00 não estoura nibble nenhum, mas $0F+$00+1 estoura",
            u8::from(carry_in)
        );
    }
}

#[test]
fn every_sum_of_the_two_operations_matches_the_two_flag_definitions() {
    for (base, uses_carry_in) in [(0x80u8, false), (0x88, true)] {
        let (mut cpu, mut bus) = machine(&[base]);

        for a in 0..=0xFFu8 {
            for operand in 0..=0xFFu8 {
                for carry_in in [false, true] {
                    // Nibble baixo em 1 e Z/N/H em 1 na entrada: flag calculada
                    // que não seja escrita fica visível, e o nibble baixo mede a
                    // ausência da máscara (invariante do 1.1).
                    cpu.registers.f = if carry_in { 0xFF } else { 0xEF };
                    cpu.registers.pc = ENTRY_ADDRESS;
                    cpu.registers.a = a;
                    cpu.registers.b = operand;

                    cpu.step(&mut bus);

                    let addend = u16::from(uses_carry_in && carry_in);
                    let total = u16::from(a) + u16::from(operand) + addend;
                    let result = cpu.registers.a;

                    // Carry que **entra** no bit 4 — derivação independente da
                    // soma de nibbles, e a mesma grandeza que a § BCD Flags nomeia.
                    let half = (a ^ operand ^ result) & 0x10 != 0;

                    let expected = u8::from(result == 0) << 7
                        | u8::from(half) << 5
                        | u8::from(total > 0xFF) << 4
                        | 0x0F;

                    assert_eq!(
                        u16::from(result),
                        total & 0xFF,
                        "${base:02X}: A={a:#04X}, B={operand:#04X}, C de entrada={}",
                        u8::from(carry_in)
                    );
                    assert_eq!(
                        cpu.registers.f,
                        expected,
                        "${base:02X} com A={a:#04X}, B={operand:#04X} e C de \
                         entrada={}: as quatro flags, uma varredura de todos os \
                         65536 pares. `N` é `0` literal na coluna e entrou `1`; \
                         `H` sai do XOR (que é o carry que entra no bit 4, não uma \
                         segunda soma de nibbles); o nibble baixo de `F` entrou \
                         `1111` e a ALU não o mascara (invariante do 1.1)",
                        u8::from(carry_in)
                    );
                }
            }
        }
    }
}

// Ler `(HL)` dentro do fetch e gastar o M2 aplicando dá os mesmos 2 M-cycles, os
// mesmos 8 T-cycles e o mesmo estado final — passou verde nos 251 testes. O que
// separa as duas versões é trocar a memória **entre** os dois `step`.
#[test]
fn the_bus_access_of_86_happens_in_the_m2_and_not_during_the_fetch() {
    const BEFORE: u8 = 0x11;
    const AFTER: u8 = 0x22;

    for opcode in [0x86u8, 0x8E] {
        let (mut cpu, mut bus) = machine(&[opcode]);
        cpu.registers.a = 0x00;
        cpu.registers.set_hl(OPERAND_SLOT);
        cpu.registers.f = 0x00;
        bus.write(OPERAND_SLOT, BEFORE);

        cpu.step(&mut bus);
        bus.write(OPERAND_SLOT, AFTER);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.a, AFTER,
            "${opcode:02X}: a coluna põe o acesso no M2, e uma chamada de \
             `Cpu::step` faz **no máximo um** acesso ao barramento (invariante do \
             1.3) — o M1 já gastou o dele lendo o opcode. Ler `(HL)` junto do \
             fetch e gastar o M2 somando dá o mesmo `A` final, os mesmos 2 \
             M-cycles e os mesmos 8 T-cycles: o único observável é o operando ter \
             mudado no meio, e aí a versão errada soma {BEFORE:#04X}. É o M11 da \
             0021 outra vez (nota 43), agora com a memória no lugar do `PC`"
        );
    }
}

#[test]
fn the_operand_of_86_is_read_in_the_second_m_cycle_and_the_sum_lands_there_too() {
    for (opcode, operand, expected) in [(0x86u8, 0x01u8, 0x10u8), (0x8E, 0x00, 0x10)] {
        let (mut cpu, mut bus) = machine(&[opcode]);
        cpu.registers.a = 0x0F;
        cpu.registers.set_hl(OPERAND_SLOT);
        cpu.registers.f = 0x10;
        bus.write(OPERAND_SLOT, operand);

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.a, 0x0F,
            "${opcode:02X}: o M1 é o fetch e não lê o operando nem soma"
        );
        assert_eq!(
            cpu.registers.f, 0x10,
            "${opcode:02X}: e não mexe nas flags — o `C` que entrou continua de pé \
             para o M2 do `$8E` consumir"
        );
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X} tem 8 T-cycles: dois M-cycles"
        );

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.a, expected,
            "${opcode:02X}: o M2 lê `(HL)` **e** soma. A coluna é `read((HL))` sem \
             seta, e pela nota 34 seta ausente é latch — mas a linha tem 8 \
             T-cycles e não 12, então não existe o terceiro M-cycle onde o latch \
             aterrissaria. Aqui a seta falta porque o byte não vai para \
             registrador nenhum: vai para a ALU"
        );
        assert!(
            cpu.is_between_instructions(),
            "${opcode:02X}: dois M-cycles e acabou — pôr um `internal` no fim daria \
             12 T-cycles, que é a linha de outra instrução"
        );
        assert!(
            cpu.registers.flag(Flag::H),
            "${opcode:02X}: e as flags saem no mesmo M2 do resultado"
        );
        assert_eq!(
            cpu.registers.pc, 0x0101,
            "${opcode:02X} tem **1** byte: o operando vem de `(HL)`, não do `PC`. \
             Quem lê do `PC` é o `$C6`/`$CE` (`ADD A,u8`/`ADC A,u8`), que é o 1.6d \
             — mesma forma de M-cycle, endereço de outro lugar"
        );
    }
}

#[test]
fn an_add_or_adc_touches_only_a_the_flags_and_the_program_counter() {
    for r8 in 0..8u8 {
        for opcode in [add_a(r8), adc_a(r8)] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed(&mut cpu.registers);
            bus.write(OPERAND_SLOT, 0x0C);

            let mut expected = cpu.registers;

            for _ in 0..m_cycles_of(opcode) {
                cpu.step(&mut bus);
            }

            expected.pc = 0x0101;
            expected.a = cpu.registers.a;
            expected.f = cpu.registers.f;

            assert_eq!(
                cpu.registers, expected,
                "${opcode:02X}: `A` é o único destino da coluna. `$84`/`$85` leem \
                 `H`/`L` e `$86` lê `(HL)` — nenhum dos três escreve de volta no \
                 par, e o `SP` não entra nesta conta"
            );
        }
    }
}

#[test]
fn no_opcode_of_this_item_writes_to_memory() {
    for r8 in 0..8u8 {
        for opcode in [add_a(r8), adc_a(r8)] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed(&mut cpu.registers);
            bus.write(OPERAND_SLOT, 0x0C);

            for _ in 0..m_cycles_of(opcode) {
                cpu.step(&mut bus);
            }

            assert_eq!(
                bus.read(OPERAND_SLOT),
                0x0C,
                "${opcode:02X}: os dois M-cycles do `$86`/`$8E` são fetch e \
                 leitura. Guardar o resultado em `(HL)` é o `INC (HL)` do 1.6e, e \
                 lá são três M-cycles"
            );
        }
    }
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
        || (0x90..=0x9F).contains(&opcode)
        || (0xB8..=0xBF).contains(&opcode)
        || matches!(
            opcode,
            0x08 | 0xE0 | 0xE2 | 0xEA | 0xF0 | 0xF2 | 0xF9 | 0xFA
        )
}

#[test]
fn the_block_this_item_decodes_is_exactly_the_sixteen_opcodes_of_1000_xrrr() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00, 0x00, 0x00]);
        seed(&mut cpu.registers);

        for _ in 0..5 {
            cpu.step(&mut bus);
        }

        let in_block = (0x80..=0x8F).contains(&opcode);

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos dezesseis `ADD`/`ADC A,r8` e o 1.6a o \
                 decodifica"
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
                "${opcode:02X} é opcode legítimo fora deste sub-item: o rótulo tem \
                 de continuar dizendo `falta implementar`. Máscara que solte o bit \
                 3 leva `SUB`/`SBC` (`10 01x rrr`, o 1.6b); soltar o bit 4 leva \
                 `AND`/`XOR` (o 1.6c); trocar `10` por `11` no topo leva o bloco \
                 `11 ooo 110` do 1.6d, que tem um byte a mais"
            );
        }
    }
}
