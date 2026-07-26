//! O laço de M-cycles (ROADMAP 1.3): fetch/decode/execute como máquina de
//! estados, um M-cycle por chamada.
//!
//! Spec: `docs/reference/03-opcodes.md`, coluna *M-cycles (passo a passo)*
//! (gbops `90b9bf296aed`), e `docs/reference/02-cpu.md` § Moved, Removed, and
//! Added Opcodes (Pan Docs `fe246067b695`).
//!
//! # O modelo de M-cycle
//!
//! A coluna da tabela lista, para cada opcode, o que o barramento faz em cada
//! M-cycle. O primeiro é sempre `fetch`, e ele **conta**: `NOP` é 1 M-cycle
//! (4 T-cycles) e esse M-cycle é o próprio fetch; `JP u16` é 4, dos quais o
//! fetch é o primeiro. Então:
//!
//! - **M-cycle 1** — lê o byte em `PC`, incrementa `PC`, decodifica.
//! - **M-cycles 2..N** — o passo correspondente da coluna, um por chamada.
//!
//! Isto é a contabilidade de gbops, e não uma afirmação sobre o pipeline do
//! silício: no hardware o último M-cycle de uma instrução se sobrepõe ao fetch
//! da seguinte. Os dois modelos dão o mesmo número de M-cycles por instrução, e
//! é o número que a suíte Mooneye cobra.
//!
//! # Por que ainda não há tabela de micro-operações
//!
//! Um `enum MicroOp { ReadImmediate, Internal, … }` seria o desenho natural
//! para os opcodes que faltam — e é código escrito por antecipação, que a
//! nota 8 do `STATUS.md` registra como o erro mais reincidente do projeto:
//! passa verde por vacuidade até algo real exercitá-lo.
//!
//! O 1.4a acrescentou 63 opcodes e **três** formas de M-cycle, todas da mesma
//! família: `fetch`, e depois no máximo um acesso ao barramento. Generalizar
//! daí seria generalizar de um terço dos dados — as outras formas do `x8/lsm`
//! (operando imediato no 1.4b, efeito colateral sobre `HL` no 1.4c, endereço
//! de 16 bits no 1.4d) ainda não existem para contradizer o desenho. A
//! decisão está marcada no ROADMAP, no 1.4d.
//!
//! O que **nasceu** aqui é menor e vem direto da spec: [`R8`], o operando de
//! três bits da § Block 1. Isso não é antecipação — é a codificação que a
//! tabela dá, e os 63 opcodes a exercitam inteira.
//!
//! # Onde entra o resto da máquina
//!
//! A R2 diz que o barramento, o timer, a PPU e a APU avançam junto, no meio da
//! instrução. Nenhum deles existe (2.1, M3, M6), e por isso [`Cpu::step`] só
//! avança a CPU. O lugar onde eles vão ser tiquetaqueados é este, uma vez por
//! chamada — e é de propósito que [`crate::bus::Bus::read`] não tiqueta por
//! conta própria: quem posiciona o acesso *dentro* da instrução é o laço.

use crate::bus::Bus;
use crate::cart::HeaderChecksum;
use crate::cpu::Registers;

/// `$00` — `NOP`. 1 M-cycle: `fetch`.
const NOP: u8 = 0x00;

/// `$C3` — `JP u16`. 4 M-cycles:
/// `fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?)`.
const JP_U16: u8 = 0xC3;

/// `$40`–`$7F` — o bloco `LD r8,r8` (ROADMAP 1.4a).
///
/// `docs/reference/02-cpu.md`, § Block 1: 8-bit register-to-register loads:
///
/// ```text
/// Bits | Campo
///    7 | 0
///    6 | 1
///  5-3 | Dest (r8)
///  2-0 | Source (r8)
/// ```
///
/// Sessenta e quatro combinações, das quais 63 são load — a 64ª é [`HALT`].
/// Os dois extremos são `0b01_000_000` e `0b01_111_111`.
const LD_R8_R8_FIRST: u8 = 0x40;
/// Ver [`LD_R8_R8_FIRST`].
const LD_R8_R8_LAST: u8 = 0x7F;

