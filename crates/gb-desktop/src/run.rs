use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use gb_core::bus::Bus;
use gb_core::cart::{self, CartridgeHeader};
use gb_core::cpu::Cpu;
use gb_core::joypad::Key;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const SCREEN_W: u32 = 160;
const SCREEN_H: u32 = 144;
const SCALE: u32 = 4;

const M_CYCLES_PER_FRAME: u32 = 17556;

const FRAMEBUFFER_PALETTE: [[u8; 4]; 4] = [
    [0xE0, 0xF8, 0xD0, 0xFF],
    [0x88, 0xC0, 0x70, 0xFF],
    [0x34, 0x68, 0x56, 0xFF],
    [0x08, 0x18, 0x20, 0xFF],
];

type AudioBuffer = Arc<Mutex<VecDeque<(f32, f32)>>>;

fn create_audio_buffer() -> AudioBuffer {
    Arc::new(Mutex::new(VecDeque::with_capacity(16384)))
}

fn drain_bus_audio(bus: &mut Bus, buffer: &AudioBuffer) {
    let mut buf = buffer.lock().expect("audio buffer mutex poisoned");
    // Em laço porque a fatia é contígua: com o anel na volta, esvaziar tudo no
    // mesmo frame custa duas passadas.
    loop {
        let available = bus.audio_samples_available();
        if available == 0 {
            return;
        }
        let samples: Vec<(f32, f32)> = bus.audio_samples()[..available].to_vec();
        bus.consume_audio_samples(available);
        for sample in samples {
            buf.push_back(sample);
        }
    }
}

fn open_audio_stream(buffer: AudioBuffer) -> Option<cpal::Stream> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;
    let config = match device.default_output_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("configuração de áudio indisponível: {e}");
            return None;
        }
    };
    let channels = config.channels() as usize;
    let stream = match device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut buf = buffer.lock().expect("audio buffer mutex poisoned");
            for frame in data.chunks_mut(channels) {
                match buf.pop_front() {
                    Some((left, right)) => {
                        if !frame.is_empty() {
                            frame[0] = left;
                        }
                        if frame.len() >= 2 {
                            frame[1] = right;
                        }
                        for sample in frame.iter_mut().skip(2) {
                            *sample = (left + right) * 0.5;
                        }
                    }
                    None => {
                        for sample in frame.iter_mut() {
                            *sample = 0.0;
                        }
                    }
                }
            }
        },
        |err| eprintln!("erro na stream de áudio: {err}"),
        None,
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("não consegui abrir stream de áudio: {e}");
            return None;
        }
    };
    if stream.play().is_err() {
        eprintln!("não consegui iniciar stream de áudio");
        return None;
    }
    Some(stream)
}

