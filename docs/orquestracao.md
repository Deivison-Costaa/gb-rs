# Registro de orquestração

> Como o trabalho foi **conduzido**, em oposição ao que foi construído. O doc de
> iteração registra decisões tomadas *dentro* de uma iteração; este registra as
> tomadas *entre* elas, que não têm outro lugar. Material do relatório final
> (ROADMAP 8.1).
>
> Uma entrada por decisão, em ordem cronológica. Cada uma diz o que mediu — e
> quando não mediu nada, diz isso também.

---

## 2026-07-26 — O agente passa a orquestrar, uma iteração por vez

**Antes:** `./scripts/loop.sh N` encadeava N iterações sem supervisão. A cadeia
de 10 de 25/07 fechou 9; a décima morreu num 429 de limite de sessão às 01:27 e
ninguém soube por dez minutos.

**Agora:** uma sessão de Claude Code orquestra, chamando `./scripts/loop.sh 1`
por vez e verificando entre elas. A tentativa de trocar `claude -p` por
subagente foi **abandonada**: subagente não devolve custo, turnos e duração, e
essa série é material do relatório. O ganho da orquestração é a verificação
entre iterações; o custo é que o orquestrador ocupa uma sessão.

**Mediu:** nada de qualidade — é mudança de processo. Mediu o buraco abaixo.

## 2026-07-26 — A métrica perdia justamente a iteração que falha

`scripts/loop.sh` grava `logs/metrics.csv` **depois** do teste de código de
saída, então iteração que falha não deixa linha. Duas falharam nesta sessão e
custaram **US$ 8,26** que o CSV não conhecia: o 429 (US$ 4,37) e uma
interrupção manual (US$ 3,89).

**Decisão:** não mexer no `loop.sh` — o orquestrador lê o `logs/iter-*.json`
mais recente **sempre**, deu certo ou não, e grava em `logs/metrics-orq.csv`
com colunas a mais (`resultado`, `head_antes`, `head_depois`, `log`, `modelo`).
Patch no script exigiria teste comportamental com `claude` falso e repositório
git de mentira — custo desproporcional a uma flag, e o teste do
`scoreboard.sh` é o precedente de como teria de ser feito.

**Em aberto:** `logs/metrics.csv` segue com o buraco. Quem ler só ele vê um
projeto onde nada falha.

## 2026-07-26 — Numeração de iteração: sufixo, nunca o inteiro seguinte

O PR #20 (limpeza de comentários, executado por uma sessão de Kimi K3 no
OpenCode) usou a branch `iter/0018-densidade-r7` sem gerar doc nem commit
`iter 0018:`. O número ficou reservado por um trabalho que o registro não tinha.

**Custo medido:** a sessão seguinte criou `iter/0019-cpu-ld16-stack`, voltou,
criou `iter/0018-cpu-ld-16bit-stack`, e abandonou as duas sem um commit — ~7
minutos gastos adivinhando o próprio número.

**Decisão:** trabalho fora da escada do ROADMAP usa **sufixo** (`0017b`), como
o PR #6 já fazia com `iter/0004b`. O PR #20 ganhou doc retroativo em
`docs/iterations/0017b-densidade-r7.md`, marcado como reconstrução a partir do
git — os campos que dependiam de ter observado o processo ficaram como *não
observados*, em vez de preenchidos por inferência.

## 2026-07-26 — `gh pr checks --watch` é uma corrida latente no protocolo

O passo 10 do `SKILL.md` manda `gh pr create` seguido de `gh pr checks --watch`.
O `--watch` **não espera** os runs aparecerem: sai com exit 1 e a mensagem
`no checks reported`. Derrubou o merge do PR #23.

**Decisão:** esperar o registro antes —
`for i in $(seq 1 30); do gh pr checks >/dev/null 2>&1 && break; sleep 10; done`.
No #23 apareceram na segunda tentativa (~10 s).

**Nota:** as iterações anteriores escaparam por acaso, por gastarem segundos
entre um comando e outro. A CI dispara em todo `pull_request`, sem filtro de
path — PR só de docs também roda checks.

## 2026-07-26 — Comparação Opus 5 × Sonnet 5

**Método:** trocar só o modelo. Mesmo prompt, mesmas ferramentas, mesmo
`--max-turns`, `STATUS.md` intacto. Comparar contra as **vizinhas imediatas**,
nunca contra a média histórica: o custo por iteração sobe com o tamanho do
projeto (US$ 4,44 na primeira da série, US$ 8,59 na décima sexta), então a média
faria o modelo novo ganhar por um motivo que não é ele.

**Linha de base:** 16 execuções, todas `claude-opus-5[1m]`. 14 fecharam, média
**US$ 6,73**, 66 turnos, 14,3 min. As 2 que falharam não falharam por código.

