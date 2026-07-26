# Iteração 0010 — decodificação de endereço e RAM interna do `Bus`

- **Data:** 2026-07-25
- **Item do roadmap:** 1.2a (sub-item criado nesta iteração, ver *Decisões*)
- **PR:** #12
- **Duração:** ~40min
- **Custo reportado:** não medido — sessão interativa, sem `--output-format json` (`STATUS.md`, nota 10)
- **Turnos:** 1

## Objetivo

Traduzir os 65536 endereços do Game Boy nas faixas da § Memory Map e atender as
que este item cobre: WRAM, echo RAM, HRAM, região proibida e o roteamento das
duas janelas do cartucho.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | § Memory Map (a tabela de 12 linhas) | `docs/reference/01-memory-map.md` |
| Pan Docs | § Echo RAM | `docs/reference/01-memory-map.md` |
| Pan Docs | § FEA0–FEFF range | `docs/reference/01-memory-map.md` |
| Pan Docs | § Console state after boot ROM hand-off → *Common remarks* (RAM não inicializada) | `docs/reference/01-memory-map.md` |

## Erros de primeira tentativa

O procedimento desta vez foi diferente das anteriores, e a diferença é o dado:
em vez de anotar de cabeça o que eu *teria* escrito, **escrevi mesmo** — um
esqueleto descartável com a versão de memória — e rodei a suíte contra ele.
O RED deixou de ser uma impressão e virou uma lista de nomes de teste.
Resultado da execução: **9 passaram, 3 falharam**; depois de corrigir o teste do
erro #3, 9 passaram e 4 falharam.

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | `$FEA0`–`$FEFF` é "não usável", logo lê `$FF` — o reflexo de tratar região proibida como barramento aberto | § FEA0–FEFF range: `$FF` **só quando a OAM está bloqueada**; no DMG, *"reads otherwise return `$00`"*. O `$FF` está preso ao modo da PPU, que não existe até o M3 — sem PPU não há bloqueio, logo `$00` | `the_not_usable_range_reads_zero_while_there_is_no_oam_blocking`, contra o esqueleto |
| 2 | endereçamento | HRAM = `$FF80`–`$FFFF`, 128 bytes | A tabela dá `$FF80`–`$FFFE` à HRAM (**127** bytes) e uma linha própria a `$FFFF` (`IE`). O byte a mais não é RAM, é um registrador de interrupções | `every_address_decodes_to_the_region_the_pandocs_table_gives_it`, contra o esqueleto |
| 3 | teste | o teste que escrevi *para* o erro #2 procurava **aliasing**: escrever em `$FFFF` e ver se `$FFFE` mudava | Uma HRAM de 128 bytes não pisa em `$FFFE` — ela dá índice 127 ao endereço anexado. O teste **passou verde contra o esqueleto errado** e só a varredura de regiões pegou o erro | eu mesmo, lendo qual teste falhou e qual não; corrigido antes da implementação (`high_ram_is_127_bytes_and_stops_before_the_ie_register` agora afirma a região, não a colisão) |
| 4 | endereçamento | "echo RAM é a WRAM espelhada" | § Memory Map: *"mirror of C000–DDFF"*. O espelho é **512 bytes mais curto que a fonte**; `$DE00`–`$DFFF` não têm endereço de echo. A implementação de memória (`& 0x1FFF`) acerta por construção — o **enunciado** é que estava errado, e um teste escrito a partir dele teria afirmado espelho onde não há | não foi pego pelo esqueleto: o `STATUS.md` da 0009 já tinha deixado o aviso escrito, e o teste nasceu certo. Registrado porque a versão de memória era falsa mesmo tendo produzido código certo |

O erro #3 é o mais útil dos quatro. Ele é a **nota 8 do `STATUS.md` com o sinal
trocado mais uma vez**: não é o guarda que passa por vacuidade (0001), nem o
vermelho pelo motivo errado (0003) — é o guarda que falha em pegar exatamente o
erro para o qual foi escrito, porque mirou no sintoma (colisão de índice) em vez
de na afirmação (`$FFFF` não pertence à HRAM). Só apareceu porque havia um
esqueleto errado contra o qual medir. Sem ele, a suíte teria ficado 13/13 verde
com um teste inútil no meio, e ninguém saberia.

## Bateria de mutação

11 mutantes em `crates/gb-core/src/bus.rs`, com `mtime` explícito (nota 14) e
contagem de casamento conferida antes de aplicar (nota 18).

| Resultado | Contagem |
|---|---|
| Mutantes pegos | **9/9** |
| Controles negativos que ficaram verdes | **2/2** |

