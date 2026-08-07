# Iteração 0088 — Gamepad XInput no `gb-desktop`, com hotplug

- **Data:** 2026-08-07
- **Item do roadmap:** 4.5

## Objetivo

Jogar de controle: `gb-desktop` passa a ler gamepads no padrão XInput via
`gilrs`, com o controle podendo ser conectado ou removido a qualquer momento.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Joypad Input (P1/JOYP) | `docs/reference/09-joypad-serial.md` |
| gilrs 0.11.2 | `ev::Button`, `ev::Axis`, filtros de `next_event` | crate (não é hardware do DMG) |

O lado Game Boy já estava pronto desde a 4.1 — esta iteração não toca em
`gb-core`. R1 não se aplica ao mapeamento de host: gamepad não é periférico do
DMG, é I/O de frontend, e portanto mora inteiro em `gb-desktop` (R3).

## Mapeamento

Por **posição física**, não por rótulo: o A do Game Boy fica à direita, então
casa com o botão leste do pad (B do Xbox, círculo do DualShock), e o B do Game
Boy com o botão sul (A do Xbox, xis do DualShock). É a convenção do RetroArch e
a que preserva a memória muscular de quem jogou no aparelho. Mapear por rótulo
(A→A) inverteria os dois botões em relação ao console real.

| Game Boy | Gamepad (gilrs) | Xbox |
|---|---|---|
| A | `Button::East` | B |
| B | `Button::South` | A |
| Start | `Button::Start` | Start/Menu |
| Select | `Button::Select` | Back/View |
| D-pad | `Button::DPad*` + stick esquerdo | D-pad + stick |

Norte, oeste, gatilhos e `Mode` ficam sem função — o DMG tem oito teclas e só.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que os testes podiam montar um `EventType` completo para exercitar a tradução | `gilrs::ev::Code` tem campo `pub(crate)`: é inconstruível fora da crate | Não compilou. A API foi refeita em duas camadas — `eixo`/`botao` puros, testáveis, e um adaptador fino sobre `EventType` |
| 2 | API-Rust | Que `let ... && let ...` (let-chains) estava disponível | Let-chains estabilizaram no 1.88; o workspace declara `rust-version = 1.85` | Pego antes de escrever, ao conferir a MSRV — a CI usa `stable` e teria passado, escondendo a quebra da MSRV declarada |
| 3 | timing | Que um limiar único bastava para o stick analógico | Stick parado na borda do limiar chacoalha a direção a cada amostra | Escrito como teste (`stick_tem_histerese_entre_pressionar_e_soltar`) antes da implementação: pressiona em 0.5, solta em 0.35 |

## Placar

Inalterado — nenhuma ROM da bateria lê gamepad, e `gb-core` não foi tocado.

| Suíte | Antes | Depois |
|---|---|---|
| total | 18/121 | 18/121 |

## Decisões de arquitetura

- **A tradução é pura; o gilrs fica na borda.** `Traducao` não conhece `Gilrs`,
  só `Button`/`Axis`/`EventType`. É o que permitiu 16 testes sem hardware.
- **Contagem por fonte, não por tecla.** Stick e hat podem pedir a mesma
  direção; soltar um não pode soltar o que o outro segura. Daí os quatro slots
  de eixo mais a máscara de botões, e o `segurada()` consultado antes de emitir.
- **`Connected`/`Disconnected` soltam as oito teclas.** Controle arrancado com
  a direita pressionada deixaria o Mario correndo para sempre. `key_up` não
  levanta interrupção de joypad (`gb-core/src/joypad.rs`), então soltar tudo é
  barato e idempotente.
- **Falha de inicialização não é fatal.** Sem `/dev/input` acessível, o
  `Gilrs::new()` falha, o erro vai para o stderr e o teclado continua.

## Notas

Com os filtros padrão o `next_event` do gilrs já converte hat em `Button::DPad*`
(`axis_dpad_to_button`), então o tratamento de `Axis::DPadX/DPadY` é um caminho
reserva para quando os filtros estiverem desligados. Ele é testado; só não é o
que roda num pad XInput típico.

**O que não foi verificado:** nada disto foi exercitado com controle físico.
A máquina não tinha gamepad plugado e `/dev/uinput` é root-only, então não deu
para injetar um pad virtual. O que está provado é a tradução (16 testes) e que
o emulador roda normalmente com o gilrs inicializado e nenhum controle
conectado. O primeiro teste com pad de verdade ainda é dívida — se o botão
leste não cair no A, o suspeito é a linha `Button::East` de `map_button`.

Esta iteração saiu da fila: a próxima acionável era a 2.4b. Foi pedido direto do
usuário, e está registrado em `docs/orquestracao.md`.
