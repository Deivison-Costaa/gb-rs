use gb_core::cart::{Cartridge, Mbc3, OPEN_BUS};

const KIB: usize = 1024;
const BANK_LEN: usize = 16 * KIB;

fn rom_with_banks(num_banks: usize) -> Mbc3 {
    let rom_len = num_banks * BANK_LEN;
    let mut rom = vec![0_u8; rom_len];
    for (i, byte) in rom.iter_mut().enumerate() {
        let bank = i / BANK_LEN;
        *byte = ((bank as u8) << 4) | (i & 0x0F) as u8;
    }
    Mbc3::new(rom, num_banks, 0).expect("rom do fixture cabe num MBC3")
}

fn rom_with_ram(num_banks: usize, ram_banks: usize) -> Mbc3 {
    let rom_len = num_banks * BANK_LEN;
    let mut rom = vec![0_u8; rom_len];
    for (i, byte) in rom.iter_mut().enumerate() {
        let bank = i / BANK_LEN;
        *byte = ((bank as u8) << 4) | (i & 0x0F) as u8;
    }
    let ram_bytes = ram_banks * 8 * KIB;
    Mbc3::new(rom, num_banks, ram_bytes).expect("rom do fixture cabe num MBC3")
}

fn rom_2_banks() -> Mbc3 {
    rom_with_banks(2)
}

fn rom_byte(bank: usize, addr: u16) -> u8 {
    ((bank as u8) << 4) | (addr as u8 & 0x0F)
}

fn rom_with_rtc(num_banks: usize) -> Mbc3 {
    let mbc = rom_with_banks(num_banks);
    mbc.with_rtc()
}

