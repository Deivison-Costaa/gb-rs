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
//! # Por que não há tabela de micro-operações
//!
//! Um `enum MicroOp { ReadImmediate, Internal, … }` seria o desenho natural
//! para os 245 opcodes que faltam — e é código escrito por antecipação, que a
//! nota 8 do `STATUS.md` registra como o erro mais reincidente do projeto:
//! passa verde por vacuidade até algo real exercitá-lo. Com duas instruções
//! decodificadas não há de onde generalizar. Os estados abaixo descrevem
//! exatamente as instruções que existem hoje, o `match` é total sem `_ =>`, e
//! quem acrescentar um opcode no 1.4 vai **precisar** mexer aqui — que é o
//! momento em que a generalização terá três casos para aprender.
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
    /// 243 dos 256, e eles chegam nos itens 1.4 a 1.11 do ROADMAP.
    ///
    /// Parar aqui, em vez de entrar em pânico, mantém o `gb-core` como máquina
    /// de estados: quem decide o que fazer com uma CPU parada é quem a roda.
    UndecodedOpcode(u8),
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
            // O tempo passa; a CPU não. Ver [`Lockup`].
            State::Locked(lockup) => State::Locked(lockup),
        };
    }

    /// Por que a CPU parou, se parou.
    #[must_use]
    pub const fn lockup(&self) -> Option<Lockup> {
        match self.state {
            State::Locked(lockup) => Some(lockup),
            State::Fetch | State::JumpImmediate(_) => None,
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
