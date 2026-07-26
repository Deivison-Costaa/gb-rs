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
//! família: `fetch`, e depois no máximo um acesso ao barramento. O 1.4b
//! acrescentou mais oito e a **primeira** forma com dois acessos —
//! `LD (HL),u8`, que lê o operando num M-cycle e escreve no seguinte. O 1.4c
//! acrescentou outros oito e nenhuma forma nova: as oito linhas são `fetch` mais
//! um acesso, como o 1.4a. O que ele trouxe foi um **efeito colateral** dentro
//! do passo do acesso (`write(A->(HL++))`), que é dimensão diferente de forma.
//!
//! Três sub-itens, e o que se repete é a família do 1.4a. O que falta para
//! decidir o desenho são as formas do 1.4d (endereço absoluto de 16 bits e a
//! página `$FF00`), que trazem o primeiro operando de dois bytes desde o
//! `JP u16`; generalizar antes delas seria a nota 8 com três quartos dos dados.
//! A decisão está marcada no ROADMAP, no 1.4d.
//!
//! O que **nasceu** aqui é menor e vem direto da spec: [`R8`], o operando de
//! três bits da § Block 1, e [`R16Mem`], o de dois bits da § Block 0. Isso não é
//! antecipação — é a codificação que a tabela dá, e os opcodes de cada sub-item
//! exercitam a sua inteira.
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
    /// **164** dos 245 que existem, e eles chegam nos itens 1.4d a 1.11 do
    /// ROADMAP. Um deles é o [`HALT`], que só é "não decodificado" porque o
    /// 2.3 ainda não chegou; os outros 163 nunca foram tentados.
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

/// O bloco `LD r8,u8` — `$06 $0E $16 $1E $26 $2E $36 $3E` (ROADMAP 1.4b).
///
/// `docs/reference/02-cpu.md`, § Block 0, layout de bits de `ld r8, imm8`:
///
/// ```text
/// Bits | Campo
///    7 | 0
///    6 | 0
///  5-3 | Dest (r8)
///    2 | 1
///    1 | 1
///    0 | 0
/// ```
///
/// O destino é o mesmo campo [`R8`] do 1.4a, nos mesmos bits 5-3. Oito
/// combinações, oito opcodes — e aqui, ao contrário da § Block 1, **não há
/// exceção**: o índice 6 dá `LD (HL),u8`, que é load como os outros sete.
///
/// Os opcodes não são contíguos (andam de 8 em 8), então o reconhecimento é por
/// máscara e não por faixa: apagar os bits do destino tem de deixar
/// [`LD_R8_U8_PATTERN`]. Os vizinhos que a máscara **não** pode deixar entrar
/// são `INC r8` (`00 ddd 100`) e `DEC r8` (`00 ddd 101`), que compartilham os
/// bits 7-6 e diferem só nos bits 2-0 — e que são o 1.6.
const LD_R8_U8_MASK: u8 = 0b1100_0111;
/// Ver [`LD_R8_U8_MASK`].
const LD_R8_U8_PATTERN: u8 = 0b0000_0110;

/// `$02 $0A $12 $1A $22 $2A $32 $3A` — o indireto por par de registradores
/// (ROADMAP 1.4c).
///
/// `docs/reference/02-cpu.md`, § Block 0, layouts de bits de `ld [r16mem], a` e
/// `ld a, [r16mem]`. As duas codificações diferem num bit só — o 3:
///
/// ```text
/// Bits | Campo                 Bits | Campo
///    7 | 0                        7 | 0
///    6 | 0                        6 | 0
///  5-4 | Dest (r16mem)          5-4 | Source (r16mem)
///    3 | 0                        3 | 1
///    2 | 0                        2 | 0
///    1 | 1                        1 | 1
///    0 | 0                        0 | 0
/// ```
///
/// O outro operando não aparece no opcode porque não varia: é sempre `A`.
///
/// A máscara tem de deixar passar os bits 5-4 (o par) e **prender** o bit 3 (a
/// direção), porque é ele que separa as duas famílias. Os vizinhos que ela não
/// pode engolir estão todos a um bit de distância: `LD r16,u16` (`00 mm 0001`,
/// o 1.5), `INC r16`/`DEC r16` (`00 mm 0011` e `00 mm 1011`, o 1.7) e
/// `INC r8`/`DEC r8` (`00 ddd 100`/`101`, o 1.6).
const LD_R16MEM_MASK: u8 = 0b1100_1111;
/// `LD (r16mem),A` — ver [`LD_R16MEM_MASK`].
const STORE_R16MEM_PATTERN: u8 = 0b0000_0010;
/// `LD A,(r16mem)` — ver [`LD_R16MEM_MASK`].
const LOAD_R16MEM_PATTERN: u8 = 0b0000_1010;

