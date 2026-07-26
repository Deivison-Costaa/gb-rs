# Invariantes estabelecidas

> Decisões que valem para o projeto inteiro. Saíram do `STATUS.md` para
> não entrar em contexto a cada iteração; o índice lá aponta para cá.


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

- **`Cpu::step(&mut Bus)` avança um M-cycle e não devolve contagem.** A
  assinatura é metade da R2: não há número de ciclos a retornar porque a
  resposta é sempre um, e não há laço interno até a instrução acabar porque
  parar no meio é o objetivo. Uma chamada faz **no máximo um** acesso ao
  barramento — os M-cycles `internal` da tabela não fazem nenhum. Timer (2.1),
  PPU (M3) e APU (M6) são tiquetaqueados aqui quando existirem.
- **O `fetch` é o primeiro M-cycle da instrução, e ele conta.** `NOP` é 1
  M-cycle e esse M-cycle é o próprio fetch; `JP u16` é 4, dos quais o fetch é o
  primeiro. É a contabilidade de gbops, **não** uma afirmação sobre o pipeline
  do silício (no hardware o último M-cycle se sobrepõe ao fetch seguinte). Os
  dois modelos dão o mesmo total por instrução, e é o total que a Mooneye cobra.
- **`JP u16` desvia no M4, não no M3.** Depois de `read(u16:upper)` o alvo já
  está inteiro dentro da CPU e ainda falta um M-cycle: a coluna é
  `fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?)`.
  Escrever o `PC` junto com o byte alto dá as mesmas 4 M-cycles e desloca o
  desvio em um — invisível em teste de instrução isolada, visível para o timer e
  a PPU. `each_step_advances_one_m_cycle_not_one_instruction` observa o `PC`
  **entre** os quatro passos justamente porque no fim os dois modelos coincidem.
- **`Cpu` não é dono do `Bus`.** Recebe `&mut Bus` a cada M-cycle
  (`CLAUDE.md` § Arquitetura). Não existe tipo `GameBoy`: quem possui os dois
  hoje são os testes, e a partir do 1.12 será o `gb-cli run`. **A aspereza que a
  0012 previu não existiu** — testar opcode sem cartucho não pediu extrair
  interface de memória do `Bus`; um cartucho de teste de 32 KiB com o programa
  em `$0100` custa seis linhas, e o `Bus` segue `struct` concreto sem vtable.
- **Não há tabela de micro-operações, e a ausência é deliberada.** `State`
  nomeia os M-cycles das duas instruções que existem e o `match` é total sem
  `_ =>`. Um `enum MicroOp` genérico agora seria a nota 8 outra vez: abstração
  sem nada que a exercite. Quem generaliza é o 1.4, que terá casos.
- **`Lockup` distingue "o SM83 não tem esse opcode" de "este emulador ainda não
  chegou nele".** Mesmo efeito, origens opostas, consertos em lugares
  diferentes: `IllegalOpcode` significa que a ROM executou lixo,
  `UndecodedOpcode` que falta implementar. Parar em vez de entrar em pânico é o
  que mantém o `gb-core` como máquina de estados (mesma escolha do `NoMbc`).
  São **onze** os inexistentes — `D3 DB DD E3 E4 EB EC ED F4 FC FD`. `D9` é
  `RETI` no Game Boy (era `EXX` no Z80) e **não** entra na lista; quem contar os
  `-` da coluna do Z80 conta outra coisa.
- **Onde a CPU para de travar é escolha, não spec.** A § Moved, Removed, and
  Added Opcodes diz que o opcode trava a CPU e não diz nada sobre o `PC`. O
  `PC = $0101` daqui é consequência do modelo de fetch — o byte é lido e o `PC`
  passa por ele antes de haver o que decodificar — e nada observável depende
  disso, porque a CPU não volta a andar.

