//! spec: `docs/reference/03-opcodes.md`. Iterações: 0013–0017, notas 26–32 do `STATUS.md`.

use crate::bus::Bus;
use crate::cart::HeaderChecksum;
use crate::cpu::alu::{self, AluOp};
use crate::cpu::{Flag, Registers};

const NOP: u8 = 0x00;
const JP_U16: u8 = 0xC3;

const LD_R8_R8_FIRST: u8 = 0x40;
const LD_R8_R8_LAST: u8 = 0x7F;

// $76 é HALT, não LD (HL),(HL) — exceção da § Block 1.
const HALT: u8 = 0x76;

// IllegalOpcode = hardware (a ROM executou lixo); UndecodedOpcode = este emulador (falta implementar).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lockup {
    IllegalOpcode(u8),
    UndecodedOpcode(u8),
}

// Sete registradores alcançáveis como r8 — o oitavo valor é memória. Sem f.
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

// r8 de três bits; índice 6 é [hl].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum R8 {
    Register(ByteRegister),
    MemoryAtHl,
}

impl R8 {
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

const LD_R8_U8_MASK: u8 = 0b1100_0111;
const LD_R8_U8_PATTERN: u8 = 0b0000_0110;

const LD_R16MEM_MASK: u8 = 0b1100_1111;
const STORE_R16MEM_PATTERN: u8 = 0b0000_0010;
const LOAD_R16MEM_PATTERN: u8 = 0b0000_1010;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum R16Mem {
    Bc,
    De,
    HlIncrement,
    HlDecrement,
}

impl R16Mem {
    const fn from_opcode(opcode: u8) -> Self {
        match (opcode >> 4) & 0b11 {
            0 => Self::Bc,
            1 => Self::De,
            2 => Self::HlIncrement,
            _ => Self::HlDecrement,
        }
    }
}

const LD_R16_U16_MASK: u8 = 0b1100_1111;
const LD_R16_U16_PATTERN: u8 = 0b0000_0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum R16 {
    Bc,
    De,
    Hl,
    Sp,
}

impl R16 {
    const fn from_opcode(opcode: u8) -> Self {
        match (opcode >> 4) & 0b11 {
            0 => Self::Bc,
            1 => Self::De,
            2 => Self::Hl,
            _ => Self::Sp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImmediatePair {
    ReadLowByte,
    ReadHighByte,
}

const R16STK_MASK: u8 = 0b1100_1111;
const PUSH_R16STK_PATTERN: u8 = 0b1100_0101;
const POP_R16STK_PATTERN: u8 = 0b1100_0001;

// Índice 3 é af, não sp — tabela vizinha da do R16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum R16Stk {
    Bc,
    De,
    Hl,
    Af,
}

impl R16Stk {
    const fn from_opcode(opcode: u8) -> Self {
        match (opcode >> 4) & 0b11 {
            0 => Self::Bc,
            1 => Self::De,
            2 => Self::Hl,
            _ => Self::Af,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Push {
    Internal,
    WriteHighByte,
    WriteLowByte,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pop {
    ReadLowByte,
    ReadHighByte,
}

// $E2 — 1 byte: C é o operando (erro #1 da 0017 — ver docs/iterations/0017).
const LDH_C_A: u8 = 0xE2;
const LDH_A_C: u8 = 0xF2;
const LDH_IMM8_A: u8 = 0xE0;
const LDH_A_IMM8: u8 = 0xF0;
const LD_IMM16_A: u8 = 0xEA;
const LD_A_IMM16: u8 = 0xFA;

const LD_SP_HL: u8 = 0xF9;
const LD_IMM16_SP: u8 = 0x08;

// `10 ooo rrr`: a operação em 5-3, o operando em 2-0. `100`/`101`/`110`
// (`AND`/`XOR`/`OR`) são o 1.6c.
const ALU_A_R8_MASK: u8 = 0b1111_1000;
const ADD_A_R8_PATTERN: u8 = 0b1000_0000;
const ADC_A_R8_PATTERN: u8 = 0b1000_1000;
const SUB_A_R8_PATTERN: u8 = 0b1001_0000;
const SBC_A_R8_PATTERN: u8 = 0b1001_1000;
const AND_A_R8_PATTERN: u8 = 0b1010_0000;
const XOR_A_R8_PATTERN: u8 = 0b1010_1000;
const OR_A_R8_PATTERN: u8 = 0b1011_0000;
const CP_A_R8_PATTERN: u8 = 0b1011_1000;

// `11 ooo 110`: mesma coluna de flag do 1.6a/1.6b/1.6c, operando vem do `PC`
// em vez de `r8` (nota 43 — o `PC` é testemunha entre os M-cycles).
const ADD_A_IMM8: u8 = 0xC6;
const ADC_A_IMM8: u8 = 0xCE;
const SUB_A_IMM8: u8 = 0xD6;
const SBC_A_IMM8: u8 = 0xDE;
const AND_A_IMM8: u8 = 0xE6;
const XOR_A_IMM8: u8 = 0xEE;
const OR_A_IMM8: u8 = 0xF6;
const CP_A_IMM8: u8 = 0xFE;

// `00 ddd 100`/`00 ddd 101`: únicos da ALU que deixam `C` intocado (1.6e).
const INC_DEC_R8_MASK: u8 = 0b1100_0111;
const INC_R8_PATTERN: u8 = 0b0000_0100;
const DEC_R8_PATTERN: u8 = 0b0000_0101;

// `00 rr 0011`/`00 rr 1011`: as quatro colunas de flag em `-` (1.7a).
const INC_DEC_R16_MASK: u8 = 0b1100_1111;
const INC_R16_PATTERN: u8 = 0b0000_0011;
const DEC_R16_PATTERN: u8 = 0b0000_1011;

// `00 rr 1001`: `ADD HL,r16`. `N`=0 literal, `H`/`C` sobre o par de 16 bits
// (carry do bit 11 e do bit 15). `Z` não é tocada.
const ADD_HL_R16_MASK: u8 = 0b1100_1111;
const ADD_HL_R16_PATTERN: u8 = 0b0000_1001;

const HIGH_PAGE: u16 = 0xFF00;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Store,
    Load,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HighPageImmediate {
    ReadOffset,
    Access,
}

// M4 é o acesso, não internal (erro #2 da 0017 — ver docs/iterations/0017).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Absolute {
    ReadLowByte,
    ReadHighByte,
    Access,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreImmediateToHl {
    ReadImmediate,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreStackPointer {
    ReadAddressLow,
    ReadAddressHigh,
    WriteLowHalf,
    WriteHighHalf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IncDecOp {
    Increment,
    Decrement,
}

// Espelha StoreImmediateToHl: o `(HL)` do 1.6e é read-modify-write, não read+apply
// como o AluFromHl do 1.6a (nota 45) — leitura e escrita são M-cycles distintos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IncDecHl {
    Read,
    Write,
}

// match de Cpu::step é total sem _ =>: estado novo quebra a compilação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Fetch,
    JumpImmediate(JumpImmediate),
    LoadFromHl(ByteRegister),
    StoreToHl(ByteRegister),
    LoadImmediate(ByteRegister),
    StoreImmediateToHl(StoreImmediateToHl),
    // O par viaja (não o endereço) para o HL± não acontecer antes do M2 (ver docs/iterations/0016).
    LoadFromR16Mem(R16Mem),
    StoreToR16Mem(R16Mem),
    LoadImmediatePair(R16, ImmediatePair),
    PushPair(R16Stk, Push),
    PopPair(R16Stk, Pop),
    HighPageC(Direction),
    HighPageImmediate(Direction, HighPageImmediate),
    Absolute(Direction, Absolute),
    // A coluna do $F9 não põe seta em passo nenhum: o instante é escolha (ver docs/iterations/0021).
    CopyHlToStackPointer,
    StoreStackPointer(StoreStackPointer),
    // `read((HL))` sem seta e em 8 T-cycles: o M2 lê e opera, e não há terceiro
    // passo onde o latch aterrissaria (ver docs/iterations/0022).
    AluFromHl(AluOp),
    AluImmediate(AluOp),
    IncDecHl(IncDecOp, IncDecHl),
    // A metade baixa já foi escrita no fetch; falta o `internal` da metade alta.
    IncDecR16(R16),
    // Source r16 carregado para o M2; destino é sempre HL. O H alto foi escrito
    // no fetch (como o 1.7a); falta escrever a metade alta no internal.
    AddHlR16(R16),
    Locked(Lockup),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JumpImmediate {
    ReadLowByte,
    ReadHighByte,
    SetProgramCounter,
}

#[derive(Debug, Clone)]
pub struct Cpu {
    pub registers: Registers,
    state: State,
    latch: u16,
}

impl Cpu {
    #[must_use]
    pub const fn after_boot_rom(checksum: HeaderChecksum) -> Self {
        Self {
            registers: Registers::after_boot_rom(checksum),
            state: State::Fetch,
            latch: 0,
        }
    }

    // Avança um M-cycle (R2). No máximo um acesso ao barramento.
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
            State::LoadImmediatePair(target, phase) => self.load_immediate_pair(bus, target, phase),
            State::PushPair(source, phase) => self.push_pair(bus, source, phase),
            State::PopPair(target, phase) => self.pop_pair(bus, target, phase),
            State::HighPageC(direction) => self.high_page_c(bus, direction),
            State::HighPageImmediate(direction, phase) => {
                self.high_page_immediate(bus, direction, phase)
            }
            State::Absolute(direction, phase) => self.absolute(bus, direction, phase),
            State::CopyHlToStackPointer => self.copy_hl_to_stack_pointer(),
            State::StoreStackPointer(phase) => self.store_stack_pointer(bus, phase),
            State::AluFromHl(op) => self.alu_from_hl(bus, op),
            State::AluImmediate(op) => self.alu_immediate(bus, op),
            State::IncDecHl(op, phase) => self.inc_dec_hl(bus, op, phase),
            State::IncDecR16(target) => self.finish_inc_dec_r16(target),
            State::AddHlR16(source) => self.finish_add_hl_r16(source),
            State::Locked(lockup) => State::Locked(lockup),
        };
    }

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
            | State::LoadImmediatePair(..)
            | State::PushPair(..)
            | State::PopPair(..)
            | State::HighPageC(_)
            | State::HighPageImmediate(..)
            | State::Absolute(..)
            | State::CopyHlToStackPointer
            | State::StoreStackPointer(_)
            | State::AluFromHl(_)
            | State::AluImmediate(_)
            | State::IncDecHl(..)
            | State::IncDecR16(_)
            | State::AddHlR16(_) => None,
        }
    }

    #[must_use]
    pub const fn is_between_instructions(&self) -> bool {
        matches!(self.state, State::Fetch)
    }

    fn fetch(&mut self, bus: &Bus) -> State {
        let opcode = self.read_at_pc(bus);

        match opcode {
            NOP => State::Fetch,
            JP_U16 => State::JumpImmediate(JumpImmediate::ReadLowByte),
            HALT => State::Locked(Lockup::UndecodedOpcode(opcode)),
            LD_R8_R8_FIRST..=LD_R8_R8_LAST => self.load_r8_r8(opcode),
            _ if opcode & LD_R8_U8_MASK == LD_R8_U8_PATTERN => Self::load_r8_u8(opcode),
            _ if opcode & LD_R16MEM_MASK == STORE_R16MEM_PATTERN => {
                State::StoreToR16Mem(R16Mem::from_opcode(opcode))
            }
            _ if opcode & LD_R16MEM_MASK == LOAD_R16MEM_PATTERN => {
                State::LoadFromR16Mem(R16Mem::from_opcode(opcode))
            }
            _ if opcode & LD_R16_U16_MASK == LD_R16_U16_PATTERN => {
                State::LoadImmediatePair(R16::from_opcode(opcode), ImmediatePair::ReadLowByte)
            }
            _ if opcode & R16STK_MASK == PUSH_R16STK_PATTERN => {
                State::PushPair(R16Stk::from_opcode(opcode), Push::Internal)
            }
            _ if opcode & R16STK_MASK == POP_R16STK_PATTERN => {
                State::PopPair(R16Stk::from_opcode(opcode), Pop::ReadLowByte)
            }
            LDH_C_A => State::HighPageC(Direction::Store),
            LDH_A_C => State::HighPageC(Direction::Load),
            LDH_IMM8_A => State::HighPageImmediate(Direction::Store, HighPageImmediate::ReadOffset),
            LDH_A_IMM8 => State::HighPageImmediate(Direction::Load, HighPageImmediate::ReadOffset),
            LD_IMM16_A => State::Absolute(Direction::Store, Absolute::ReadLowByte),
            LD_A_IMM16 => State::Absolute(Direction::Load, Absolute::ReadLowByte),
            LD_SP_HL => State::CopyHlToStackPointer,
            LD_IMM16_SP => State::StoreStackPointer(StoreStackPointer::ReadAddressLow),
            _ if opcode & ALU_A_R8_MASK == ADD_A_R8_PATTERN => self.alu_a_r8(AluOp::Add, opcode),
            _ if opcode & ALU_A_R8_MASK == ADC_A_R8_PATTERN => {
                self.alu_a_r8(AluOp::AddWithCarry, opcode)
            }
            _ if opcode & ALU_A_R8_MASK == SUB_A_R8_PATTERN => {
                self.alu_a_r8(AluOp::Subtract, opcode)
            }
            _ if opcode & ALU_A_R8_MASK == SBC_A_R8_PATTERN => {
                self.alu_a_r8(AluOp::SubtractWithCarry, opcode)
            }
            _ if opcode & ALU_A_R8_MASK == AND_A_R8_PATTERN => self.alu_a_r8(AluOp::And, opcode),
            _ if opcode & ALU_A_R8_MASK == XOR_A_R8_PATTERN => self.alu_a_r8(AluOp::Xor, opcode),
            _ if opcode & ALU_A_R8_MASK == OR_A_R8_PATTERN => self.alu_a_r8(AluOp::Or, opcode),
            _ if opcode & ALU_A_R8_MASK == CP_A_R8_PATTERN => self.alu_a_r8(AluOp::Compare, opcode),
            ADD_A_IMM8 => State::AluImmediate(AluOp::Add),
            ADC_A_IMM8 => State::AluImmediate(AluOp::AddWithCarry),
            SUB_A_IMM8 => State::AluImmediate(AluOp::Subtract),
            SBC_A_IMM8 => State::AluImmediate(AluOp::SubtractWithCarry),
            AND_A_IMM8 => State::AluImmediate(AluOp::And),
            XOR_A_IMM8 => State::AluImmediate(AluOp::Xor),
            OR_A_IMM8 => State::AluImmediate(AluOp::Or),
            CP_A_IMM8 => State::AluImmediate(AluOp::Compare),
            _ if opcode & INC_DEC_R8_MASK == INC_R8_PATTERN => {
                self.inc_dec_r8(IncDecOp::Increment, opcode)
            }
            _ if opcode & INC_DEC_R8_MASK == DEC_R8_PATTERN => {
                self.inc_dec_r8(IncDecOp::Decrement, opcode)
            }
            _ if opcode & INC_DEC_R16_MASK == INC_R16_PATTERN => {
                self.inc_dec_r16(IncDecOp::Increment, opcode)
            }
            _ if opcode & INC_DEC_R16_MASK == DEC_R16_PATTERN => {
                self.inc_dec_r16(IncDecOp::Decrement, opcode)
            }
            _ if opcode & ADD_HL_R16_MASK == ADD_HL_R16_PATTERN => self.add_hl_r16(opcode),
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                State::Locked(Lockup::IllegalOpcode(opcode))
            }
            _ => State::Locked(Lockup::UndecodedOpcode(opcode)),
        }
    }

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
            JumpImmediate::SetProgramCounter => {
                self.registers.pc = self.latch;
                State::Fetch
            }
        }
    }

