# STATUS

> Este arquivo é a **memória do projeto entre iterações**. O contexto do agente
> é descartado a cada iteração; este arquivo não. Mantenha-o curto e verdadeiro.

**Última iteração concluída:** 0012 — registradores de hardware no hand-off da boot ROM ([doc](docs/iterations/0012-bus-io-boot-state.md)). Fecha o **1.2b** e com ele o **1.2 inteiro**. As quatro armadilhas que a 0011 deixou anotadas se materializaram todas as quatro, e nenhuma delas era a que se esperava: a coluna DMG saiu certa de memória (`DIV`/`STAT`/`LY`), e o que errou foi folclore de emulador (`DMA=$00`, `OBP*=$FF`, `BANK=$01`) mais a suposição estrutural de que a faixa de I/O é um array plano.
**Próxima tarefa:** ROADMAP 1.3 — laço de M-cycle: `Cpu::step()` avança **um** M-cycle e volta; fetch/decode/execute como máquina de estados. Spec: `docs/reference/02-cpu.md` e a tabela de timing do `03-opcodes.md` (gbops `90b9bf296aed`). **É a R2, e é a regra mais cara de violar do projeto** — "executa a instrução inteira e depois soma N ciclos" passa nos testes unitários, quebra a suíte Mooneye e é refatoração de tudo. **O que já está pronto para ser ligado:** `Registers::after_boot_rom(checksum)` (1.2b-i) e `Bus::new(cart)` (1.2a + 1.2b-ii) dão o estado inicial completo, e nada no código ainda junta os dois — quem cria o dono dos dois é esta iteração. **Duas asperezas conhecidas:** (a) `Bus::new` exige um `Box<dyn Cartridge>`, então testar opcode sem cartucho pede ou um cartucho de teste (é o que `bus_boot_state.rs` faz, em 6 linhas) ou extrair a interface de memória — o 1.2a registrou que extrair **então** é mudança local, e este é o "então"; (b) `Bus::read`/`write` não avançam o tempo de propósito, e é o laço quem tem de chamá-los uma vez por M-cycle, **no ponto certo dentro da instrução** — é isso que a Mooneye mede.
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

Testes do workspace: **131** (eram 122 antes da 0012). Este número não é o placar
— ele mede o que o projeto afirma sobre si mesmo, não o que o hardware cobra.

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

- **`F` carrega os 8 bits: o `gb-core` não mascara o nibble baixo.** O folclore
  diz que os bits 3–0 de `F` são sempre zero e que `POP AF` os descarta. Pode
  ser verdade no silício, mas **não está na spec deste projeto**: a tabela do
  § Flags Register termina no bit 4, e a string `POP AF` não aparece em nenhum
  dos 75 arquivos do Pan Docs no commit fixado (`03-opcodes.md` descreve `F1`
  sem mencionar máscara). Pela R1, o que não está na spec não vira código.
  `f_keeps_the_bits_the_spec_does_not_describe` fixa a **ausência** da máscara,
  para que ela não entre depois por hábito. **Previsão registrada, a conferir e
  não a retroajustar:** se a máscara for necessária, quem cobra é a blargg
  `cpu_instrs/01-special` no 1.13 — e nesse dia a fonte entra em
  `docs/reference/` junto com ela.
- **Flags: `Z`=7, `N`=6, `H`=5, `C`=4, e o Z80 não vale de guia.** O § CPU
  Comparison with Z80 diz que sinal e paridade/overflow **foram removidos** —
  quem portar tabela de flags de Z80 põe `S` onde mora `Z`. As quatro posições
  estão presas por teste ancorado na spec, não por ida e volta pelos acessores.
- **`Registers`: campos de 8 bits públicos, pares de 16 bits são métodos.** Um
  banco de registradores não tem invariante a proteger, e um acessor por
  registrador só engrossaria o decodificador do 1.4. Onde há cálculo (compor e
  dividir os pares), há método. `F` não é campo endereçável no sentido do `r8`:
  a lista `r8` da spec é `b c d e h l [hl] a`, sem `f`.
- **`Registers::default()` é tudo zero, e isso não é o estado pós-boot.**
  O estado pós-boot é `Registers::after_boot_rom(HeaderChecksum)` (1.2b-i).
  Zero é ausência de decisão, não decisão errada — e não dava para ser
  `Default` nem querendo: o estado pós-boot **depende da ROM carregada**.