/// O operando `r16mem` da spec: dois bits, quatro valores.
///
/// `docs/reference/02-cpu.md`, § CPU Instruction Set, tabela de placeholders:
///
/// ```text
/// 0 | bc     2 | hl+
/// 1 | de     3 | hl-
/// ```
///
/// É o **terceiro** dos cinco blocos que a nota 24 do `STATUS.md` registra como
/// emendados numa tabela só pela conversão para Markdown, sem cabeçalho e com os
/// índices 0–3 repetidos quatro vezes. Não é o `r16` (`bc de hl sp`, do 1.5) nem
/// o `r16stk` (`bc de hl af`, do PUSH/POP): quem confirma que a leitura é esta é
/// a tabela de gbops, que enumera os oito opcodes um a um — `$22` é
/// `LD (HL+),A` e `$32` é `LD (HL-),A`.
///
/// Os índices 2 e 3 são o conceito novo do sub-item: o único operando do projeto
/// até aqui que **modifica** o registrador que usou como endereço.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum R16Mem {
    /// Índice 0 — `bc`, sem efeito colateral.
    Bc,
    /// Índice 1 — `de`, sem efeito colateral.
    De,
    /// Índice 2 — `hl+`: o acesso é em `HL`, e `HL` sobe um depois.
    HlIncrement,
    /// Índice 3 — `hl-`: o acesso é em `HL`, e `HL` desce um depois.
    HlDecrement,
}

impl R16Mem {
    /// Decodifica os dois bits de um campo `r16mem`, que moram nos bits 5-4.
    ///
    /// O `_` do último braço é `0b11` e só ele, pelo mesmo motivo de
    /// [`R8::from_bits`]: `match` sobre `u8` não é exaustivo sem ele, e a
    /// alternativa seria um `unreachable!` que a R6 não quer no `gb-core`.
    const fn from_opcode(opcode: u8) -> Self {
        match (opcode >> 4) & 0b11 {
            0 => Self::Bc,
            1 => Self::De,
            2 => Self::HlIncrement,
            _ => Self::HlDecrement,
        }
    }
}

/// `$E2` — `LD (FF00+C),A`. 1 byte, 2 M-cycles: `fetch → write(A->(FF00+C))`.
///
/// **Um** byte, e é onde a memória deste agente estava errada (0017): tabelas de
/// opcode que circulam há décadas listam `LDH (C),A` como instrução de dois
/// bytes, com um operando que não existe. O registrador `C` **é** o operando, e
/// ele já está na CPU — não há byte a buscar no fluxo de instruções.
///
/// A coluna `Bytes` de gbops diz `1`. Os 8 T-cycles não desempatam: eles fixam o
/// número de M-cycles, não o de bytes, e um `PC` que ande dois deixa os dois
/// M-cycles no lugar e o efeito na memória certo. O que quebra é a instrução
/// **seguinte**, que passa a ser buscada um byte adiante.
const LDH_C_A: u8 = 0xE2;
/// `$F2` — `LD A,(FF00+C)`. 1 byte, 2 M-cycles: `fetch → read((FF00+C)->A)`.
/// Ver [`LDH_C_A`] para o byte que não existe.
const LDH_A_C: u8 = 0xF2;
/// `$E0` — `LD (FF00+u8),A`. 2 bytes, 3 M-cycles:
/// `fetch → read(u8) → write(A->(FF00+u8))`.
const LDH_IMM8_A: u8 = 0xE0;
/// `$F0` — `LD A,(FF00+u8)`. 2 bytes, 3 M-cycles:
/// `fetch → read(u8) → read((FF00+u8)->A)`.
const LDH_A_IMM8: u8 = 0xF0;
/// `$EA` — `LD (u16),A`. 3 bytes, 4 M-cycles:
/// `fetch → read(u16:lower) → read(u16:upper) → write(A->(u16))`.
const LD_IMM16_A: u8 = 0xEA;
/// `$FA` — `LD A,(u16)`. 3 bytes, 4 M-cycles:
/// `fetch → read(u16:lower) → read(u16:upper) → read((u16)->A)`.
const LD_A_IMM16: u8 = 0xFA;