    fn load_r8_r8(&mut self, opcode: u8) -> State {
        match (R8::from_bits(opcode >> 3), R8::from_bits(opcode)) {
            (R8::Register(dest), R8::Register(source)) => {
                let value = self.read_r8(source);
                self.write_r8(dest, value);
                State::Fetch
            }
            (R8::Register(dest), R8::MemoryAtHl) => State::LoadFromHl(dest),
            (R8::MemoryAtHl, R8::Register(source)) => State::StoreToHl(source),
            (R8::MemoryAtHl, R8::MemoryAtHl) => State::Locked(Lockup::UndecodedOpcode(opcode)),
        }
    }

    fn load_from_hl(&mut self, bus: &Bus, dest: ByteRegister) -> State {
        let value = bus.read(self.registers.hl());
        self.write_r8(dest, value);
        State::Fetch
    }

    fn store_to_hl(&mut self, bus: &mut Bus, source: ByteRegister) -> State {
        bus.write(self.registers.hl(), self.read_r8(source));
        State::Fetch
    }

    const fn load_r8_u8(opcode: u8) -> State {
        match R8::from_bits(opcode >> 3) {
            R8::Register(dest) => State::LoadImmediate(dest),
            R8::MemoryAtHl => State::StoreImmediateToHl(StoreImmediateToHl::ReadImmediate),
        }
    }

