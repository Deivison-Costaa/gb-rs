//! spec: `docs/reference/09-joypad-serial.md` § Joypad Input (o lado Game Boy).
//! O porquê do mapeamento por posição: `docs/iterations/0088-gamepad-xinput.md`.

use gb_core::joypad::Key;
use gilrs::Gilrs;
use gilrs::ev::{Axis, Button, EventType};

const TECLAS: [Key; 8] = [
    Key::Right,
    Key::Left,
    Key::Up,
    Key::Down,
    Key::A,
    Key::B,
    Key::Start,
    Key::Select,
];

// Histerese: o stick parado na borda do limiar chacoalharia a direção a cada
// amostra se pressionar e soltar usassem o mesmo valor.
const LIMIAR_PRESSIONA: f32 = 0.5;
const LIMIAR_SOLTA: f32 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acao {
    Pressiona(Key),
    Solta(Key),
}

#[derive(Debug, Default)]
pub struct Traducao {
    eixos: [Option<Key>; 4],
    botoes: u8,
}

impl Traducao {
    pub fn traduz(&mut self, evento: &EventType, saida: &mut Vec<Acao>) {
        match *evento {
            EventType::ButtonPressed(botao, _) => self.botao(botao, true, saida),
            EventType::ButtonReleased(botao, _) => self.botao(botao, false, saida),
            EventType::AxisChanged(eixo, valor, _) => self.eixo(eixo, valor, saida),
            EventType::Connected | EventType::Disconnected => self.solta_tudo(saida),
            _ => {}
        }
    }

    fn botao(&mut self, botao: Button, pressionado: bool, saida: &mut Vec<Acao>) {
        let Some(key) = map_button(botao) else {
            return;
        };
        let bit = 1u8 << indice(key);

        if pressionado {
            let ja_segurada = self.segurada(key);
            self.botoes |= bit;
            if !ja_segurada {
                saida.push(Acao::Pressiona(key));
            }
        } else {
            self.botoes &= !bit;
            if !self.segurada(key) {
                saida.push(Acao::Solta(key));
            }
        }
    }

    fn eixo(&mut self, eixo: Axis, valor: f32, saida: &mut Vec<Acao>) {
        let Some((slot, negativa, positiva)) = slot_do_eixo(eixo) else {
            return;
        };

        let antes = self.eixos[slot];
        let depois = direcao(valor, antes, negativa, positiva);
        if antes == depois {
            return;
        }

        let novo_ja_segurado = depois.is_some_and(|key| self.segurada(key));
        self.eixos[slot] = depois;

        if let Some(key) = antes {
            if !self.segurada(key) {
                saida.push(Acao::Solta(key));
            }
        }
        if let Some(key) = depois {
            if !novo_ja_segurado {
                saida.push(Acao::Pressiona(key));
            }
        }
    }

    fn solta_tudo(&mut self, saida: &mut Vec<Acao>) {
        self.eixos = [None; 4];
        self.botoes = 0;
        saida.extend(TECLAS.map(Acao::Solta));
    }

    fn segurada(&self, key: Key) -> bool {
        self.botoes & (1u8 << indice(key)) != 0 || self.eixos.contains(&Some(key))
    }
}

pub struct Gamepad {
    gilrs: Option<Gilrs>,
    traducao: Traducao,
}

impl Gamepad {
    pub fn new() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(error) => {
                eprintln!("controle indisponível, seguindo só com o teclado: {error}");
                None
            }
        };

        Self {
            gilrs,
            traducao: Traducao::default(),
        }
    }

    pub fn poll(&mut self, saida: &mut Vec<Acao>) {
        let Some(gilrs) = self.gilrs.as_mut() else {
            return;
        };
        while let Some(evento) = gilrs.next_event() {
            self.traducao.traduz(&evento.event, saida);
        }
    }
}