- **`$76` é `HALT`, e é o único buraco de um bloco perfeitamente regular.** A
  § Block 1 do `02-cpu.md` chama isso de **exceção**, com todas as letras:
  *"trying to encode `ld [hl], [hl]` instead yields the `halt` instruction"*.
  Um decodificador escrito direto dos bits acerta 63 e transforma o 64º numa
  leitura e numa escrita no mesmo endereço — sem efeito visível, sem travar, e
  qualquer ROM que use `HALT` para esperar interrupção gira para sempre. Até o
  2.3 ele para a CPU com `UndecodedOpcode`, que é o rótulo certo: o opcode
  existe, falta implementar. **Está garantido por dois caminhos** (o `const
  HALT` antes da faixa, e o braço `(MemoryAtHl, MemoryAtHl)` que só existe para
  o `match` ser total) — e a bateria da 0014 mediu que **nenhum teste os
  distingue**: remover qualquer um deixa a suíte verde. Redundância conhecida,
  não descuido.
- **`LD r,(HL)` faz a leitura e a escrita no registrador no mesmo M2.** Dois
  M-cycles, 8 T-cycles, coluna `fetch → read((HL)->r)`. Não há terceiro M-cycle
  `internal` onde a escrita "aconteça de verdade" — e supor que houvesse é a
  descoberta da 0013 (`JP u16` desvia no M4, não no M3) aplicada onde ela não
  vale. **A coluna é por instrução; não há regra geral sobre quando o efeito
  acontece.** Ver nota 26.
- **`R8` e `ByteRegister` são dois tipos.** O operando `r8` tem oito valores e
  o oitavo — o índice 6 — é memória, não registrador. Separar é o que deixa
  `State::LoadFromHl(ByteRegister)` carregar um destino que **é** registrador
  sem `unreachable!` (a R6 não quer pânico no `gb-core`), e o que faz as três
  formas de M-cycle do bloco caírem sozinhas do `match` sobre os dois campos.
  O mapeamento `ByteRegister` → campo de `Registers` mora no **decodificador**,
  coerente com a decisão do 1.1 de não pôr acessor por registrador no banco.
- **`LD (HL),u8` (`$36`) lê o imediato no M2 e escreve em `(HL)` no M3.** Três
  M-cycles, 12 T-cycles, coluna `fetch → read(u8) → write((HL))`. É a primeira
  instrução do projeto com **dois** acessos ao barramento, e eles são M-cycles
  diferentes — o que também é o que mantém a invariante de que uma chamada de
  `Cpu::step` faz no máximo um acesso. Juntar os dois no M2 e gastar o M3 num
  `internal` dá os mesmos 12 T-cycles e o mesmo estado final, e adianta a
  escrita em um. **Nove dos dez testes do 1.4b passam contra a versão errada**:
  quem a reprova é a única asserção que lê a memória *entre* o M2 e o M3. Ver
  nota 30.
- **`LD r,u8` faz a leitura e a escrita no registrador no mesmo M2.** Dois
  M-cycles, coluna `fetch → read(u8->r)` — a mesma forma do `LD r,(HL)`, com o
  endereço vindo de `PC` (que anda) em vez de `HL` (que não). Dois bytes, não um.
- **O bloco `00 ddd 110` não tem exceção.** A § Block 1 tem o `$76`; a § Block 0
  não tem nada equivalente para `ld r8, imm8`. O índice 6 do destino dá o `$36`,
  que é load como os outros sete. Procurar a exceção e **não** achar é resultado,
  não descuido — está registrado para não ser procurado de novo.
- **Opcode reconhecido por máscara pede controle negativo dos 256.** O bloco do
  1.4b anda de 8 em 8, então o `fetch` o reconhece com
  `opcode & 0b1100_0111 == 0b0000_0110` e não com um `..=`. Os vizinhos que uma
  máscara frouxa engole são `INC r8` (`00 ddd 100`), `DEC r8` (`00 ddd 101`) e
  `RLCA`/`RRCA`/… (`00 ddd 111`) — e **nenhum teste de comportamento do
  sub-item os menciona**, porque eles não são deste sub-item. A bateria mediu:
  os dois mutantes de máscara só morrem na varredura dos 256.