    fn load_immediate(&mut self, bus: &Bus, dest: ByteRegister) -> State {
        let value = self.read_at_pc(bus);
        self.write_r8(dest, value);
        State::Fetch
    }

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

    fn load_from_r16_mem(&mut self, bus: &Bus, source: R16Mem) -> State {
        let address = self.address_from_r16_mem(source);
        self.registers.a = bus.read(address);
        State::Fetch
    }

    fn store_to_r16_mem(&mut self, bus: &mut Bus, dest: R16Mem) -> State {
        let address = self.address_from_r16_mem(dest);
        bus.write(address, self.registers.a);
        State::Fetch
    }

    // Devolve o endereço de antes do HL±. Chamada do M2, nunca do fetch —
    // adiantar move o instante do efeito com o mesmo estado final (erro da 0016).
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

    // Meia metade por M-cycle: `read(u16:lower->C)` → `read(u16:upper->B)`.
    // Sem latch — latchar os dois e escrever o par no M3 é o erro #1 da 0018.
    fn load_immediate_pair(&mut self, bus: &Bus, target: R16, phase: ImmediatePair) -> State {
        let byte = self.read_at_pc(bus);

        match phase {
            ImmediatePair::ReadLowByte => {
                self.write_r16_low(target, byte);
                State::LoadImmediatePair(target, ImmediatePair::ReadHighByte)
            }
            ImmediatePair::ReadHighByte => {
                self.write_r16_high(target, byte);
                State::Fetch
            }
        }
    }

