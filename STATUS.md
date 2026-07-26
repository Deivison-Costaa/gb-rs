# STATUS

> Este arquivo é a **memória do projeto entre iterações**. O contexto do agente
> é descartado a cada iteração; este arquivo não. Mantenha-o curto e verdadeiro.

**Última iteração concluída:** 0008 — guarda do `fetch-test-roms.sh` ([doc](docs/iterations/0008-fetch-test-roms-guard.md)). **Fecha o M0.**
**Próxima tarefa:** ROADMAP 1.1 — registradores AF/BC/DE/HL/SP/PC, flags Z/N/H/C, pares de 8/16 bits. Primeiro item de hardware desde a 0007, então a R1 volta a morder: ler `docs/reference/02-cpu.md` **inteiro** antes de escrever, e `grep` pelo registrador no arquivo todo, não só na seção que o `docs/reference/README.md` aponta (nota 15). **Primeira pergunta a responder pela spec, não pela memória:** a tabela do § Flags Register em `02-cpu.md` lista só os bits 7–4 (`z n h c`) e **não diz nada** sobre os bits 3–0. "O nibble baixo de `F` é sempre zero, inclusive depois de `POP AF`" é folclore que eu ia escrever aqui como fato e não achei escrito em `docs/reference/` nenhum (`03-opcodes.md` l. 289 descreve `POP AF` sem mencionar máscara). Ou se acha a fonte e ela entra em `docs/reference/`, ou o 1.1 não implementa máscara nenhuma.
**Marco atual:** M1 — CPU (sem gráficos)

**Repositório:** https://github.com/Deivison-Costaa/gb-rs

## Placar de ROMs de teste

**121 ROMs baixadas, 0 passando** — ainda não existe emulador. Os totais abaixo
são os que `scripts/scoreboard.sh` mede de fato, e divergem um pouco dos que o
scaffold estimava (a diferença é que cada suíte tem as ROMs individuais **mais**
a ROM agregada).

Desde a 0001 o status das linhas é `crash`, não `skip`: o `gb-cli` existe mas
sai `2` (`EXIT_NOT_IMPLEMENTED`) em qualquer invocação. Ambos contam 0 passando
— **não é regressão**, é o rótulo ficando honesto. Quem plotar o 8.2 tem de
agrupar `skip` e `crash` como "não passa", ou o gráfico inventa um evento.

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
- `gb-core` não tem dependência de I/O. **Agora isso é testado, não prometido:**
  `crates/gb-core/tests/purity.rs` reprova qualquer dependência fora de
  `ALLOWED_DEPENDENCIES` (hoje vazia) e exige `#![forbid(unsafe_code)]` em
  `lib.rs`. Para admitir uma dependência (o 7.3 vai querer `serde`), adicione à
  lista **e justifique no doc da iteração**.
- **Identificadores em inglês; comentários, docs e mensagens de teste em
  português.** A API fala de hardware, cujos nomes são ingleses de origem, e
  `CLAUDE.md` § Arquitetura já usa `bus.rs`/`Cartridge`/`NoMbc`. A prosa é do
  trabalho, e o trabalho é em português.
- **Workspace:** edition 2024, `resolver = "3"`, `rust-version = "1.85"`,
  metadados herdados de `[workspace.package]`. `[profile.release] debug = 1`
  (pânico de opcode em release sem símbolo é indepurável); sem LTO até o fim do
  M1, quando houver tempo de execução real para justificar o custo de CI.
- **Códigos de saída do `gb-cli`** (contrato do `scoreboard.sh`): `0` pass,
  `1` fail, `124` timeout, **qualquer outro** = erro do emulador. Enquanto não
  houver emulador, `run` sai `2`. Nunca reaproveite `0`/`1` para "não
  implementado" — isso planta um veredito falso no `scoreboard.csv`.
  **Os erros do próprio `gb-cli` usam `sysexits.h`:** `64` uso, `65` dado
  inválido (ROM lida, conteúdo não serve), `66` entrada ilegível (caminho
  errado, diretório, sem permissão). Começam em 64 justamente para não colidir
  com código de aplicação. Moram em `crates/gb-cli/src/exit.rs`, com o porquê;
  `crates/gb-cli/tests/info_command.rs` guarda cada um, inclusive o `2` do
  `run` — que some sem ninguém notar quando se mexe no despacho de argumentos.