- **`decoded_elsewhere` mora só em `tests/support/mod.rs` desde a 0026 — a
  duplicação de propósito foi revertida.** Esta mesma linha, até a 0025,
  defendia o oposto: que a cópia por arquivo obrigava quem acrescentasse
  opcode a vir declarar o que acrescentou. O ROADMAP 0.6 decidiu que o custo
  cresceu rápido demais (9 arquivos na 0023, 12 na 0025) para o benefício
  continuar valendo a pena, e consolidou. Cada consumidor ganha
  `mod support; use support::decoded_elsewhere;` — opcode novo ainda se
  declara, só que num lugar só. Ver nota 47: consolidar tirou uma proteção
  acidental que a redundância dava de graça, e os 12 `sweep`s precisaram de
  um ramo a mais para não perdê-la.
- **Ainda não há tabela de micro-operações, e a data mudou.** A 0013 previu que
  ela nasceria no 1.4. Com o 1.4 quebrado em quatro, o 1.4a tem três formas e
  todas da mesma família (`fetch` + no máximo um acesso), e generalizar dali
  seria a nota 8 com um terço dos dados. A decisão está escrita **no ROADMAP**,
  no 1.4d, onde as quatro formas de endereçamento do `x8/lsm` já existirão.
  **O 1.4c não mexeu nisso, e por um motivo que vale registrar:** ele
  acrescentou oito opcodes e **nenhuma forma nova** — as oito linhas são `fetch`
  mais um acesso, a família do 1.4a. O que ele trouxe foi um efeito colateral
  *dentro* do passo do acesso, que é dimensão diferente de forma. Três sub-itens
  e o que se repete é a mesma família; quem tem forma nova é o 1.4d (4 M-cycles,
  operando de dois bytes, e o último passo sendo o acesso e não um `internal`).

- **Os oito opcodes `r16mem` são 2 M-cycles, e o `HL±` é do M2.** A coluna
  escreve o efeito **dentro** do passo do acesso — `write(A->(HL++))`, não
  `write(A->(HL))` seguido de nada —, então depois do fetch o par ainda vale o
  que valia. Resolver o endereço no decodificador e guardá-lo no `latch` é o
  caminho que o código pede (o fetch já tem `&mut self`, o `latch` existe para
  isso) e adianta o `HL±` em um M-cycle, com o **mesmo** estado final e os mesmos
  8 T-cycles. **10 dos 11 testes do 1.4c passam contra essa versão**; quem a
  reprova é a única asserção que lê `HL` *entre* os dois `step`. É por isso que
  `State::LoadFromR16Mem` carrega o **par** e não o endereço: com o par, o efeito
  não tem onde acontecer antes do M2. Ver nota 32.

- **`LD r16,u16` escreve meia metade por M-cycle, e não o par no fim.** Três
  M-cycles, 12 T-cycles, coluna `fetch → read(u16:lower->C) → read(u16:upper->B)`:
  a metade **baixa** já está no registrador ao fim do M2. O `JP u16` está no
  mesmo arquivo, tem operando do mesmo tamanho, e latcha os dois bytes para
  escrever o `PC` de uma vez — mas ele tem **quatro** passos e o quarto é
  `internal`; aqui os três passos são todos acesso e não há onde o par
  "acontecer de verdade". **7 dos 8 testes do 1.5a passam contra a versão que
  latcha.** Ver nota 34.
- **A seta da coluna faz parte do passo.** `read(u16:lower->C)` e
  `read(u16:lower)` são M-cycles diferentes, e gbops escreve os dois: o `$FA`
  (`LD A,(u16)`) tem o segundo, porque ali o byte vai mesmo para um latch
  interno. Ler a seta como decoração foi o erro #1 da 0018.
- **`r16` é `bc de hl sp`; `af` é do `r16stk`, que é outra tabela.** São dois dos
  quatro placeholders de par que a § CPU Instruction Set define, e a conversão
  para Markdown fundiu os quatro numa tabela só, sem cabeçalho (nota 24) — o que
  deixa a distinção legível apenas pela **ordem** dos blocos: `r8`, `r16`,
  `r16stk`, `r16mem`, `cond`. `the_fourth_pair_of_r16_is_sp_and_not_af` fixa o
  índice 3 do `r16` para que a tabela vizinha, que chega no 1.5b, não o
  sobrescreva por contágio.