/// `$76` — `HALT`, e **não** `LD (HL),(HL)`.
///
/// A § Block 1 chama isto de exceção, com todas as letras:
///
/// > **Exception**: trying to encode `ld [hl], [hl]` instead yields the `halt`
/// > instruction
///
/// É o único buraco de um bloco que fora dele é perfeitamente regular, e um
/// decodificador escrito direto dos bits não o vê: `0b01_110_110` é destino 6,
/// fonte 6, e a fórmula responde "leia `(HL)` e escreva em `(HL)`" — uma
/// instrução sem efeito observável, que não trava, e que faria qualquer ROM
/// que use `HALT` para esperar uma interrupção girar para sempre.
///
/// `HALT` é o ROADMAP 2.3 (com o bug do `HALT` junto), então até lá ele para a
/// CPU com [`Lockup::UndecodedOpcode`] — o rótulo que diz "falta implementar",
/// como qualquer outro opcode legítimo que ainda não chegou.
const HALT: u8 = 0x76;

/// Por que a CPU parou de buscar instruções.
///
/// As duas variantes têm o mesmo efeito — a CPU não anda mais — e origens
/// opostas, e é por isso que são duas. Uma é o hardware; a outra é este
/// emulador. Quem lê um relatório do `gb-cli` conserta cada uma num lugar
/// diferente: a primeira significa "a ROM executou lixo", a segunda significa
/// "falta implementar".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lockup {
    /// Opcode que o SM83 não tem. A § Moved, Removed, and Added Opcodes lista
    /// onze deles como `-` na coluna `GB CPU` e fecha a seção com:
    ///
    /// > Note: The unused (-) opcodes will lock up the Game Boy CPU when used.
    ///
    /// A tabela de gbops concorda por outro caminho: marca os onze como
    /// `unused` com **0 T-cycles**. Instrução sem timing é instrução que não
    /// termina. Não são `NOP`.
    IllegalOpcode(u8),
    /// Opcode legítimo do SM83 que este emulador ainda não decodifica — hoje
    /// **180** dos 245 que existem, e eles chegam nos itens 1.4b a 1.11 do
    /// ROADMAP. Um deles é o [`HALT`], que só é "não decodificado" porque o
    /// 2.3 ainda não chegou; os outros 179 nunca foram tentados.
    ///
    /// Parar aqui, em vez de entrar em pânico, mantém o `gb-core` como máquina
    /// de estados: quem decide o que fazer com uma CPU parada é quem a roda.
    UndecodedOpcode(u8),
}

/// Um dos **sete** registradores de 8 bits alcançáveis como operando.
///
/// Sete, e não oito: o oitavo valor de [`R8`] é a memória, não um registrador.
/// Manter os dois em tipos separados é o que faz o `match` de [`Cpu::fetch`]
/// distinguir as três formas de M-cycle sem um braço impossível — e o que
/// permite a [`State::LoadFromHl`] carregar um destino que **é** registrador,
/// sem `unreachable!` (a R6 não quer pânico no `gb-core`).
///
/// `F` não está aqui, e a ausência é da spec: a lista `r8` é
/// `b c d e h l [hl] a`, sem `f`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ByteRegister {
    B,
    C,
    D,
    E,
    H,
    L,
    A,
}

/// O operando `r8` da spec: três bits, oito valores.
///
/// `docs/reference/02-cpu.md`, § CPU Instruction Set, tabela de placeholders:
///
/// ```text
/// 0 | b      4 | h
/// 1 | c      5 | l
/// 2 | d      6 | [hl]
/// 3 | e      7 | a
/// ```
///
/// O índice 6 é o que dá as três formas de M-cycle do bloco: sem ele a
/// instrução é `fetch` e acabou; com ele no lugar da fonte há uma leitura no
/// M2; com ele no lugar do destino há uma escrita no M2.
///
/// **A tabela dessa seção está corrompida na conversão** (nota 24 do
/// `STATUS.md`): ela emenda, sem cabeçalho, os placeholders `r8`, `r16`,
/// `r16stk`, `r16mem` e `cond` numa lista só, com os índices 0–3 repetidos
/// quatro vezes. Os oito valores acima são o primeiro bloco, e quem confirma
/// que a leitura é essa é a tabela de gbops em `03-opcodes.md`, que enumera os
/// 63 opcodes um a um — e a própria § Block 1, cuja exceção
/// (`ld [hl], [hl]` → `halt`) só faz sentido com `[hl]` no índice 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum R8 {
    /// Índices 0–5 e 7.
    Register(ByteRegister),
    /// Índice 6 — a memória apontada por `HL`, não um registrador.
    MemoryAtHl,
}

