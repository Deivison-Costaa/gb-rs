use gb_core::cart::{Cartridge, Mbc2, OPEN_BUS};

const KIB: usize = 1024;
const BANK_LEN: usize = 16 * KIB;

fn rom_with_banks(num_banks: usize) -> Mbc2 {
    let rom_len = num_banks * BANK_LEN;
    let mut rom = vec![0_u8; rom_len];
    for (i, byte) in rom.iter_mut().enumerate() {
        let bank = i / BANK_LEN;
        *byte = ((bank as u8) << 4) | (i & 0x0F) as u8;
    }
    Mbc2::new(rom, num_banks).expect("rom do fixture cabe num MBC2")
}

fn rom_2_banks() -> Mbc2 {
    rom_with_banks(2)
}

fn rom_byte(bank: usize, addr: u16) -> u8 {
    ((bank as u8) << 4) | (addr as u8 & 0x0F)
}

// ── ROM ───────────────────────────────────────────────────────────────

#[test]
fn bank_zero_reads_from_rom_region_0000_3fff() {
    let mbc = rom_2_banks();
    let addr: u16 = 0x0100;
    assert_eq!(mbc.read(addr), rom_byte(0, addr));
}

#[test]
fn default_bank_one_reads_from_region_4000_7fff() {
    let mbc = rom_2_banks();
    let addr: u16 = 0x5678;
    assert_eq!(mbc.read(addr), rom_byte(1, addr));
}

#[test]
fn switching_rom_bank_changes_region_4000_7fff() {
    let mut mbc = rom_with_banks(4);
    let addr: u16 = 0x4321;
    assert_eq!(mbc.read(addr), rom_byte(1, addr));

    mbc.write(0x2100, 0x02);
    assert_eq!(mbc.read(addr), rom_byte(2, addr));

    mbc.write(0x2100, 0x03);
    assert_eq!(mbc.read(addr), rom_byte(3, addr));
}

#[test]
fn bank_zero_is_treated_as_one_for_rom() {
    let mut mbc = rom_with_banks(2);
    mbc.write(0x2100, 0x00);
    let addr: u16 = 0x5000;
    assert_eq!(mbc.read(addr), rom_byte(1, addr));
}

#[test]
fn only_lower_four_bits_select_rom_bank() {
    let mut mbc = rom_with_banks(3);
    mbc.write(0x2100, 0xF2);
    let addr: u16 = 0x5000;
    assert_eq!(mbc.read(addr), rom_byte(2, addr));
}

#[test]
fn max_bank_is_fifteen() {
    let mbc = rom_with_banks(16);
    let addr: u16 = 0x4000;
    assert_eq!(mbc.read(addr), rom_byte(1, addr));
}

// ── RAM ───────────────────────────────────────────────────────────────

#[test]
fn ram_is_disabled_by_default_returns_open_bus() {
    let mbc = rom_2_banks();
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
    assert_eq!(mbc.read(0xA100), OPEN_BUS);
}

#[test]
fn enabling_ram_makes_it_readable_and_writable() {
    let mut mbc = rom_2_banks();

    mbc.write(0x0000, 0x0A);

    mbc.write(0xA000, 0x03);
    assert_eq!(mbc.read(0xA000), 0x03);
}

#[test]
fn ram_only_uses_lower_four_bits() {
    let mut mbc = rom_2_banks();

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0xF7);
    assert_eq!(mbc.read(0xA000), 0x07);
}

#[test]
fn disabling_ram_makes_writes_ignored_and_reads_return_open_bus() {
    let mut mbc = rom_2_banks();

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x05);

    mbc.write(0x0002, 0x00);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);

    mbc.write(0xA000, 0x09);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
}

#[test]
fn lower_nibble_not_a_does_not_enable_ram() {
    let mut mbc = rom_2_banks();
    mbc.write(0x0000, 0x0B);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
}

#[test]
fn lower_nibble_a_at_any_even_upper_byte_enables_ram() {
    let mut mbc = rom_2_banks();

    mbc.write(0x0200, 0x0A);
    mbc.write(0xA000, 0x03);
    assert_eq!(mbc.read(0xA000), 0x03);
}

#[test]
fn ram_address_range_is_a000_to_a1ff() {
    let mut mbc = rom_2_banks();

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x01);
    mbc.write(0xA1FF, 0x02);

    assert_eq!(mbc.read(0xA000), 0x01);
    assert_eq!(mbc.read(0xA1FF), 0x02);
}