- **`gb-cli info <rom>` relata, não julga.** Tipo de cartucho fora da tabela,
  RAM sem tamanho atestado, checksum que não bate: tudo sai `0` e vira texto.
  O único erro de conteúdo é estrutural (ROM que acaba antes de `$014F` → `65`).
  A ROM que trava o boot ROM é exatamente a que alguém quer diagnosticar.
  Relatório em `stdout`, erro em `stderr`, e com erro o `stdout` fica **vazio**:
  relatório pela metade tem cara de dado bom.
- **Os três passos de qualidade do job `check` são incondicionais.** Nada de
  `if:` em `cargo fmt` / `cargo clippy -- -D warnings` / `cargo test` — passo
  pulado deixa o job verde sem ter medido nada, e é `check` verde que a proteção
  de `main` exige. `crates/gb-cli/tests/ci_workflow.rs` reprova quem
  reintroduzir a condicional, ou quem tirar o `-D warnings`. O teste mora em
  `cargo test --all`, não no workflow: guarda dentro da coisa guardada some
  junto com ela.
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
  **Agora isso é testado, não prometido:** `crates/gb-cli/tests/fetch_test_roms.rs`
  roda o script de verdade contra um bundle falso local, servido por `file://`,
  e afirma as quatro promessas — as três suítes chegam, `cgb_sound` é podada,
  sha256 divergente derruba o script, segunda execução é no-op. Sem rede, em
  0,1s. O par `TEST_ROMS_BUNDLE_URL`/`TEST_ROMS_BUNDLE_SHA256` é **costura de
  teste**: existe só para apontar o download para um zip local, anda em par (URL
  trocada sozinha esbarra no sha fixado e mata o script), e
  `ci_does_not_override_the_pinned_bundle` reprova o dia em que o `ci.yml`
  mencionar o prefixo. Seam que a produção possa usar por acidente é como se
  perde uma fixação por sha256 sem ninguém notar.
- **`scoreboard.csv` é acumulativo e versionado.** Cada execução anexa; nunca
  truncar. É a série temporal que vira gráfico no ROADMAP 8.2.
- **A série completa mora na branch `scoreboard-data`, não em `main`.** Push em
  `main` dispara `scripts/publish-scoreboard.sh`, que publica lá a **união** do
  que já estava publicado com o que o runner mediu. União, e não substituição:
  o runner mede em cima do CSV do commit que fez checkout, então o CSV local é
  sempre um recorte da série. Não é `main` porque a proteção de `main` exige PR
  e o `GITHUB_TOKEN` não tem bypass — o porquê inteiro está no cabeçalho do
  script e no [doc da 0004](docs/iterations/0004-ci-serie-persistida.md).
  Quem for plotar o 8.2 lê `scoreboard-data`; o CSV de `main` é só o que as
  iterações commitaram à mão.
- **O `GITHUB_TOKEN` deste repositório é `read` por padrão.** Job que precise
  escrever declara `permissions:` **no job** (o `scoreboard` declara
  `contents: write`). Não mova isso para o topo do workflow: daria escrita ao
  `check`, que não precisa. `ci_workflow.rs` reprova quem tirar.
- **`scripts/scoreboard.sh` sai != 0 quando não anexa nenhuma linha.** "Rodou
  sem medir nada" é erro, não sucesso: sair `0` ali deixaria a CI verde com a
  série congelada e o artefato repetindo o CSV do dia anterior.
  `crates/gb-cli/tests/scoreboard_script.rs` guarda isso — e exige a mensagem,
  não só o código de saída, porque morrer por outro motivo já satisfaria um
  teste que só olhasse o código.
- **O job `scoreboard` não pode engolir o veredito do script.** Nada de `if:`
  nem `continue-on-error: true` nos passos de `fetch-test-roms.sh` e
  `scoreboard.sh` — `ci_workflow.rs` reprova. O `upload-artifact` é a exceção e
  leva `if: always()` de propósito: é na execução que morreu que se quer ler o
  CSV parcial.