impl R8 {
    /// Decodifica os três bits de um campo `r8`.
    ///
    /// O `_` do último braço é `0b111` e só ele: a máscara deixa oito valores,
    /// sete estão nomeados acima. Um `match` sobre `u8` não tem como ser
    /// exaustivo sem ele, e a alternativa seria um `unreachable!` — pânico que
    /// a R6 não quer no `gb-core`.
    const fn from_bits(bits: u8) -> Self {
        match bits & 0b111 {
            0 => Self::Register(ByteRegister::B),
            1 => Self::Register(ByteRegister::C),
            2 => Self::Register(ByteRegister::D),
            3 => Self::Register(ByteRegister::E),
            4 => Self::Register(ByteRegister::H),
            5 => Self::Register(ByteRegister::L),
            6 => Self::MemoryAtHl,
            _ => Self::Register(ByteRegister::A),
        }
    }
}

/// Em que ponto de qual instrução a CPU está.
///
/// O `match` de [`Cpu::step`] sobre este enum é total **sem** `_ =>`: estado
/// novo tem de quebrar a compilação, pela mesma razão que
/// [`crate::bus::Region::of`] não tem braço genérico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Entre instruções: o próximo M-cycle busca o opcode em `PC`.
    Fetch,
    /// Dentro de `JP u16`. O `fetch` já aconteceu — foi ele que criou este
    /// estado —, então o que resta são os três M-cycles de [`JumpImmediate`].
    JumpImmediate(JumpImmediate),
    /// Dentro de `LD r,(HL)`. O M2 é `read((HL)->r)`: a leitura no barramento
    /// e a escrita no registrador acontecem no **mesmo** M-cycle, e o
    /// registrador que vai receber o byte viaja aqui.
    ///
    /// Não há um terceiro M-cycle onde a escrita "aconteça de verdade". A
    /// tentação de pôr um vem do `JP u16`, cujo desvio é do M4 e não do M3 —
    /// mas ali a coluna tem quatro passos e aqui tem dois, e os 8 T-cycles não
    /// deixam espaço para inventar o terceiro.
    LoadFromHl(ByteRegister),
    /// Dentro de `LD (HL),r`. O M2 é `write(r->(HL))`.
    StoreToHl(ByteRegister),
    /// A CPU parou, e não volta.
    Locked(Lockup),
}

/// Os três M-cycles de `JP u16` que vêm depois do `fetch`.
///
/// Os nomes são a coluna da tabela, traduzida um para um. O que a intuição erra
/// aqui é o último: depois de `read(u16:upper)` o endereço de destino já está
/// inteiro dentro da CPU, e ainda assim falta um M-cycle antes de o `PC` mudar.
/// Escrever o `PC` junto com o byte alto dá as mesmas 4 M-cycles no total e
/// desloca o desvio em um — invisível em teste de instrução isolada, visível
/// para o timer e a PPU, que é o que a suíte Mooneye mede.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JumpImmediate {
    /// M2 — `read(u16:lower)`.
    ReadLowByte,
    /// M3 — `read(u16:upper)`.
    ReadHighByte,
    /// M4 — `internal(branch decision?)`.
    SetProgramCounter,
}

/// A CPU SM83 e o ponto em que ela está dentro da instrução corrente.
///
/// Não é dona do [`Bus`]: recebe `&mut Bus` a cada M-cycle, como o `CLAUDE.md`
/// § Arquitetura manda. Quem possui os dois é quem roda a máquina — hoje os
/// testes, e o `gb-cli run` a partir do 1.12.
#[derive(Debug, Clone)]
pub struct Cpu {
    /// O banco de registradores. Público pelo mesmo motivo que os campos de
    /// [`Registers`] são: `regs.a = value` é o que a instrução faz.
    pub registers: Registers,
    /// Onde a CPU está dentro da instrução. **Não** é público: é o invariante
    /// que a R2 protege, e escrever nele de fora seria pular M-cycles.
    state: State,
    /// Os bytes do operando que já chegaram do fluxo de instruções e que um
    /// M-cycle posterior vai consumir.
    ///
    /// Não é registrador que a spec nomeie — é estado interno do laço, e por
    /// isso não aparece em [`Registers`]. Um `JP u16` guarda aqui o byte baixo
    /// no M2 e o alto no M3 para usá-los no M4.
    latch: u16,
}

