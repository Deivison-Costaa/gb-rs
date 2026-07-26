# Iteração 0008 — guarda do `fetch-test-roms.sh`

- **Data:** 2026-07-25
- **Item do roadmap:** 0.5
- **PR:** #10
- **Duração:** ~20min
- **Custo reportado:** não medido — sessão interativa, sem `--output-format json`.
  Oitava iteração seguida com essa dívida; ver nota 10 do `STATUS.md`.
- **Turnos:** 1

## Objetivo

Fechar o 0.5 como o `STATUS.md` mandava: **verificar** que
`scripts/fetch-test-roms.sh` cumpre o item em vez de reimplementá-lo, e cobrir
o buraco que a verificação achou — era o único dos quatro scripts do projeto
sem teste nenhum.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| — | — | — |

**Nenhuma.** O 0.5 é item de ferramental: não há comportamento de hardware
envolvido, então a R1 não morde aqui. As fontes de verdade desta iteração foram
o texto do próprio item ("baixa blargg, mooneye, dmg-acid2 para `tests/roms/`"),
a nota 6 do `STATUS.md` (`cgb_sound` é CGB e fica de fora) e o cabeçalho do
script, que promete fixação por tag + sha256 e no-op quando as ROMs já estão lá.
Cada uma dessas quatro promessas virou um teste.

## O que a verificação encontrou

O script cumpre o item **ao pé da letra**, e isso foi medido, não suposto:
121 ROMs em `tests/roms/`, distribuídas nas três suítes que o item nomeia
(blargg 45 + `halt_bug.gb`, mooneye/acceptance 75, dmg-acid2 1), carimbo
`v7.0`, `cgb_sound` ausente. Bate com o total do `scoreboard.csv` desde a 0001.

O que faltava era guarda. E o modo de falha que ela cobre é o pior tipo: um
`fetch` que entrega **menos** ROMs do que prometeu não pinta nada de vermelho.
O `scoreboard.sh` mede o que achou, sai `0` (a 0.2b só reprova zero linhas, não
"menos linhas que ontem"), e a série da apresentação passa a medir um universo
menor com exatamente a mesma cara. É a 0.2b um degrau acima.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era de fato | Como foi pego |
|---|---|---|---|---|
| 1 | `ferramental` | Que `curl -o out "file:///caminho/com espaço/x.zip"` funcionaria — o plano inteiro do teste hermético dependia disso. | curl 8.18 recusa **antes de tocar no disco**: `curl: (3) URL rejected: Malformed input to a URL function`. Só a forma percent-encoded passa. E o repositório mora sob `Área de trabalho/Programação com Agentes/`: seria falha garantida, e por causa do caminho do checkout, não do script. | Sondagem de 30s no shell antes de escrever o teste. Virou o `file_url()`, que codifica byte a byte. |
| 2 | `teste` | Que `download_bundle 2>/dev/null \|\| download_fallback` era mutação **equivalente** — só silencia stderr, não muda comportamento — e a usei como controle negativo. | Não é equivalente: engole a mensagem do `die`, e o teste do sha256 afirma a *mensagem*, não só o código de saída. O controle foi pego, e com razão — motivo da falha sumindo do log da CI **é** perda de comportamento. | A própria bateria, marcando `!!! INESPERADO`. Sem a expectativa declarada por escrito, eu teria lido "7/7 pegos" e comemorado. |
| 3 | `hardware` | Ao escrever o ponteiro da próxima tarefa no `STATUS.md`, escrevi de memória que "o nibble baixo de `F` é sempre zero no hardware" — como fato, para o 1.1 usar. | Pode até ser verdade, mas **não está em `docs/reference/`**: a tabela do § Flags Register lista só os bits 7–4 e não diz nada dos 3–0; `03-opcodes.md` (l. 289) descreve `POP AF` sem mencionar máscara. Numa iteração de ferramental, sem R1 morder, plantei folclore no arquivo que a próxima iteração lê como spec. | `grep` nos próprios `docs/reference/` antes de commitar — o reflexo que a R1 treina. Virou pergunta em aberto no ponteiro, não afirmação. |
| 4 | `teste` | Que o vermelho do RED valeria para os cinco testes. | Valeu para dois. Os outros três passaram: um por vacuidade (`ci_does_not_override_the_pinned_bundle`, guarda de futuro) e **dois medindo o bundle real baixado da internet** — o script ignorava a costura e ia buscar a release verdadeira, que por acaso satisfaz as asserções de poda e de idempotência. | O `-marker` no nome das ROMs falsas, posto de propósito. Sem ele, `fetch_lands_the_three_suites` teria ficado verde no RED e a suíte inteira estaria medindo a internet. |

**Sobre o #4 — o RED custou 100 segundos e três downloads de 3,6 MiB.** Depois
da costura, os mesmos cinco testes rodam em **0,10s** sem rede. A diferença não
é conforto: teste que depende de release do GitHub falha por motivo alheio ao
que mede, e falha justamente no dia em que a CI está com pressa.