**Primeira em Sonnet 5 (0023, ROADMAP 1.6b):**

| | Opus 0020 | Opus 0022 | Sonnet 0023 |
|---|---|---|---|
| custo | 6,85 | 8,59 | **5,96** |
| turnos | 75 | 72 | **87** |
| leitura de cache | 7,5M | 9,9M | **13,1M** |
| custo por turno | 0,091 | 0,119 | **0,068** |

Por turno, 43% mais barato. De ponta a ponta, 31% contra a vizinha — porque
precisou de 21% mais turnos e leu 31% mais contexto. **O preço promocional do
Sonnet 5 não está valendo:** reconstruindo o custo a partir dos tokens, US$
5,9554 calculados às taxas cheias contra US$ 5,9565 cobrados.

**Qualidade — o que sobreviveu:** registrou três erros de primeira tentativa e
aplicou a nota 41 corretamente, escrevendo que o "nenhum de conta" *mede o aviso
da 0022, não uma descoberta*. Achou um erro processual que o Opus não achou (a
lista `decoded_elsewhere` duplicada em nove arquivos, o nono com nome de
variável divergente, quase escapando do `grep`). Carregou adiante o achado da
vizinha, escrevendo o teste que troca a memória entre os dois `step`.

**Qualidade — o que caiu:** não rodou bateria de mutação. Primeira em sete sem
uma (a série Opus vinha 9/9, 10/10, 11/11, 15/15, 16/16). Justificou pela
`alu_from_hl` já corrigida na 0022 — argumento que cobre *timing* e não cobre as
flags novas (`N` literal, `H` como empréstimo, `SBC` consumindo o carry de
entrada no nibble), onde havia mutante fácil e ninguém escreveu.

**Segunda em Sonnet 5 (0024, ROADMAP 1.6c):** US$ 4,94, 81 turnos, 9 min. A
mais barata da sessão. Também sem bateria de mutação, e também sem erro de
hardware — mas o 1.6c é o sub-item mais fácil do grupo (nas lógicas as flags são
constantes), então "nenhum erro" ali não distingue modelo nenhum.

**Veredito, com 2 iterações de cada lado:**

| | Opus (0020, 0021r, 0022) | Sonnet (0023, 0024) |
|---|---|---|
| custo médio | US$ 7,25 | **US$ 5,45** |
| turnos | 70 | 84 |
| bateria de mutação | 3 de 3 | **0 de 2** |

**Decisão: fica o Sonnet 5** — 25% mais barato, e a única regressão consistente
tem causa identificada que não é o modelo.

**Achado de fundo, e é o mais importante desta comparação:** a bateria de
mutação **não está no `SKILL.md` nem no `CLAUDE.md`**. Existe só como prática
registrada no `STATUS.md`, 31 menções. É costume, não regra — o Opus a herdava
por osmose ao ler aquilo, o Sonnet leu a mesma coisa e tratou como o que ela
formalmente é. **Um processo que depende de o modelo inferir o rigor a partir de
prosa não é um processo; é sorte com testemunha.** A bateria virou passo
explícito do protocolo neste mesmo PR, e a próxima medição justa é "Sonnet com a
bateria escrita" — não "Sonnet contra Opus".

## 2026-07-26 — O orçamento de contexto, medido

~150k tokens por turno. De onde vem:

| | tokens | fatia |
|---|---|---|
| arquivos de teste | ~61k | ~41% |
| `STATUS.md` | 27k | 18% |
| spec do passo 3 | 12k | 8% |
| fonte | 13k | 9% |
| `ROADMAP.md` + `CLAUDE.md` + skill | 7k | 5% |

Os testes cresceram **5.189 → 7.761 linhas em oito iterações** (+50%). O
`STATUS.md` tinha 1.458 linhas, das quais **19 eram estado atual** — 96% era
história no caminho quente.

**Decisões que saíram daqui,** cada uma medida separadamente para não confundir
uma com a outra:

1. Cortar o `STATUS.md` (índice fica, corpo vai para `docs/notas.md` e
   `docs/invariantes.md`). Estimado em ~US$ 0,65/iteração, ~11%.
2. Consolidar a lista `decoded_elsewhere`, hoje duplicada em nove arquivos de
   teste — item de ROADMAP com teste próprio, não remendo.

**Risco assumido no corte 1:** as notas são o que faz as iterações boas — a 0020
disse que suas três armadilhas "vinham pré-anunciadas no `STATUS.md`". A aposta
é que o mecanismo que entrega não é a seção de notas, e sim o parágrafo
**Próxima tarefa**, que é denso e **cita as notas por número**. O índice mantém
a descoberta; o parágrafo mantém o handoff. Se a qualidade cair, a bateria de
mutação mede.