impl Cpu {
    /// A CPU no estado em que a boot ROM do DMG entrega o controle ao cartucho.
    ///
    /// Os registradores são os do 1.2b-i — a tabela mora lá, e esta função não
    /// tem uma segunda cópia dela. O que o 1.3 acrescenta é o resto do estado:
    /// a máquina começa **entre instruções**, com o próximo M-cycle buscando o
    /// opcode em `PC = $0100`.
    ///
    /// O parâmetro é [`HeaderChecksum`] porque `F` depende dele; o porquê inteiro
    /// está em [`Registers::after_boot_rom`].
    #[must_use]
    pub const fn after_boot_rom(checksum: HeaderChecksum) -> Self {
        Self {
            registers: Registers::after_boot_rom(checksum),
            state: State::Fetch,
            latch: 0,
        }
    }

    /// Avança **um** M-cycle.
    ///
    /// Esta é a R2 do `CLAUDE.md` inteira, e a assinatura é metade dela: não há
    /// contagem de ciclos a devolver porque a resposta é sempre um, e não há
    /// laço interno até a instrução acabar porque parar no meio é o objetivo.
    ///
    /// Uma chamada faz **no máximo um** acesso ao barramento: os M-cycles
    /// `internal` da tabela não fazem nenhum.
    pub fn step(&mut self, bus: &mut Bus) {
        self.state = match self.state {
            State::Fetch => self.fetch(bus),
            State::JumpImmediate(phase) => self.jump_immediate(bus, phase),
            State::LoadFromHl(dest) => self.load_from_hl(bus, dest),
            State::StoreToHl(source) => self.store_to_hl(bus, source),
            // O tempo passa; a CPU não. Ver [`Lockup`].
            State::Locked(lockup) => State::Locked(lockup),
        };
    }

    /// Por que a CPU parou, se parou.
    #[must_use]
    pub const fn lockup(&self) -> Option<Lockup> {
        match self.state {
            State::Locked(lockup) => Some(lockup),
            State::Fetch | State::JumpImmediate(_) | State::LoadFromHl(_) | State::StoreToHl(_) => {
                None
            }
        }
    }

    /// `true` quando a instrução anterior acabou e a próxima ainda não começou.
    ///
    /// Uma CPU travada **não** está entre instruções: não há próxima.
    #[must_use]
    pub const fn is_between_instructions(&self) -> bool {
        matches!(self.state, State::Fetch)
    }