- **Contrato do `gb-cli`** (definido em `scripts/scoreboard.sh` antes de o
  binário existir, conforme R5) — os itens 0.3 e 1.12 têm de cumprir:
  `gb-cli run <rom> --headless --max-cycles <n>`, saindo `0` = pass, `1` = fail,
  outro = crash, com o token `cycles=<n>` em algum ponto da saída.
- **No `cart`, código desconhecido não é erro; `None` não é número inventado.**
  `CartridgeType`/`RomSize`/`RamSize` guardam o byte cru e devolvem `None`
  quando ele não está na tabela do Pan Docs — o único erro de `parse` é
  estrutural (`TooShort`). Em particular, RAM `$01` e ROM `$52`/`$53`/`$54` são
  `None` **de propósito**: a spec diz que são valores sem cartucho conhecido e
  de origem desconhecida. Não os mapeie "para ficar completo" — o campo vai ser
  impresso e lido como medição. Ver erros #1 e #2 da
  [0005](docs/iterations/0005-cart-header.md).
- **Tabela de RAM não é fórmula.** `$04` são 128 KiB e `$05` são 64 KiB: a
  tabela não é monotônica, e qualquer `32 KiB << n` acerta parte dela e erra
  esses dois. `ram_size_is_a_table_and_it_is_not_monotonic` guarda.
- **O cartucho fala com o barramento por dois métodos, e só.** `Cartridge` é
  `read(u16) -> u8` e `write(u16, u8)`, porque é isso que o hardware expõe: o
  MBC fica entre o barramento e os chips, e banco selecionado, RAM habilitada e
  modo de banking são estado **interno** dele, escrito pelas mesmas escritas em
  `$0000`–`$7FFF` que num cartucho sem MBC não fazem nada. Por isso `write`
  existe em `NoMbc`, onde é no-op: quem chama é o `Bus`, que não sabe qual
  mapeador está do outro lado. O `Bus` (1.2) roteia **só** `$0000`–`$7FFF` e
  `$A000`–`$BFFF`; fora dali o cartucho responde `OPEN_BUS` em vez de entrar em
  pânico — `read` de emulador é o pior lugar para descobrir erro de roteamento.
  **Custo a medir no M1:** `Box<dyn Cartridge>` põe despacho dinâmico no caminho
  mais quente que existe. Trocar por `enum` depois é mudança local.
- **`OPEN_BUS = $FF` é constante nomeada.** O Pan Docs escreve "often `$FF`, but
  not guaranteed" (§ MBC1, `$A000`–`$BFFF`): é valor típico de linha solta, não
  número que algum chip produz. O nome carrega o "not guaranteed" para onde
  alguém for depender dele.
- **`cart::load` despacha por `$0147` e não julga o cabeçalho.** Checksum
  errado, título ilegível e tamanho declarado divergente montam normalmente —
  quem trava a máquina por checksum é o boot ROM, que este emulador pula. Hoje
  só `$00` (ROM ONLY) monta. **`$08`/`$09` são recusados de propósito**, embora
  sejam cartucho sem MBC: são a RAM opcional da § No MBC, e a nota de rodapé da
  tabela de `$0147` diz que nenhum cartucho licenciado os usa e que *"the exact
  behavior is unknown"*. Aceitá-los daria uma RAM que lê `$FF` e engole escrita
  — save perdido em silêncio. Ver erro #1 da [0007](docs/iterations/0007-cart-nombc.md).
- **`NoMbc` mapeia direto e não espelha.** `$0000`–`$7FFF` é o índice na ROM,
  sem tradução; acima de 32 KiB, `NoMbc::new` recusa. Acima do fim de uma ROM
  **menor** que a janela vem `OPEN_BUS`, e não o começo espelhado
  (`addr % rom.len()`, o idioma comum): a spec não descreve chip menor, então
  espelhar seria inventar fiação — e `% 0` ainda entra em pânico com ROM vazia.
- **O título do cartucho é o trecho inicial de ASCII imprimível.** Nada no
  cabeçalho diz se ele tem 16, 15 ou 11 bytes úteis — o que sobra é código do
  fabricante (`$013F`–`$0142`, ASCII!) e CGB flag (`$0143`). Parar no primeiro
  byte não imprimível é diferente de filtrar os não imprimíveis, e a diferença
  só aparece com título curto; ver erro #3 da 0005. **Validado contra ROM real
  na 0006:** `blargg/mem_timing-2/rom_singles/03-modify_timing.gb` tem
  `03-MODIFY_TIMIN` seguido de `$80` em `$0143` — sem a regra, o CGB flag iria
  impresso junto.