fn map_button(botao: Button) -> Option<Key> {
    match botao {
        Button::East => Some(Key::A),
        Button::South => Some(Key::B),
        Button::Start => Some(Key::Start),
        Button::Select => Some(Key::Select),
        Button::DPadUp => Some(Key::Up),
        Button::DPadDown => Some(Key::Down),
        Button::DPadLeft => Some(Key::Left),
        Button::DPadRight => Some(Key::Right),
        _ => None,
    }
}

fn slot_do_eixo(eixo: Axis) -> Option<(usize, Key, Key)> {
    match eixo {
        Axis::LeftStickX => Some((0, Key::Left, Key::Right)),
        Axis::LeftStickY => Some((1, Key::Down, Key::Up)),
        Axis::DPadX => Some((2, Key::Left, Key::Right)),
        Axis::DPadY => Some((3, Key::Down, Key::Up)),
        _ => None,
    }
}

fn direcao(valor: f32, atual: Option<Key>, negativa: Key, positiva: Key) -> Option<Key> {
    let limiar = if atual.is_some() {
        LIMIAR_SOLTA
    } else {
        LIMIAR_PRESSIONA
    };

    if valor >= limiar {
        Some(positiva)
    } else if valor <= -limiar {
        Some(negativa)
    } else {
        None
    }
}