    const fn read_r16(&self, which: R16) -> u16 {
        match which {
            R16::Bc => self.registers.bc(),
            R16::De => self.registers.de(),
            R16::Hl => self.registers.hl(),
            R16::Sp => self.registers.sp,
        }
    }

    const fn write_r16_low(&mut self, which: R16, value: u8) {
        match which {
            R16::Bc => self.registers.c = value,
            R16::De => self.registers.e = value,
            R16::Hl => self.registers.l = value,
            R16::Sp => self.registers.sp = (self.registers.sp & 0xFF00) | value as u16,
        }
    }

    const fn write_r16_high(&mut self, which: R16, value: u8) {
        match which {
            R16::Bc => self.registers.b = value,
            R16::De => self.registers.d = value,
            R16::Hl => self.registers.h = value,
            R16::Sp => self.registers.sp = (self.registers.sp & 0x00FF) | ((value as u16) << 8),
        }
    }

    // `write(upper->(--SP))`: o `--SP` é do passo da escrita, como o `HL++` do
    // 1.4c. Decrementar no `internal` do M2 é o erro #1 da 0019.
    fn push_pair(&mut self, bus: &mut Bus, source: R16Stk, phase: Push) -> State {
        match phase {
            Push::Internal => State::PushPair(source, Push::WriteHighByte),
            Push::WriteHighByte => {
                let [high, _] = self.read_r16_stk(source).to_be_bytes();
                self.push_byte(bus, high);
                State::PushPair(source, Push::WriteLowByte)
            }
            Push::WriteLowByte => {
                let [_, low] = self.read_r16_stk(source).to_be_bytes();
                self.push_byte(bus, low);
                State::Fetch
            }
        }
    }

