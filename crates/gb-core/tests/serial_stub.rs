//! ROADMAP 1.12 — stub da porta serial ($FF01/$FF02).

use gb_core::bus::Bus;
use gb_core::cart::{Cartridge, OPEN_BUS};

const SB: u16 = 0xFF01;
const SC: u16 = 0xFF02;

struct SilentCartridge;

impl Cartridge for SilentCartridge {
    fn read(&self, _addr: u16) -> u8 {
        OPEN_BUS
    }

    fn write(&mut self, _addr: u16, _value: u8) {}
}

fn bus() -> Bus {
    Bus::new(Box::new(SilentCartridge))
}

const PROBE: u8 = 0xA3;

#[test]
fn sb_starts_at_zero_and_is_readable_writable() {
    let mut bus = bus();

    assert_eq!(
        bus.read(SB),
        0x00,
        "$FF01 (SB) no hand-off da boot ROM é $00"
    );

    bus.write(SB, PROBE);
    assert_eq!(
        bus.read(SB),
        PROBE,
        "$FF01 (SB) devolve o byte escrito: é um registrador de verdade"
    );
}

#[test]
fn sc_starts_at_7e_and_is_readable_writable() {
    let mut bus = bus();

    assert_eq!(
        bus.read(SC),
        0x7E,
        "$FF02 (SC) no hand-off da boot ROM é $7E"
    );

    bus.write(SC, 0x01);
    assert_eq!(
        bus.read(SC),
        0x01,
        "$FF02 (SC) devolve o byte escrito: é um registrador de verdade"
    );
}

#[test]
fn writing_81_to_sc_with_internal_clock_triggers_transfer_and_clears_bit_7() {
    let mut bus = bus();

    bus.write(SB, 0x42);
    bus.write(SC, 0x81);

    assert_eq!(
        bus.read(SC) & 0x80,
        0x00,
        "depois da transferência o bit 7 de SC deve voltar a 0"
    );
    assert_eq!(
        bus.read(SC) & 0x01,
        0x01,
        "o bit 0 (clock select = internal) permanece como foi escrito"
    );
}

#[test]
fn write_to_sc_without_bit_7_set_does_not_trigger_output() {
    let mut bus = bus();

    bus.write(SB, 0x77);
    bus.write(SC, 0x01);

    let output = bus.take_serial_output();
    assert!(
        output.is_empty(),
        "escrever em SC sem o bit 7 setado não dispara transferência"
    );
}

#[test]
fn writing_81_to_sc_enqueues_sb_byte_in_the_output() {
    let mut bus = bus();

    bus.write(SB, 0x42);
    bus.write(SC, 0x81);

    let output = bus.take_serial_output();
    assert_eq!(
        output.len(),
        1,
        "uma transferência disparada produz um byte"
    );
    assert_eq!(output[0], 0x42, "o byte transmitido é o que estava em SB");

    let output2 = bus.take_serial_output();
    assert!(output2.is_empty(), "depois de `take` a fila está vazia");
}

#[test]
fn external_clock_does_not_trigger_output_even_with_bit_7_set() {
    let mut bus = bus();

    bus.write(SB, 0x99);
    bus.write(SC, 0x80);

    assert_eq!(
        bus.read(SC),
        0x80,
        "SC com bit 7=1 e bit 0=0 (external clock) não dispara: o stub não \
         tem escravo para fornecer o clock, e o bit 7 fica setado aguardando"
    );
    assert!(bus.take_serial_output().is_empty());
}

#[test]
fn writing_81_when_bit_7_is_already_set_does_not_trigger_again() {
    let mut bus = bus();

    bus.write(SB, 0x99);
    bus.write(SC, 0x80);
    assert!(bus.take_serial_output().is_empty());

    bus.write(SB, 0x42);
    bus.write(SC, 0x81);

    assert!(
        bus.take_serial_output().is_empty(),
        "escrever $81 com SC.7 já setado (do $80 anterior) não dispara: \
         o stub exige borda de subida do bit 7"
    );
}

#[test]
fn sc_masks_bits_6_through_2_to_zero() {
    let mut bus = bus();

    bus.write(SC, 0x80);
    bus.write(SC, 0xFF);
    assert_eq!(
        bus.read(SC),
        0x83,
        "$FF02 (SC) descarta os bits 6–2: $FF & $83 = $83"
    );
}

#[test]
fn consecutive_transfers_accumulate_in_fifo_order() {
    let mut bus = bus();

    bus.write(SB, 0x10);
    bus.write(SC, 0x81);
    bus.write(SB, 0x20);
    bus.write(SC, 0x81);
    bus.write(SB, 0x30);
    bus.write(SC, 0x81);

    assert_eq!(bus.take_serial_output(), vec![0x10, 0x20, 0x30]);
}