- **`SP` é a única metade de par que não é um campo de 8 bits.** `B`/`C`,
  `D`/`E`, `H`/`L` são `u8` no banco; `SP` é `u16`. Por isso existem
  `write_r16_low`/`write_r16_high`: "escrever a metade baixa" é atribuição em
  três casos e máscara no quarto. Esquecer o `& 0xFF00` é invisível no estado
  final — a metade alta é reescrita no M3 logo em seguida — e só o teste que lê
  o par **entre** os M-cycles o pega.
- **`LD HL,SP+i8` (`$F8`) não é do `x16/lsm`.** gbops o classifica em `x16/alu`
  e o ROADMAP o tem no 1.7. A intuição o põe junto do `$F9` (`LD SP,HL`, que é
  `lsm`) porque o mnemônico começa igual e ele escreve num par de 16 bits; o que
  o separa é calcular flags — e as dele são as contraintuitivas (`H`/`C` sobre o
  **byte baixo**).
- **O endereço é o valor de antes do `HL±`, e quem confirma isso não é a § Block
  0.** O `++`/`--` postfixo de gbops afirma; a confirmação independente está na
  § OAM Corruption Bug do `06-ppu.md`, que fala do conteúdo de 16 bits
  *"(before the operation)"* — arquivo que o `README.md` do `docs/reference/`
  mapeia para o **M3**, três marcos adiante. A mesma seção descreve o `HL±` como
  evento de **barramento** (a IDU põe o valor nas linhas de endereço mesmo sem
  acesso assertado, e é por isso que essas quatro instruções corrompem a OAM
  *duas* vezes). Isso **não** está implementado — é o 7.2 —, mas desmente a
  leitura natural de que o incremento seja aritmética silenciosa de registrador.
- **`LD (HL+),A` com `HL` apontando para o destino não é caso especial.** A 0015
  anotou isso como a armadilha em que a ordem de operações viraria valor
  observável. Foi conferido: `HL` não é memória mapeada, então "escreve em
  `(HL)` e incrementa" e "incrementa e escreve em `(HL)-1`" são indistinguíveis
  em qualquer endereço, `$FFFF` incluído. Armadilha prevista que **não existe** —
  registrada para não ser procurada de novo (nota 22).
- **A máscara dos oito `r16mem` tem duas formas, e as duas estão em uso de
  propósito.** `opcode & 0b1100_0111 == 0b0000_0010` reconhece o conjunto (é a
  **mesma** máscara do 1.4b, com outro padrão) e é o que os controles negativos
  usam; o decodificador usa `0b1100_1111` com dois padrões, porque ele precisa
  da **direção**, que mora no bit 3 e que a máscara de 7 bits apaga. Unificar as
  duas colapsaria `LD (BC),A` e `LD A,(BC)` na mesma instrução.

- **Os seis do 1.4d são reconhecidos um a um, sem máscara — e isso fecha o
  `x8/lsm`.** Não há máscara que pegue `$E0`, `$E2` e `$EA` sem levar `$E8`
  (`ADD SP,i8`) e `$F8` (`LD HL,SP+i8`), que são o 1.7: os bits constantes dos
  seis deixariam passar dezesseis opcodes. O `fetch` tem seis braços literais e
  o controle negativo trava os seis exatos na varredura dos 256. **A "tabela de
  micro-operações" se decidiu contra si mesma:** quatro sub-itens esperando
  dados, e o desenho escolhido é `State` por variante de forma + **uma** função
  compartilhada, `Cpu::access(bus, direction, address)` — o último passo das
  três formas do 1.4d, e a primeira função de M-cycle do projeto que serve a
  instruções diferentes. A abstração nasceu onde a repetição existia, não onde
  se apostava: quatro linhas, e não um `enum MicroOp`. O 1.5 (`PUSH`/`POP`:
  dois acessos com `SP` modificado no meio) e o 1.6 (ALU: efeito em flags) são
  os próximos a testar a decisão.

