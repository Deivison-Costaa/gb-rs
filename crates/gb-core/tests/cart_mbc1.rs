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

fn rom_with_full_bank_mark(nbanks: usize, ram_bytes: usize) -> Mbc1 {
    let rom_len = nbanks * BANK_LEN;
    let mut rom = vec![0u8; rom_len];
    for (i, byte) in rom.iter_mut().enumerate() {
        *byte = (i / BANK_LEN) as u8;
    }
    Mbc1::new(rom, nbanks, ram_bytes).expect("ROM do fixture cabe num MBC1")
}

fn rom_64_banks() -> Mbc1 {
    rom_with_full_bank_mark(64, 0)
}

// ---------------------------------------------------------------------------
// Banking: secondary register (4000-5FFF) como upper bits do ROM bank
// ---------------------------------------------------------------------------

#[test]
fn effective_rom_bank_combines_secondary_register_for_region_4000_7fff() {
    let mut mbc = rom_64_banks();
    mbc.write(0x4000, 0x01);
    mbc.write(0x2100, 0x01);
    let addr: u16 = 0x5000;
    assert_eq!(
        mbc.read(addr),
        0x21,
        "banco efetivo = (secondary<<5)|rom_bank = 32|1 = 33 = 0x21"
    );
}

#[test]
fn secondary_register_with_rom_bank_zero_selects_bank_one_plus_upper_bits() {
    let mut mbc = rom_64_banks();
    mbc.write(0x4000, 0x01);
    mbc.write(0x2100, 0x00);
    let addr: u16 = 0x5000;
    assert_eq!(
        mbc.read(addr),
        0x21,
        "rom_bank=0 vira 1; efetivo = (1<<5)|1 = 33 = 0x21"
    );
}

#[test]
fn bank_mask_caps_effective_bank_to_rom_size() {
    let mut mbc = rom_with_full_bank_mark(8, 0);
    mbc.write(0x2100, 0x07);
    mbc.write(0x4000, 0x01);
    let addr: u16 = 0x5000;
    assert_eq!(
        mbc.read(addr),
        0x07,
        "8 bancos (128 KiB): máscara de 3 bits descarta o bit 5 do secondary"
    );
}

// ---------------------------------------------------------------------------
// Mode 1: 0000-3FFF afetado pelo secondary register
// ---------------------------------------------------------------------------

#[test]
fn mode_0_locks_0000_3fff_to_bank_zero() {
    let mut mbc = rom_64_banks();
    mbc.write(0x4000, 0x01);
    mbc.write(0x2100, 0x03);
    assert_eq!(
        mbc.read(0x0150),
        0x00,
        "modo 0 prende 0000-3FFF ao banco 0 independente do secondary"
    );
}

#[test]
fn mode_1_applies_secondary_bank_to_region_0000_3fff() {
    let mut mbc = rom_64_banks();
    mbc.write(0x6000, 0x01);
    mbc.write(0x4000, 0x01);
    assert_eq!(
        mbc.read(0x0150),
        0x20,
        "modo 1 com secondary=1 expõe banco 0x20 em 0000-3FFF"
    );
}

#[test]
fn switching_secondary_in_mode_1_changes_both_regions() {
    let mut mbc = rom_with_full_bank_mark(128, 0);
    mbc.write(0x6000, 0x01);
    mbc.write(0x2100, 0x01);
    mbc.write(0x4000, 0x02);
    assert_eq!(
        mbc.read(0x0150),
        0x40,
        "secondary=2: banco 0x40 em 0000-3FFF"
    );
    assert_eq!(
        mbc.read(0x5000),
        0x41,
        "secondary=2 com rom_bank=1: banco 0x41 em 4000-7FFF"
    );
}

// ---------------------------------------------------------------------------
// RAM banking com modo 0/1
// ---------------------------------------------------------------------------

const RAM_LEN_16K: usize = 16 * KIB;

#[test]
fn mode_0_locks_ram_to_bank_zero() {
    let mut mbc = rom_with_banks(2, RAM_LEN_16K);
    mbc.write(0x0000, 0x0A);
    mbc.write(0x4000, 0x01);
    mbc.write(0xA000, 0x77);
    mbc.write(0x4000, 0x00);
    mbc.write(0xA000, 0xCC);
    assert_eq!(
        mbc.read(0xA000),
        0xCC,
        "modo 0: secondary=1 nāo troca de banco RAM; a segunda escrita sobrescreve"
    );
}

#[test]
fn mode_1_allows_ram_bank_switching() {
    let mut mbc = rom_with_banks(2, RAM_LEN_16K);
    mbc.write(0x0000, 0x0A);
    mbc.write(0x6000, 0x01);
    mbc.write(0x4000, 0x00);
    mbc.write(0xA000, 0x42);
    mbc.write(0x4000, 0x01);
    mbc.write(0xA000, 0x99);
    mbc.write(0x4000, 0x00);
    assert_eq!(
        mbc.read(0xA000),
        0x42,
        "banco 0 preservado após alternar para banco 1 e voltar"
    );
}

// ---------------------------------------------------------------------------
// Auto-rams nos asserts de erro
// ---------------------------------------------------------------------------

fn rom_size_code(banks: usize) -> u8 {
    match banks {
        2 => 0x00,
        4 => 0x01,
        8 => 0x02,
        16 => 0x03,
        32 => 0x04,
        64 => 0x05,
        _ => 0x00,
    }
}

fn rom_mbc1_ram_battery(rom_banks: usize, ram_size: usize) -> Vec<u8> {
    let mut rom = vec![0u8; rom_banks * BANK_LEN];
    if rom.len() >= 0x0150 {
        rom[0x0147] = 0x03;
        rom[0x0148] = rom_size_code(rom_banks);
        rom[0x0149] = if ram_size == 0 {
            0x00
        } else if ram_size == 8 * KIB {
            0x02
        } else if ram_size == 32 * KIB {
            0x03
        } else {
            0x02
        };
    }
    rom
}

#[test]
fn load_mbc1_ram_battery_creates_ram() {
    let rom = rom_mbc1_ram_battery(8, 8 * KIB);
    let mut cart = gb_core::cart::load(rom).expect("MBC1+RAM+BATTERY deve montar");
    cart.write(0x0000, 0x0A);
    cart.write(0xA000, 0x55);
    assert_eq!(cart.read(0xA000), 0x55);
}
