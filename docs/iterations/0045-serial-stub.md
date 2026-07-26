# Iteração 0045 — stub da porta serial ($FF01/$FF02)

- **Data:** 2026-07-26
- **Item do roadmap:** 1.12

## Objetivo

Criar o módulo `serial.rs` dono de SB ($FF01) e SC ($FF02), rotear leitura/escrita no
`Bus`, e expor uma fila de saída (`take_serial_output`) para o `gb-cli` consumir.
Escrever `$81` em SC com clock interno dispara a transferência do byte em SB.
Sem temporização: SC.7 é limpo imediatamente após a transferência.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Serial Data Transfer (Link Cable) | `docs/reference/09-joypad-serial.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que SB e SC seriam tratados como os outros registradores de I/O (byte cru no array `io`), sem lógica de transferência. Bastaria expor leitura/escrita. | O byte em SB é transmitido quando SC recebe `$81` com clock interno; SC.7 é limpo ao fim da transferência. Sem a lógica de transferência, as ROMs blargg jamais produziriam saída — o `gb-cli` ficaria mudo. | Leitura da spec durante o passo 3; a descrição do STATUS.md já antecipava o mecanismo ("escrever `$81` em `SC` dispara e o byte em `SB` sai"). |
| 2 | flags/timing | Que a interrupção serial (IF.3) precisava ser implementada neste item para as ROMs funcionarem. | As ROMs blargg tipicamente fazem polling de SC.7, não dependem da interrupção serial. A interrupção é do M2 (2.2). | Leitura da spec: o Pan Docs menciona a interrupção como **notificação**, mas o polling de SC.7 é suficiente para o fluxo básico. |
| 3 | API-Rust | Que os valores iniciais de SB/SC viriam do módulo `boot` (como os demais registradores). `boot` é `mod boot` privado dentro de `bus`, inacessível de `serial.rs` no crate root. | — (não é questão de spec, é de visibilidade Rust). | Erro de compilação ao tentar `use crate::bus::boot`. Resolvido com constantes locais `SB_INITIAL`/`SC_INITIAL`. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |
| Todas as demais | 0/* | 0/* |

Testes do workspace: **577** (eram **568** antes da 0044 — +9 do novo arquivo `serial_stub`).

## Revisão cruzada (segundo modelo)

- **Modelo:** —
- **Achados:** 0
- **Procedentes:** 0
- **Falso positivo mais interessante:** —

O `scripts/review.sh` ainda não está configurado (nota 5).

## Decisões de arquitetura

1. **`Serial` é módulo no crate root, não sub-módulo de `bus`.** O `STATUS.md` descreve `serial.rs` como par de `bus.rs`, e a visibilidade entre crate root e `bus` é mais limpa com `mod serial` em `lib.rs` do que com `pub(super) mod serial` dentro de `bus/mod.rs`.

2. **SC mascara bits 6–2 (`value & 0x83`).** A spec só documenta bits 7, 1 e 0 de SC; bits 6–2 não têm função no DMG. A máscara evita que escritas espúrias nesses bits afetem o estado. A decisão inversa (não mascarar) também seria defensável — o teste `sc_masks_bits_6_through_2_to_zero` fixa a escolha e avisa se alguém mudar.

3. **Gatilho exige borda de subida do bit 7.** O trigger só dispara quando `(value & 0x81) == 0x81` **e** `SC.7` estava em 0 antes da escrita. Isso evita que uma segunda escrita de `$81` enquanto o bit 7 ainda está setado (ex.: após `$80` com clock externo) produza uma transferência espúria. O teste `writing_81_when_bit_7_is_already_set_does_not_trigger_again` guarda essa escolha.

4. **`take_serial_output` consome a fila.** `Vec<u8>` em vez de `&[u8]`: o `gb-cli` drena a saída a cada frame (ou a cada `max_cycles`) e imprime em stdout. Consumir evita reimpressão.

## Notas

A bateria de mutação revelou dois buracos na primeira versão da suíte de testes (7 testes iniciais):
- **M1 (sem rising edge):** o teste `external_clock_does_not_trigger_output_even_with_bit_7_set` escrevia `$80` e verificava ausência de saída, mas não testava o que acontece ao escrever `$81` em seguida. Adicionado `writing_81_when_bit_7_is_already_set_does_not_trigger_again`.
- **M5 (sem máscara em SC):** o teste `sc_starts_at_7e_and_is_readable_writable` escrevia `$01` e verificava `$01` — indistinguível com ou sem máscara. Adicionado `sc_masks_bits_6_through_2_to_zero`, que escreve `$FF` via pré-carga de `$80` (evitando trigger) e verifica `$83`.

Placar final da bateria: **6/6 pegos, 2/2 controles verdes**.

A máscara `$83` em SC interage com o teste `sc_starts_at_7e_and_is_readable_writable`: o valor inicial é `$7E` (setado diretamente em `Serial::new`, sem passar pela máscara), mas escritas subsequentes são mascaradas. Isso é intencional — o boot state é o que a spec dá, e a máscara só se aplica a writes do software.
