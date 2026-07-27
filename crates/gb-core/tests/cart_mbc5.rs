use gb_core::cart::{Cartridge, Mbc5, OPEN_BUS};

const KIB: usize = 1024;
const BANK_LEN: usize = 16 * KIB;

fn rom_with_banks(num_banks: usize) -> Mbc5 {
    let rom_len = num_banks * BANK_LEN;
    let mut rom = vec![0_u8; rom_len];
    for (i, byte) in rom.iter_mut().enumerate() {
        let bank = i / BANK_LEN;
        *byte = ((bank as u8) << 4) | (i & 0x0F) as u8;
    }
    Mbc5::new(rom, num_banks, 0).expect("rom do fixture cabe num MBC5")
}

fn rom_with_ram(num_banks: usize, ram_banks: usize) -> Mbc5 {
    let rom_len = num_banks * BANK_LEN;
    let mut rom = vec![0_u8; rom_len];
    for (i, byte) in rom.iter_mut().enumerate() {
        let bank = i / BANK_LEN;
        *byte = ((bank as u8) << 4) | (i & 0x0F) as u8;
    }
    let ram_bytes = ram_banks * 8 * KIB;
    Mbc5::new(rom, num_banks, ram_bytes).expect("rom do fixture cabe num MBC5")
}

fn rom_2_banks() -> Mbc5 {
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
fn bank_zero_is_accessible_from_4000_7fff_unlike_other_mbcs() {
    let mut mbc = rom_with_banks(2);
    mbc.write(0x2100, 0x00);
    let addr: u16 = 0x5000;
    assert_eq!(mbc.read(addr), rom_byte(0, addr));
}

#[test]
fn rom_bank_register_low_is_8_bits() {
    let num_banks = 130;
    let rom_len = num_banks * BANK_LEN;
    let mut rom = vec![0_u8; rom_len];
    for bank in 0..num_banks {
        rom[bank * BANK_LEN + 0x1000] = bank as u8;
    }
    let mut mbc = Mbc5::new(rom, num_banks, 0).expect("rom do fixture cabe num MBC5");

    mbc.write(0x2200, 0x80);
    let addr: u16 = 0x5000;
    assert_eq!(mbc.read(addr), 128);

    mbc.write(0x2200, 0x03);
    assert_eq!(mbc.read(addr), 3);
}

#[test]
fn rom_bank_ninth_bit_via_separate_register() {
    let mut mbc = rom_with_banks(260);
    let addr: u16 = 0x5000;

    mbc.write(0x2100, 0x00);
    mbc.write(0x3000, 0x00);
    assert_eq!(mbc.read(addr), rom_byte(0, addr));

    mbc.write(0x3000, 0x01);
    assert_eq!(mbc.read(addr), rom_byte(256, addr));

    mbc.write(0x2100, 0x03);
    assert_eq!(mbc.read(addr), rom_byte(259, addr));
}

#[test]
fn ninth_bit_is_masked_to_one_bit() {
    let mut mbc = rom_with_banks(260);
    let addr: u16 = 0x5000;

    mbc.write(0x2100, 0x00);
    mbc.write(0x3000, 0xFE);
    assert_eq!(mbc.read(addr), rom_byte(0, addr));

    mbc.write(0x3000, 0xFF);
    assert_eq!(mbc.read(addr), rom_byte(256, addr));
}

// ── RAM ───────────────────────────────────────────────────────────────

#[test]
fn ram_is_disabled_by_default_returns_open_bus() {
    let mbc = rom_with_ram(2, 1);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
}

#[test]
fn enabling_ram_makes_it_readable_and_writable() {
    let mut mbc = rom_with_ram(2, 1);

    mbc.write(0x0000, 0x0A);

    mbc.write(0xA000, 0x42);
    assert_eq!(mbc.read(0xA000), 0x42);
}

#[test]
fn ram_stores_full_byte_unlike_mbc2() {
    let mut mbc = rom_with_ram(2, 1);

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0xFF);
    assert_eq!(mbc.read(0xA000), 0xFF);
}

#[test]
fn disabling_ram_makes_writes_ignored_and_reads_return_open_bus() {
    let mut mbc = rom_with_ram(2, 1);

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x55);

    mbc.write(0x0002, 0x00);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);

    mbc.write(0xA000, 0x09);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
}

#[test]
fn write_while_disabled_does_not_mutate_ram() {
    let mut mbc = rom_with_ram(2, 1);

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x42);

    mbc.write(0x0002, 0x00);
    mbc.write(0xA000, 0xFF);

    mbc.write(0x0000, 0x0A);
    assert_eq!(mbc.read(0xA000), 0x42);
}