- **O `--SP` do `PUSH` é do passo da escrita, e o `internal` do M2 não faz nada.**
  Quatro M-cycles, 16 T-cycles, coluna
  `fetch → internal → write(B->(--SP)) → write(C->(--SP))`: **pré**-decremento
  escrito dentro do acesso, exatamente a notação do `write(A->(HL++))` do 1.4c.
  Decrementar no `internal` e pós-decrementar nas escritas dá o mesmo estado
  final, os mesmos 16 T-cycles e a mesma memória — **8 dos 10 testes passam
  contra essa versão**. O que a delata é o `SP` lido depois do M2 e depois do M3.
  É a nota 26/30/34 pela quarta vez, e a fonte da regra falsa é o **Z80**, onde o
  decremento mora no T-cycle extra do M1 (ver nota 36).
- **A metade alta vai primeiro, para o endereço mais alto.** `write(B->(--SP))`
  antes de `write(C->(--SP))`: a pilha fica little-endian (byte baixo no endereço
  baixo), e é o `POP` do 1.5c que tem de ler na ordem inversa para fechar. Isso
  **está** no estado final, ao contrário do erro acima — inverter é visível em
  qualquer teste que leia a pilha.
- **`Cpu::push_byte` é indivisível de propósito.** Decrementa e escreve, nessa
  ordem, numa função só — porque é separar os dois que produz o erro do
  `internal`. Segunda função de M-cycle compartilhada do projeto, depois do
  `Cpu::access` do 1.4d; o `POP` (1.5c) ganha a simétrica e o `CALL`/`RST` (1.10)
  reusam esta. **A simetria é de papel, não de notação:** aqui é `(--SP)`
  (pré-decremento), lá é `(SP++)` (pós-incremento).
- **`R16Stk` e `R16` são dois tipos, e a quarta variante é a diferença inteira.**
  `bc de hl af` × `bc de hl sp`. Fundir os dois num tipo com parâmetro "qual
  tabela" poria numa posição de argumento a distinção que a § CPU Instruction Set
  define como duas tabelas — e é `af` × `sp` no índice 3 que os separa.
  `the_fourth_pair_of_r16stk_is_af_and_not_sp` e
  `the_fourth_pair_of_r16_is_sp_and_not_af` guardam os dois lados.
- **`PUSH AF` escreve os 8 bits de `F`.** É a metade da decisão do 1.1 que
  *escreve* o nibble baixo não mascarado; `POP AF` (1.5c) é a que lê.
  `push_af_writes_the_whole_f_byte_including_the_low_nibble` fixa a **ausência**
  da máscara aqui, e a previsão do 1.13 continua de pé, não retroajustada.
- **O `SP` dá a volta abaixo de `$0000`:** `PUSH` com `SP = $0000` escreve em
  `$FFFF` (o `IE`) e `$FFFE` (o último byte da HRAM). `wrapping_sub`, e o teste
  que o fixa é o **único** algoz do mutante `saturating_sub`.

- **O `(SP++)` do `POP` é pós-incremento, e a metade baixa vem primeiro.** Três
  M-cycles, 12 T-cycles, coluna `fetch → read((SP++)->C) → read((SP++)->B)`: lê
  **em** `SP` e só então anda. É a construção do `write(A->(HL++))` do 1.4c outra
  vez, com o sinal e o lado trocados. A metade baixa sai do endereço mais baixo,
  o que é o que faz a pilha do `PUSH` (metade alta primeiro, para o endereço mais
  alto) fechar.
- **`Cpu::pop_byte` é a simétrica de `push_byte`, e a simetria é de papel.**
  `push_byte` decrementa e escreve; `pop_byte` lê e incrementa. **Trocar as duas
  linhas de lugar não produz a outra** — `(--SP)` é pré, `(SP++)` é pós. Terceira
  função de M-cycle compartilhada do projeto, depois do `Cpu::access` (1.4d) e do
  `push_byte` (1.5b); o `RET`/`RETI` (1.10) reusa esta.