/// A base da página alta: `$FF00`, o começo da faixa de I/O.
///
/// A § Comparison with the Z80 diz por que estes opcodes existem: *"Unlike the
/// 8080 and Z80, the Game Boy has no dedicated I/O bus and no IN/OUT opcodes.
/// Instead, I/O ports are accessed directly by normal LD instructions, or by new
/// LD (FF00+n) opcodes."* A página não é espaço de endereçamento à parte — é o
/// fim do mapa de memória, e estes quatro opcodes só encurtam o caminho.
///
/// O `|` do cálculo do endereço podia ser `+` sem diferença: o deslocamento tem
/// 8 bits e a base tem os 8 bits baixos zerados, então a soma nunca estoura e a
/// página tem exatamente tantos endereços quantos o operando tem valores.
const HIGH_PAGE: u16 = 0xFF00;

/// Em que lado do acesso está `A` — o bit 4 do opcode (ROADMAP 1.4d).
///
/// Os três sub-itens anteriores do 1.4 eram blocos de bits com um campo
/// variável; este é **três pares**, e dentro de cada par o bit 4 escolhe a
/// direção: `$Ex` escreve, `$Fx` lê. Fora dos pares o bit 4 não significa nada,
/// e é por isso que o decodificador reconhece os seis opcode a opcode, e não por
/// máscara — qualquer máscara frouxa o bastante para pegar `$E0`, `$E2` e `$EA`
/// de uma vez leva junto `$E8` (`ADD SP,i8`) e `$F8` (`LD HL,SP+i8`), que são o
/// 1.7.
///
/// O outro operando não aparece no opcode porque não varia: é sempre `A`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    /// `$Ex` — `A` vai para a memória.
    Store,
    /// `$Fx` — a memória vai para `A`.
    Load,
}

/// Os dois M-cycles de `$E0`/`$F0` que vêm depois do `fetch`.
///
/// Mesma forma do [`StoreImmediateToHl`] do 1.4b — um M-cycle para o operando,
/// outro para o acesso —, e o mesmo erro à espreita: juntar os dois no M2 e
/// gastar o M3 num `internal` dá os mesmos 12 T-cycles e o mesmo estado final, e
/// adianta o acesso em um M-cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HighPageImmediate {
    /// M2 — `read(u8)`. O deslocamento chega e vira endereço no latch; a página
    /// alta não é tocada.
    ReadOffset,
    /// M3 — o acesso, na direção que o bit 4 escolheu.
    Access,
}

/// Os três M-cycles de `$EA`/`$FA` que vêm depois do `fetch`.
///
/// É o primeiro operando de dois bytes desde o `JP u16`, e o parecido acaba no
/// M4: lá o quarto passo é `internal(branch decision?)` e aqui é o **acesso**.
/// Escrever (ou ler) junto com o byte alto e gastar o M4 num `internal` dá os
/// mesmos 16 T-cycles e o mesmo estado final — foi o segundo erro do esqueleto
/// da 0017, e dois dos dezenove testes o reprovaram.
///
/// A regra que gera esses dois enganos é sempre a mesma e não existe: **a coluna
/// é por instrução**. Ver as notas 26, 30 e 32 do `STATUS.md`, que são três
/// iterações seguidas errando *em qual* M-cycle o efeito cai.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Absolute {
    /// M2 — `read(u16:lower)`.
    ReadLowByte,
    /// M3 — `read(u16:upper)`. Depois deste passo o endereço está inteiro dentro
    /// da CPU, e ainda falta um M-cycle antes de a memória ser tocada.
    ReadHighByte,
    /// M4 — o acesso, na direção que o bit 4 escolheu.
    Access,
}

