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

## 2026-07-26 — O corte do `STATUS.md` não baixou o custo, e a previsão errada é o dado

Previsão: cortar 91 KB do `STATUS.md` (~23k tokens) baixaria o contexto por
turno, e essa parte seria **atribuível** porque a bateria de mutação mexeria em
turnos e saída, não em contexto por turno.

Medido, na primeira iteração sob as duas mudanças (0025): contexto por turno
**subiu** de 150k para 154k, e depois para 167k na 0027.

A premissa da atribuição estava errada. A bateria muta o fonte, roda a suíte,
reverte e repete — cada ciclo desses despeja saída de ferramenta no contexto, de
modo que ela infla **as duas** grandezas. Some-se que os testes cresceram de
5.189 para mais de 8.000 linhas no mesmo período, e o corte foi engolido.

**Conclusão que sobrevive:** o custo por iteração é função do tamanho do
repositório, não da escolha de modelo. A economia de trocar Opus 5 por Sonnet 5
foi consumida em menos de dez iterações. Nenhuma escolha de modelo muda essa
inclinação; só reduzir o que entra em contexto muda — e reduzir documentação não
basta quando o que cresce é o código de teste.

## 2026-07-26 — Série de abortos sem causa identificada

Seis execuções morreram sem entregar nada, **US$ 6,76**, todas com a mesma forma:
o processo é cortado de fora, `api_error_status` nulo, zero negações de
permissão, `stderr` vazio quando foi capturado.

| horário | razão | turnos | custo |
|---|---|---|---|
| 05:29 | `aborted_streaming` | 33 | 1,36 |
| 06:12 | `aborted_streaming` | 54 | 2,58 |
| 06:19 | `aborted_streaming` | 27 | 1,33 |
| 11:49 | `aborted_streaming` | 4 | 0,14 |
| 11:55 | `aborted_tools` | 7 | 0,05 |
| 11:56 | `aborted_streaming` | 35 | 1,30 |

**Eliminado por medição, não por opinião:** suspensão da máquina (journal ativo
durante as janelas; última suspensão em 25/07 17:47), OOM killer (nenhuma
ocorrência), `systemd-oomd` (serviço inativo), pressão de memória (PSI em 0,00),
rede (nenhum evento do NetworkManager nas janelas), processos órfãos acumulando
(só a sessão-mãe viva), e o `claude -p` em si (sonda trivial completa em 4,5 s).

**Não eliminado:** o mecanismo de tarefa em background do harness que hospeda o
orquestrador. Todas as execuções, boas e ruins, passaram por ele — nunca esteve
sob controle experimental. O teste que separa é rodar `./scripts/loop.sh 1` num
terminal fora da sessão.

**O que conteve o prejuízo** foi instrumentação criada antes, por outro motivo:
métrica gravada mesmo na falha (senão esses US$ 6,76 seriam invisíveis) e a
guarda de árvore limpa com preservação em commit `wip:` (nenhum aborto perdeu
trabalho).

## 2026-07-26 — O orquestrador violou a própria R1

Diagnostiquei a primeira dupla de abortos como "provavelmente a máquina
suspendendo", **sem medir**, e gravei isso na mensagem de um commit. Quatro
comandos depois a medição desmentiu: o journal registrou atividade contínua em
06:13, 06:17, 06:19 e 06:24.

É exatamente o erro que a **R1** existe para impedir — intuição confiante sobre
um mecanismo que se podia consultar — cometido na camada de processo, onde não
há regra escrita para impedir. O `CLAUDE.md` protege o agente contra achismo
sobre o SM83; ninguém protegia o orquestrador contra achismo sobre a máquina.

Corrigido na origem (a branch não tinha ido para o `origin`, então o commit foi
emendado em vez de deixar história falsa). Registrado aqui porque um projeto que
mede erro de primeira tentativa não pode esconder os do próprio condutor.

Correção adicional da mesma série: afirmei que os abortos estavam "acelerando"
com três pontos (54 → 27 → 4 turnos). O quarto e o sexto vieram com 7 e 35. Não
havia tendência; havia três pontos e uma reta imaginada.

## 2026-07-26 — Duas vezes o processo sobreviveu por redundância acidental

Padrão que só apareceu porque as duas coisas foram achadas no mesmo dia:

1. **Cobertura do controle negativo de decodificação** (achado da 0026): nenhum
   teste verificava a afirmação *positiva* de `decoded_elsewhere`. A proteção
   existia só porque a lista estava duplicada em 12 arquivos. Consolidar —
   correto e desejável — teria apagado a proteção com tudo verde. Só a bateria
   de mutação, obrigatória havia uma iteração, pegou.

