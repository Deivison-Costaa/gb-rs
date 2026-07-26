//! ROADMAP 1.3 — o laço de M-cycles: `Cpu::step()` avança **um** M-cycle.
//!
//! Spec: `docs/reference/03-opcodes.md`, coluna *M-cycles (passo a passo)*
//! (gbops `90b9bf296aed`), e `docs/reference/02-cpu.md` § Moved, Removed, and
//! Added Opcodes (Pan Docs `fe246067b695`).
//!
//! Esta é a **R2** do `CLAUDE.md`, e a regra mais cara de violar do projeto:
//!
//! > `Cpu::step()` avança **um M-cycle** e retorna. […] Não implemente "executa
//! > a instrução inteira e depois soma N ciclos" — isso quebra a suíte Mooneye e
//! > é caríssimo de refatorar depois.
//!
//! O problema de testar a R2 é que ela é uma afirmação sobre *quando* as coisas
//! acontecem, e uma implementação instruction-stepped chega ao mesmo lugar no
//! fim. O que a separa é o **meio da instrução**: com `JP u16`, que a tabela
//! descreve como `fetch → read(u16:lower) → read(u16:upper) → internal`, um
//! `step()` instruction-stepped já entrega `PC = $1234` na primeira chamada,
//! enquanto o cycle-stepped passa por `$0101`, `$0102` e `$0103` antes. É por
//! isso que `each_step_advances_one_m_cycle_not_one_instruction` observa o `PC`
//! **entre** os quatro passos, e não só no fim: no fim os dois coincidem.
//!
//! O que **não** está sendo medido aqui: o resto da máquina. Timer (2.1), PPU
//! (M3) e APU (M6) também deveriam avançar a cada M-cycle, e não existem para
//! avançar. O laço é o lugar onde eles entram, e entram nos seus itens.
//!
//! `unwrap`/`expect` são permitidos aqui: a R6 proíbe fora de teste.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup, Registers};

/// `$0100` — o endereço em que a boot ROM entrega o controle ao cartucho.
const ENTRY: usize = 0x0100;

/// Os opcodes que o SM83 **não tem**, transcritos da coluna `GB CPU` da tabela
/// § Moved, Removed, and Added Opcodes, onde aparecem como `-`. A frase que os
/// qualifica é a última linha da seção:
///
/// > Note: The unused (-) opcodes will lock up the Game Boy CPU when used.
///
/// A tabela de gbops concorda por outro caminho: marca os onze como `unused`
/// com **0 T-cycles** — instrução que não tem timing é instrução que não volta.
///
/// Onze, e não doze: `D9` é `RETI` no Game Boy (era `EXX` no Z80), e `CB 3X` é
/// `SWAP`, não `SLL`. Quem contar os `-` da coluna do Z80 conta outra coisa.
const ILLEGAL_OPCODES: [u8; 11] = [
    0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
];

/// Uma ROM de 32 KiB com `program` em `$0100`. O resto é `$00`, que é `NOP`:
/// uma CPU que escape do programa continua andando em vez de travar por acaso,
/// e travar por acaso pareceria o comportamento que os testes de opcode ilegal
/// procuram.
fn rom_with(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);
    rom
}