    // `read((SP++)->C)`: meia metade por M-cycle, como o 1.5a — latchar os dois
    // é o erro #1 da 0018.
    fn pop_pair(&mut self, bus: &Bus, target: R16Stk, phase: Pop) -> State {
        let byte = self.pop_byte(bus);

        match phase {
            Pop::ReadLowByte => {
                self.write_r16_stk_low(target, byte);
                State::PopPair(target, Pop::ReadHighByte)
            }
            Pop::ReadHighByte => {
                self.write_r16_stk_high(target, byte);
                State::Fetch
            }
        }
    }

    fn push_byte(&mut self, bus: &mut Bus, value: u8) {
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        bus.write(self.registers.sp, value);
    }

    // Pós-incremento: lê em `SP` e só então anda. A simetria com `push_byte` é de
    // papel, não de notação — `(--SP)` lá, `(SP++)` aqui.
    fn pop_byte(&mut self, bus: &Bus) -> u8 {
        let byte = bus.read(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        byte
    }

    const fn read_r16_stk(&self, which: R16Stk) -> u16 {
        match which {
            R16Stk::Bc => self.registers.bc(),
            R16Stk::De => self.registers.de(),
            R16Stk::Hl => self.registers.hl(),
            R16Stk::Af => self.registers.af(),
        }
    }

    const fn write_r16_stk_low(&mut self, which: R16Stk, value: u8) {
        match which {
            R16Stk::Bc => self.registers.c = value,
            R16Stk::De => self.registers.e = value,
            R16Stk::Hl => self.registers.l = value,
            R16Stk::Af => self.registers.f = value,
        }
    }

    const fn write_r16_stk_high(&mut self, which: R16Stk, value: u8) {
        match which {
            R16Stk::Bc => self.registers.b = value,
            R16Stk::De => self.registers.d = value,
            R16Stk::Hl => self.registers.h = value,
            R16Stk::Af => self.registers.a = value,
        }
    }

    fn high_page_c(&mut self, bus: &mut Bus, direction: Direction) -> State {
        let address = HIGH_PAGE | u16::from(self.registers.c);
        self.access(bus, direction, address);
        State::Fetch
    }

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

    fn alu_a_r8(&mut self, op: AluOp, opcode: u8) -> State {
        match R8::from_bits(opcode) {
            R8::Register(source) => {
                let operand = self.read_r8(source);
                alu::apply(&mut self.registers, op, operand);
                State::Fetch
            }
            R8::MemoryAtHl => State::AluFromHl(op),
        }
    }

    fn alu_from_hl(&mut self, bus: &Bus, op: AluOp) -> State {
        let operand = bus.read(self.registers.hl());
        alu::apply(&mut self.registers, op, operand);
        State::Fetch
    }

    fn alu_immediate(&mut self, bus: &Bus, op: AluOp) -> State {
        let operand = self.read_at_pc(bus);
        alu::apply(&mut self.registers, op, operand);
        State::Fetch
    }

    fn inc_dec_r8(&mut self, op: IncDecOp, opcode: u8) -> State {
        match R8::from_bits(opcode >> 3) {
            R8::Register(register) => {
                let value = self.read_r8(register);
                let result = match op {
                    IncDecOp::Increment => alu::increment(&mut self.registers, value),
                    IncDecOp::Decrement => alu::decrement(&mut self.registers, value),
                };
                self.write_r8(register, result);
                State::Fetch
            }
            R8::MemoryAtHl => State::IncDecHl(op, IncDecHl::Read),
        }
    }

    // Leitura no M2, escrita no M3 — o mesmo endereço em passos diferentes.
    // Juntar os dois no M2 dá a mesma memória final com um M-cycle a menos
    // (erro #1 da 0015 numa forma nova).
    fn inc_dec_hl(&mut self, bus: &mut Bus, op: IncDecOp, phase: IncDecHl) -> State {
        match phase {
            IncDecHl::Read => {
                let value = bus.read(self.registers.hl());
                let result = match op {
                    IncDecOp::Increment => alu::increment(&mut self.registers, value),
                    IncDecOp::Decrement => alu::decrement(&mut self.registers, value),
                };
                self.latch = u16::from(result);
                State::IncDecHl(op, IncDecHl::Write)
            }
            IncDecHl::Write => {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "o latch recebeu um byte no M2"
                )]
                bus.write(self.registers.hl(), self.latch as u8);
                State::Fetch
            }
        }
    }