## Bloqueios

_(nenhum)_

A proteção de branch **funcionou** no plano atual — não foi preciso o
contorno previsto no prompt de bootstrap.

## Notas para a próxima iteração

1. ~~**A guarda na CI some sozinha.**~~ **Removida na 0002.** E a 0001 a
   descreveu errado: não era código morto, era condicional **viva** dando
   `true`. Código morto não roda e não falha; condicional viva desliga os três
   passos no dia em que o valor virar, sem pintar nada de vermelho. Hoje os
   passos são incondicionais e há teste guardando isso.

2. ~~**As linhas geradas pela CI se perdem.**~~ **Resolvida na 0004** — mas não
   como o item pedia. O 0.2c dizia "commit-back em `main`"; isso é inalcançável
   pela CI, porque a proteção de `main` exige PR e o `GITHUB_TOKEN` não tem
   bypass. A série passou a ser publicada na branch `scoreboard-data`. Ver a
   invariante correspondente e a nota 11.

3. **`scoreboard.csv` vai gerar conflito** se duas iterações mexerem nele em
   paralelo. Projeto é sequencial, então na prática não dói; se doer, resolva
   concatenando os dois lados, nunca escolhendo um. A 0004 **não** piorou isso:
   a CI publica em branch própria e nunca escreve em `main`.

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

7. ~~**O `scoreboard.sh` tinha um bug latente e a 0001 o destravou.**~~
   **Resolvida na 0003 — e a segunda metade dela estava errada.** O bug do
   `grep 'cycles='` sob `set -e` (corrigido com `|| true` na 0001) é real. Mas
   a afirmação de que "o job `scoreboard` não falha quando o script morre assim"
   **não procede**: `run: ./scripts/scoreboard.sh` roda sob `bash -e {0}`, e
   saída != 0 já reprovava o passo e o job. Era inferência, nunca conferida.
   O que faltava mesmo era o inverso — script saindo `0` sem anexar nada — e é
   isso que a 0003 implementou. Ver o erro #3 do
   [doc da 0003](docs/iterations/0003-ci-scoreboard-falha.md).

8. **Escrever teste antes da implementação não torna o teste testado.** O erro
   #1 e #3 da 0001 são a mesma coisa vista de dois lados: código escrito "por
   antecipação" (o `scoreboard.sh` do bootstrap; os guardas de invariante) passa
   verde por vacuidade até algo real exercitá-lo. Quando escrever um guarda,
   force-o a falhar uma vez — nem que seja mutando o alvo à mão e revertendo.

   **Reincidiu na 0002** e o procedimento funcionou:
   `ci_check_job_runs_fmt_clippy_and_tests` passou de primeira, e só passou a
   valer alguma coisa depois de duas mordidas (remover o passo do clippy;
   tirar-lhe o `-D warnings`). Trate "passou de primeira" como suspeita, não
   como notícia boa.

   **E na 0003 apareceu a versão invertida, que engana melhor:** o teste
   *falhou* de primeira — resultado que o ciclo RED→GREEN ensina a comemorar —
   mas falhou por um bug diferente do que ele pretendia medir (`declare -A`
   não associado sob `set -u`, três linhas antes). **Vermelho não é prova; é
   prova o vermelho com a mensagem certa.** Guarda nova deve afirmar o
   *motivo* da falha, não só o código de saída.

   **Terceira reincidência na 0005**, agora medida em vez de suposta: das 12
   mutações aplicadas ao parser, 11 foram pegas e **1 passou verde**, apontando
   o único caso que a suíte de 19 testes não cobria. Rodar a bateria custa
   minutos e diz *qual* teste falta — reler os testes com mais atenção não diz.
   **Inclua sempre um controle negativo** (mutação equivalente, que deve ficar
   verde): sem ele, "tudo foi pego" não distingue suíte boa de suíte que quebra
   com qualquer mudança.

   **Na 0007 o procedimento virou rotina e ganhou duas leituras novas.** (a) O
   primeiro vermelho foi `error[E0432]` — módulo inexistente. Erro de compilação
   não mede asserção nenhuma; para ver o RED de verdade foi preciso um esqueleto
   descartável, e contra ele **4 dos 13 testes passaram**, três por vacuidade
   (afirmam *ausência* de comportamento). Esses três são guarda de regressão
   futura, não medição do código de hoje — e vale saber a diferença. (b) A
   bateria deu 9/9 pegas e 2/2 controles verdes, e isso **não** quer dizer que a
   suíte está completa: os mutantes foram escritos por quem escreveu os testes,
   na mesma sessão, então o ponto cego tende a ser o mesmo nos dois. 9/9 autoriza
   dizer que os nove modos de falha imaginados doem; nada além disso.

