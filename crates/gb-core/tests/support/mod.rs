//! Controle negativo de decodificação — ROADMAP 0.6: só existe aqui.

pub fn decoded_elsewhere(opcode: u8) -> bool {
    opcode == 0x00
        || opcode == 0xC3
        || ((0x40..=0x7F).contains(&opcode) && opcode != 0x76)
        || opcode & 0b1100_0111 == 0b0000_0110
        || opcode & 0b1100_0111 == 0b0000_0010
        || opcode & 0b1100_1111 == 0b0000_0001
        || opcode & 0b1100_1111 == 0b1100_0101
        || opcode & 0b1100_1111 == 0b1100_0001
        || matches!(
            opcode,
            0x08 | 0xE0 | 0xE2 | 0xE8 | 0xEA | 0xF0 | 0xF2 | 0xF8 | 0xF9 | 0xFA
        )
        || (0x80..=0x8F).contains(&opcode)
        || (0x90..=0x9F).contains(&opcode)
        || (0xA0..=0xB7).contains(&opcode)
        || (0xB8..=0xBF).contains(&opcode)
        || matches!(
            opcode,
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE
        )
        || opcode & 0b1100_0111 == 0b0000_0100
        || opcode & 0b1100_0111 == 0b0000_0101
        || opcode & 0b1100_1111 == 0b0000_0011
        || opcode & 0b1100_1111 == 0b0000_1011
        || opcode & 0b1100_1111 == 0b0000_1001
        || matches!(opcode, 0x07 | 0x0F | 0x17 | 0x1F)
        || opcode == 0xCB
        || matches!(opcode, 0x18 | 0x20 | 0x28 | 0x30 | 0x38)
        || matches!(opcode, 0xC2 | 0xCA | 0xD2 | 0xDA | 0xE9)
        || matches!(opcode, 0xC4 | 0xCC | 0xCD | 0xD4 | 0xDC)
        || matches!(opcode, 0xC0 | 0xC8 | 0xC9 | 0xD0 | 0xD8 | 0xD9)
}