/// Os dois M-cycles de `LD (HL),u8` (`$36`) que vêm depois do `fetch`.
///
/// A coluna de gbops é `fetch → read(u8) → write((HL))`: **três** M-cycles, 12
/// T-cycles, e o operando e a escrita são passos diferentes. O byte chega do
/// fluxo de instruções no M2 e só vai para a memória no M3.
///
/// Juntar os dois no M2 e gastar o M3 num `internal` dá os mesmos 12 T-cycles,
/// o mesmo estado final, e adianta a escrita em um M-cycle — para o timer e a
/// PPU, que é o que a suíte Mooneye mede, não é a mesma instrução. Foi o
/// esqueleto desta iteração, e **um** teste da suíte o reprovou: os outros
/// nove passam contra ele, porque só olham o começo e o fim.
///
/// A tentação vem de generalizar o 1.4a, onde `LD r,(HL)` faz a leitura e a
/// escrita no mesmo M2 e não tem terceiro M-cycle. Lá a coluna tem dois passos;
/// aqui tem três. **A coluna é por instrução** — ver a nota 26 do `STATUS.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreImmediateToHl {
    /// M2 — `read(u8)`. O byte fica no latch; a memória não é tocada.
    ReadImmediate,
    /// M3 — `write((HL))`.
    Write,
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
    /// Dentro de `LD r,u8`. O M2 é `read(u8->r)`: o byte é buscado no fluxo de
    /// instruções e escrito no registrador no **mesmo** M-cycle, como no
    /// `LD r,(HL)` do 1.4a. A diferença é de onde ele vem — de `PC`, que anda,
    /// e não de `HL`, que não.
    LoadImmediate(ByteRegister),
    /// Dentro de `LD (HL),u8`. Ver [`StoreImmediateToHl`], onde está a única
    /// decisão de timing deste sub-item.
    StoreImmediateToHl(StoreImmediateToHl),
    /// Dentro de `LD A,(r16mem)`. O M2 é `read((rr)->A)`, e para os pares
    /// `hl+`/`hl-` é **também** onde `HL` se move: a coluna escreve o efeito
    /// dentro do passo do acesso (`read((HL++)->A)`), não num passo à parte.
    ///
    /// O par viaja aqui em vez de o endereço já resolvido justamente por isso.
    /// Guardar o endereço no `latch` no M1 daria o mesmo estado final e os
    /// mesmos 8 T-cycles, e adiantaria o `HL±` em um M-cycle.
    LoadFromR16Mem(R16Mem),
    /// Dentro de `LD (r16mem),A`. O M2 é `write(A->(rr))`. Ver
    /// [`State::LoadFromR16Mem`] para o porquê de o par viajar aqui.
    StoreToR16Mem(R16Mem),
    /// Dentro de `LD (FF00+C),A` / `LD A,(FF00+C)`. O M2 é o acesso, e o
    /// endereço sai de `C` — que já estava na CPU quando o opcode chegou.
    ///
    /// Não há operando no fluxo de instruções: [`LDH_C_A`] tem **um** byte.
    HighPageC(Direction),
    /// Dentro de `LD (FF00+u8),A` / `LD A,(FF00+u8)`. Ver [`HighPageImmediate`].
    HighPageImmediate(Direction, HighPageImmediate),
    /// Dentro de `LD (u16),A` / `LD A,(u16)`. Ver [`Absolute`].
    Absolute(Direction, Absolute),
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
            State::LoadImmediate(dest) => self.load_immediate(bus, dest),
            State::StoreImmediateToHl(phase) => self.store_immediate_to_hl(bus, phase),
            State::LoadFromR16Mem(source) => self.load_from_r16_mem(bus, source),
            State::StoreToR16Mem(dest) => self.store_to_r16_mem(bus, dest),
            State::HighPageC(direction) => self.high_page_c(bus, direction),
            State::HighPageImmediate(direction, phase) => {
                self.high_page_immediate(bus, direction, phase)
            }
            State::Absolute(direction, phase) => self.absolute(bus, direction, phase),
            // O tempo passa; a CPU não. Ver [`Lockup`].
            State::Locked(lockup) => State::Locked(lockup),
        };
    }

    /// Por que a CPU parou, se parou.
    #[must_use]
    pub const fn lockup(&self) -> Option<Lockup> {
        match self.state {
            State::Locked(lockup) => Some(lockup),
            State::Fetch
            | State::JumpImmediate(_)
            | State::LoadFromHl(_)
            | State::StoreToHl(_)
            | State::LoadImmediate(_)
            | State::StoreImmediateToHl(_)
            | State::LoadFromR16Mem(_)
            | State::StoreToR16Mem(_)
            | State::HighPageC(_)
            | State::HighPageImmediate(..)
            | State::Absolute(..) => None,
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
            // O bloco `00 ddd 110` não é contíguo — os oito opcodes andam de 8
            // em 8 —, então é máscara e não faixa. Ver [`LD_R8_U8_MASK`].
            _ if opcode & LD_R8_U8_MASK == LD_R8_U8_PATTERN => Self::load_r8_u8(opcode),
            // As duas famílias do 1.4c. Nada acontece no M1 além do fetch: o
            // endereço vem de um par de registradores, e resolvê-lo aqui
            // adiantaria o `HL±` em um M-cycle. Ver [`State::LoadFromR16Mem`].
            _ if opcode & LD_R16MEM_MASK == STORE_R16MEM_PATTERN => {
                State::StoreToR16Mem(R16Mem::from_opcode(opcode))
            }
            _ if opcode & LD_R16MEM_MASK == LOAD_R16MEM_PATTERN => {
                State::LoadFromR16Mem(R16Mem::from_opcode(opcode))
            }
            // Os seis do 1.4d, um a um. **Não** há máscara aqui, e a ausência é
            // da spec: os seis são três pares, e os bits que eles têm em comum
            // deixariam passar dezesseis opcodes. Ver [`Direction`].
            //
            // Nada acontece no M1 além do fetch, nem mesmo nos dois que já têm o
            // endereço à mão (`$E2`/`$F2`): a coluna põe o acesso no M2, e o
            // fetch é o M1 inteiro.
            LDH_C_A => State::HighPageC(Direction::Store),
            LDH_A_C => State::HighPageC(Direction::Load),
            LDH_IMM8_A => State::HighPageImmediate(Direction::Store, HighPageImmediate::ReadOffset),
            LDH_A_IMM8 => State::HighPageImmediate(Direction::Load, HighPageImmediate::ReadOffset),
            LD_IMM16_A => State::Absolute(Direction::Store, Absolute::ReadLowByte),
            LD_A_IMM16 => State::Absolute(Direction::Load, Absolute::ReadLowByte),
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

    /// Decodifica um opcode do bloco `00 ddd 110`.
    ///
    /// Nada acontece no M1 além do fetch: as duas formas leem o operando, e ele
    /// está no byte seguinte, que é o M2. Por isso a função não toca em `self` —
    /// ela só escolhe qual das duas colunas a instrução vai percorrer.
    ///
    /// | Dest | Bytes | T-cycles | Coluna |
    /// |---|---|---|---|
    /// | reg | 2 | 8 | `fetch → read(u8->r)` |
    /// | `[hl]` | 2 | 12 | `fetch → read(u8) → write((HL))` |
    ///
    /// As duas são de **2 bytes**, e é a única coisa que todo o bloco tem em
    /// comum com o do 1.4a: lá o segundo M-cycle é um acesso a `(HL)` e o `PC`
    /// anda um só.
    const fn load_r8_u8(opcode: u8) -> State {
        match R8::from_bits(opcode >> 3) {
            R8::Register(dest) => State::LoadImmediate(dest),
            R8::MemoryAtHl => State::StoreImmediateToHl(StoreImmediateToHl::ReadImmediate),
        }
    }

    /// M2 de `LD r,u8`: `read(u8->r)`.
    ///
    /// Um acesso ao barramento — o operando, em `PC` — e a escrita no
    /// registrador, no mesmo M-cycle. É [`Cpu::load_from_hl`] com outra fonte de
    /// endereço, e a diferença é que esta faz o `PC` andar.
    fn load_immediate(&mut self, bus: &Bus, dest: ByteRegister) -> State {
        let value = self.read_at_pc(bus);
        self.write_r8(dest, value);
        State::Fetch
    }

    /// M2 e M3 de `LD (HL),u8`.
    ///
    /// Dois M-cycles, dois acessos ao barramento, um em cada — o que mantém a
    /// invariante de que uma chamada de [`Cpu::step`] faz no máximo um. O
    /// porquê de a escrita ser do M3 está em [`StoreImmediateToHl`].
    fn store_immediate_to_hl(&mut self, bus: &mut Bus, phase: StoreImmediateToHl) -> State {
        match phase {
            StoreImmediateToHl::ReadImmediate => {
                self.latch = u16::from(self.read_at_pc(bus));
                State::StoreImmediateToHl(StoreImmediateToHl::Write)
            }
            StoreImmediateToHl::Write => {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "o latch recebeu um byte no M2"
                )]
                bus.write(self.registers.hl(), self.latch as u8);
                State::Fetch
            }
        }
    }

    /// M2 de `LD A,(r16mem)`: `read((rr)->A)`.
    ///
    /// Um acesso ao barramento e a escrita em `A`, no mesmo M-cycle — a forma do
    /// [`Cpu::load_from_hl`], com o endereço vindo de um par escolhido pelo
    /// opcode em vez de sempre `HL`.
    fn load_from_r16_mem(&mut self, bus: &Bus, source: R16Mem) -> State {
        let address = self.address_from_r16_mem(source);
        self.registers.a = bus.read(address);
        State::Fetch
    }

    /// M2 de `LD (r16mem),A`: `write(A->(rr))`.
    fn store_to_r16_mem(&mut self, bus: &mut Bus, dest: R16Mem) -> State {
        let address = self.address_from_r16_mem(dest);
        bus.write(address, self.registers.a);
        State::Fetch
    }

    /// O endereço que um operando `r16mem` aponta, **e** o efeito colateral que
    /// `hl+`/`hl-` deixam em `HL`.
    ///
    /// Devolve o valor de **antes** da modificação: é o que o `++`/`--` postfixo
    /// da coluna de gbops diz (`write(A->(HL++))`), e a § OAM Corruption Bug
    /// confirma pelo outro lado, ao dizer que o bug dispara conforme o conteúdo
    /// de 16 bits *"(before the operation)"* caia na faixa da OAM.
    ///
    /// A função é `&mut self` porque metade dela é o efeito colateral, e é
    /// chamada **do M2**, nunca do fetch: adiantá-la um M-cycle deixaria o
    /// estado final e o total de T-cycles idênticos e moveria o instante em que
    /// `HL` muda — o que o timer e a PPU veem, e o que a suíte Mooneye mede.
    ///
    /// `wrapping_add`/`wrapping_sub` porque um par de 16 bits dá a volta, como o
    /// `PC` de [`Cpu::read_at_pc`]. Não é caso de erro, é a aritmética do
    /// registrador.
    ///
    /// **O que esta função não modela:** em hardware o `HL±` não é aritmética
    /// silenciosa. A § OAM Corruption Bug explica que a IDU põe o valor nas
    /// linhas de endereço mesmo sem leitura nem escrita assertada, e é por isso
    /// que essas quatro instruções corrompem a OAM *duas* vezes — uma pelo
    /// acesso e outra pelo incremento. Isso é o ROADMAP 7.2.
    fn address_from_r16_mem(&mut self, which: R16Mem) -> u16 {
        match which {
            R16Mem::Bc => self.registers.bc(),
            R16Mem::De => self.registers.de(),
            R16Mem::HlIncrement => {
                let address = self.registers.hl();
                self.registers.set_hl(address.wrapping_add(1));
                address
            }
            R16Mem::HlDecrement => {
                let address = self.registers.hl();
                self.registers.set_hl(address.wrapping_sub(1));
                address
            }
        }
    }

    /// M2 de `LD (FF00+C),A` e `LD A,(FF00+C)`.
    ///
    /// A instrução tem 1 byte, então o `PC` já está no opcode seguinte desde o
    /// fetch — ver [`LDH_C_A`]. O endereço não custa M-cycle nenhum: `C` está
    /// na CPU e a base é constante.
    fn high_page_c(&mut self, bus: &mut Bus, direction: Direction) -> State {
        let address = HIGH_PAGE | u16::from(self.registers.c);
        self.access(bus, direction, address);
        State::Fetch
    }

    /// M2 e M3 de `LD (FF00+u8),A` e `LD A,(FF00+u8)`.
    ///
    /// Dois acessos ao barramento, um em cada M-cycle — o que mantém a
    /// invariante de que uma chamada de [`Cpu::step`] faz no máximo um. O
    /// deslocamento vira endereço já no M2 porque a base é constante; guardar o
    /// byte cru e somar no M3 daria o mesmo, e o latch é de 16 bits de qualquer
    /// forma.
    fn high_page_immediate(
        &mut self,
        bus: &mut Bus,
        direction: Direction,
        phase: HighPageImmediate,
    ) -> State {
        match phase {
            HighPageImmediate::ReadOffset => {
                self.latch = HIGH_PAGE | u16::from(self.read_at_pc(bus));
                State::HighPageImmediate(direction, HighPageImmediate::Access)
            }
            HighPageImmediate::Access => {
                self.access(bus, direction, self.latch);
                State::Fetch
            }
        }
    }

    /// M2 a M4 de `LD (u16),A` e `LD A,(u16)`.
    ///
    /// Os dois primeiros passos são os do `JP u16`, literalmente: `read`,
    /// `read`, little-endian, montados no mesmo latch. O quarto é que diverge —
    /// lá é `internal(branch decision?)`, aqui é o acesso. Ver [`Absolute`].
    fn absolute(&mut self, bus: &mut Bus, direction: Direction, phase: Absolute) -> State {
        match phase {
            Absolute::ReadLowByte => {
                self.latch = u16::from(self.read_at_pc(bus));
                State::Absolute(direction, Absolute::ReadHighByte)
            }
            Absolute::ReadHighByte => {
                self.latch |= u16::from(self.read_at_pc(bus)) << 8;
                State::Absolute(direction, Absolute::Access)
            }
            Absolute::Access => {
                self.access(bus, direction, self.latch);
                State::Fetch
            }
        }
    }

    /// O passo do acesso, que é o mesmo nas três formas do 1.4d: só o endereço
    /// muda de origem, e a [`Direction`] escolhe o lado. `A` é sempre o outro
    /// operando, e nenhuma das seis linhas toca em flag.
    ///
    /// É a primeira vez no projeto que um passo de M-cycle é compartilhado por
    /// instruções diferentes — três formas de resolver endereço convergindo num
    /// acesso só. Ver a § "Por que ainda não há tabela de micro-operações" no
    /// topo do arquivo, onde isso é o dado que faltava para a decisão.
    fn access(&mut self, bus: &mut Bus, direction: Direction, address: u16) {
        match direction {
            Direction::Store => bus.write(address, self.registers.a),
            Direction::Load => self.registers.a = bus.read(address),
        }
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
