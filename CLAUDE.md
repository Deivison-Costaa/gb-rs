# gb-rs — Emulador de Game Boy (DMG) em Rust

## Contexto

Projeto acadêmico da disciplina **Programação com Agentes** (CI-UFPB).
Objetivo duplo, e o segundo importa tanto quanto o primeiro:

1. Um emulador de Game Boy DMG correto, com MBC1/2/3/5 e APU completa.
2. Um **registro empírico** de como o trabalho foi conduzido com agente —
   onde o agente acertou, onde errou, o que custou. Esse registro vira a
   apresentação final.

Se em algum momento houver conflito entre "avançar rápido" e "registrar
direito o que aconteceu", **registrar ganha**.

---

## Regras invioláveis

### R1 — Nunca implemente comportamento de hardware de memória

Você conhece Z80 melhor do que conhece o SM83, e os dois divergem em pontos
específicos (`DAA`, rotações, bug do `HALT`, flags de `ADD SP,e8`). Sua
intuição sobre flags e timing **não é confiável aqui**.

Antes de implementar qualquer opcode, registrador ou comportamento de
periférico, **leia o arquivo correspondente em `docs/reference/`**. Se a
informação não estiver lá, pare, baixe a seção relevante do Pan Docs para
`docs/reference/`, commite isso, e só então implemente.

Quando você perceber que implementou algo de memória e depois descobriu que
estava errado, **isso não é vergonha — é o dado mais valioso do projeto**.
Registre no doc da iteração, no campo `Erros de primeira tentativa`.

### R2 — CPU é cycle-stepped, não instruction-stepped

`Cpu::step()` avança **um M-cycle** e retorna. O barramento, o timer, a PPU e
a APU avançam junto, no meio da instrução. Não implemente "executa a instrução
inteira e depois soma N ciclos" — isso quebra a suíte Mooneye e é caríssimo de
refatorar depois.

### R3 — `gb-core` não tem I/O

O crate `gb-core` é uma máquina de estados pura: sem `std::fs`, sem `winit`,
sem `cpal`, sem `println!`. Ele expõe framebuffer, buffer de áudio e porta
serial como dados. Quem faz I/O é `gb-cli` e `gb-desktop`.

Isso é o que permite rodar as ROMs de teste headless na CI. Não quebre.

### R4 — Uma micro-funcionalidade por iteração, e então PARE

Ao terminar o passo 10 do protocolo de iteração, **encerre o turno**. Não
comece a próxima tarefa "já que estou aqui". O contexto é limpo entre
iterações de propósito.

### R5 — Teste antes de implementar

Todo item do roadmap vira teste primeiro (unitário, ou uma ROM adicionada ao
scoreboard). Sem exceção.

### R6 — Nada de `unsafe`, nada de `unwrap()` fora de teste

`#![forbid(unsafe_code)]` em `gb-core`. `clippy -D warnings` passa limpo.

---

## Arquitetura

```
gb-rs/
├── crates/
│   ├── gb-core/      # máquina de estados pura (sem I/O)
│   │   ├── cpu/      # registradores, decode, execução por M-cycle
│   │   ├── bus.rs    # MMU, mapa de memória
│   │   ├── cart/     # header, NoMBC, MBC1, MBC2, MBC3, MBC5
│   │   ├── ppu/      # renderizador por scanline
│   │   ├── apu/      # 4 canais + frame sequencer + mixer
│   │   ├── timer.rs
│   │   └── joypad.rs
│   ├── gb-cli/       # runner headless: roda ROM de teste, lê porta serial
│   └── gb-desktop/   # winit + pixels + cpal
├── docs/
│   ├── reference/    # Pan Docs, gbops — a fonte de verdade (R1)
│   ├── iterations/   # um arquivo por iteração (material da apresentação)
│   └── prompts/
├── tests/roms/       # ROMs de teste (blargg, mooneye, dmg-acid2)
├── scripts/
├── ROADMAP.md        # a escada de micro-funcionalidades
└── STATUS.md         # estado atual — LEIA ISTO PRIMEIRO
```

**`Bus` é o dono de tudo.** Componentes recebem `&mut Bus`. Não tente modelar
com `Rc<RefCell<>>` espalhado — emulador é estado mutável compartilhado por
natureza e essa briga com o borrow checker não se ganha, se contorna.

---

## Protocolo de iteração

Cada iteração é **um PR**. Sempre estes 10 passos, nesta ordem:

1. Ler `STATUS.md` e `ROADMAP.md`. Confirmar que está em `main`, limpo e atualizado.
2. Escolher **exatamente uma** micro-funcionalidade: a próxima não concluída.
3. Criar branch `iter/NNNN-slug`.
4. Ler a spec relevante em `docs/reference/` (R1). Se faltar, buscar e commitar antes.
5. Escrever o teste que falha.
6. Implementar o mínimo para o teste passar.
7. Rodar: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`,
   `./scripts/scoreboard.sh`.
8. Escrever `docs/iterations/NNNN-slug.md` a partir de `docs/iterations/TEMPLATE.md`.
   Preencher **honestamente**, principalmente `Erros de primeira tentativa`.
9. Atualizar `STATUS.md`: iteração atual, próxima tarefa, placar, pendências.
10. Commitar (Conventional Commits), `git push`, `gh pr create`, aguardar CI verde,
    `gh pr merge --squash --delete-branch`. **PARAR.**

Se um passo falhar três vezes seguidas, **não insista**: registre o bloqueio em
`STATUS.md` na seção `Bloqueios`, abra o PR como draft, e pare. Um humano decide.

## Commits

`feat(cpu):`, `fix(ppu):`, `test(apu):`, `docs(iter):`, `chore(ci):`
Um commit por unidade lógica. Prefira 4 commits pequenos a 1 grande.
