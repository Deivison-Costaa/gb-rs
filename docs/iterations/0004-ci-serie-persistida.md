# Iteração 0004 — A série gerada pela CI para de morrer com o runner

- **Data:** 2026-07-25
- **Item do roadmap:** 0.2c
- **PR:** #5
- **Duração:** ~40min
- **Custo reportado:** não medido — sessão interativa, sem `--output-format json`.
  Quarta iteração seguida com essa dívida; ver nota 10 do `STATUS.md`.
- **Turnos:** 1

## Objetivo

Fazer as linhas que a CI mede sobreviverem ao fim do job, publicando o
`scoreboard.csv` acumulado numa branch de dados a cada push em `main`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| — | — | — |

Sem spec de hardware, mesmo motivo das três anteriores: a R1 vincula "opcode,
registrador ou comportamento de periférico", e a tabela de
`docs/reference/README.md` começa no 0.3.

A "spec" desta iteração foi a configuração do próprio repositório, lida pela API
antes de escrever qualquer coisa — e ela contradisse o enunciado do item:

```
$ gh api repos/Deivison-Costaa/gb-rs/branches/main/protection
  required_pull_request_reviews: { required_approving_review_count: 0 }
  enforce_admins:                { enabled: false }
$ gh api repos/Deivison-Costaa/gb-rs/actions/permissions/workflow
  { "default_workflow_permissions": "read" }
$ gh api repos/Deivison-Costaa/gb-rs --jq .owner.type
  User
```

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | `ferramental` | Que o item 0.2c era implementável como escrito: `git commit` + `git push origin main` num passo do job, porque "o `GITHUB_TOKEN` tem escrita no próprio repositório". | `required_pull_request_reviews` está **ligado** em `main` (com 0 aprovações, mas ligado), e isso bloqueia push direto para quem não tem bypass. `enforce_admins: false` dá bypass ao dono humano, não ao `github-actions[bot]`. `bypass_pull_request_allowances` é só para repositório de organização; este é de usuário (`owner.type: User`). | Ler `branches/main/protection` antes de escrever o passo. |
| 2 | `ferramental` | Que bastava `git push` e o token do Actions escreveria — o padrão do GitHub para repositórios novos é `read and write`. | Neste repositório `default_workflow_permissions` é `read`. Sem `permissions: contents: write` **no job**, o push morre por permissão, não por proteção — e o diagnóstico apontaria para o lugar errado. | Ler `actions/permissions/workflow` na mesma leva do #1. |
| 3 | `API-Rust` | Que dedentar o corpo de um job pela indentação do **cabeçalho** do job (`  scoreboard:`, coluna 2) poria as chaves dele (`permissions:`, `steps:`) na coluna zero — foi o que `job_body` fez na primeira escrita. | As chaves de um job são o mapeamento **aninhado sob** o nome dele: ficam um nível mais fundo (coluna 4). Dedentar por 2 deixou tudo na coluna 2, e `indent_of(l) == 0` nunca casou. Quem manda é a indentação da primeira chave, não a do cabeçalho. | `permission_scanner_reads_the_job_level_permissions_block` falhou com `left: None` — teste de parser, escrito justamente como guarda do guarda. |
| 4 | `ferramental` | Que dava para chamar o script novo de `publish-scoreboard.sh` sem mexer nas guardas existentes. | `scoreboard.sh` é substring de `publish-scoreboard.sh`, e `ci_scoreboard_steps_cannot_fail_silently` acha o passo com `.find()` — o **primeiro** que casa. Hoje a ordem salva: o passo da medição vem antes. Bastaria reordenar os passos para a guarda passar a examinar o passo de publicação achando que examina o da medição, **sem ficar vermelha**. | Reler a guarda antes de acrescentar o passo. Nenhum teste falhou — este é o modo de falha que não se anuncia. |

**Sobre o #1 — o que está verificado e o que está inferido.** A configuração da
proteção está verificada: veio da API, está transcrita acima. A consequência —
"o `GITHUB_TOKEN` seria rejeitado ao empurrar para `main`" — é **inferência** a
partir do comportamento documentado do GitHub, não observação. Não vi a
rejeição acontecer, porque observá-la exigiria um push de verdade em `main` a
partir do Actions.

Isso é dito assim de propósito. A nota 7 do `STATUS.md` registra uma inferência
que foi anotada como fato e depois não procedeu — e quase virou trabalho
inventado na 0003. A lição de lá não é "não infira", é "não anote inferência com
cara de medição". O experimento que fecharia a questão está descrito na nota 11
do `STATUS.md`: um `DATA_BRANCH=main` numa execução de push e a leitura do erro.

**Sobre o #3 — o teste de parser se pagou pela segunda vez.** Os testes de
parser existem desde a 0002 porque um `steps_of_job` que devolvesse sempre
`vec![]` deixaria as guardas verdes por vacuidade. Desta vez ele pegou o
inverso: o parser **novo** estava errado, e sem ele o
`ci_scoreboard_job_asks_for_write_access` teria falhado com a mensagem "o job
não pede `contents: write`" — que é falsa e teria me mandado editar o `ci.yml`
já correto até desistir. Dois testes falharam ao mesmo tempo; só um estava
apontando para o bug.

**Sobre o #4 — a guarda que continua verde guardando a coisa errada.** É o mesmo
gênero da nota 8, mas nem "passou de primeira" nem "falhou pelo motivo errado":
é "vai passar para sempre, e um dia deixa de significar o que significava". A
correção foi ancorar os fragmentos no `run: ./scripts/`, que não é substring de
nada, e dizer isso no comentário do `SCOREBOARD_STEPS` — porque a próxima pessoa
a acrescentar um passo com nome parecido não vai reler este documento.

