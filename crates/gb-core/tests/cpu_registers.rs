//! ROADMAP 1.1 — o banco de registradores do SM83.
//!
//! Spec: `docs/reference/02-cpu.md` § CPU registers and flags (Pan Docs
//! `fe246067b695`). A tabela inteira que rege este arquivo cabe aqui:
//!
//! > | 16-bit | Hi | Lo | Name/Function |
//! > | AF | A | - | Accumulator & Flags |
//! > | BC | B | C | BC |
//! > | DE | D | E | DE |
//! > | HL | H | L | HL |
//! > | SP | - | - | Stack Pointer |
//! > | PC | - | - | Program Counter/Pointer |
//!
//! e, para `F`:
//!
//! > | Bit | Name | Explanation |
//! > | 7 | z | Zero flag |
//! > | 6 | n | Subtraction flag (BCD) |
//! > | 5 | h | Half Carry flag (BCD) |
//! > | 4 | c | Carry flag |
//!
//! **O que a spec não diz é tão importante quanto o que ela diz.** A tabela
//! acima para no bit 4: sobre os bits 3–0 de `F` não há uma linha no Pan Docs
//! inteiro do commit fixado. Ver `f_keeps_the_bits_the_spec_does_not_describe`
//! e o doc da iteração 0009 — o teste que *não* está aqui é uma decisão
//! registrada, não um esquecimento.
//!
//! `unwrap`/`expect` são permitidos aqui: a R6 proíbe fora de teste.

use gb_core::cpu::{Flag, Registers};

/// Os quatro flags, na ordem em que a spec os lista (bit 7 → bit 4).
const FLAGS: [Flag; 4] = [Flag::Z, Flag::N, Flag::H, Flag::C];

/// Registradores zerados com `F` posto no valor dado.
///
/// Atalho dos testes de flag, que precisam montar um `F` cru e olhar o que os
/// acessores fazem com ele.
fn with_f(value: u8) -> Registers {
    Registers {
        f: value,
        ..Default::default()
    }
}

// --- pares de 8/16 bits ---------------------------------------------------

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

    // `B` é o Hi de `BC` — trocar a ordem dá `$3412`, que é o erro que este
    // teste existe para pegar.
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
    // Varre os 256 valores de cada metade: um `>> 8` trocado por `>> 4`, ou um
    // `as u8` que trunca o lado errado, sobrevive a um punhado de exemplos
    // escolhidos a dedo mas não sobrevive à varredura.
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
    // A tabela da spec deixa Hi e Lo de SP e PC como `-`: eles não se dividem
    // em registradores de 8 bits endereçáveis.
    let regs = Registers {
        sp: 0xFFFE,
        pc: 0x0100,
        ..Default::default()
    };

    assert_eq!(regs.sp, 0xFFFE);
    assert_eq!(regs.pc, 0x0100);
}

// --- flags ----------------------------------------------------------------

#[test]
fn each_flag_sits_on_the_bit_the_spec_assigns_it() {
    // O teste ancorado na spec: Z=7, N=6, H=5, C=4. Trocar dois flags de lugar
    // é o erro clássico de portar Z80 (onde o bit 7 é `S`, de sinal), e passa
    // despercebido por qualquer teste que só faça ida e volta pelos acessores.
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
    // Um `^=` no lugar de um `|=` passa no teste de "setar funciona" e falha
    // aqui — é a mutação mais barata de escrever e a mais fácil de não notar.
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
    // A spec não descreve os bits 3–0. Sejam eles o que forem, nenhum flag
    // mora lá — quem ler `Flag::C` num F com o nibble baixo cheio tem de
    // receber `false`.
    let regs = with_f(0b0000_1111);

    for flag in FLAGS {
        assert!(
            !regs.flag(flag),
            "{flag:?} não pode sair do nibble baixo de F"
        );
    }
}

// --- o nibble que a spec não descreve -------------------------------------

#[test]
fn f_keeps_the_bits_the_spec_does_not_describe() {
    // **Decisão registrada da iteração 0009.**
    //
    // O folclore diz que o nibble baixo de `F` é sempre zero, e que `POP AF`
    // o mascara. Isso pode até ser verdade no silício, mas **não está no Pan
    // Docs** do commit fixado: a tabela de flags para no bit 4, e a string
    // `POP AF` não aparece em nenhum dos 75 arquivos daquele commit. Pela R1,
    // o que não está na spec não vira código.
    //
    // Então o 1.1 não mascara nada, e este teste fixa a ausência para que a
    // máscara não entre depois sem alguém decidir que ela entra. O dia em que
    // a blargg `cpu_instrs/01-special` reprovar por causa disto é o dia em que
    // existe *evidência* para trazer a máscara — com a fonte junto.
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

// --- estado inicial -------------------------------------------------------

#[test]
fn the_default_is_zeroed_and_is_not_the_post_boot_state() {
    // Guarda de regressão, não medição: afirma uma *ausência*. O estado
    // pós-boot (A=$01, F=Z…, SP=$FFFE, PC=$0100 no DMG) é do ROADMAP 1.2, e
    // `docs/reference/01-memory-map.md` § Console state after boot ROM
    // hand-off é onde ele está escrito. Se alguém o adiantar para cá, este
    // teste avisa — e a conversa é sobre qual item da escada faz isso.
    let regs = Registers::default();

    assert_eq!(regs.af(), 0x0000);
    assert_eq!(regs.bc(), 0x0000);
    assert_eq!(regs.de(), 0x0000);
    assert_eq!(regs.hl(), 0x0000);
    assert_eq!(regs.sp, 0x0000);
    assert_eq!(regs.pc, 0x0000, "PC pós-boot é $0100, e isso é o 1.2");
}