9. **Bash: `declare -A m` sem atribuição é variável NÃO associada.** Sob
   `set -u`, `${#m[@]}` e `${!m[@]}` abortam o script — inclusive dentro do
   `if` escrito para tratar o caso vazio. Sempre `declare -A m=()`.
   Testado em bash 5.3; custou a 0003 um teste verde por engano.

10. **O custo por iteração não está sendo medido.** As 0001 a 0004 têm o
    campo `Custo reportado` vazio — sessão interativa, sem
    `--output-format json`. O ROADMAP 8.2 pede exatamente esse gráfico. Nenhum
    item do ROADMAP cobre a coleta hoje; ou o `scripts/loop.sh` passa a
    registrar, ou o 8.2 nasce com quatro pontos faltando.

11. **A publicação da série funciona — observada.** O push de merge da 0004
    (run `30174085591`, commit `dddba9c`) rodou o passo com `success` e criou
    `scoreboard-data` com **726 linhas de dado**, autor `github-actions[bot]`,
    mensagem `chore(scoreboard): +726 linhas de dddba9c`. As 726 são as 605
    commitadas em `main` mais as 121 que aquela execução mediu — a união
    funcionou contra dado real. Confere com
    `git show origin/scoreboard-data --stat`.

    Na execução de PR do mesmo código o passo saiu `skipped`, como o `if:`
    manda. Os dois lados da condição estão observados, não supostos.

    **O que continua inferência:** "o `GITHUB_TOKEN` seria rejeitado ao empurrar
    para `main`" — o fato que decidiu o desenho todo. Veio das configurações
    lidas na API, não de uma rejeição vista. O experimento que fecha a questão:
    rodar o passo uma vez com `DATA_BRANCH=main` e ler o erro. Se for
    `protected branch hook declined`, procede; se passar, o 0.2c podia ter sido
    literal e vale registrar. Custa uma execução; ninguém fez ainda. Ver nota 7
    para o preço de anotar inferência como medição.

12. ~~**O parser do cabeçalho nunca viu ROM de verdade.**~~ **Resolvida na
    0006, e a previsão errou para o lado bom.** As 121 ROMs de `tests/roms/`
    passaram por `gb-cli info`: 121/121 parseadas, 121/121 com checksum
    válido, 121/121 com o tamanho declarado em `$0148` igual ao tamanho do
    arquivo. A regra do título — a apontada como mais provável de destoar —
    **não destoou**, e ganhou uma ROM real que a discrimina (ver invariante).

    **O que a varredura não cobriu:** `desconhecido` aparece **zero** vezes nas
    121. Os ramos de RAM `$01`, ROM `$52` e tipo fora da tabela continuam
    cobertos só por ROM sintética. O corpus é homogêneo — toolchain de homebrew
    moderna, não o mercado de cartuchos. Não conclua da varredura que aqueles
    ramos são código morto.

13. **A MSRV é promessa que ninguém verifica.** `rust-version = "1.85"` está no
    `Cargo.toml` e **nada** o testa: a CI usa `dtolnay/rust-toolchain@stable`,
    então API nova compila, passa no clippy e passa nos testes. A 0006 escreveu
    `u32::is_multiple_of` (Rust 1.87) e só não entrou porque foi notado à mão —
    ironicamente, é o que o próprio clippy recomenda no lugar de `% == 0`.
    Fecharia com um job de CI em `1.85` ou um `cargo-msrv`; nenhum item do
    ROADMAP cobre isso hoje. Ou se fecha, ou se apaga a linha do `Cargo.toml`:
    declaração que ninguém checa é pior que declaração nenhuma.