Os nove: região proibida lendo `$FF`; HRAM de 128 bytes engolindo o `IE`;
máscara da WRAM de 4 KiB; echo não lendo a WRAM; echo não escrevendo na WRAM;
VRAM roteada para o cartucho; escrita em ROM não chegando ao mapeador; fronteira
OAM/região-proibida deslocada em um byte; RAM interna começando em `$FF`.

Os dois controles: `& WRAM_ADDRESS_MASK` trocado por `% (WRAM_ADDRESS_MASK + 1)`
e o braço do `$FFFF` movido para o topo do `match` — mudanças de texto sem
mudança de comportamento, e a suíte não reclamou de nenhuma. Vale a ressalva que
a 0007 já registrou: **os mutantes foram escritos por quem escreveu os testes, na
mesma sessão**, então 9/9 autoriza dizer que os nove modos de falha imaginados
doem, e nada além disso.

## Placar

Nenhuma suíte mudou — ainda não há emulador, `gb-cli run` continua saindo `2`.

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |
| todas as demais | 0/109 | 0/109 |
| **TOTAL** | **0/121** | **0/121** |

Testes do workspace: **98 → 111** (13 novos em `tests/bus_memory_map.rs`).
121 ROMs medidas pelo `scoreboard.sh`, então não houve queda para o fallback da
nota 17.

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
  (`STATUS.md`, nota 5), e o `loop.sh` tem a revisão cruzada desligada por padrão
  desde `b8e3a8f`.
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

**O 1.2 foi quebrado em 1.2a e 1.2b.** O item juntava duas coisas com specs
diferentes: quem responde a cada endereço (§ Memory Map) e com que valores a
máquina começa (§ Console state after boot ROM hand-off). Esta iteração fez só a
primeira. A quebra está commitada separada (`02a5bd7`), antes de qualquer código.

**O `Bus` é `struct`, não `trait`** — contra a letra do item, a favor do
`CLAUDE.md` § Arquitetura, que diz que o `Bus` é o dono de tudo e que os
componentes recebem `&mut Bus`. Um trait com um único implementador poria vtable
no caminho mais quente de um emulador sem comprar nada; se o 1.3 quiser memória
plana para testar opcodes sem cartucho, extrair a interface então é mudança
local. A divergência está escrita no próprio ROADMAP, não só aqui.

**`Region` é um tipo público, separado do `Bus`.** O mapa de memória é um fato
sobre o hardware, não sobre o que já está implementado: `$8000` é VRAM hoje,
mesmo sem PPU para atender. Separá-lo torna a tabela testável contra a spec
endereço por endereço, independentemente de qual componente existe — e é o que
permitiu o teste que pegou o erro #2. O `match` de `Region::of` é total **sem**
`_ =>`, para que faixa nova quebre a compilação em vez de cair num ramo genérico.

**Regiões sem dono leem `OPEN_BUS` e engolem escrita, com teste fixando isso.**
Não é uma afirmação sobre o hardware — é a ausência de componente ligado, e o
teste existe para que seja decisão visível em vez de lacuna a descobrir depurando
a PPU. Pânico seria pior: `read` de emulador é o último lugar onde se quer achar
erro de roteamento (mesma escolha do `NoMbc`, 0.4).

**RAM interna começa zerada, e isso é escolha, não hardware.** A spec diz que
WRAM e HRAM são aleatórias ao ligar e que os emuladores divergem — constante
(`$00` ou `$FF`) ou sorteio. Constante é o que dá teste reprodutível; o teste
nomeia a escolha para que ela quebre se mudar.

## Notas

O cartucho-espião do teste (`SpyCartridge` + `Rc<SpyLog>`) foi o que permitiu
medir **roteamento** em vez de conteúdo. Com um `NoMbc` de verdade não daria:
fora das suas janelas ele responde `$FF`, e as regiões sem dono também — não há
como separar "o `Bus` roteou e o cartucho não tinha nada" de "o `Bus` não
roteou". O marcador `$C7` existe só para quebrar esse empate, e é ele que faz
`only_the_two_cartridge_windows_reach_the_cartridge_on_reads` afirmar as duas
metades de uma vez: nenhuma janela do cartucho ficou de fora, nenhum endereço de
fora dele entrou.

O `the_transcribed_table_covers_the_address_space_exactly_once` não testa o
código, testa a **fixture**: se a transcrição da tabela tivesse buraco, a
varredura seguinte simplesmente não visitaria os endereços faltantes e ficaria
verde tendo medido menos do que anuncia. É a nota 8 aplicada ao dado do teste, e
não ao alvo dele.

MSRV conferida à mão outra vez (nota 13): `cargo +1.85 test --all` deu 111/111
em `cargo 1.85.1`. Continua sendo um ponto de dado que depende de alguém lembrar,
não uma guarda — a nota segue aberta.