fn rom_with_rtc_and_ram(num_banks: usize, ram_banks: usize) -> Mbc3 {
    let mbc = rom_with_ram(num_banks, ram_banks);
    mbc.with_rtc()
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
fn rom_bank_register_is_7_bits() {
    let mut mbc = rom_with_banks(3);
    mbc.write(0x2100, 0x82);
    let addr: u16 = 0x5000;
    assert_eq!(mbc.read(addr), rom_byte(2, addr));
}

#[test]
fn bank_zero_is_not_accessible_from_4000_7fff() {
    let mbc = rom_with_banks(2);
    let addr: u16 = 0x4123;
    assert_ne!(mbc.read(addr), rom_byte(0, addr));
}

#[test]
fn bank_twenty_is_directly_accessible_unlike_mbc1() {
    let mbc = rom_with_banks(33);
    let addr: u16 = 0x4123;
    assert_eq!(mbc.read(addr), rom_byte(1, addr));
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

    mbc.write(0x4000, 0x04);
    assert_eq!(mbc.read(0xA000), 0x77);
}

#[test]
fn ram_bank_selection_masks_to_three_bits() {
    let mut mbc = rom_with_ram(2, 4);

    mbc.write(0x0000, 0x0A);

    mbc.write(0x4000, 0x08);
    mbc.write(0xA000, 0x0B);

    mbc.write(0x4000, 0x00);
    mbc.write(0xA000, 0xAA);

    assert_eq!(mbc.read(0xA000), 0xAA);
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
fn load_mbc3_creates_cartridge_without_ram() {
    let rom = min_rom(0x11);
    let cart = gb_core::cart::load(rom).expect("MBC3 devia carregar");
    assert_eq!(cart.read(0x0100), 0x00);
}

#[test]
fn load_mbc3_ram_has_external_ram() {
    let mut rom = min_rom(0x12);
    rom[0x0149] = 0x02;
    let check = rom[0x0134..0x014D]
        .iter()
        .fold(0u8, |c, &b| c.wrapping_sub(b).wrapping_sub(1));
    rom[0x014D] = check;

    let mut cart = gb_core::cart::load(rom).expect("MBC3+RAM devia carregar");
    cart.write(0x0000, 0x0A);
    cart.write(0xA000, 0xBE);
    assert_eq!(cart.read(0xA000), 0xBE);
}

#[test]
fn load_mbc3_ram_battery_has_ram_and_battery_flag() {
    let mut rom = min_rom(0x13);
    rom[0x0149] = 0x02;
    let check = rom[0x0134..0x014D]
        .iter()
        .fold(0u8, |c, &b| c.wrapping_sub(b).wrapping_sub(1));
    rom[0x014D] = check;

    let cart = gb_core::cart::load(rom).expect("MBC3+RAM+BATTERY devia carregar");
    assert!(cart.ram_data().is_some());
}

#[test]
fn load_mbc3_timer_battery_has_no_external_ram() {
    let rom = min_rom(0x0F);
    let cart = gb_core::cart::load(rom).expect("MBC3+TIMER+BATTERY devia carregar");
    assert!(cart.ram_data().is_none());
}

#[test]
fn load_mbc3_timer_ram_battery_has_ram() {
    let mut rom = min_rom(0x10);
    rom[0x0149] = 0x02;
    let check = rom[0x0134..0x014D]
        .iter()
        .fold(0u8, |c, &b| c.wrapping_sub(b).wrapping_sub(1));
    rom[0x014D] = check;

    let cart = gb_core::cart::load(rom).expect("MBC3+TIMER+RAM+BATTERY devia carregar");
    assert!(cart.ram_data().is_some());
}

// ── RTC ────────────────────────────────────────────────────────────────

#[test]
fn rtc_registers_require_ram_enabled() {
    let mbc = rom_with_rtc(2);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
}

#[test]
fn rtc_select_08_reads_and_writes_seconds() {
    let mut mbc = rom_with_rtc(2);

    mbc.write(0x0000, 0x0A);
    mbc.write(0x4000, 0x08);

    mbc.write(0xA000, 0x2A);
    assert_eq!(mbc.read(0xA000), 0x2A);
}

#[test]
fn rtc_all_five_registers_08_to_0c_are_accessible() {
    let mut mbc = rom_with_rtc(2);

    mbc.write(0x0000, 0x0A);

    for reg in 0x08..=0x0C {
        mbc.write(0x4000, reg);
        let value = reg.wrapping_mul(7);
        mbc.write(0xA000, value);
    }

    for reg in 0x08..=0x0C {
        mbc.write(0x4000, reg);
        assert_eq!(mbc.read(0xA000), reg.wrapping_mul(7));
    }
}

#[test]
fn rtc_values_persist_when_select_changes() {
    let mut mbc = rom_with_rtc(2);

    mbc.write(0x0000, 0x0A);

    mbc.write(0x4000, 0x08);
    mbc.write(0xA000, 0x3B);

    mbc.write(0x4000, 0x09);
    mbc.write(0xA000, 0x17);

    mbc.write(0x4000, 0x08);
    assert_eq!(mbc.read(0xA000), 0x3B);
}

#[test]
fn rtc_day_high_bit_6_is_halt() {
    let mut mbc = rom_with_rtc(2);

    mbc.write(0x0000, 0x0A);
    mbc.write(0x4000, 0x0C);

    mbc.write(0xA000, 0x40);
    assert_eq!(mbc.read(0xA000), 0x40);

    mbc.write(0xA000, 0x00);
    assert_eq!(mbc.read(0xA000), 0x00);
}

#[test]
fn rtc_day_high_bit_7_is_day_carry() {
    let mut mbc = rom_with_rtc(2);

    mbc.write(0x0000, 0x0A);
    mbc.write(0x4000, 0x0C);

    mbc.write(0xA000, 0x80);
    assert_eq!(mbc.read(0xA000), 0x80);

    mbc.write(0xA000, 0x00);
    assert_eq!(mbc.read(0xA000), 0x00);
}

#[test]
fn rtc_day_high_bit_0_is_day_bit_8() {
    let mut mbc = rom_with_rtc(2);

    mbc.write(0x0000, 0x0A);
    mbc.write(0x4000, 0x0C);

    mbc.write(0xA000, 0x01);
    assert_eq!(mbc.read(0xA000), 0x01);
}

#[test]
fn rtc_writes_are_ignored_when_ram_disabled() {
    let mut mbc = rom_with_rtc(2);

    mbc.write(0x4000, 0x08);
    mbc.write(0xA000, 0x2A);

    mbc.write(0x0000, 0x0A);
    assert_eq!(mbc.read(0xA000), 0x00);
}

#[test]
fn rtc_accessible_even_without_external_ram() {
    let mut mbc = rom_with_rtc(2);

    mbc.write(0x0000, 0x0A);
    mbc.write(0x4000, 0x08);

    mbc.write(0xA000, 0x17);
    assert_eq!(mbc.read(0xA000), 0x17);
}

#[test]
fn rtc_ram_and_rtc_registers_are_independent() {
    let mut mbc = rom_with_rtc_and_ram(2, 4);

    mbc.write(0x0000, 0x0A);

    mbc.write(0x4000, 0x08);
    mbc.write(0xA000, 0x08);

    mbc.write(0x4000, 0x00);
    mbc.write(0xA000, 0x00);

    mbc.write(0x4000, 0x08);
    assert_eq!(mbc.read(0xA000), 0x08);

    mbc.write(0x4000, 0x00);
    assert_eq!(mbc.read(0xA000), 0x00);
}

#[test]
fn latch_writing_00_then_01_to_6000_is_accepted() {
    let mut mbc = rom_with_rtc(2);

    mbc.write(0x6000, 0x00);
    mbc.write(0x6000, 0x01);

    mbc.write(0x0000, 0x0A);
    mbc.write(0x4000, 0x08);
    assert_eq!(mbc.read(0xA000), 0x00);
}

#[test]
fn rtc_registers_not_mapped_for_non_rtc_types() {
    let mut mbc = rom_with_banks(2);

    mbc.write(0x0000, 0x0A);
    mbc.write(0x4000, 0x08);

    mbc.write(0xA000, 0x42);
    assert_eq!(mbc.read(0xA000), OPEN_BUS);
}

#[test]
fn writing_to_latch_range_does_not_corrupt_rtc_values() {
    let mut mbc = rom_with_rtc(2);

    mbc.write(0x0000, 0x0A);
    mbc.write(0x4000, 0x08);
    mbc.write(0xA000, 0x55);

    mbc.write(0x6000, 0x00);
    mbc.write(0x6000, 0x01);
    mbc.write(0x6000, 0xFF);

    mbc.write(0x4000, 0x08);
    assert_eq!(mbc.read(0xA000), 0x55);
}
