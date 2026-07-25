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

Iteração concluída, próxima tarefa, placar atualizado, invariantes novas,
bloqueios. Curto.

## Passo 9 — Marcar o ROADMAP

`[x]` no item concluído.

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