- **`after_boot_rom` copia a coluna DMG, e as cinco colunas são consoles
  diferentes.** DMG0 é a vizinha e é outro aparelho (`B=$FF`, `E=$C1`,
  `HL=$8403`, `F=$00`): copiar errado dá um estado inteiro, plausível, que
  boota, e que só destoa dentro de um jogo. `this_is_the_dmg_column_and_not_
  one_of_the_other_four` transcreve as outras quatro como controle negativo,
  porque nenhum `assert_eq!` de registrador isolado separa DMG de DMG0.
- **O `F` pós-boot sai do checksum **gravado** em `$014D`, não do calculado.**
  A nota de rodapé da coluna DMG diz que `H` e `C` são limpos quando "the
  header checksum" é `$00` e setados nos outros 255 casos, e liga a palavra à
  § 014D, que abre com *"This byte contains an 8-bit checksum"* — o sujeito é o
  byte gravado; o calculado é a variável `checksum` do trecho em C. **Em
  hardware a distinção não existe** (checksum divergente trava o boot ROM), e
  por isso ela é invisível em toda ROM real. Existe aqui porque este emulador
  pula o boot ROM e `cart::load` não julga o cabeçalho (0.4). É também por isso
  que o parâmetro é `HeaderChecksum` e não `u8`: um byte solto deixaria a
  escolha em cada chamador, onde ela some. Ver nota 21.

- **O `Bus` é `struct`, não `trait`, e é o dono do estado.** O ROADMAP 1.2 dizia
  "trait"; o `CLAUDE.md` § Arquitetura diz que o `Bus` é o dono de tudo e que os
  componentes recebem `&mut Bus`. Ganhou o `CLAUDE.md`: trait com um único
  implementador põe vtable no caminho mais quente que existe num emulador e não
  compra nada. Se o 1.3 quiser memória plana para testar opcodes sem cartucho,
  extrair a interface **então** é mudança local. A divergência está escrita no
  próprio ROADMAP, não só no doc da iteração.
- **`Bus::read`/`write` não avançam o tempo.** A R2 (cycle-stepped) vive no laço
  do 1.3, que os chamará uma vez por M-cycle. Esconder o tique aqui dentro faria
  cada acesso tiquetaquear por conta própria e tiraria a possibilidade de
  posicionar o acesso *dentro* da instrução — que é exatamente o que a suíte
  Mooneye mede.
- **`Region` é público e separado do `Bus`.** O mapa de memória é fato sobre o
  hardware, não sobre o que já está implementado: `$8000` é VRAM hoje, mesmo sem
  PPU para atender. Isso deixa a tabela da § Memory Map testável endereço por
  endereço contra a spec, e foi o que pegou a HRAM de 128 bytes. O `match` de
  `Region::of` é total **sem** `_ =>`: faixa nova tem de quebrar a compilação.
- **A região proibida `$FEA0`–`$FEFF` lê `$00`, não `$FF`.** A § FEA0–FEFF range
  dá `$FF` **só quando a OAM está bloqueada**; no DMG, fora do bloqueio,
  *"reads otherwise return $00"*. Como o bloqueio é estado da PPU e não há PPU
  até o M3, a leitura é `$00`. Quando o 3.6 trouxer o bloqueio por modo, `$00`
  vira o ramo "fora de bloqueio" de uma decisão de dois; a corrupção de OAM que
  a spec cita na mesma frase é o 7.2. Escrita ali se perde — a spec descreve a
  leitura como constante, o que já implica não haver célula.
- **A HRAM tem 127 bytes: `$FF80`–`$FFFE`.** `$FFFF` é o `IE` e tem linha
  própria na tabela. O byte a mais não é RAM, é registrador de interrupções
  (2.2) — e **teste que procura aliasing não pega esse erro**, porque uma HRAM
  de 128 bytes não pisa em `$FFFE`, só anexa um endereço. Ver nota 20.