    // Nenhuma flag: as quatro colunas do grupo são `-`. `fetch` computa o par
    // inteiro e escreve a metade baixa; `internal` escreve a alta — a coluna
    // anota "Probably writes to X here" em cada passo, sem seta.
    fn inc_dec_r16(&mut self, op: IncDecOp, opcode: u8) -> State {
        let target = R16::from_opcode(opcode);
        let value = self.read_r16(target);
        let result = match op {
            IncDecOp::Increment => value.wrapping_add(1),
            IncDecOp::Decrement => value.wrapping_sub(1),
        };
        self.latch = result;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "grava só a metade baixa do par calculado"
        )]
        self.write_r16_low(target, result as u8);
        State::IncDecR16(target)
    }

    fn finish_inc_dec_r16(&mut self, target: R16) -> State {
        let [high, _] = self.latch.to_be_bytes();
        self.write_r16_high(target, high);
        State::Fetch
    }

    // `ADD HL,r16`: mesma forma de M-cycle do 1.7a. `Z` não é tocada; `H`
    // (carry do bit 11) e `C` (carry do bit 15) são calculados sobre o par
    // inteiro. `N` = 0 literal.
    fn add_hl_r16(&mut self, opcode: u8) -> State {
        let source = R16::from_opcode(opcode);
        let hl = self.registers.hl();
        let operand = self.read_r16(source);
        let result = hl.wrapping_add(operand);

        let half = ((hl & 0x0FFF).wrapping_add(operand & 0x0FFF)) >> 12;
        let carry = (hl as u32).wrapping_add(operand as u32) >> 16;

        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, half != 0);
        self.registers.set_flag(Flag::C, carry != 0);

        self.latch = result;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "grava só a metade baixa de HL"
        )]
        self.write_r16_low(R16::Hl, result as u8);
        State::AddHlR16(source)
    }

    fn finish_add_hl_r16(&mut self, _source: R16) -> State {
        let [high, _] = self.latch.to_be_bytes();
        self.write_r16_high(R16::Hl, high);
        State::Fetch
    }

    const fn copy_hl_to_stack_pointer(&mut self) -> State {
        self.registers.sp = self.registers.hl();
        State::Fetch
    }

    // `read(u16:lower)` sem seta: o endereço é latch, como no $FA. As duas
    // escritas são M-cycles distintos e a de baixo vai no endereço mais baixo.
    fn store_stack_pointer(&mut self, bus: &mut Bus, phase: StoreStackPointer) -> State {
        match phase {
            StoreStackPointer::ReadAddressLow => {
                self.latch = u16::from(self.read_at_pc(bus));
                State::StoreStackPointer(StoreStackPointer::ReadAddressHigh)
            }
            StoreStackPointer::ReadAddressHigh => {
                self.latch |= u16::from(self.read_at_pc(bus)) << 8;
                State::StoreStackPointer(StoreStackPointer::WriteLowHalf)
            }
            StoreStackPointer::WriteLowHalf => {
                let [_, low] = self.registers.sp.to_be_bytes();
                bus.write(self.latch, low);
                State::StoreStackPointer(StoreStackPointer::WriteHighHalf)
            }
            StoreStackPointer::WriteHighHalf => {
                let [high, _] = self.registers.sp.to_be_bytes();
                bus.write(self.latch.wrapping_add(1), high);
                State::Fetch
            }
        }
    }

    // Compartilhado pelas três formas do 1.4d (ver docs/iterations/0017).
    fn access(&mut self, bus: &mut Bus, direction: Direction, address: u16) {
        match direction {
            Direction::Store => bus.write(address, self.registers.a),
            Direction::Load => self.registers.a = bus.read(address),
        }
    }

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

    // wrapping_add: PC dá a volta — instrução em $FFFF segue em $0000.
    fn read_at_pc(&mut self, bus: &Bus) -> u8 {
        let byte = bus.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        byte
    }
}