#[test]
fn lower_nibble_not_a_does_not_enable_ram() {
    let mut mbc = rom_with_ram(2, 1);
    mbc.write(0x0000, 0x0B);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
}

#[test]
fn any_value_with_lower_nibble_a_enables_ram() {
    let mut mbc = rom_with_ram(2, 1);

    mbc.write(0x0000, 0x1A);
    mbc.write(0xA000, 0x77);
    assert_eq!(mbc.read(0xA000), 0x77);

    mbc.write(0x0002, 0x00);
    mbc.write(0x0004, 0x5A);
    mbc.write(0xA001, 0x88);
    assert_eq!(mbc.read(0xA001), 0x88);
}

#[test]
fn ram_without_ram_bytes_always_returns_open_bus() {
    let mut mbc = rom_with_banks(2);

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x55);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
}

// ── RAM banking ────────────────────────────────────────────────────────

#[test]
fn selecting_ram_bank_switches_a000_bfff() {
    let mut mbc = rom_with_ram(2, 4);

    mbc.write(0x0000, 0x0A);

    mbc.write(0x4000, 0x00);
    mbc.write(0xA000, 0x11);

    mbc.write(0x4000, 0x01);
    mbc.write(0xA000, 0x22);

    mbc.write(0x4000, 0x00);
    assert_eq!(mbc.read(0xA000), 0x11);

    mbc.write(0x4000, 0x01);
    assert_eq!(mbc.read(0xA000), 0x22);
}

#[test]
fn ram_bank_default_is_zero() {
    let mut mbc = rom_with_ram(2, 4);

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x99);

    assert_eq!(mbc.read(0xA000), 0x99);
}

#[test]
fn unmapped_ram_bank_wraps_around() {
    let mut mbc = rom_with_ram(2, 1);

    mbc.write(0x0000, 0x0A);

    mbc.write(0x4000, 0x00);
    mbc.write(0xA000, 0x77);

    mbc.write(0x4000, 0x08);
    assert_eq!(mbc.read(0xA000), 0x77);
}

#[test]
fn ram_bank_selection_masks_to_four_bits() {
    let mut mbc = rom_with_ram(2, 4);

    mbc.write(0x0000, 0x0A);

    mbc.write(0x4000, 0x10);
    mbc.write(0xA000, 0x0B);

    mbc.write(0x4000, 0x00);
    assert_eq!(mbc.read(0xA000), 0x0B);
}

#[test]
fn mbc5_supports_up_to_16_ram_banks() {
    let mut mbc = rom_with_ram(2, 16);

    mbc.write(0x0000, 0x0A);

    for bank in 0..15 {
        mbc.write(0x4000, bank);
        mbc.write(0xA000, bank + 1);
    }

    for bank in 0..15 {
        mbc.write(0x4000, bank);
        assert_eq!(mbc.read(0xA000), bank + 1);
    }
}

// ── RAM within window ──────────────────────────────────────────────────

#[test]
fn ram_occupies_full_a000_bfff_window() {
    let mut mbc = rom_with_ram(2, 1);

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0xAB);
    mbc.write(0xBFFF, 0xCD);

    assert_eq!(mbc.read(0xA000), 0xAB);
    assert_eq!(mbc.read(0xBFFF), 0xCD);
}

// ── Battery / SRAM ────────────────────────────────────────────────────

#[test]
fn ram_data_with_battery_returns_some() {
    let mut mbc = rom_with_ram(2, 1).with_battery();

    mbc.write(0x0000, 0x0A);
    mbc.write(0xA000, 0x42);

    let data = mbc.ram_data();
    assert!(data.is_some());
    assert_eq!(data.unwrap()[0x000], 0x42);
}

#[test]
fn ram_data_without_battery_returns_none() {
    let mbc = rom_with_ram(2, 1);
    assert!(mbc.ram_data().is_none());
}

#[test]
fn load_ram_populates_values_readable_through_read() {
    let mut mbc = rom_with_ram(2, 1).with_battery();
    let saved = vec![0xA5; 8 * KIB];
    mbc.load_ram(&saved);

    mbc.write(0x0000, 0x0A);
    assert_eq!(mbc.read(0xA000), 0xA5);
    assert_eq!(mbc.read(0xA100), 0xA5);
}

#[test]
fn load_ram_excess_is_truncated() {
    let mut mbc = rom_with_ram(2, 1).with_battery();
    let saved = vec![0xFF; 16 * KIB];
    mbc.load_ram(&saved);
    assert_eq!(mbc.ram_data().unwrap().len(), 8 * KIB);
}

// ── load() dispatch ───────────────────────────────────────────────────