// ── RAM echo ──────────────────────────────────────────────────────────

#[test]
fn ram_echoes_at_a200_bfff() {
    let mut mbc = rom_2_banks();

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x05);

    assert_eq!(mbc.read(0xA200), 0x05);
    assert_eq!(mbc.read(0xA400), 0x05);
    assert_eq!(mbc.read(0xB000), 0x05);
    assert_eq!(mbc.read(0xB200), 0x05);
}

// ── Register: bit 8 selects function ──────────────────────────────────

#[test]
fn bit_8_set_selects_rom_bank() {
    let mut mbc = rom_with_banks(4);

    mbc.write(0x2100, 0x03);
    assert_eq!(mbc.read(0x5000), rom_byte(3, 0x5000));
}

#[test]
fn bit_8_clear_selects_ram_enable() {
    let mut mbc = rom_with_banks(4);

    mbc.write(0x2000, 0x0A);
    mbc.write(0xA000, 0x09);
    assert_eq!(mbc.read(0xA000), 0x09);
}

#[test]
fn ram_enable_and_rom_bank_are_independent() {
    let mut mbc = rom_with_banks(4);

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x05);

    mbc.write(0x2100, 0x02);
    assert_eq!(mbc.read(0x5000), rom_byte(2, 0x5000));
    assert_eq!(mbc.read(0xA000), 0x05);
}

// ── Battery / SRAM ────────────────────────────────────────────────────

#[test]
fn ram_data_with_battery_returns_some() {
    let mut mbc = rom_2_banks().with_battery();

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x01);

    let data = mbc.ram_data();
    assert!(data.is_some());
    assert_eq!(data.unwrap().len(), 512);
    assert_eq!(data.unwrap()[0x000], 0x01);
}

#[test]
fn ram_data_without_battery_returns_none() {
    let mbc = rom_2_banks();
    assert!(mbc.ram_data().is_none());
}

#[test]
fn load_ram_populates_nibbles_readable_through_read() {
    let mut mbc = rom_2_banks().with_battery();
    let saved = vec![0x03, 0x07];
    mbc.load_ram(&saved);

    mbc.write(0x0000, 0x0A);
    assert_eq!(mbc.read(0xA000), 0x03);
    assert_eq!(mbc.read(0xA001), 0x07);
}

#[test]
fn load_ram_excess_is_truncated() {
    let mut mbc = rom_2_banks().with_battery();
    let saved = vec![0xFF; 1024];
    mbc.load_ram(&saved);
    assert_eq!(mbc.ram_data().unwrap().len(), 512);
}

// ── load() dispatch ───────────────────────────────────────────────────

#[test]
fn load_mbc2_creates_cartridge_with_512_half_byte_ram() {
    let mut rom = vec![0u8; 32 * KIB];
    rom[0x0147] = 0x05;
    rom[0x0148] = 0x00;
    rom[0x0149] = 0x00;
    rom[0x014A] = 0x00;
    rom[0x014B] = 0x00;
    rom[0x014D] = 0x00;
    let check = rom[0x014D..0x014F]
        .iter()
        .fold(0u8, |c, &b| c.wrapping_sub(b).wrapping_sub(1));
    rom[0x014D] = check;

    let mut cart = gb_core::cart::load(rom).expect("MBC2 devia carregar");
    cart.write(0x0000, 0x0A);
    cart.write(0xA000, 0x07);
    assert_eq!(cart.read(0xA000), 0x07);
}

#[test]
fn load_mbc2_battery_has_ram_and_battery_flag() {
    let mut rom = vec![0u8; 32 * KIB];
    rom[0x0147] = 0x06;
    rom[0x0148] = 0x00;
    rom[0x0149] = 0x00;
    rom[0x014A] = 0x00;
    rom[0x014B] = 0x00;
    rom[0x014D] = 0x00;
    let check = rom[0x014D..0x014F]
        .iter()
        .fold(0u8, |c, &b| c.wrapping_sub(b).wrapping_sub(1));
    rom[0x014D] = check;

    let cart = gb_core::cart::load(rom).expect("MBC2+BATTERY devia carregar");
    assert!(cart.ram_data().is_some());
}