**As cinco mutações.** Nenhuma guarda do script foi aceita sem ser forçada a
falhar. Os cinco testes passaram **de primeira**, que a nota 8 manda tratar como
suspeita:

| Mutação | Testes que reprovaram |
|---|---|
| `merge_csv` ignora o publicado e copia só o CSV local | `publish_keeps_rows_that_another_run_had_already_published` |
| saída antecipada de "nada novo" removida (`if false`) | `publish_is_a_no_op_when_there_is_nothing_new` |
| `PUSH_ATTEMPTS` de 3 para 1 | `publish_retries_when_the_push_is_rejected` |
| checagem de CSV sem linha de dado afrouxada para `>= 0` | `publish_fails_when_the_csv_has_no_data_rows` |
| push para `refs/heads/branch-errada` | os 4 acima **menos** o do CSV vazio |

As quatro primeiras matam **exatamente um** teste cada — é isso que diz que as
guardas medem coisas diferentes, e não a mesma coisa cinco vezes. A quinta é o
controle grosso: quebrar o destino derruba tudo que depende de publicação, e
deixa de pé só o teste que exige que **nada** seja publicado.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |

Sem regressão e sem avanço — nenhuma linha de emulação foi escrita.
`scoreboard.csv`: 484 → 605 linhas de dado (121 novas, todas `crash`).

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum.
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

Segue sem `REVIEWER_CMD` configurado (nota 5 do `STATUS.md`). Campo vazio por
ausência de ferramenta, não por esquecimento.

## Decisões de arquitetura

1. **A série vai para `scoreboard-data`, não para `main`.** Forçado pelo erro
   #1. As alternativas eram afrouxar a proteção de `main` (invariante do
   projeto, e o item que a criou custou uma iteração) ou provisionar um PAT de
   admin à mão (fora do alcance de um agente, e um segredo de admin no CI é
   preço alto por um CSV). Efeito colateral bem-vindo: `main` fica só com
   commits de iteração, e o CSV deixa de ser campo de conflito entre a CI e a
   iteração seguinte — a nota 3 do `STATUS.md` fica menor, não maior.
2. **União, não substituição.** O runner faz checkout de um commit e mede em
   cima do CSV **daquele** commit: o CSV local é sempre um recorte da série.
   Empurrar o recorte por cima apagaria o que outra execução publicou. A chave
   da união é a linha inteira; ela carrega timestamp, commit, suíte, ROM, status
   e ciclos, então duas medições distintas só colidem no mesmo segundo, no mesmo
   commit, para a mesma ROM. O troco é a idempotência: republicar o mesmo CSV
   não gera commit.
3. **Commit montado com plumbing, não com worktree.**
   `hash-object` + `mktree` + `commit-tree` não tocam o índice nem o diretório
   de trabalho. Importa: depois deste passo o job ainda sobe o artefato lendo o
   `scoreboard.csv` do checkout, e um `git checkout` de branch no meio do job
   trocaria o arquivo debaixo dele.
4. **Retentar é obrigatório, e refazendo a união.** Push rejeitado significa que
   o topo mudou; reempurrar o mesmo commit só repetiria a rejeição. O laço
   refaz `fetch` → união → `commit-tree` a cada tentativa.
5. **O passo só roda em push para `main`.** Em execução de PR o commit medido
   pode nunca entrar em `main` — a série ganharia pontos de código descartado —
   e PR vindo de fork recebe token somente-leitura, o que faria o passo falhar
   sempre. Como o `if: always()` do artefato, este `if:` é afirmado por um teste
   para ser decisão, não descuido.
6. **`permissions:` no job, não no workflow.** Declarar no topo daria escrita
   também ao `check`, que não tem por quê.

## Notas

**O que faltava observar, e o que se observou.** No fechamento do PR este campo
dizia que o push do Actions para `scoreboard-data` ainda não tinha sido visto de
ponta a ponta — os testes cobriam o script contra um remoto local, e o smoke
test cobria o CSV real (605 linhas, incluindo `03-op sp,hl.gb`, com vírgula
dentro das aspas) num round-trip byte a byte, mas o par
`permissions: contents: write` + credencial do `actions/checkout` só aparece
numa execução de verdade.

O push de merge resolveu: run `30174085591`, passo `success`, branch criada com
**726 linhas** (as 605 de `main` mais as 121 daquela execução), autor
`github-actions[bot]`. Na execução de PR do mesmo código o passo saiu `skipped`.
Os dois lados do `if:` estão observados.

Fica registrado nos dois estados de propósito: o valor deste log não é parecer
que tudo foi previsto, é mostrar em que ordem as coisas passaram de suposição a
medição. A que **não** passou é a do erro #1 — ver nota 11 do `STATUS.md`.

**A branch de dados não é um lugar bom para esconder erro.** Ela não aparece em
`git log` de `main`, ninguém faz checkout dela, e nada quebra se ela ficar
parada — que é exatamente o perfil de um dado que apodrece sem sinal. O que
protege hoje: o passo é incondicional dentro do `if:` de push em `main`, sem
`continue-on-error`, e o script sai != 0 quando não tem o que publicar. O que
**não** existe: alguém percebendo que a branch parou de crescer. Um item futuro
(8.2 é o candidato natural, já que é ele quem lê a série) devia comparar o
número de commits da branch com o número de pushes em `main`.

**Custo não medido, quatro vezes.** Mesma dívida das 0001–0003.
