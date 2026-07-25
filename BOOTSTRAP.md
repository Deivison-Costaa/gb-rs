# Como pôr isso pra rodar

## 1. Preparar a pasta

```bash
mkdir gb-rs && cd gb-rs
# copie o conteúdo deste scaffold para cá
chmod +x scripts/*.sh
gh auth status   # precisa estar logado
```

## 2. O prompt de bootstrap

Abra `claude` na pasta e cole **exatamente isto**, uma vez só:

---

Este diretório contém o scaffold de um projeto novo. Leia `CLAUDE.md`,
`ROADMAP.md` e `STATUS.md` antes de fazer qualquer coisa.

Sua tarefa agora é **só o bootstrap** — não implemente nada do ROADMAP ainda.

1. `git init`, primeiro commit com o scaffold como está.
2. Crie o repositório no GitHub com `gh repo create gb-rs --public --source=. --push`.
3. Proteja `main`: exija PR e CI verde antes de merge (`gh api` se necessário; se
   a proteção de branch não estiver disponível no plano, registre isso em
   `STATUS.md` e siga sem ela).
4. Crie `scripts/fetch-test-roms.sh`, que baixa para `tests/roms/`:
   - blargg test roms (cpu_instrs, instr_timing, mem_timing, mem_timing-2,
     halt_bug, oam_bug, interrupt_time, dmg_sound)
   - mooneye-gb acceptance suite
   - dmg-acid2
   Adicione `tests/roms/` ao `.gitignore` — não commite ROMs.
5. Crie `scripts/scoreboard.sh`: roda todas as ROMs baixadas via
   `gb-cli` em modo headless, com timeout por ROM, e **anexa** uma linha a
   `scoreboard.csv` no formato:
   `timestamp,commit,suite,rom,status,ciclos`
   Enquanto `gb-cli` não existir, o script deve sair com sucesso reportando 0
   de tudo — ele precisa funcionar desde o dia 1 para a CI não quebrar.
6. Popule `docs/reference/` com o material que será a fonte de verdade
   (regra R1 do CLAUDE.md): baixe do Pan Docs (gbdev.io/pandocs) as seções de
   mapa de memória, CPU/instruções, timers, interrupções, PPU/rendering, APU e
   MBCs, além da tabela de opcodes (gbops). Um arquivo markdown por tema.
   Este passo é o mais importante do bootstrap — sem ele o projeto inteiro
   vira chute.
7. Commite tudo em commits pequenos e separados por assunto.
8. Atualize `STATUS.md`.

**Pare aqui.** Não comece o item 0.1 do ROADMAP. Ao terminar, me diga o que
ficou pronto e o que faltou.

---

## 3. A partir daí, o loop

Cada iteração é uma **sessão nova de contexto**. Duas formas:

**Supervisionado (comece por aqui):**

```
/clear
/iterate
```

Repita. Você lê o resumo entre uma e outra, e intervém quando algo cheirar mal.

**Desatendido:**

```bash
./scripts/loop.sh 3
```

Cada volta do `for` é um processo `claude -p` novo — contexto limpo por
construção, sem risco de estourar a janela.

## 4. Cuidados

- **Comece com `loop.sh 1`.** Só suba para 3, depois 5, depois de ver
  iterações consecutivas saudáveis. Um loop desatendido com CLAUDE.md ruim
  produz 8 PRs ruins em vez de 1.
- **O loop consome sua cota do plano Max.** Rodar a noite inteira pode esgotar
  a janela de 5 horas. `--max-turns` no script é o freio; ajuste se as
  iterações começarem a ser cortadas no meio.
- **Leia `STATUS.md` toda manhã.** É o único lugar onde o projeto sabe de si.
- **Não deixe `docs/iterations/` para depois.** Se você pular a documentação em
  10 iterações "para ganhar tempo", perdeu a apresentação e ganhou um emulador
  igual a qualquer outro do GitHub.
