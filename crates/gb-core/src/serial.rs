//! spec: `docs/reference/09-joypad-serial.md` § Serial Data Transfer (Link Cable).
//! Stub que despacha o byte de SB para uma fila de saída quando o bit 7 de SC é
//! escrito com 1 e o bit 0 (clock internal) também. Sem temporização — SC.7 é
//! limpo imediatamente.

const SB_ADDR: u16 = 0xFF01;
const SC_ADDR: u16 = 0xFF02;

const SB_INITIAL: u8 = 0x00;
const SC_INITIAL: u8 = 0x7E;

pub(crate) struct Serial {
    sb: u8,
    sc: u8,
    output: Vec<u8>,
}

impl Serial {
    pub(crate) fn new() -> Self {
        Self {
            sb: SB_INITIAL,
            sc: SC_INITIAL,
            output: Vec::new(),
        }
    }

    pub(crate) fn read(&self, addr: u16) -> u8 {
        match addr {
            SB_ADDR => self.sb,
            SC_ADDR => self.sc,
            _ => unreachable!("serial só atende $FF01/$FF02, recebeu ${addr:04X}"),
        }
    }

    pub(crate) fn write(&mut self, addr: u16, value: u8) {
        match addr {
            SB_ADDR => self.sb = value,
            SC_ADDR => {
                let trigger = (value & 0x81) == 0x81 && (self.sc & 0x80) == 0x00;
                self.sc = value & 0x83;
                if trigger {
                    self.output.push(self.sb);
                    self.sc &= !0x80;
                }
            }
            _ => unreachable!("serial só atende $FF01/$FF02, recebeu ${addr:04X}"),
        }
    }

    pub(crate) fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.output)
    }
}
