# STATUS

> Este arquivo é a **memória do projeto entre iterações**. O contexto do agente
> é descartado a cada iteração; este arquivo não. Mantenha-o curto e verdadeiro.

**Última iteração concluída:** 0000 — bootstrap (sem código de emulador)
**Próxima tarefa:** ROADMAP 0.1 — workspace Cargo (`gb-core`, `gb-cli`, `gb-desktop`)
**Marco atual:** M0 — Fundação

**Repositório:** https://github.com/Deivison-Costaa/gb-rs

## Placar de ROMs de teste

Baseline do dia 0: **121 ROMs baixadas, 0 passando** — não existe emulador
ainda. Os totais abaixo são os que `scripts/scoreboard.sh` mede de fato, e
divergem um pouco dos que o scaffold estimava (a diferença é que cada suíte tem
as ROMs individuais **mais** a ROM agregada).

| Suíte | Passando | Total |
|---|---|---|
| blargg cpu_instrs | 0 | 12 |
| blargg instr_timing | 0 | 1 |
| blargg mem_timing | 0 | 4 |
| blargg mem_timing-2 | 0 | 4 |
| blargg halt_bug | 0 | 1 |
| blargg oam_bug | 0 | 9 |
| blargg interrupt_time | 0 | 1 |
| blargg dmg_sound | 0 | 13 |
| dmg-acid2 | 0 | 1 |
| mooneye acceptance | 0 | 66 |
| mooneye acceptance (outros modelos) | 0 | 9 |

## Invariantes já estabelecidas

- CPU é cycle-stepped (M-cycle). PPU é scanline renderer, não pixel FIFO.
- `gb-core` não tem dependência de I/O.
- **`main` é protegida:** merge só via PR, com os jobs `check` e `scoreboard`
  verdes e branch atualizada. 0 aprovações exigidas (projeto solo), histórico
  linear, sem force-push. `enforce_admins=false` de propósito: se o loop
  travar, um humano ainda consegue destravar sem desmontar a proteção.
- **`docs/reference/` é a fonte de verdade e é commitado.** Pan Docs fixado em
  `fe246067b695`, gbops em `90b9bf296aed`. Regenerado só por
  `scripts/fetch-reference-docs.sh`; os arquivos `01-`…`09-` são gerados e não
  devem ser editados à mão. Ver `docs/reference/README.md` para o mapa
  "item do ROADMAP → arquivo a ler antes de implementar".
- **ROMs de teste não entram no git.** `tests/roms/` é gitignored;
  `scripts/fetch-test-roms.sh` baixa o bundle fixado por tag e sha256.
- **`scoreboard.csv` é acumulativo e versionado.** Cada execução anexa; nunca
  truncar. É a série temporal que vira gráfico no ROADMAP 8.2.
- **Contrato do `gb-cli`** (definido em `scripts/scoreboard.sh` antes de o
  binário existir, conforme R5) — os itens 0.3 e 1.12 têm de cumprir:
  `gb-cli run <rom> --headless --max-cycles <n>`, saindo `0` = pass, `1` = fail,
  outro = crash, com o token `cycles=<n>` em algum ponto da saída.

## Bloqueios

_(nenhum)_

A proteção de branch **funcionou** no plano atual — não foi preciso o
contorno previsto no prompt de bootstrap.

## Notas para a próxima iteração

1. **A guarda na CI some sozinha.** O job `check` pula `fmt`/`clippy`/`test`
   enquanto não houver `Cargo.toml` na raiz. Assim que a 0.1 criar o workspace,
   os três passos voltam a rodar sem tocar em `.github/workflows/ci.yml`.
   Confira na 0.1 que eles realmente rodaram — se continuarem pulando, o
   `Cargo.toml` não está na raiz.

2. **O artefato `scoreboard.csv` da CI não acumula.** Cada job roda em checkout
   limpo, então o CSV enviado como artefato tem só as linhas daquela execução.
   O histórico de verdade é o `scoreboard.csv` versionado, que cresce a cada
   iteração que commita. Se a apresentação precisar do histórico completo da CI,
   isso é trabalho a fazer no ROADMAP 0.2 — não está feito.

3. **`scoreboard.csv` vai gerar conflito** se duas iterações mexerem nele em
   paralelo. Projeto é sequencial, então na prática não dói; se doer, resolva
   concatenando os dois lados, nunca escolhendo um.

4. **9 ROMs mooneye são de outros modelos** (`-dmg0`, `-mgb`, `-S`, `-sgb`,
   `-sgb2`). Elas rodam, mas na suíte `mooneye/acceptance-nondmg`. Não são
   regressão; não tente fazer passar.

5. **`scripts/review.sh` ainda não foi configurado.** Ele espera `REVIEWER_CMD`
   (padrão `opencode run`). Enquanto não houver um segundo modelo disponível, a
   revisão cruzada de cada iteração fica vazia — e o campo correspondente do
   `docs/iterations/NNNN-*.md` deve dizer isso, não ficar em branco.

6. **`blargg/cgb_sound` foi deliberadamente excluída** do download: é suíte de
   Game Boy Color e este emulador é DMG. Se aparecer no placar, algo regrediu
   em `scripts/fetch-test-roms.sh`.