fn indice(key: Key) -> u32 {
    match key {
        Key::Right => 0,
        Key::Left => 1,
        Key::Up => 2,
        Key::Down => 3,
        Key::A => 4,
        Key::B => 5,
        Key::Start => 6,
        Key::Select => 7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gilrs::ev::{Axis, Button};

    fn eixo(estado: &mut Traducao, eixo: Axis, valor: f32) -> Vec<Acao> {
        let mut saida = Vec::new();
        estado.eixo(eixo, valor, &mut saida);
        saida
    }

    fn botao(estado: &mut Traducao, botao: Button, pressionado: bool) -> Vec<Acao> {
        let mut saida = Vec::new();
        estado.botao(botao, pressionado, &mut saida);
        saida
    }

    #[test]
    fn botao_leste_e_o_a_do_game_boy() {
        assert_eq!(map_button(Button::East), Some(Key::A));
    }

    #[test]
    fn botao_sul_e_o_b_do_game_boy() {
        assert_eq!(map_button(Button::South), Some(Key::B));
    }

    #[test]
    fn start_e_select_mapeiam_direto() {
        assert_eq!(map_button(Button::Start), Some(Key::Start));
        assert_eq!(map_button(Button::Select), Some(Key::Select));
    }

    #[test]
    fn dpad_mapeia_para_as_quatro_direcoes() {
        assert_eq!(map_button(Button::DPadUp), Some(Key::Up));
        assert_eq!(map_button(Button::DPadDown), Some(Key::Down));
        assert_eq!(map_button(Button::DPadLeft), Some(Key::Left));
        assert_eq!(map_button(Button::DPadRight), Some(Key::Right));
    }

    #[test]
    fn botoes_sem_equivalente_no_game_boy_sao_ignorados() {
        assert_eq!(map_button(Button::North), None);
        assert_eq!(map_button(Button::West), None);
        assert_eq!(map_button(Button::Mode), None);
        assert_eq!(map_button(Button::LeftTrigger), None);
    }

    #[test]
    fn botao_pressionado_e_solto_vira_par_de_acoes() {
        let mut estado = Traducao::default();

        assert_eq!(
            botao(&mut estado, Button::East, true),
            vec![Acao::Pressiona(Key::A)]
        );
        assert_eq!(
            botao(&mut estado, Button::East, false),
            vec![Acao::Solta(Key::A)]
        );
    }

    #[test]
    fn botao_sem_mapeamento_nao_gera_acao() {
        let mut estado = Traducao::default();
        assert!(botao(&mut estado, Button::North, true).is_empty());
    }

    #[test]
    fn stick_alem_do_limiar_pressiona_a_direcao() {
        let mut estado = Traducao::default();
        assert_eq!(
            eixo(&mut estado, Axis::LeftStickX, 0.8),
            vec![Acao::Pressiona(Key::Right)]
        );
    }

    #[test]
    fn stick_dentro_da_zona_morta_nao_gera_acao() {
        let mut estado = Traducao::default();
        assert!(eixo(&mut estado, Axis::LeftStickX, 0.2).is_empty());
    }

    #[test]
    fn stick_tem_histerese_entre_pressionar_e_soltar() {
        let mut estado = Traducao::default();
        eixo(&mut estado, Axis::LeftStickX, 0.8);

        assert!(
            eixo(&mut estado, Axis::LeftStickX, 0.4).is_empty(),
            "0.4 está acima do limiar de soltar"
        );
        assert_eq!(
            eixo(&mut estado, Axis::LeftStickX, 0.1),
            vec![Acao::Solta(Key::Right)]
        );
    }

    #[test]
    fn inverter_o_stick_solta_uma_direcao_e_pressiona_a_oposta() {
        let mut estado = Traducao::default();
        eixo(&mut estado, Axis::LeftStickX, 0.8);

        assert_eq!(
            eixo(&mut estado, Axis::LeftStickX, -0.9),
            vec![Acao::Solta(Key::Right), Acao::Pressiona(Key::Left)]
        );
    }

    #[test]
    fn eixo_y_positivo_e_para_cima() {
        let mut estado = Traducao::default();
        assert_eq!(
            eixo(&mut estado, Axis::LeftStickY, 0.9),
            vec![Acao::Pressiona(Key::Up)]
        );
        assert_eq!(
            eixo(&mut estado, Axis::LeftStickY, -0.9),
            vec![Acao::Solta(Key::Up), Acao::Pressiona(Key::Down)]
        );
    }

    #[test]
    fn dpad_analogico_tambem_vira_direcao() {
        let mut estado = Traducao::default();
        assert_eq!(
            eixo(&mut estado, Axis::DPadX, -1.0),
            vec![Acao::Pressiona(Key::Left)]
        );
    }

    #[test]
    fn eixo_sem_uso_no_game_boy_e_ignorado() {
        let mut estado = Traducao::default();
        assert!(eixo(&mut estado, Axis::RightStickX, 1.0).is_empty());
    }

    // O stick e o hat podem pedir a mesma direção: soltar um não pode soltar a
    // tecla que o outro ainda segura.
    #[test]
    fn soltar_uma_fonte_nao_solta_direcao_que_a_outra_segura() {
        let mut estado = Traducao::default();
        eixo(&mut estado, Axis::LeftStickX, 1.0);

        assert!(
            eixo(&mut estado, Axis::DPadX, 1.0).is_empty(),
            "a direita já estava pressionada pelo stick"
        );
        assert!(
            eixo(&mut estado, Axis::LeftStickX, 0.0).is_empty(),
            "o hat ainda segura a direita"
        );
        assert_eq!(
            eixo(&mut estado, Axis::DPadX, 0.0),
            vec![Acao::Solta(Key::Right)]
        );
    }

    #[test]
    fn hotplug_solta_todas_as_teclas() {
        let mut estado = Traducao::default();
        eixo(&mut estado, Axis::LeftStickX, 1.0);

        let mut acoes = Vec::new();
        estado.solta_tudo(&mut acoes);

        for key in TECLAS {
            assert!(
                acoes.contains(&Acao::Solta(key)),
                "{key:?} continuou pressionada depois do controle sair"
            );
        }
    }

    #[test]
    fn depois_do_hotplug_o_estado_dos_eixos_zera() {
        let mut estado = Traducao::default();
        eixo(&mut estado, Axis::LeftStickX, 1.0);
        estado.solta_tudo(&mut Vec::new());

        assert_eq!(
            eixo(&mut estado, Axis::LeftStickX, 1.0),
            vec![Acao::Pressiona(Key::Right)],
            "sem zerar os eixos, a direita ficaria surda até o stick voltar ao centro"
        );
    }
}
