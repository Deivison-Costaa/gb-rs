# Iteração 0079 — Saída de áudio via cpal no gb-desktop

- **Data:** 2026-07-27
- **Item do roadmap:** 6.7b

## Objetivo

Conectar o buffer de áudio do `Apu` (já exposto via `Bus::audio_samples()` / `consume_audio_samples()`) a um dispositivo de áudio real usando `cpal` no `gb-desktop`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| crates.io | cpal 0.15 API | `Cargo.toml` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | — | — | — |

Sem erros de hardware nesta iteração — o trabalho foi puramente de integração com biblioteca Rust (`cpal`), não de comportamento de Game Boy.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| instr_timing | 1/1 | 1/1 |
| mem_timing | 4/4 | 4/4 |
| mem_timing-2 | 0/4 | 0/4 |
| halt_bug | 0/1 | 0/1 |
| oam_bug | 0/9 | 0/9 |
| interrupt_time | 0/1 | 0/1 |
| dmg_sound | 0/13 | 0/13 |
| dmg-acid2 | 1/1 | 1/1 |
| mooneye acceptance | 0/66 | 0/66 |
| mooneye (outros modelos) | 0/9 | 0/9 |

Testes do workspace: **886** (eram 881 — 5 novos em `gb-desktop`).

## Revisão cruzada (segundo modelo)

- **Modelo:** não disponível
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

**Buffer compartilhado via `Arc<Mutex<VecDeque<(f32, f32)>>>`.** O `Bus` é `&mut` no laço principal do `winit`; o callback do `cpal` roda em thread separada. A sincronização é feita drenando o buffer do `Apu` ao fim de cada frame (via `drain_bus_audio`) e empurrando as amostras para uma `VecDeque` protegida por `Mutex`. O callback do `cpal` consome dessa fila.

**Dispositivo de áudio ausente não é erro fatal.** `open_audio_stream` retorna `Option<cpal::Stream>`: se não houver dispositivo padrão ou a configuração falhar, o emulador roda sem som (avisa via `eprintln!`).

**Dreno condicional.** Se `has_audio` for `false`, `drain_bus_audio` não é chamado, evitando acúmulo infinito de amostras no buffer compartilhado.

**Callback preenche silêncio quando o buffer está vazio.** Se a fila esvaziar (underrun), o callback escreve `0.0` em todas as amostras da fatia, em vez de deixar ruído.

## Bateria de mutação

| Mutação | Tipo | Veredito |
|---|---|---|
| Remover `consume_audio_samples` | erro | FALHOU |
| Remover `push_back` no buffer | erro | FALHOU |
| Capacidade do buffer 16384→32 | controle | PASSOU |
| Inverter condição `available == 0` | erro | FALHOU |

**3/3 pegos, 1/1 controles verdes.**

A callback do `cpal` (dentro de `open_audio_stream`) sobrevive a qualquer mutação nos testes unitários — o código que interage com o dispositivo de áudio só é exercitado em runtime com hardware real. É uma lacuna de cobertura conhecida e inerente à dependência de dispositivo físico.

## Notas

O `gb-desktop` agora tem 5 testes de áudio (3 de buffer + 2 de integração com `Bus`/`Cpu`). A callback do `cpal` não é testável sem dispositivo real, mas o contrato é simples: consumir da `VecDeque` e preencher a fatia de saída.

A iteração 0078 entregou o downsampler e ring buffer no `gb-core`; esta iteração fecha o ciclo conectando ao hardware de áudio. O próximo passo (6.8 — blargg `dmg_sound`) depende de a saída de áudio estar funcionando para verificação auditiva, mas as ROMs podem ser testadas headless pelo `gb-cli` (que lê a porta serial, não o áudio).
