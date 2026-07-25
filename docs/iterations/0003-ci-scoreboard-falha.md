# Iteração 0003 — Job `scoreboard` que falha quando não mede nada

- **Data:** 2026-07-25
- **Item do roadmap:** 0.2b
- **PR:** #4
- **Duração:** ~25min
- **Custo reportado:** não medido — sessão interativa, sem `--output-format json`.
  Terceira iteração seguida com essa dívida; ver § Notas.
- **Turnos:** 1

## Objetivo

Fazer o job `scoreboard` ficar vermelho quando o placar não mediu nada — seja
porque o script morreu, seja porque ele terminou bem sem anexar linha nenhuma
ao `scoreboard.csv`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| — | — | — |

Sem spec de hardware, mesmo motivo da 0001 e da 0002: a R1 vincula "opcode,
registrador ou comportamento de periférico", e a tabela de
`docs/reference/README.md` começa no 0.3. O diff é bash, YAML de CI e testes.

A "spec" desta iteração foi o próprio `bash(1)` sob `set -u`, consultado por
experimento — ver erro #2.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | `ferramental` | Que o teste negativo (`scoreboard_fails_when_no_row_is_appended`) tinha nascido RED contra o script antigo, porque saiu vermelho na primeira execução. | Ele saiu vermelho pelo **motivo errado**: o script já morria três linhas antes do ponto de interesse, com `total: variável não associada` (erro #2). O comportamento sob teste — "detectou que o CSV não cresceu" — não existia nem passou a ser exercido pelo teste. | Rodar `bash -x` no script em vez de aceitar o vermelho. |
| 2 | `ferramental` | Que `declare -A total pass` seguido de `${#total[@]}` devolve `0` quando nenhuma chave foi atribuída — e que por isso o `if (( ${#total[@]} > 0 ))` do resumo protegia o caso de zero ROMs. | Bash 5.3: array associativo **declarado e nunca atribuído** conta como não associado. Sob `set -u`, `${#total[@]}` aborta o script. A guarda escrita para tratar o caso vazio é exatamente o que explode no caso vazio. Corrigido com `declare -A total=() pass=()`. | O `bash -x` do erro #1. |
| 3 | `ferramental` | Que o job `scoreboard` ficava verde quando o script morria — foi assim que a nota 7 do `STATUS.md` descreveu, e foi assim que o enunciado do 0.2b foi escrito. | Metade disso não procede. `run: ./scripts/scoreboard.sh` executa sob `bash -e {0}`; saída != 0 reprova o passo e o job. Essa metade já funcionava por padrão do Actions. O que **não** existia era a outra metade: script saindo `0` sem ter anexado nada. | Reler o contrato do `run:` antes de "consertar" o que já funcionava. |

**Sobre o #1 — a vacuidade da nota 8, invertida.** A nota 8 do `STATUS.md`
manda desconfiar do teste que passa de primeira. Aqui o sintoma foi o oposto e
enganou melhor: o teste *falhou* de primeira, que é o resultado que o ciclo
RED→GREEN ensina a comemorar. Só que "falhou" e "falhou pelo motivo que eu
quero medir" são coisas diferentes, e um vermelho não distingue as duas. O
teste ficou exigindo a mensagem (`não anexou nenhuma linha`), não só o código
de saída — sem isso ele continuaria verde contra um script que morresse por
qualquer outro motivo, medindo um bug com o outro.

**Sobre o #3 — metade do item já estava pronta e ninguém sabia.** O enunciado
do 0.2b, herdado da nota 7, prometia dois consertos. Um deles não era conserto,
era comportamento padrão do Actions que ninguém tinha verificado. Isso não
esvazia a iteração: o que passou a existir é (a) a detecção do CSV parado, que
faltava mesmo, e (b) um teste que impede alguém de *desfazer* a metade que já
funcionava, pendurando um `continue-on-error:` no passo. Vale o registro porque
o custo aqui foi de uma nota de `STATUS.md` escrita por inferência e nunca
conferida — e ela quase virou trabalho inventado.

**As seis mutações.** Nenhuma guarda foi aceita sem ser forçada a falhar:

| Mutação | Teste que reprovou |
|---|---|
| `continue-on-error: true` no passo do `scoreboard.sh` | `ci_scoreboard_steps_cannot_fail_silently` |
| passo do `fetch-test-roms.sh` removido | `ci_scoreboard_job_fetches_roms_and_runs_the_scoreboard` |
| `continue-on-error: false` (valor benigno) | **nenhum** — controle negativo, como projetado |
| `if: always()` retirado do `upload-artifact` | `ci_uploads_the_scoreboard_csv_even_on_failure` |
| checagem de crescimento removida do script | `scoreboard_fails_when_no_row_is_appended` (saiu `0`) |
| `declare -A total=()` revertido para `declare -A total` | `scoreboard_fails_when_no_row_is_appended` (motivo errado) |

As duas últimas são o par que interessa: uma isola o comportamento novo, a
outra isola o bug que estava mascarando o comportamento novo. Rodar só a
primeira teria deixado o erro #1 de pé.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |

Sem regressão e sem avanço — nenhuma linha de emulação foi escrita.
`scoreboard.csv`: 363 → 484 linhas de dado (121 novas, todas `crash`).

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum.
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

Segue sem `REVIEWER_CMD` configurado (nota 5 do `STATUS.md`). Campo vazio por
ausência de ferramenta, não por esquecimento.

## Decisões de arquitetura

1. **A checagem de crescimento mora no script, não no YAML.** Podia ser um
   passo `wc -l` antes e depois no `ci.yml`. Ficou no `scoreboard.sh` porque
   assim vale também para quem roda local (passo 6 do protocolo de iteração), e
   porque o job não pode garantir sozinho uma propriedade do script.
2. **O YAML garante só o que é do YAML.** Ao job cabe não engolir o veredito —
   e é isso que `ci_scoreboard_steps_cannot_fail_silently` tranca. Divisão
   limpa: script diz a verdade, workflow não a esconde.
3. **`continue-on-error` é reprovado pelo valor, não pela presença.**
   `continue-on-error: false` é inofensivo e não deve quebrar o build de
   ninguém; `true` é o modo de falha. O teste distingue.
4. **`if: always()` no upload é o único `if:` desejado no job.** A 0.2a baniu
   `if:` dos passos de qualidade; aqui um `if:` é obrigatório, e pelo mesmo
   raciocínio invertido — sem ele o artefato só existe quando ninguém precisa
   dele. Por isso a lista de passos guardados é explícita (`SCOREBOARD_STEPS`) e
   não "todos os passos do job".
5. **Testes de script bash rodam de dentro do `cargo test`.** É o único portão
   que a proteção de `main` exige. Um `bats` ou um `*.sh` de teste ficaria fora
   do `check` e ninguém o rodaria. `GB_CLI` aponta para o binário que o próprio
   `cargo test` construiu, para não disparar um `cargo build` aninhado disputando
   o lock de `target/`.

## Notas

**O sandbox nunca toca o CSV da raiz.** `SCOREBOARD_CSV` e `ROMS_DIR` vão para
`target/tests-tmp/<caso>/`. Linha de teste no `scoreboard.csv` versionado
corromperia a série que vira gráfico no 8.2 — e seria invisível, porque teria a
mesma forma das linhas verdadeiras.

**Custo não medido, três vezes.** As iterações 0001, 0002 e 0003 têm o campo
`Custo reportado` vazio pelo mesmo motivo: sessão interativa. A apresentação
pede "custo por iteração" (8.2) e esse dado não está sendo coletado. Não é
tarefa de nenhum item do ROADMAP hoje — virou pendência no `STATUS.md`.

**Um slide, talvez.** A 0002 registrou uma nota de `STATUS.md` que descreveu o
problema errado (guarda "morta" que era condicional viva). A 0003 registra
outra que descreveu um problema **inexistente** (job verde com script morto).
Duas iterações seguidas em que o registro do agente sobre o próprio trabalho
saiu mais confiável do que a inferência do agente sobre o que ainda faltava — e
em ambos os casos quem corrigiu foi ir olhar o artefato de novo, não pensar
mais forte.