**Sobre o #2 — controle negativo também precisa ser desenhado.** A nota 8 do
`STATUS.md` manda incluir controle para distinguir "suíte boa" de "suíte que
quebra com qualquer mudança". Ela não avisava que escolher mal o controle
produz o mesmo ruído do outro lado: eu chamei de equivalente algo que não era.
O procedimento que salvou foi declarar `should_pass` **antes** de rodar — a
tabela reprova a expectativa, não só o mutante.

## Bateria de mutação

Sete mutantes e três controles, no script (não em Rust — o teste lê o `.sh` em
tempo de execução, então a armadilha de mtime da nota 14 não se aplica aqui).
Cada padrão foi conferido por casar **exatamente uma vez** antes de aplicar.

| # | Mutação | Esperado | Resultado |
|---|---|---|---|
| M1 | remove a chamada de `prune_blargg` | pego | pego |
| M2 | `sha256sum -c \|\| die` vira `\|\| true` | pego | pego (`..._sha256_does_not_match_is_refused`) |
| M3 | mooneye não é movida para `$ROMS_DIR` | pego | pego |
| M4 | `dmg-acid2/*` sai dos padrões do `unzip` | pego | pego |
| M5 | `already_current` sempre falso | pego | pego |
| M6 | poda perde a exceção do `halt_bug.gb` | pego | pego |
| M7 | carimbo grava outra string | pego | pego |
| C1 | mensagem de `log` reescrita | verde | verde |
| C2 | `REQUIRED_BLARGG` reordenada | verde | verde |
| C3 | `download_bundle 2>/dev/null` | verde | **pego** — controle mal desenhado, ver erro #2 |

A primeira tentativa do M2 não chegou a rodar: o padrão casou
0 vezes porque a mensagem tem acento e eu havia escrito o literal sem ele. O
script imprimiu `PADRAO CASOU 0x — mutante invalido` em vez de dar um veredito
— que é o comportamento certo, e vale mais do que parece: um harness de mutação
que aplica silenciosamente "nada" e reporta verde é a nota 14 com outra roupa.

**7/7 mutantes pegos não quer dizer suíte completa.** Vale o de sempre (nota 8):
os mutantes saíram da mesma cabeça que escreveu os testes, na mesma sessão.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas | 0/121 | 0/121 |

Sem emulador, sem mudança — como esperado. O `scoreboard.csv` foi de 968 para
1089 linhas. O que esta iteração muda é a confiança no **denominador**: até
hoje o 121 era um número que ninguém defendia.

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum. `scripts/review.sh` continua sem `REVIEWER_CMD`
  configurado (nota 5 do `STATUS.md`).
- **Achados:** —
- **Procedentes:** —

## Decisões de arquitetura

**A costura de teste é explícita e nomeada para não virar configuração.**
`TEST_ROMS_BUNDLE_URL` / `TEST_ROMS_BUNDLE_SHA256` existem só para o teste
apontar o download para um zip local. Duas defesas, de propósito: (a) andam em
par — quem trocar só a URL esbarra no sha256 fixado e o script morre, que é
exatamente o que a fixação existe para fazer; (b)
`ci_does_not_override_the_pinned_bundle` reprova o dia em que o `ci.yml`
mencionar o prefixo. Abrir seam para teste é barato; abrir seam que a produção
possa usar por acidente é como se perde uma fixação por sha256 sem ninguém
perceber.

**O bundle falso é montado com `zip`, e o teste falha se ele não existir** —
não pula. Suíte que se desliga sozinha quando o ambiente encolhe é a vacuidade
da nota 8 com outro nome. O script já exige `curl`/`unzip`/`tar`/`sha256sum`.

## Notas

**O que a verificação achou e esta iteração deliberadamente NÃO consertou.**
O caminho de fallback do script (bundle fora do ar) **não baixa a mooneye** —
está escrito lá, com o motivo: não existe release pré-montada upstream, só o
fonte, que exige RGBDS. Só que ele avisa e sai **`0`**, e o `verify()` imprime a
contagem da mooneye sem nunca conferi-la. Resultado: numa execução de fallback,
a CI fica verde medindo 46 ROMs em vez de 121, e a única pista é uma linha de
`ATENÇÃO:` no meio do log.

Não consertei porque isso é decisão de projeto, não defeito: o script escolheu
explicitamente manter a CI viva em vez de derrubá-la quando a rede falha, e
inverter essa escolha é outro item, não um detalhe de guarda. Trocar por conta
própria seria contrabandear design dentro de uma iteração de verificação. Virou
a nota 17 do `STATUS.md`.

**Sobre o 0.5 ter ficado sete iterações desmarcado.** Ele estava pronto desde o
scaffold. O custo de não ter sido marcado é pequeno; o de ter sido marcado sem
olhar teria sido maior — a verificação é o que produziu os cinco testes e a
nota 17. Vale para o resto do ROADMAP: item entregue pelo scaffold merece uma
iteração de conferência, não um `[x]` de confiança.