2. **Caixa do ROADMAP 1.4**: fechada nos quatro sub-itens e **aberta no pai**
   desde a 0017. A regra do passo 1 é "a próxima caixa não marcada, em ordem", e
   literalmente a próxima era um item pronto havia dez iterações. Funcionou
   porque o parágrafo **Próxima tarefa** do `STATUS.md` aponta para outro lugar e
   os agentes seguem ele.

Nos dois casos a fonte formal de verdade estava errada e uma redundância informal
segurou o processo. Redundância que ninguém projetou não é robustez: é uma dívida
que cobra no dia em que alguém a remove por ser redundante — e no caso 1 esse dia
quase foi o mesmo em que foi descoberta.

## 2026-07-26 — O cabeçalho do doc pedia quatro números que o autor não podia ver

O `TEMPLATE.md` abria com `PR`, `Duração`, `Custo reportado` e `Turnos`. O passo 7
escreve o doc; o PR nasce no passo 10, e custo, turnos e duração são medidos pelo
processo que hospeda o agente. **Nenhum dos quatro existe no instante em que o
campo é preenchido.**

O resultado, em 36 iterações:

| campo | o que ficou escrito | o que era |
|---|---|---|
| `Turnos` | `1` em praticamente todos | 52 a 118 |
| `Custo reportado` | `não medido`, `n/d`, `N/D`, `—` | US$ 0,07 a 8,95 |
| `Duração` | `~30min` na 0033, `~40min` na 0034 | 14 min e 12 min |
| `PR` | correto por 30 iterações, depois `#`, `#N` e um push direto em `main` | — |

O `PR` é o caso instrutivo. Funcionou enquanto um agente teve o hábito de abrir o
PR e só então commitar o doc na mesma branch — **acerto por hábito, não por
protocolo**, e por isso não sobreviveu à troca de agente. Das quatro seguintes,
duas (0033 e 0034) deixaram `#` vazio, a 0035 copiou o placeholder `#N` literal, e
a 0036 percebeu o problema e o resolveu com `docs(iter): preenche número do PR
#45` empurrado direto para `main`, fora de PR e fora de CI. Deu certo, e é por dar
certo que preocupa: "cada iteração é um PR" é convenção, não é imposta pelo
GitHub, e na primeira vez que um agente esbarrou nela passou por cima sem atrito.

**A regra que sai daí:** campo que seu autor não pode observar não fica em branco
— fica preenchido com ficção, e ficção formatada é indistinguível de dado. Três
dos quatro campos vinham mentindo desde a primeira iteração, em documento que é
insumo do relatório final.

**Correção:** os quatro saem do cabeçalho. O `git log` já carrega `(#45)` no
título do squash, e a medição passa a morar em `docs/metricas.csv`, casada com a
iteração por `head_antes`/`head_depois`. Os docs 0033 a 0036 foram limpos por
carregarem valores demonstravelmente falsos; de 0001 a 0032 ficam como estão —
são registro histórico, e o projeto não reescreve registro (mesma razão de nunca
renumerar nota).

### Sobre `docs/metricas.csv`

O `.gitignore` já dizia, desde o começo, "métricas consolidadas vão em `docs/`".
Ninguém tinha feito: as 31 execuções medidas viviam só em `logs/`, ignorado pelo
git, numa máquina só. O arquivo consolida as duas fontes e **preserva as falhas** —
`morta-pelo-usuario`, `abortada:aborted_streaming`, `ok-retomada`.

Três ressalvas, para quem for citar o arquivo:

1. `turnos` não é comparável entre linhas: é `num_turns` no `claude -p` e contagem
   de `step_finish` no `opencode`. Mesma coluna, duas definições.
2. As 9 linhas com `fonte=loop.sh` não têm `head_antes`/`head_depois` — o
   `loop.sh` não os registrava — e portanto não se atribuem a uma iteração
   específica, só à janela de tempo.
3. Só as linhas com `fonte=orquestrador` incluem falha. O `loop.sh` grava a
   métrica **depois** do teste de código de saída, então iteração que morre não
   deixa linha. Foi por isso que os nove abortos quase ficaram invisíveis.

A diferença de custo entre os modelos está no arquivo e dispensa comentário aqui:
duas ordens de grandeza entre as linhas `claude-*` e as `opencode-*`, com o mesmo
protocolo e o mesmo repositório.

