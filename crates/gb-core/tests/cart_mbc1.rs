use gb_core::cart::{Cartridge, Mbc1, OPEN_BUS};

const KIB: usize = 1024;
const BANK_LEN: usize = 16 * KIB;
const RAM_LEN_8K: usize = 8 * KIB;

fn rom_with_banks(num_banks: usize, ram_bytes: usize) -> Mbc1 {
    let rom_len = num_banks * BANK_LEN;
    let mut rom = vec![0_u8; rom_len];
    for (i, byte) in rom.iter_mut().enumerate() {
        let bank = i / BANK_LEN;
        *byte = ((bank as u8) << 4) | (i & 0x0F) as u8;
    }
    Mbc1::new(rom, num_banks, ram_bytes).expect("rom+ram do fixture cabe num MBC1")
}

fn rom_2_banks() -> Mbc1 {
    rom_with_banks(2, 0)
}

fn rom_4_banks() -> Mbc1 {
    rom_with_banks(4, 0)
}

#[test]
fn bank_zero_reads_from_rom_region_0000_3fff() {
    let mbc = rom_2_banks();
    let addr: u16 = 0x0100;
    assert_eq!(mbc.read(addr), rom_byte(0, addr));
}

#[test]
fn default_bank_one_reads_from_region_4000_7fff() {
    let mbc = rom_4_banks();
    let addr: u16 = 0x5000;
    assert_eq!(mbc.read(addr), rom_byte(1, addr));
}

#[test]
fn switching_rom_bank_changes_region_4000_7fff() {
    let mut mbc = rom_4_banks();
    mbc.write(0x2100, 0x02);
    let addr: u16 = 0x5000;
    assert_eq!(mbc.read(addr), rom_byte(2, addr));
}

#[test]
fn bank_zero_is_treated_as_one_for_rom() {
    let mut mbc = rom_4_banks();
    mbc.write(0x2100, 0x00);
    let addr: u16 = 0x5000;
    assert_eq!(mbc.read(addr), rom_byte(1, addr));
}

#[test]
fn ram_is_disabled_by_default_returns_open_bus() {
    let mbc = rom_with_banks(2, RAM_LEN_8K);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
}

#[test]
fn enabling_ram_makes_it_readable_and_writable() {
    let mut mbc = rom_with_banks(2, RAM_LEN_8K);
    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x42);
    assert_eq!(mbc.read(0xA000), 0x42);
}

#[test]
fn disabling_ram_makes_writes_ignored_and_reads_return_open_bus() {
    let mut mbc = rom_with_banks(2, RAM_LEN_8K);
    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x42);
    mbc.write(0x0000, 0x00);
    mbc.write(0xA000, 0x99);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
    mbc.write(0x0000, 0x0A);
    assert_eq!(mbc.read(0xA000), 0x42);
}

#[test]
fn ram_without_ram_in_header_has_no_ram() {
    let mbc = rom_with_banks(2, 0);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
}

#[test]
fn writing_lower_nibble_not_a_does_not_enable_ram() {
    let mut mbc = rom_with_banks(2, RAM_LEN_8K);
    mbc.write(0x1234, 0xA7);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
}

#[test]
fn lower_nibble_a_at_any_address_0000_1fff_enables_ram() {
    let mut mbc = rom_with_banks(2, RAM_LEN_8K);
    mbc.write(0x1234, 0x7A);
    assert_eq!(mbc.read(0xA000), 0x00);
}

fn rom_byte(bank: usize, addr: u16) -> u8 {
    let offset = bank * BANK_LEN + (addr as usize & (BANK_LEN - 1));
    ((bank as u8) << 4) | (offset & 0x0F) as u8
}