- **O mesmo erro de instante é barulhento no `POP` e silencioso no `PUSH`, e a
  causa é a instrução, não a suíte.** Deslocar o `±SP` em um passo: no `PUSH`
  escreve nos mesmos dois endereços, na mesma ordem, com o mesmo `SP` final —
  **8 dos 10 testes passam**. No `POP` lê em `SP+1` e `SP+2` em vez de `SP` e
  `SP+1` — **7 dos 10 reprovam**. O `PUSH` decide o endereço *antes* do acesso; o
  `POP` decide *no* acesso. **Corolário para o 1.10:** `RET` verde não autoriza
  concluir nada sobre o instante do `CALL`. Ver nota 40.
- **`write_r16_stk_low`/`_high` não reusam as do `R16`.** A quarta variante é
  `af` × `sp`, e `SP` é o único par cuja metade não é campo de 8 bits (1.5a).
  Converter entre as duas tabelas seria fazê-lo exatamente onde elas divergem.
- **`POP AF` lê os 8 bits de `F`.** É a metade da decisão do 1.1 que *lê* o
  nibble baixo não mascarado; `PUSH AF` (1.5b) é a que escreve. A previsão do
  1.13 continua de pé e **não** foi retroajustada. `pop_af_loads_the_whole_f_byte_
  including_the_low_nibble` fixa a ausência da máscara.
- **`$F1` é a única linha do bloco com flags, e o teste do `PUSH` não se
  espelha.** `no_push_touches_the_flags` vale para os quatro `PUSH` — inclusive o
  `PUSH AF`, que *lê* `F`. O espelho literal (`no_pop_touches_the_flags`) estaria
  errado num quarto dos casos, e quem denuncia é a coluna de flags: `$C1 $D1 $E1`
  têm `-`, `$F1` tem `Z N H C`. O teste que ficou no lugar afirma a assimetria.