## 2026-07-26 — Os abortos eram do mecanismo de background do orquestrador

Fecha a entrada anterior, que listava o harness como única hipótese não testada.

**O experimento:** o mesmo `logs/oc-iter.sh`, mesmo modelo
(`opencode-go/deepseek-v4-pro`), mesmo prompt, mesma máquina, mesmos minutos —
mudando só **quem lança o processo**.

| lançado por | tentativas no ROADMAP 1.7d | abortos |
|---|---|---|
| sessão do orquestrador (tarefa em background) | 3 | **3** |
| terminal do usuário | 1 | **0** |

A que rodou no terminal fechou em 9 minutos, 58 passos, US$ 0,070, PR #38.

**Série completa:** 9 abortos, US$ 6,87. Seis em `claude -p` (US$ 6,76, Opus 5 e
Sonnet 5) e três em `opencode` (US$ 0,11, DeepSeek V4 Pro).

**Eliminado por medição, em ordem:** suspensão da máquina (journal ativo nas
janelas), OOM killer, `systemd-oomd` (inativo), pressão de memória (PSI 0,00),
rede (nenhum evento do NetworkManager), processos órfãos acumulando, o
`claude -p` em si (sonda trivial completa em 4,5 s), o modelo (três), o CLI
(dois) e o bloqueio de tela (abortou com a tela destravada).

**Consequência prática:** as iterações passam a rodar no terminal do usuário
(`logs/oc-loop.sh`), e o orquestrador fica com verificação entre iterações,
auditoria, registro e PRs de infraestrutura. Olhando a sessão, é onde ele rendeu
mais de qualquer forma: a bateria de mutação virando regra escrita, o corte do
`STATUS.md`, a consolidação do controle negativo e as três auditorias de
protocolo saíram todas daí, e nenhuma dependia de ele executar a iteração.

**O que conteve o prejuízo** foi instrumentação feita antes, por outro motivo:
métrica gravada mesmo na falha (sem ela os US$ 6,87 seriam invisíveis) e a
guarda de árvore limpa com preservação em commit `wip:` — nenhum dos nove
abortos perdeu trabalho.

## 2026-07-26 — O título do PR era o nome da branch, e ninguém tinha reparado

O merge é squash, então **o título do PR vira o título do commit em `main` para
sempre** — é o índice que `git log` oferece do projeto.

`gh pr create --fill` usa o assunto do commit quando há **um** commit, e cai
para o **nome da branch** quando há mais de um. O `CLAUDE.md` manda "prefira 4
commits pequenos a 1 grande" e o passo 10 mandava `--fill`: seguir as duas
instruções garantia o título degradado.

Medido nos PRs #26 a #39: todos os de 1 commit saíram com título legível; todos
os de 2 ou mais viraram `iter/0028 cpu inc dec r16`, `iter/0025 cpu alu a imm8`,
`iter/0029b roadmap 1.4` — cinco PRs que não dizem o que mudou.

**Detalhe que fecha o raciocínio:** os PRs do opencode saíram com título bom
porque ele faz **1 commit por PR** — ou seja, acertava o título por desobedecer
a orientação de commits pequenos.

Corrigido no PR #40: o passo 10 escreve o título no formato
`iter NNNN: <o que entrega> (ROADMAP X.Y)`.

## 2026-07-26 — Duas lacunas do protocolo que só apareceram na repetição

1. **Caixa do pai.** O passo 9 dizia apenas "`[x]` no item concluído". Quando o
   último sub-item de um grupo fechava, o pai ficava aberto — e a regra do passo
   1 ("a próxima caixa não marcada, em ordem") passava a apontar para trabalho
   feito. O 1.4 ficou assim por dez iterações; o 1.7 repetiu **no mesmo dia** em
   que o 1.4 foi consertado. Corrigido no PR #39.

2. **A corrida do `gh pr checks --watch`** estava registrada neste documento e
   nos prompts avulsos do orquestrador desde a manhã, mas **nunca no `SKILL.md`**
   — o arquivo que os agentes de fato leem. Derrubou o merge do #23 e reapareceu
   no #35, onde os runs só apareceram na sétima tentativa (~60 s). Corrigido no
   PR #40.

Nos dois casos a primeira ocorrência pareceu descuido do agente e a segunda
revelou defeito do processo. **Regra que sai daí:** falha que se repete em
agentes diferentes não se conserta no artefato, conserta-se no protocolo.

