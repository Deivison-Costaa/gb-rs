//! ROADMAP 1.1 — o banco de registradores do SM83.

use gb_core::cpu::{Flag, Registers};

const FLAGS: [Flag; 4] = [Flag::Z, Flag::N, Flag::H, Flag::C];

fn with_f(value: u8) -> Registers {
    Registers {
        f: value,
        ..Default::default()
    }
}

#[test]
fn the_pair_puts_the_first_register_in_the_high_byte() {
    let regs = Registers {
        a: 0xDE,
        f: 0xF0,
        b: 0x12,
        c: 0x34,
        d: 0x56,
        e: 0x78,
        h: 0x9A,
        l: 0xBC,
        ..Default::default()
    };

    assert_eq!(regs.bc(), 0x1234, "BC = B no byte alto, C no baixo");
    assert_eq!(regs.de(), 0x5678, "DE = D no byte alto, E no baixo");
    assert_eq!(regs.hl(), 0x9ABC, "HL = H no byte alto, L no baixo");
    assert_eq!(regs.af(), 0xDEF0, "AF = A no byte alto, F no baixo");
}

#[test]
fn writing_the_pair_splits_it_into_the_two_halves() {
    let mut regs = Registers::default();

    regs.set_bc(0x1234);
    regs.set_de(0x5678);
    regs.set_hl(0x9ABC);
    regs.set_af(0xDEF0);

    assert_eq!((regs.b, regs.c), (0x12, 0x34), "BC divide em B e C");
    assert_eq!((regs.d, regs.e), (0x56, 0x78), "DE divide em D e E");
    assert_eq!((regs.h, regs.l), (0x9A, 0xBC), "HL divide em H e L");
    assert_eq!((regs.a, regs.f), (0xDE, 0xF0), "AF divide em A e F");
}

#[test]
fn the_pair_round_trips_every_byte_value() {
    for high in 0..=u8::MAX {
        for low in 0..=u8::MAX {
            let value = u16::from(high) << 8 | u16::from(low);
            let mut regs = Registers::default();

            regs.set_bc(value);
            assert_eq!(
                regs.bc(),
                value,
                "BC não sobreviveu ao ciclo com ${value:04X}"
            );
            assert_eq!(
                (regs.b, regs.c),
                (high, low),
                "BC dividiu errado ${value:04X}"
            );

            regs.set_hl(value);
            assert_eq!(
                regs.hl(),
                value,
                "HL não sobreviveu ao ciclo com ${value:04X}"
            );
        }
    }
}

#[test]
fn each_pair_is_independent_of_the_others() {
    let mut regs = Registers::default();
    regs.set_bc(0xFFFF);
    regs.set_de(0xFFFF);
    regs.set_hl(0xFFFF);
    regs.set_af(0xFFFF);

    regs.set_de(0x0000);

    assert_eq!(regs.bc(), 0xFFFF, "escrever DE não pode tocar em BC");
    assert_eq!(regs.hl(), 0xFFFF, "escrever DE não pode tocar em HL");
    assert_eq!(regs.af(), 0xFFFF, "escrever DE não pode tocar em AF");
    assert_eq!(regs.de(), 0x0000);
}

#[test]
fn sp_and_pc_are_sixteen_bit_and_have_no_halves() {
    let regs = Registers {
        sp: 0xFFFE,
        pc: 0x0100,
        ..Default::default()
    };

    assert_eq!(regs.sp, 0xFFFE);
    assert_eq!(regs.pc, 0x0100);
}

#[test]
fn each_flag_sits_on_the_bit_the_spec_assigns_it() {
    let cases = [
        (Flag::Z, 0b1000_0000u8),
        (Flag::N, 0b0100_0000),
        (Flag::H, 0b0010_0000),
        (Flag::C, 0b0001_0000),
    ];

    for (flag, expected) in cases {
        let mut regs = Registers::default();
        regs.set_flag(flag, true);
        assert_eq!(
            regs.f, expected,
            "{flag:?} sozinho deveria deixar F = ${expected:02X}, veio ${:02X}",
            regs.f
        );
    }
}

#[test]
fn reading_a_flag_reads_its_bit_out_of_f() {
    let regs = with_f(0b1010_0000); // Z e H setados, N e C limpos.

    assert!(regs.flag(Flag::Z), "bit 7 setado => Z");
    assert!(!regs.flag(Flag::N), "bit 6 limpo => N limpo");
    assert!(regs.flag(Flag::H), "bit 5 setado => H");
    assert!(!regs.flag(Flag::C), "bit 4 limpo => C limpo");
}

#[test]
fn setting_a_flag_leaves_the_other_three_alone() {
    for target in FLAGS {
        let mut regs = Registers::default();
        for flag in FLAGS {
            regs.set_flag(flag, true);
        }

        regs.set_flag(target, false);

        assert!(!regs.flag(target), "{target:?} deveria ter sido limpo");
        for other in FLAGS.into_iter().filter(|f| *f != target) {
            assert!(
                regs.flag(other),
                "limpar {target:?} não pode mexer em {other:?}"
            );
        }
    }
}

#[test]
fn setting_a_flag_twice_is_the_same_as_setting_it_once() {
    for flag in FLAGS {
        let mut regs = Registers::default();

        regs.set_flag(flag, true);
        let once = regs.f;
        regs.set_flag(flag, true);

        assert_eq!(regs.f, once, "setar {flag:?} duas vezes mudou F");

        regs.set_flag(flag, false);
        let cleared = regs.f;
        regs.set_flag(flag, false);

        assert_eq!(regs.f, cleared, "limpar {flag:?} duas vezes mudou F");
    }
}

#[test]
fn the_flags_ignore_the_bits_below_bit_four() {
    let regs = with_f(0b0000_1111);

    for flag in FLAGS {
        assert!(
            !regs.flag(flag),
            "{flag:?} não pode sair do nibble baixo de F"
        );
    }
}

#[test]
fn f_keeps_the_bits_the_spec_does_not_describe() {
    let mut regs = Registers::default();

    regs.set_af(0x120F);

    assert_eq!(regs.f, 0x0F, "1.1 não mascara F: a spec não pede");
    assert_eq!(regs.af(), 0x120F, "o que entrou em AF tem de sair igual");
}

#[test]
fn f_round_trips_all_eight_bits() {
    for value in 0..=u8::MAX {
        let regs = with_f(value);
        assert_eq!(regs.af() as u8, value, "F perdeu bits em ${value:02X}");
    }
}

#[test]
fn the_default_is_zeroed_and_is_not_the_post_boot_state() {
    let regs = Registers::default();

    assert_eq!(regs.af(), 0x0000);
    assert_eq!(regs.bc(), 0x0000);
    assert_eq!(regs.de(), 0x0000);
    assert_eq!(regs.hl(), 0x0000);
    assert_eq!(regs.sp, 0x0000);
    assert_eq!(regs.pc, 0x0000, "PC pós-boot é $0100, e isso é o 1.2");
}