pub fn execute(path: &Path) {
    let rom = match std::fs::read(path) {
        Ok(rom) => rom,
        Err(error) => {
            eprintln!("não consegui ler {}: {error}", path.display());
            return;
        }
    };

    let checksum = match CartridgeHeader::parse(&rom) {
        Ok(header) => header.checksum(),
        Err(error) => {
            eprintln!("{}: {error}", path.display());
            return;
        }
    };

    let sav_path = path.with_extension("sav");
    let sav_data = std::fs::read(&sav_path).ok();

    let mut cartridge = match cart::load(rom) {
        Ok(cart) => cart,
        Err(error) => {
            eprintln!("{}: {error}", path.display());
            return;
        }
    };

    if let Some(ref data) = sav_data {
        cartridge.load_ram(data);
    }

    let mut bus = Bus::new(cartridge);
    let mut cpu = Cpu::after_boot_rom(checksum);

    let audio_buffer = create_audio_buffer();
    let stream = open_audio_stream(audio_buffer.clone());
    let has_audio = stream.is_some();

    let event_loop: EventLoop<()> = EventLoop::new();

    let window = match WindowBuilder::new()
        .with_title("gb-rs")
        .with_inner_size(LogicalSize::new(SCREEN_W * SCALE, SCREEN_H * SCALE))
        .build(&event_loop)
    {
        Ok(w) => w,
        Err(error) => {
            eprintln!("não consegui criar janela: {error}");
            return;
        }
    };

    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = match Pixels::new(SCREEN_W, SCREEN_H, surface_texture) {
        Ok(p) => p,
        Err(error) => {
            eprintln!("não consegui criar renderer: {error}");
            return;
        }
    };

    event_loop.run(move |event, _elwt, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(vk) = input.virtual_keycode {
                        let gb_key = map_key(vk);
                        match input.state {
                            ElementState::Pressed => {
                                if let Some(k) = gb_key {
                                    bus.key_down(k);
                                }
                            }
                            ElementState::Released => {
                                if let Some(k) = gb_key {
                                    bus.key_up(k);
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                for _ in 0..M_CYCLES_PER_FRAME {
                    if cpu.lockup().is_some() || cpu.is_stopped() {
                        break;
                    }
                    cpu.step(&mut bus);
                }

                if has_audio {
                    drain_bus_audio(&mut bus, &audio_buffer);
                }

                let frame = pixels.frame_mut();
                framebuffer_to_rgba(bus.framebuffer(), frame);
                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}

fn map_key(vk: VirtualKeyCode) -> Option<Key> {
    match vk {
        VirtualKeyCode::Right => Some(Key::Right),
        VirtualKeyCode::Left => Some(Key::Left),
        VirtualKeyCode::Up => Some(Key::Up),
        VirtualKeyCode::Down => Some(Key::Down),
        VirtualKeyCode::Z => Some(Key::A),
        VirtualKeyCode::X => Some(Key::B),
        VirtualKeyCode::Return => Some(Key::Start),
        VirtualKeyCode::RShift => Some(Key::Select),
        _ => None,
    }
}

fn framebuffer_to_rgba(fb: &[u8; (SCREEN_W * SCREEN_H) as usize], output: &mut [u8]) {
    for (i, &color) in fb.iter().enumerate() {
        let palette = FRAMEBUFFER_PALETTE
            .get(color as usize)
            .copied()
            .unwrap_or([0xFF, 0x00, 0xFF, 0xFF]);
        let idx = i * 4;
        let end = idx + 4;
        if end <= output.len() {
            output[idx..end].copy_from_slice(&palette);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_key_seta_direita_mapeia_para_key_right() {
        assert_eq!(map_key(VirtualKeyCode::Right), Some(Key::Right));
    }

    #[test]
    fn map_key_seta_esquerda_mapeia_para_key_left() {
        assert_eq!(map_key(VirtualKeyCode::Left), Some(Key::Left));
    }

    #[test]
    fn map_key_seta_cima_mapeia_para_key_up() {
        assert_eq!(map_key(VirtualKeyCode::Up), Some(Key::Up));
    }

    #[test]
    fn map_key_seta_baixo_mapeia_para_key_down() {
        assert_eq!(map_key(VirtualKeyCode::Down), Some(Key::Down));
    }

    #[test]
    fn map_key_z_mapeia_para_key_a() {
        assert_eq!(map_key(VirtualKeyCode::Z), Some(Key::A));
    }

    #[test]
    fn map_key_x_mapeia_para_key_b() {
        assert_eq!(map_key(VirtualKeyCode::X), Some(Key::B));
    }

    #[test]
    fn map_key_enter_mapeia_para_key_start() {
        assert_eq!(map_key(VirtualKeyCode::Return), Some(Key::Start));
    }

    #[test]
    fn map_key_shift_direito_mapeia_para_key_select() {
        assert_eq!(map_key(VirtualKeyCode::RShift), Some(Key::Select));
    }

    #[test]
    fn map_key_tecla_nao_mapeada_retorna_none() {
        assert_eq!(map_key(VirtualKeyCode::A), None);
        assert_eq!(map_key(VirtualKeyCode::Space), None);
        assert_eq!(map_key(VirtualKeyCode::Escape), None);
    }

    #[test]
    fn framebuffer_to_rgba_converte_zero_para_branco_esverdeado() {
        let fb = [0u8; (SCREEN_W * SCREEN_H) as usize];
        let mut output = [0u8; (SCREEN_W * SCREEN_H * 4) as usize];
        framebuffer_to_rgba(&fb, &mut output);
        assert_eq!(&output[0..4], &[0xE0, 0xF8, 0xD0, 0xFF]);
    }

    #[test]
    fn framebuffer_to_rgba_converte_tres_para_preto_esverdeado() {
        let fb = [3u8; (SCREEN_W * SCREEN_H) as usize];
        let mut output = [0u8; (SCREEN_W * SCREEN_H * 4) as usize];
        framebuffer_to_rgba(&fb, &mut output);
        assert_eq!(&output[0..4], &[0x08, 0x18, 0x20, 0xFF]);
    }

    #[test]
    fn framebuffer_to_rgba_todos_os_pixels_sao_convertidos() {
        let fb = [1u8; (SCREEN_W * SCREEN_H) as usize];
        let mut output = [0u8; (SCREEN_W * SCREEN_H * 4) as usize];
        framebuffer_to_rgba(&fb, &mut output);
        for i in 0..(SCREEN_W * SCREEN_H) as usize {
            let idx = i * 4;
            assert_eq!(
                &output[idx..idx + 4],
                &[0x88, 0xC0, 0x70, 0xFF],
                "pixel {i} com valor 1 deveria ser verde médio"
            );
        }
    }

    #[test]
    fn framebuffer_to_rgba_cada_valor_dois_bits_tem_cor_diferente() {
        let mut fb = [0u8; (SCREEN_W * SCREEN_H) as usize];
        fb[0] = 0;
        fb[1] = 1;
        fb[2] = 2;
        fb[3] = 3;
        let mut output = [0u8; (SCREEN_W * SCREEN_H * 4) as usize];
        framebuffer_to_rgba(&fb, &mut output);
        assert_ne!(
            &output[0..4],
            &output[4..8],
            "cor 0 e 1 deveriam ser diferentes"
        );
        assert_ne!(
            &output[4..8],
            &output[8..12],
            "cor 1 e 2 deveriam ser diferentes"
        );
        assert_ne!(
            &output[8..12],
            &output[12..16],
            "cor 2 e 3 deveriam ser diferentes"
        );
    }

    #[test]
    fn audio_buffer_vazio_retorna_silencio() {
        let buf = create_audio_buffer();
        let mut frames = Vec::new();
        {
            let mut inner = buf.lock().expect("mutex válido");
            for _ in 0..4 {
                match inner.pop_front() {
                    Some(s) => frames.push(s),
                    None => frames.push((0.0, 0.0)),
                }
            }
        }
        assert_eq!(frames, vec![(0.0, 0.0); 4]);
    }

    #[test]
    fn audio_buffer_mantem_amostras_em_ordem() {
        let buf = create_audio_buffer();
        {
            let mut inner = buf.lock().expect("mutex válido");
            inner.push_back((1.0, -1.0));
            inner.push_back((0.5, 0.5));
        }
        {
            let mut inner = buf.lock().expect("mutex válido");
            assert_eq!(inner.pop_front(), Some((1.0, -1.0)));
            assert_eq!(inner.pop_front(), Some((0.5, 0.5)));
            assert_eq!(inner.pop_front(), None);
        }
    }

    #[test]
    fn drain_bus_audio_sem_amostras_nao_altera_buffer() {
        let rom = vec![0u8; 32 * 1024];
        let header = CartridgeHeader::parse(&rom).expect("header válido");
        let checksum = header.checksum();
        let cartridge = cart::load(rom).expect("cartucho válido");
        let mut bus = Bus::new(cartridge);
        let _cpu = Cpu::after_boot_rom(checksum);

        let buffer = create_audio_buffer();
        assert_eq!(bus.audio_samples_available(), 0);
        drain_bus_audio(&mut bus, &buffer);

        let inner = buffer.lock().expect("mutex válido");
        assert!(inner.is_empty());
    }

    #[test]
    fn drain_bus_audio_transfere_amostras_para_o_buffer() {
        let rom = vec![0u8; 32 * 1024];
        let header = CartridgeHeader::parse(&rom).expect("header válido");
        let checksum = header.checksum();
        let cartridge = cart::load(rom).expect("cartucho válido");
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::after_boot_rom(checksum);

        bus.write(0xFF26, 0x80);

        for _ in 0..100 {
            cpu.step(&mut bus);
        }

        let available = bus.audio_samples_available();
        assert!(available > 0, "deveria ter amostras de áudio após ciclos");

        let buffer = create_audio_buffer();
        drain_bus_audio(&mut bus, &buffer);

        let inner = buffer.lock().expect("mutex válido");
        assert_eq!(inner.len(), available);
    }

    #[test]
    fn consumir_amostras_zera_disponiveis_no_bus() {
        let rom = vec![0u8; 32 * 1024];
        let header = CartridgeHeader::parse(&rom).expect("header válido");
        let checksum = header.checksum();
        let cartridge = cart::load(rom).expect("cartucho válido");
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::after_boot_rom(checksum);

        bus.write(0xFF26, 0x80);

        for _ in 0..100 {
            cpu.step(&mut bus);
        }

        let available = bus.audio_samples_available();
        assert!(available > 0);

        let buffer = create_audio_buffer();
        drain_bus_audio(&mut bus, &buffer);

        assert_eq!(bus.audio_samples_available(), 0);
    }
}