- **O echo é mais curto que a fonte.** `$E000`–`$FDFF` espelha `$C000`–`$DDFF`:
  os 512 bytes finais da WRAM (`$DE00`–`$DFFF`) **não têm endereço de echo**. A
  máscara de 13 bits que a § Echo RAM descreve (*"only the lower 13 bits of the
  address lines are connected"*) dá isso de graça; o enunciado "echo é a WRAM
  espelhada" é que é falso, e um teste escrito a partir dele afirmaria espelho
  onde não há.
- **Região sem dono lê `OPEN_BUS` e engole escrita — e há teste fixando isso.**
  VRAM e OAM estão no mapa e não têm componente. Isso não é afirmação sobre o
  hardware, é ausência de quem responda; o teste existe para que seja decisão
  visível em vez de lacuna a descobrir depurando a PPU. Pânico seria pior —
  `read` de emulador é o último lugar onde se quer achar erro de roteamento
  (mesma escolha do `NoMbc`, 0.4). Quem ligar um desses componentes vai derrubar
  `the_regions_without_an_owner_are_open_bus_and_swallow_writes`: é o teste
  avisando que chegou a hora, não atrapalhando. **`$FF00`–`$FF7F` e `IE` saíram
  dessa lista na 0012**, e só em parte — ver a invariante da faixa de I/O.
- **A RAM interna começa zerada, e isso é escolha, não hardware.** A § Console
  state after boot ROM hand-off diz que WRAM e HRAM são **aleatórias** ao ligar e
  que os emuladores divergem (constante `$00`/`$FF`, ou sorteio). Constante é o
  que dá teste reprodutível; jogo que dependa disso tem bug, e a própria spec
  desaconselha. O teste nomeia a escolha para que ela quebre se mudar.

- **A faixa de I/O tem dono por endereço, não por região — 41 / 15 / 72.** Dos
  128 endereços de `$FF00`–`$FF7F`, a tabela § Hardware registers nomeia **41**
  (que ganharam célula e valor inicial), marca **15** como `---` (`KEY0`, `KEY1`,
  `VBK`, `BANK`, `HDMA1`–`HDMA5`, `RP`, `BGPI/D`, `OGPI/D`, `SVBK`: registradores
  de CGB, que este console não tem) e **não menciona 72** — entre eles a wave RAM
  inteira, `$FF30`–`$FF3F`, que chega com a APU (6.4). Os 87 últimos leem
  `OPEN_BUS` e engolem escrita, e os dois motivos são diferentes ainda que o
  código não os distinga: `---` é a spec **afirmando** ausência, os 72 são a spec
  se **calando**. Quem decide é `bus/boot.rs::IO_HAS_OWNER`, construído em tempo
  de compilação a partir da mesma tabela que dá os valores — uma lista só.
- **`Bus::new` é o estado de hand-off; não existe `Bus::after_boot_rom`.** A
  assimetria com `Registers::after_boot_rom` é deliberada: lá o estado depende do
  checksum da ROM, e havia o que um construtor parametrizado carregar. Aqui a
  coluna DMG / MGB é literal e este emulador não tem outro estado em que estar —
  ele nunca roda a boot ROM. O que sai de `Bus::new` mistura **spec** (os 41
  registradores) com **escolha** (RAM interna e `OBP0`/`OBP1` zerados), e cada
  escolha tem um teste que a nomeia.
- **`OBP0`/`OBP1` são `??` na spec, e `$00` aqui por escolha.** A nota de rodapé
  diz *"left entirely uninitialized […] tends to be most often $00 or $FF, but
  the value is especially not reliable"*: tendência observada, não fiação. `$FF`
  — o reflexo, "paleta branca" — seria inventar. Mas **não** são `---`: são as
  paletas de objeto da PPU, a célula existe e a escrita pega.
- **Valor inicial não é semântica, e a fronteira está fixada por teste.** Os 41
  registradores são byte cru: sem máscara de bits não usados, sem read-only, sem
  efeito colateral. `DIV` vale `$AB` e vai valer para sempre até o 2.1; `LY`
  aceita escrita; escrever em `DIV` devia zerá-lo e não zera. Divergência
  conhecida e delimitada — `the_named_registers_have_storage_and_no_read_
  semantics_yet` diz "se o componente dono chegou, este teste é que está velho".
- **`DMA` é `$FF` no DMG, não `$00`.** `$00` é a coluna CGB / AGB. É o erro de
  memória mais escorregadio da tabela porque o número é plausível e vem de uma
  coluna vizinha de verdade; ver erro #1 da [0012](docs/iterations/0012-bus-io-boot-state.md).

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

    **A 0009 conferiu uma vez, à mão, e passou:** ela introduziu `const fn` com
    `&mut self` (estável desde 1.83) e atribuição desestruturante em contexto
    const, e `cargo +1.85 test --all` deu **98/98** em `rustc 1.85.1`. Isso é um
    ponto de dado, não uma guarda: dependeu de alguém lembrar de checar. O
    próximo `const fn` — ou o primeiro `let ... else` mais novo que a MSRV —
    entra sem ninguém olhar. A nota continua **aberta**.

    **A 0011 conferiu de novo, à mão, e passou:** `cargo +1.85 test --all` deu
    **122/122**. Segundo ponto de dado, mesma fragilidade — dois acertos
    seguidos por lembrança não são uma guarda, e o custo de fechar isso (um job
    de CI em `1.85`) segue menor que o de descobrir o contrário num PR
    vermelho. Continua **aberta**.

    **Terceiro ponto de dado na 0012:** **131/131** em `1.85`, e desta vez com
    algo novo em jogo — `if let` e `while` dentro de bloco `const`, para montar
    as duas tabelas de `bus/boot.rs` em tempo de compilação. Passou. Continua
    **aberta**, e o 1.3 vai encostar nisso outra vez: máquina de estados de
    M-cycle é onde `const fn` e `match` exaustivo aparecem em quantidade.

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

19. **A R1 protege contra spec não lida. Não protege contra spec omissa.** A
    regra supõe que o modo de falha seja "o agente implementou de memória sem
    ler". A 0009 encontrou o outro: a spec foi lida, e **não dizia nada** sobre
    o ponto em questão (os bits 3–0 de `F`). Omissão não parece omissão quando
    já se tem uma convicção pronta para preencher o buraco — `02-cpu.md` não
    contradiz a máscara do nibble, ele simplesmente não fala dela, e "não
    contradisse" é fácil de ler como "confirmou".

    O que funcionou: transformar a convicção numa **pergunta com endereço** —
    *em que arquivo, em que linha está escrito?* — e ir procurar a linha. Como
    a resposta foi "em lugar nenhum", a busca teve de sair das seções que o
    `docs/reference/README.md` importa e varrer o repositório inteiro do Pan
    Docs no SHA fixado (75 arquivos). Ausência só se demonstra no corpus todo;
    é por isso que a nota 15 não basta aqui.

    E o que se faz com a resposta: **não implementar, e fixar a ausência com um
    teste**, para que a decisão não se desfaça por hábito na iteração seguinte.
    Junto, registrar a previsão falsificável de qual ROM cobraria o contrário —
    assim, se o folclore estiver certo, o projeto descobre por evidência e com
    data, em vez de retroajustar a memória.

20. **Escreva o erro de memória em código, não em prosa — e depois leia qual
    teste falhou.** Até a 0009, o campo `Erros de primeira tentativa` era
    preenchido de cabeça: "eu teria escrito X". Isso é lembrança do que se
    pensou, e lembrança é justamente o que a R1 diz não ser confiável. A 0010
    trocou o procedimento: primeiro os testes lidos da spec, depois um
    **esqueleto descartável com a versão de memória**, e a suíte rodada contra
    ele. O RED virou uma lista de nomes de teste em vez de uma impressão.

    O que só apareceu por causa disso: **um dos testes falhou em pegar o erro
    para o qual foi escrito.** O erro era "HRAM tem 128 bytes e engole o
    `$FFFF`", e o teste procurava *aliasing* — escrever em `$FFFF` e ver se
    `$FFFE` mudava. Não muda: uma HRAM de 128 bytes não colide com nada, ela
    apenas anexa um endereço que pertence ao `IE`. O teste passou verde contra o
    esqueleto errado, e quem pegou o erro foi a varredura de regiões, escrita
    para outra coisa.

    É a nota 8 com o sinal trocado pela terceira vez (quarta, com a 0011) — depois do guarda vacuoso
    (0001) e do vermelho pelo motivo errado (0003), agora o **guarda que mira no
    sintoma em vez da afirmação**. A correção foi afirmar a afirmação:
    `Region::of(0xFFFF) == Region::InterruptEnable`, e não a ausência de colisão.
    Sem o esqueleto, a suíte teria fechado 13/13 verde com um teste inútil dentro
    e ninguém saberia. **Um esqueleto errado é barato e mede o que a prosa não
    mede** — vale repetir em todo item de comportamento de hardware.

    **Repetido na 0011, com resultado limpo e uma leitura a mais.** 8 dos 11
    testes passaram contra o esqueleto e 3 pegaram o erro, que era um só e
    estava exatamente onde a iteração anterior previu. Dois dos oito verdes
    foram os **controles negativos da própria previsão** — a coluna DMG estava
    certa de memória — e um foi verde por acidente: o teste do nibble baixo de
    `F` passa contra `f: 0xB0`, porque `$B0` também tem o nibble zerado. Ele só
    mede alguma coisa contra o mutante `f: 0x0F`. Sem o esqueleto isso passaria
    por cobertura.

    **Corolário de método:** o esqueleto tem de ter a **assinatura final**, não
    só o corpo errado. Com assinatura diferente o vermelho é `error[E0432]`/
    `E0061` e não mede asserção nenhuma — é a armadilha (a) da 0007 outra vez.

    **Quinta repetição na 0012, e a mais produtiva até agora: 5 dos 9 testes
    reprovaram o esqueleto, apontando 4 erros de memória distintos.** Aqui o
    procedimento foi barato porque a API não mudou (`Bus::new`/`read`/`write` já
    existiam), então não houve corolário de assinatura a pagar — quando a
    iteração só muda comportamento de coisa já construída, o esqueleto custa
    dois `Edit` e devolve a lista inteira.

    **A leitura nova é sobre o RED "de verdade":** contra a implementação
    *anterior* (tudo `OPEN_BUS`), 5 dos 9 testes já passavam — todos os que
    afirmam ausência (`---`, endereços sem dono, controle negativo de coluna).
    Com tudo respondendo `$FF`, "não é `$00`" é verdade de graça. **O esqueleto
    foi o único momento em que esses cinco tiveram algo real para reprovar**, e
    quatro reprovaram. Sem ele o PR fecharia 9/9 verde com cinco testes que
    nunca tinham sido exercitados contra nada.

21. **O terceiro modo de falha da R1: spec ambígua.** A regra supôs "o agente
    não leu" (original); a nota 19 achou "a spec é omissa"; a 0011 achou
    **"a spec é ambígua, e as duas leituras coincidem em todo caso real"**.

    A nota de rodapé do `F` pós-boot diz "if the header checksum is $00", e
    "header checksum" nomeia duas coisas: o byte gravado em `$014D` e o valor
    calculado de `$0134`–`$014C`. Num Game Boy os dois sempre batem — checksum
    divergente **trava o boot ROM** — então a leitura errada nunca produziria
    diferença observável em ROM comercial, em ROM de teste ou em jogo. Só em
    ROM corrompida, que é o caso que a 0007 decidiu tratar como cidadão de
    primeira classe.

    Isso é pior que omissão: omissão deixa um buraco visível, ambiguidade
    entrega um valor plausível dos dois lados. E é pior que erro comum, porque
    **nenhuma ROM do scoreboard vai falsificá-lo** — nem a Mooneye
    `boot_regs-dmgABC` do 7.1, cuja ROM tem checksum válido como qualquer
    outra. Decisão que nenhum teste futuro pode derrubar tem de ser fixada por
    teste no dia em que é tomada, ou some.

    O que funcionou é o movimento da nota 19 de novo: virar a dúvida em
    **pergunta com endereço** — *qual byte, em que endereço?* — e ir atrás da
    frase. A resposta estava em **outro arquivo** do `docs/reference/`
    (`08-cartridges-mbc.md` § 014D, primeira frase), alcançada pelo link da
    própria nota de rodapé. Reforça a nota 15: seguir os links da seção, não só
    ler a seção.

22. **A previsão de qual armadilha vai doer erra — e o registro é o que mostra
    isso.** A 0011 deixou quatro avisos para a 0012, todos corretos e todos
    materializados. Mas o aviso implícito mais forte — *cuidado para não copiar
    a coluna DMG0* — não era o risco: `DIV=$AB`, `STAT=$85` e `LY=$00`, as três
    únicas células que separam DMG0 de DMG / MGB, saíram **certas** de memória
    nas duas iterações seguidas em que foram medidas.

    O que errou foi outra classe: **folclore de emulador**. `DMA=$00`,
    `OBP0/OBP1=$FF`, `BANK=$01` — três valores que circulam em tutoriais e
    tabelas de terceiros, não em nenhuma coluna do Pan Docs. `$00` para `DMA`
    até é uma coluna real (CGB / AGB), o que faz o erro parecer "coluna errada"
    sem ter sido.

    **Consequência prática:** o controle negativo por coluna continua valendo,
    mas não é o que pega mais. O que pega é a lista de valores conferida linha a
    linha contra a tabela transcrita **no teste** — e as notas de rodapé, que na
    0012 foram onde morava a informação que desqualificava dois dos três erros.

23. **A nota 15, terceira reincidência, agora a 30 linhas de distância.** O que
    desqualifica `OBP0`/`OBP1` não está na linha da tabela: está numa nota de
    rodapé abaixo dela, referenciada por um marcador que na renderização é só um
    `??`. A 0007 tinha o alvo a 360 linhas e em outra seção; a 0011, em outro
    arquivo; a 0012, logo abaixo — e ainda assim é fácil ler a tabela sem seguir
    os marcadores. **`??` e `---` numa tabela de spec são ponteiros, não
    valores.** Nunca traduza um deles para número sem ler a nota que o define.
