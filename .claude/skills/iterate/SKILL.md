---
name: iterate
description: Executa exatamente UMA iteração do gb-rs — escolhe a próxima micro-funcionalidade do ROADMAP, implementa com teste, documenta, abre PR e faz merge. Use quando o usuário pedir para avançar o projeto, rodar uma iteração, ou invocar /iterate. Sempre para ao final de uma única iteração.
---

# Uma iteração do gb-rs

Você vai executar **uma** iteração completa e então **parar**. Não encadeie
iterações. O contexto será limpo depois — tudo que a próxima iteração precisa
saber tem que estar em `STATUS.md`, não na sua cabeça.

## Passo 0 — Orientação (sempre primeiro)

Leia, nesta ordem: `STATUS.md`, `ROADMAP.md`, `CLAUDE.md`.
Confirme: `git status` limpo, branch `main`, `git pull` feito.

O `STATUS.md` traz **índices** de notas e de invariantes, não o corpo delas. O
parágrafo **Próxima tarefa** cita por número as notas que valem para esta
iteração: abra **essas** em `docs/notas.md`, e a invariante que o item tocar em
`docs/invariantes.md`. Não leia os dois arquivos inteiros — somam 1.500 linhas
e foram separados justamente para não entrar em contexto de graça.

Se `STATUS.md` listar bloqueios não resolvidos, **pare e reporte**.

## Passo 1 — Escolher a tarefa

A **próxima** caixa não marcada do `ROADMAP.md`, em ordem. Exatamente uma.

Se ela for grande demais para um PR pequeno (mais de ~300 linhas de diff ou
mais de um conceito), quebre-a em sub-itens no `ROADMAP.md`, commite essa
quebra, e faça só o primeiro sub-item.

## Passo 2 — Branch

```
git checkout -b iter/NNNN-slug
```
`NNNN` = número da iteração com 4 dígitos.

## Passo 3 — Consultar a spec (obrigatório)

Antes de escrever qualquer linha de comportamento de hardware, leia o arquivo
relevante em `docs/reference/`.

Se a informação não estiver lá: busque no Pan Docs (gbdev.io/pandocs) ou na
tabela gbops, salve a seção em `docs/reference/`, commite (`docs(ref):`), e só
então prossiga.

**Anote mentalmente o que você teria escrito de memória antes de ler a spec.**
Você vai precisar disso no Passo 7.

## Passo 4 — Teste primeiro

Escreva o teste que falha. Rode e confirme que falha pelo motivo certo.

## Passo 5 — Implementar

O mínimo para o teste passar. Sem generalizar para casos que o roadmap ainda
não pediu.

## Passo 6 — Verificar

```
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all
./scripts/scoreboard.sh
```

Tudo verde. Se o scoreboard **regrediu** em qualquer suíte, isso é bloqueante:
conserte ou reverta.

**Bateria de mutação — obrigatória.** Verde não prova que o teste mede: mutante
que sobrevive é teste que não olha. Liste as maneiras de errar o que você acabou
de implementar — valor de flag, instante do M-cycle, operando trocado, escrita a
mais ou a menos — aplique **uma de cada vez** no fonte, e confirme que algum
teste reprova. Some **controles**: mutações que *devem* passar, para provar que a
suíte não reprova qualquer coisa. Registre o placar (`N/N pegos, M/M controles
verdes`) no doc da iteração.

Mutante que sobrevive é buraco de cobertura **medido**: escreva o teste que o
mata antes de seguir. Cuidado com o rebuild por mtime — nota 14.

## Passo 7 — Documentar a iteração

Copie `docs/iterations/TEMPLATE.md` para `docs/iterations/NNNN-slug.md` e
preencha.

O campo **`Erros de primeira tentativa`** é o mais importante do projeto
inteiro. Seja específico e honesto:

- Você implementou algo de memória e a spec te contradisse? Escreva o quê,
  o que você achou que era, e o que era de fato.
- Um teste passou por acidente? Um comportamento você chutou?
- Se não houve erro nenhum, escreva "nenhum" — mas só se for verdade.

Ninguém vai te penalizar por erro registrado. O projeto **depende** desses
dados. Um log de iterações onde tudo sempre deu certo é um log inútil.

## Passo 8 — Atualizar STATUS.md

Iteração concluída, próxima tarefa, placar, bloqueios. **Curto** — há teste
guardando o teto de tamanho.

O parágrafo **Próxima tarefa** é o handoff, e é o que mais rende no projeto
inteiro: diga o que muda de forma em relação ao item anterior, que armadilhas a
spec esconde, e **cite por número** as notas que a próxima iteração precisa
abrir. Foi ele — não a seção de notas — que pré-anunciou as cinco armadilhas
que a 0023 recebeu prontas.

Nota ou invariante nova: **corpo** em `docs/notas.md` / `docs/invariantes.md`,
**uma linha** de índice no `STATUS.md`. A numeração das notas é citada em
comentários do código e nos docs de iteração — nota nova entra com o próximo
número livre, e **nunca se renumera**.

## Passo 9 — Marcar o ROADMAP

`[x]` no item concluído. **Se ele era o último sub-item aberto de um grupo,
marque também a caixa do pai.** Sem isso o grupo fica aberto para sempre e a
regra do passo 1 — "a próxima caixa não marcada, em ordem" — passa a apontar
para trabalho já feito. Aconteceu duas vezes: o 1.4 ficou dez iterações assim, e
o 1.7 repetiu no mesmo dia em que o 1.4 foi consertado.

Sub-item usa **dois espaços** de indentação. Três também renderiza, e foi
justamente o que escondeu dois sub-itens do 1.7 de uma varredura.

## Passo 10 — PR e merge

```
git add -A
git commit -m "feat(escopo): descrição"     # Conventional Commits
git push -u origin iter/NNNN-slug
gh pr create --fill
gh pr checks --watch
gh pr merge --squash --delete-branch
git checkout main && git pull
```

## Passo 11 — PARAR

Escreva um resumo de no máximo 5 linhas: o que foi feito, placar antes/depois,
qual a próxima tarefa. **Encerre o turno.** Não comece a próxima iteração.

---

## Se algo falhar

Três tentativas no mesmo erro e você para. Registre em `STATUS.md` →
`Bloqueios` o que tentou e o que observou, abra o PR como `--draft`, e encerre.
Insistir queima contexto e orçamento sem convergir.