- **O instante do `LD SP,HL` é escolha deste projeto, e a spec local aponta para
  o outro lado.** A coluna do `$F9` é `fetch → internal`, sem seta e **sem
  anotação** — primeiro caso do projeto em que ela não decide (nota 21). As duas
  únicas linhas do `03-opcodes.md` que dizem quando o `SP` recebe um valor são o
  `$33` (`fetch(Probably writes to SP:lower here) → internal(Probably writes to
  SP:upper here)`) e o `$E8`, e as duas **partem o par em duas metades, a baixa
  primeiro**. Não foram aplicadas por analogia porque são `x16/alu` e não
  `x16/lsm`, e porque `Probably` é o gbops declarando chute. O implementado é o
  par inteiro no `internal`, como o `JP u16` — cujo `internal` é o único que a
  tabela anota de verdade (`branch decision?`). O `$F8`, vizinho de mnemônico,
  tem `internal` **pelado** e não sustenta nada: citá-lo como precedente foi o
  erro #2 da 0021. `the_write_to_sp_is_in_the_internal_and_the_column_does_not_
  say_so` prende a escolha e diz, no próprio assert, que ela é escolha.
- **O `(u16+1)` do `$08` é endereço no mapa inteiro, não índice dentro da
  região.** `LD ($FFFE),SP` grava a metade baixa no último byte da HRAM e a alta
  no `IE`; `LD ($FFFF),SP` dá a volta para `$0000`, que é ROM e engole a metade
  alta. `wrapping_add`, como o `SP` do `PUSH` — `saturating_add` poria as duas
  metades no mesmo endereço.
- **O `$08` e o `PUSH` guardam o mesmo layout little-endian escrevendo em ordens
  opostas.** Aqui a metade **baixa** vai primeiro, para o endereço mais baixo; lá
  a **alta** vai primeiro, porque o endereço desce a cada escrita. Copiar a ordem
  do vizinho inverte o par na memória — é barulhento (4 algozes), ao contrário
  dos erros de instante.
- **As quatro fases do `$08` são dois valores de 16 bits, não um.**
  `ReadAddressLow`/`ReadAddressHigh` latcham o **endereço** vindo do imediato;
  `WriteLowHalf`/`WriteHighHalf` gravam as metades do **`SP`**. Os nomes antigos
  (todos terminados em `Byte`) sugeriam um só valor e foram reprovados pelo
  `clippy::enum_variant_names` — a regra de estilo acertou uma imprecisão de
  modelo por acidente.
- **O `$08` duplica o latch de dois bytes do `Absolute`, e a duplicação é
  decisão.** `Absolute::ReadLowByte`/`ReadHighByte` e
  `StoreStackPointer::ReadAddressLow`/`ReadAddressHigh` têm corpo idêntico. Não
  foram extraídos: pela doutrina do 1.4d a abstração nasce onde a repetição
  **existe**, e com dois sítios ainda se adivinha a forma. O terceiro chega no
  1.10 (`CALL u16`, `JP cond`), que é quando extrair.

- **`H` é o carry do nibble baixo e `C` o do byte — duas grandezas, dois bits de
  origem.** A § The BCD Flags define `H` como *"carry for the lower 4 bits of the
  result"* e a § The Carry Flag define `C` como o resultado de 8 bits ficar
  *"higher than $FF"*. Não se deduz uma da outra, e calcular `H` do bit 7 (o
  mesmo bit do `C`) é o erro que a bateria da 0022 pôs como M1. `N` é `0`
  **literal** na coluna das 16 linhas — não é resultado de conta, e o 1.6b tem
  `1` literal no mesmo lugar.
- **O carry de entrada do `ADC` faz parte do resultado, então conta para `H`
  também.** `A=$0F` + operando `$00` + `C=1` dá `$10`, e `$0F+$00` não estoura
  nibble nenhum: uma ALU que some o carry só no total de 8 bits acerta `A` e erra
  `H`. É o único caso que separa as duas versões, e `the_incoming_carry_of_adc_
  counts_for_the_half_carry_too` é quem o fixa. O `SBC` do 1.6b tem o espelho.
- **`ADD A,(HL)` são 2 M-cycles, e a nota 34 não vale aqui.** A coluna é
  `fetch → read((HL))`, **sem seta**, e a nota 34 diz que seta ausente é latch —
  mas latchar exige um passo onde o valor aterrisse, e a linha tem **8**
  T-cycles, não 12. A seta falta porque o destino do byte não é registrador
  nenhum: é a ALU, que não aparece na notação. **Antes de aplicar a nota 34,
  conte os passos** (é o mesmo procedimento que a nota 34 já pedia para o
  `JP u16` × `LD r16,u16`, aplicado à outra ponta).
- **O operando de uma ALU não tem testemunha entre os M-cycles, e por isso o
  estímulo é que mede o instante.** `HL` (1.4c), `SP` (1.5b/c) e `PC` (1.5d)
  ficam legíveis entre os passos; o byte que vai para a ALU some. Ler `(HL)`
  dentro do fetch e gastar o M2 aplicando dá o mesmo `A`, as mesmas flags, os
  mesmos 2 M-cycles e os mesmos 8 T-cycles — **passou verde nos 251 testes**. Quem
  o pega é trocar o conteúdo de `(HL)` **entre** os dois `step`:
  `the_bus_access_of_86_happens_in_the_m2_and_not_during_the_fetch`, algoz único.
- **`alu.rs` é módulo próprio e `apply` é função livre sobre `Registers`.** A ALU
  não precisa do `Bus` nem do estado da máquina. A fronteira separa "quem decide o
  instante" (`mcycle.rs`) de "quem faz a conta" (`alu.rs`), que são as duas
  classes de erro deste projeto e não se olham. A quarta função de M-cycle
  compartilhada **não** nasceu aqui: "ler `(HL)` e entregar à ALU" tem um sítio
  só, e `AluFromHl(AluOp)` já cobre o 1.6b e o 1.6c sem linha nova.

- **`H`/`C` do `SUB`/`SBC`/`CP` são empréstimo, não carry — mesma letra do
  1.6a, grandeza invertida.** A § The Carry Flag desempata o caso em que a §
  The BCD Flags não decide sozinha (define `H` uma vez só, sem dizer de qual
  operação fala): numa subtração `C` é o resultado ficar *"lower than zero"*.
  `N` é `1` **literal** nas 24 linhas, ao contrário do `0` literal do 1.6a.
- **O empréstimo de entrada do `SBC` conta para o `H` também, espelho exato do
  `ADC`.** `A=$10` + operando `$00` + `C=1` dá `$0F`, e `$10-$00` não pede
  nibble nenhum: uma ALU que só empreste do byte total acerta `A` e erra `H`.
  `the_incoming_borrow_of_sbc_counts_for_the_half_borrow_too` fixa o caso.
- **`CP` é a primeira das oito operações da ALU que não escreve em `A`.**
  `alu::subtract` recebe um booleano `writes_result` em vez de uma quarta
  variante de função: a conta de `SUB`/`SBC`/`CP` é idêntica, letra por letra,
  e só a escrita diverge. **`CP A,A` e `SUB A,A` têm o mesmo `Z=1`** — só um
  teste que olhe `A` depois de cada um separa "escreveu o resultado" de "só
  comparou"; o `Z` sozinho não distingue os dois.
- **`decoded_elsewhere`/`previously_decoded` precisou de atualização em nove
  arquivos, não oito — a lista tem nomes diferentes entre arquivos antigos e
  novos.** `cpu_ld_r8_u8.rs` usa uma variável local (`previously_decoded`) em
  vez da função-padrão; buscar só por `fn decoded_elsewhere` deixa esse
  arquivo passar batido. O padrão estável para achar todos é procurar por
  `0x00..=0xFF` / `for opcode in`, que é como cada arquivo varre os 256
  opcodes — não o nome que cada um deu à variável.

- **`H`/`C` do `AND`/`XOR`/`OR` são constantes na coluna, e `alu::logic`
  recebe `H` como parâmetro em vez de calculá-lo.** `AND` tem `H=1`/`C=0`;
  `XOR`/`OR` têm `H=0`/`C=0` os três — nenhuma das duas é carry nem
  empréstimo, ao contrário de `add`/`subtract` (1.6a/1.6b). `C` é sempre `0`
  nos três, hardcoded dentro de `logic`, não parâmetro: nenhuma das oito
  operações da ALU até aqui tem `C` variável fora de `add`/`subtract`.
- **`decoded_elsewhere`/`previously_decoded` chegou a dez arquivos.** A partir
  da 0024, `cpu_sub_sbc_cp_r8.rs` também precisa da lista — deixou de ser o
  sub-item mais recente e ganhou vizinho (`0xA0..=0xB7`) que ele mesmo não
  decodifica. O procedimento (procurar por `0x00..=0xFF`/`for opcode in`)
  continua sendo o que acha todos de uma vez.

- **Os 12 `sweep`s verificam a alegação positiva de `decoded_elsewhere`, não só
  a negativa.** Até a 0026, cada `sweep` só testava `!decoded_elsewhere(opcode)
  → assert Undecoded`; se a função dissesse `true`, o `sweep` pulava em
  silêncio, sem checar se o opcode estava mesmo decodificado. Agora o
  `if/else if` tem um ramo a mais: `decoded_elsewhere(opcode) → assert None`.
  A bateria de mutação da 0026 é o que provou que o ramo que faltava importava
  — ver nota 47.

- **`INC`/`DEC r8` são a primeira ALU que deixa `C` intocado — nem calculado
  (1.6a/1.6b) nem literal (1.6c).** `alu::increment`/`decrement` não têm
  `set_flag(Flag::C, ...)` linha nenhuma; devolvem o resultado em vez de
  escrever em `registers.a`, porque o operando é qualquer `r8` ou `(HL)`, não
  só o acumulador. `H` aqui é carry (`INC`) ou empréstimo (`DEC`) do nibble
  baixo, mesma letra do 1.6a/1.6b — ver nota 48.
- **`$34`/`$35` (`INC`/`DEC (HL)`) espelham `StoreImmediateToHl` (1.4b), não
  `AluFromHl` (1.6a).** Leitura no M2, escrita no M3, com o resultado num
  latch entre os dois — dois acessos ao mesmo endereço em M-cycles distintos,
  ao contrário do `(HL)` do 1.6a, que só lê.