/// Uma CPU no hand-off da boot ROM e um barramento com `program` em `$0100`.
///
/// Não é preciso extrair uma interface de memória do `Bus` para isto: um
/// cartucho de teste custa três linhas, e trocar o `Bus` concreto por um trait
/// poria despacho dinâmico no caminho mais quente do emulador (ver a invariante
/// do 1.2a no `STATUS.md`).
fn machine(program: &[u8]) -> (Cpu, Bus) {
    let rom = rom_with(program);
    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

// ---------------------------------------------------------------------------
// O estado inicial
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// A R2: um step() é um M-cycle, não uma instrução
// ---------------------------------------------------------------------------

#[test]
fn each_step_advances_one_m_cycle_not_one_instruction() {
    // `C3 34 12` = `JP $1234`. A tabela dá 4 M-cycles (16 T-cycles):
    // fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?)
    let (mut cpu, mut bus) = machine(&[0xC3, 0x34, 0x12]);

    // M1 — fetch: o opcode chega, o PC passa dele.
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

    // M2 — read(u16:lower).
    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0102, "um byte de operando por M-cycle");
    assert!(!cpu.is_between_instructions(), "faltam dois M-cycles");

    // M3 — read(u16:upper). O alvo já está inteiro dentro da CPU, e mesmo
    // assim o PC ainda é o da instrução seguinte: o desvio é do M4.
    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0103,
        "os dois bytes do operando chegaram, mas o desvio ainda não aconteceu"
    );
    assert!(!cpu.is_between_instructions(), "falta um M-cycle");

    // M4 — internal: é aqui, e só aqui, que o PC vira o alvo.
    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x1234, "o quarto M-cycle é o que desvia");
    assert!(
        cpu.is_between_instructions(),
        "quatro M-cycles depois a instrução acabou"
    );
}

#[test]
fn a_run_of_nops_advances_the_pc_one_byte_per_step() {
    // `00` = `NOP`: 1 M-cycle, e esse M-cycle é o próprio fetch. Oito passos
    // têm de andar oito bytes — nem mais (dois fetches num step) nem menos.
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
    // `JP $0106`, com opcodes ilegais nos bytes pulados: se o laço buscar em
    // $0103 em vez de $0106, a CPU trava e o teste diz qual byte foi lido a
    // mais. Um `assert_eq!` no PC sozinho não distinguiria "pulou certo" de
    // "não pulou e andou três NOPs".
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

// ---------------------------------------------------------------------------
// O que cada instrução decodificada mexe — e o que não mexe
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Opcodes que não existem
// ---------------------------------------------------------------------------

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
    // Controle negativo. Sem ele, "os onze travam" não distingue a lista certa
    // de uma implementação que trave em qualquer opcode ainda não decodificado
    // — que hoje são 243 dos 256.
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
fn an_opcode_this_emulator_has_not_reached_is_not_an_illegal_one() {
    // `04` = `INC B`: existe no SM83 e chega no 1.6. Confundir os dois rótulos
    // faria o `gb-cli` reportar "ROM usa opcode inválido" quando o problema é
    // o emulador estar incompleto — e as duas coisas se consertam em lugares
    // diferentes.
    //
    // Era `$06` (`LD B,u8`) até o 1.4b implementá-lo. Este teste precisa de um
    // opcode que ainda **não** exista aqui, então ele muda de exemplo a cada
    // vez que o exemplo anterior fica pronto — e o dia em que não houver mais
    // nenhum é o dia em que ele sai. `INC B` é o vizinho de `$06` na tabela
    // (`00 ddd 100`, contra `00 ddd 110`), o que o torna também um guarda da
    // máscara do 1.4b: uma máscara frouxa nos bits 2-0 engoliria os dois.
    let (mut cpu, mut bus) = machine(&[0x04, 0x42]);

    cpu.step(&mut bus);

    assert_eq!(
        cpu.lockup(),
        Some(Lockup::UndecodedOpcode(0x04)),
        "`INC B` não é opcode inexistente: é opcode que ainda não foi feito"
    );
}

#[test]
fn locking_up_happens_after_the_opcode_byte_was_fetched() {
    // Escolha deste emulador, não spec: a § Moved, Removed, and Added Opcodes
    // diz que o opcode trava a CPU e não diz nada sobre onde o PC para. O
    // $0101 daqui é consequência do modelo de fetch — o byte é lido e o PC
    // passa por ele antes de haver o que decodificar —, e nada observável
    // depende disso, porque a CPU não volta a andar. O teste existe para
    // nomear a escolha, de modo que mudá-la seja uma decisão e não um efeito.
    let (mut cpu, mut bus) = machine(&[0xD3]);

    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.pc, 0x0101,
        "o opcode foi buscado como qualquer outro; o que não houve foi a \
         instrução"
    );
}