fn min_rom(cart_type: u8) -> Vec<u8> {
    let mut rom = vec![0u8; 32 * KIB];
    rom[0x0147] = cart_type;
    rom[0x0148] = 0x00;
    rom[0x0149] = 0x00;
    rom[0x014A] = 0x00;
    rom[0x014B] = 0x00;
    let check = rom[0x0134..0x014D]
        .iter()
        .fold(0u8, |c, &b| c.wrapping_sub(b).wrapping_sub(1));
    rom[0x014D] = check;
    rom
}

#[test]
fn load_mbc5_creates_cartridge_without_ram() {
    let rom = min_rom(0x19);
    let cart = gb_core::cart::load(rom).expect("MBC5 devia carregar");
    assert_eq!(cart.read(0x0100), 0x00);
}

#[test]
fn load_mbc5_ram_has_external_ram() {
    let mut rom = min_rom(0x1A);
    rom[0x0149] = 0x02;
    let check = rom[0x0134..0x014D]
        .iter()
        .fold(0u8, |c, &b| c.wrapping_sub(b).wrapping_sub(1));
    rom[0x014D] = check;

    let mut cart = gb_core::cart::load(rom).expect("MBC5+RAM devia carregar");
    cart.write(0x0000, 0x0A);
    cart.write(0xA000, 0xBE);
    assert_eq!(cart.read(0xA000), 0xBE);
}

#[test]
fn load_mbc5_ram_battery_has_ram_and_battery_flag() {
    let mut rom = min_rom(0x1B);
    rom[0x0149] = 0x02;
    let check = rom[0x0134..0x014D]
        .iter()
        .fold(0u8, |c, &b| c.wrapping_sub(b).wrapping_sub(1));
    rom[0x014D] = check;

    let cart = gb_core::cart::load(rom).expect("MBC5+RAM+BATTERY devia carregar");
    assert!(cart.ram_data().is_some());
}

#[test]
fn load_mbc5_rumble_creates_cartridge_without_ram() {
    let rom = min_rom(0x1C);
    let cart = gb_core::cart::load(rom).expect("MBC5+RUMBLE devia carregar");
    assert!(cart.ram_data().is_none());
}

#[test]
fn load_mbc5_rumble_ram_has_external_ram() {
    let mut rom = min_rom(0x1D);
    rom[0x0149] = 0x02;
    let check = rom[0x0134..0x014D]
        .iter()
        .fold(0u8, |c, &b| c.wrapping_sub(b).wrapping_sub(1));
    rom[0x014D] = check;

    let mut cart = gb_core::cart::load(rom).expect("MBC5+RUMBLE+RAM devia carregar");
    cart.write(0x0000, 0x0A);
    cart.write(0xA000, 0xEF);
    assert_eq!(cart.read(0xA000), 0xEF);
}

#[test]
fn load_mbc5_rumble_ram_battery_has_battery() {
    let mut rom = min_rom(0x1E);
    rom[0x0149] = 0x02;
    let check = rom[0x0134..0x014D]
        .iter()
        .fold(0u8, |c, &b| c.wrapping_sub(b).wrapping_sub(1));
    rom[0x014D] = check;

    let cart = gb_core::cart::load(rom).expect("MBC5+RUMBLE+RAM+BATTERY devia carregar");
    assert!(cart.ram_data().is_some());
}

// ── Rumble é ignorado ──────────────────────────────────────────────────

#[test]
fn rumble_bit_does_not_affect_ram_banking() {
    // 16 RAM banks para que bank 8 e bank 0 sejam físicos distintos (sem wrap).
    let mut mbc = rom_with_ram(2, 16);

    mbc.write(0x0000, 0x0A);

    mbc.write(0x4000, 0x00);
    mbc.write(0xA000, 0xAA);

    mbc.write(0x4000, 0x08);
    mbc.write(0xA000, 0xBB);

    mbc.write(0x4000, 0x00);
    assert_eq!(mbc.read(0xA000), 0xAA);

    mbc.write(0x4000, 0x08);
    assert_eq!(mbc.read(0xA000), 0xBB);
}

// ── ROM bank 0 é banco 0 (divergência de todos os MBCs anteriores) ─────

#[test]
fn bank_zero_set_explicitly_is_still_bank_zero_unlike_mbc3() {
    let mut mbc = rom_with_banks(4);
    let addr: u16 = 0x5000;

    mbc.write(0x2100, 0x03);
    assert_eq!(mbc.read(addr), rom_byte(3, addr));

    mbc.write(0x2100, 0x00);
    assert_eq!(mbc.read(addr), rom_byte(0, addr));

    mbc.write(0x2100, 0x01);
    assert_eq!(mbc.read(addr), rom_byte(1, addr));
}