Vale notar o viés próprio: a lacuna 2 foi escrita pelo orquestrador num
documento de registro e não no protocolo executável, o que é a mesma confusão
entre "está anotado" e "está valendo" que a bateria de mutação já tinha exposto.

## 2026-07-26 — O alvo era outro, e ficou como está

Na 38ª iteração o usuário registrou que, ao iniciar o projeto, tinha em mente o
**Game Boy Advance**, não o DMG. Decidiu ficar no DMG: o GBA seria mais
impressionante, mas não caberia no prazo de **cerca de uma semana**.

O que a troca custaria, medido no instante da descoberta: 16.057 linhas de Rust
(13.641 delas em teste), das quais praticamente todas são específicas do SM83. O
ARM7TDMI não é parente — são dois conjuntos de instruções (ARM de 32 bits e Thumb
de 16), pipeline de 3 estágios com prefetch visível, PPU de seis modos com
backgrounds afins. Muda a fonte de verdade (Pan Docs → GBATEK) e mudam as suítes
de teste (blargg/mooneye → jsmolka/mGBA).

O que **não** se perderia é o que o curso avalia: o protocolo de 11 passos, a
bateria de mutação, o `STATUS.md` como handoff, os docs de iteração, este
arquivo, o `docs/metricas.csv`, a CI, o scoreboard e as regras R3/R5/R6/R7.

**O que fica registrado disso** é menos sobre consoles e mais sobre o método: um
mal-entendido no enunciado sobreviveu 37 iterações sem ser detectado porque
**nada no processo verifica a premissa** — o protocolo checa a próxima caixa do
ROADMAP, não se o ROADMAP é do aparelho certo. Todos os controles do projeto
(teste antes, bateria de mutação, spec obrigatória, revisão cruzada) olham para
dentro do item. Nenhum olha para cima.

Consequência prática para o prazo: o M8 (apresentação) é o entregável que o curso
avalia e está no **fim** do ROADMAP. Com uma semana, o risco não é o emulador
ficar incompleto — é ele ficar bom e o relatório não existir.

## 2026-07-26 — A série de dez que parou na nona

Primeira série longa rodada inteiramente pelo terminal do usuário, depois que o
experimento dos abortos mostrou de quem era a culpa.

| | |
|---|---|
| Concluídas | 8 de 10 |
| Custo | US$ 0,88 |
| Passos | 675 |
| Tempo | 148 min |
| Entregue | ROADMAP 1.8, 1.9 inteiro (256 opcodes do prefixo CB) e 1.10a |

As oito que fecharam levaram de 10 a 17 minutos. A nona **estourou o teto de 45
minutos com 17 passos** — travou duas vezes, a segunda por quinze minutos sem
escrever no log, com 7% de CPU e sem processo filho para culpar. Na primeira vez
havia um `cargo test` dormindo a 0% de CPU havia dez minutos; ele saiu sozinho
antes de eu conseguir ler o `wchan`, e **o motivo não foi determinado**. Fica
como não-explicado, e não como hipótese travestida de causa — o projeto já tem
uma entrada sobre o custo de fazer o contrário.

O `timeout` matar valeu mais do que eu matar: código de saída 124 distingue
"estourou o teto" de "morreu de outra coisa" na coluna `resultado` do
`docs/metricas.csv`. Os 346 linhas e 17 testes que o agente chegou a escrever
ficaram preservados em `iter/0040-jp-cond-hl`, num commit `wip:`.

### Proteção de branch, e por que ela quase não serviria

Ligada depois que dois colegas ganharam acesso de escrita. Três decisões, e as
três eram armadilhas:

1. **Vale para admin.** O agente empurra autenticado como o dono do repositório.
   Isentar admin deixaria a proteção valendo para todo mundo **menos** para quem
   fez o push direto da 0036.
2. **Zero aprovações exigidas.** Exigir uma revisão impediria o agente de mergear
   qualquer PR — ele abre e fecha sozinho, e não há ninguém no laço às 3 da manhã.
   Fica o que interessa (passa por PR, passa por CI) sem travar a cadeia.
3. **Sem exigir branch atualizada.** No mesmo dia, o PR #46 entrou entre o começo
   e o fim da iteração 0037; com essa regra ligada, o #47 teria sido bloqueado até
   alguém atualizar a branch — e o passo 10 não sabe fazer isso.

O padrão das três é o mesmo: a configuração "mais segura" de cada campo, marcada
por reflexo, quebraria o processo sem proteger nada.