14. **Bateria de mutação: o cargo decide rebuild por mtime.** Reverter o fonte
    com `mv arquivo.bak arquivo` — ou aplicar uma mutação que falha em silêncio
    — devolve o arquivo com mtime **anterior** ao do artefato, e o `cargo test`
    seguinte roda contra o binário do mutante **anterior**. Aconteceu duas vezes
    na 0006 (erros #5 e #6): uma varredura leu `64 MiB` num cartucho e uma
    mutação do checksum recebeu o veredito da mutação de código de saída. O
    modo de falha é silencioso porque o resultado existe e é plausível — só
    pertence a outro experimento. Escreva o mutante com `touch`/`os.utime`
    explícito, e confira que a substituição casou **exatamente uma vez** antes
    de rodar. **Funcionou na 0007** — 11 mutantes, zero resultado trocado.

15. **A R1 diz "leia a seção correspondente"; a seção correspondente não é a
    única.** Na 0007 a § No MBC descreve, em pé de igualdade com a ROM, uma RAM
    opcional de 8 KiB — e a informação que a desqualifica mora 360 linhas acima,
    numa **nota de rodapé** da tabela de `$0147`: nenhum cartucho licenciado usa
    aquilo e "the exact behavior is unknown". Ler só a seção do item teria
    produzido 8 KiB de RAM inventada com aparência de spec.

    Isso vai reincidir no M1, onde o mesmo comportamento aparece em `02-cpu.md`,
    na tabela do `03-opcodes.md` e nas notas de rodapé das duas. **Antes de
    implementar, `grep` pelo registrador/opcode no arquivo inteiro** — não só na
    seção que o `docs/reference/README.md` aponta.

16. ~~**O 0.5 já está feito, mas não marcado.**~~ **Resolvida na 0008, e a
    conferência valeu o turno.** O script cumpre o item ao pé da letra — 121
    ROMs medidas, três suítes, `cgb_sound` ausente — mas era o único dos quatro
    scripts do projeto sem teste nenhum, e a verificação virou cinco guardas e a
    nota 17. **A moral vale para o resto do ROADMAP:** item entregue pelo
    scaffold merece uma iteração de conferência, não um `[x]` de confiança.

17. **O fallback do `fetch-test-roms.sh` entrega menos do que promete, e sai
    `0`.** Quando o bundle está fora do ar, o script cai nas fontes oficiais por
    suíte — e **não baixa a mooneye**: não existe release pré-montada upstream,
    só o fonte, que exige RGBDS. Isso está escrito e é decisão consciente
    (manter a CI viva em vez de derrubá-la por falha de rede). O problema é o
    resto: `verify()` **imprime** a contagem da mooneye e nunca a confere, e
    `main` sai `0` de qualquer jeito. Numa execução de fallback a CI fica verde
    medindo 46 ROMs em vez de 121, e a única pista é uma linha de `ATENÇÃO:` no
    meio do log — o denominador do placar encolhe 62% sem nada ficar vermelho.

    A 0008 **não** consertou de propósito: inverter "sobrevive à queda da rede"
    para "morre na queda da rede" é decisão de projeto, e contrabandear design
    dentro de uma iteração de verificação é pior do que a nota ficar aberta.
    Quem for mexer decide entre (a) `verify()` conferir a mooneye e `main`
    propagar o veredito, (b) o `scoreboard.sh` recusar rodar com menos ROMs do
    que a execução anterior mediu, ou (c) aceitar e documentar. A (b) é a que
    protege a série da apresentação, que é o ativo de verdade.

18. **Guarda de script bash não sofre a armadilha de mtime da nota 14.** O teste
    lê o `.sh` em tempo de execução, então mutar o script não exige rebuild e o
    veredito nunca é de outro experimento. Mas ganha uma armadilha própria: o
    padrão de busca tem de casar **exatamente uma vez**, conferido antes de
    aplicar. Na 0008 um mutante casou 0 vezes (o literal tinha acento e eu
    escrevi sem) — o harness reportou `mutante inválido` em vez de um verde, que
    é a diferença entre medir e se enganar.