    /// M-cycle 1 de qualquer instrução: o opcode chega e é decodificado.
    fn fetch(&mut self, bus: &Bus) -> State {
        let opcode = self.read_at_pc(bus);

        match opcode {
            // Um M-cycle, que é este; nada a fazer e nada a lembrar.
            NOP => State::Fetch,
            JP_U16 => State::JumpImmediate(JumpImmediate::ReadLowByte),
            // A exceção **antes** do bloco, e a ordem é o ponto: `$76` cai
            // dentro da faixa abaixo e não é um load. Ver [`HALT`].
            HALT => State::Locked(Lockup::UndecodedOpcode(opcode)),
            LD_R8_R8_FIRST..=LD_R8_R8_LAST => self.load_r8_r8(opcode),
            // Os onze `-` da coluna `GB CPU`. Ver [`Lockup::IllegalOpcode`].
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                State::Locked(Lockup::IllegalOpcode(opcode))
            }
            _ => State::Locked(Lockup::UndecodedOpcode(opcode)),
        }
    }

    /// Os M-cycles 2 a 4 de `JP u16`.
    fn jump_immediate(&mut self, bus: &Bus, phase: JumpImmediate) -> State {
        match phase {
            JumpImmediate::ReadLowByte => {
                self.latch = u16::from(self.read_at_pc(bus));
                State::JumpImmediate(JumpImmediate::ReadHighByte)
            }
            JumpImmediate::ReadHighByte => {
                self.latch |= u16::from(self.read_at_pc(bus)) << 8;
                State::JumpImmediate(JumpImmediate::SetProgramCounter)
            }
            // `internal`: nenhum acesso ao barramento, e é só agora que o
            // desvio acontece.
            JumpImmediate::SetProgramCounter => {
                self.registers.pc = self.latch;
                State::Fetch
            }
        }
    }

    /// Decodifica um opcode do bloco `01 ddd sss` e faz o que couber no M1.
    ///
    /// As três formas da tabela de gbops saem das três combinações possíveis
    /// dos dois campos, e a quarta — os dois em `[hl]` — não chega aqui: é o
    /// [`HALT`], desviado antes.
    ///
    /// | Dest | Source | T-cycles | Coluna |
    /// |---|---|---|---|
    /// | reg | reg | 4 | `fetch` |
    /// | reg | `[hl]` | 8 | `fetch → read((HL)->r)` |
    /// | `[hl]` | reg | 8 | `fetch → write(r->(HL))` |
    ///
    /// A primeira acaba **aqui**, dentro do fetch: a coluna tem um passo só, e
    /// mover um byte entre dois registradores não toca o barramento.
    fn load_r8_r8(&mut self, opcode: u8) -> State {
        match (R8::from_bits(opcode >> 3), R8::from_bits(opcode)) {
            (R8::Register(dest), R8::Register(source)) => {
                let value = self.read_r8(source);
                self.write_r8(dest, value);
                State::Fetch
            }
            (R8::Register(dest), R8::MemoryAtHl) => State::LoadFromHl(dest),
            (R8::MemoryAtHl, R8::Register(source)) => State::StoreToHl(source),
            // `$76`, que [`HALT`] já desviou. O braço existe para que o
            // `match` seja total sem `_ =>`, como o do [`State`].
            (R8::MemoryAtHl, R8::MemoryAtHl) => State::Locked(Lockup::UndecodedOpcode(opcode)),
        }
    }

    /// M2 de `LD r,(HL)`: `read((HL)->r)`.
    ///
    /// Um acesso ao barramento e a escrita no registrador, no mesmo M-cycle.
    /// O endereço é o `HL` de agora — nada neste bloco mexe em `HL`, e por
    /// isso `LD H,(HL)` lê de onde `HL` apontava antes de `H` mudar.
    fn load_from_hl(&mut self, bus: &Bus, dest: ByteRegister) -> State {
        let value = bus.read(self.registers.hl());
        self.write_r8(dest, value);
        State::Fetch
    }

    /// M2 de `LD (HL),r`: `write(r->(HL))`.
    fn store_to_hl(&mut self, bus: &mut Bus, source: ByteRegister) -> State {
        bus.write(self.registers.hl(), self.read_r8(source));
        State::Fetch
    }

    /// O valor de um dos sete registradores de 8 bits.
    ///
    /// Este mapeamento mora no decodificador, e não em [`Registers`], de
    /// propósito: o 1.1 decidiu que o banco de registradores tem campos
    /// públicos e nenhum acessor por registrador, porque quem precisa de um
    /// nome de três bits para um campo é quem decodifica opcode.
    const fn read_r8(&self, which: ByteRegister) -> u8 {
        match which {
            ByteRegister::B => self.registers.b,
            ByteRegister::C => self.registers.c,
            ByteRegister::D => self.registers.d,
            ByteRegister::E => self.registers.e,
            ByteRegister::H => self.registers.h,
            ByteRegister::L => self.registers.l,
            ByteRegister::A => self.registers.a,
        }
    }

    /// Escreve num dos sete registradores de 8 bits. Ver [`Cpu::read_r8`].
    const fn write_r8(&mut self, which: ByteRegister, value: u8) {
        match which {
            ByteRegister::B => self.registers.b = value,
            ByteRegister::C => self.registers.c = value,
            ByteRegister::D => self.registers.d = value,
            ByteRegister::E => self.registers.e = value,
            ByteRegister::H => self.registers.h = value,
            ByteRegister::L => self.registers.l = value,
            ByteRegister::A => self.registers.a = value,
        }
    }

    /// Lê o byte em `PC` e passa o `PC` por ele. Um M-cycle de barramento.
    ///
    /// `wrapping_add` porque o `PC` é de 16 bits e dá a volta: uma instrução
    /// que comece em `$FFFF` continua em `$0000`. Isso não é caso de erro, é a
    /// aritmética do registrador.
    fn read_at_pc(&mut self, bus: &Bus) -> u8 {
        let byte = bus.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        byte
    }
}
