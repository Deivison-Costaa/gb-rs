# STATUS

> Este arquivo é a **memória do projeto entre iterações**. O contexto do agente
> é descartado a cada iteração; este arquivo não. Mantenha-o curto e verdadeiro.

**Última iteração concluída:** 0024 — `AND a,r8`, `XOR a,r8` e `OR a,r8`, os 24 opcodes `$A0`–`$B7` ([doc](docs/iterations/0024-cpu-and-xor-or-r8.md)). A armadilha muda de natureza em relação ao 1.6a/1.6b: `H`/`C` são **constantes na coluna**, não carry nem empréstimo — `AND` tem `H=1`/`C=0`, `XOR`/`OR` têm `H=0`/`C=0`. **Erro de conta: nenhum** — a tabela de flags veio pronta do handoff da 0023 e `alu::logic` recebe `H` como parâmetro literal por chamada em vez de calculá-lo, que era exatamente a forma que a nota da 0023 pedia para evitar. **O atrito foi de ferramenta, não de spec:** `clippy::type_complexity` (tipo de função inline sem `type` nomeado) e `clippy::identity_op` (`0xFF & AFTER` no teste do M2, `0xFF` sendo o neutro do `&`) reprovaram antes do `cargo test --all`. O controle negativo `decoded_elsewhere`/`previously_decoded` ganhou `(0xA0..=0xB7)` em **dez** arquivos — os nove da 0023 mais o próprio `cpu_sub_sbc_cp_r8.rs`, que passou a precisar dele por não ser mais o sub-item mais recente.
**Iteração anterior:** 0023 — `SUB a,r8`, `SBC a,r8` e `CP a,r8`, os 24 opcodes `$90`–`$9F` e `$B8`–`$BF` ([doc](docs/iterations/0023-cpu-sub-sbc-cp-r8.md)). Mesma letra do 1.6a, grandeza invertida: `N` é `1` literal, `H`/`C` são empréstimo em vez de carry, e `CP` é a primeira das oito operações da ALU que **não escreve** em `A`. **Erro de conta: nenhum** — as cinco armadilhas (N literal, H invertido, `SBC` consumindo o `C` de entrada no nibble, `CP` sem escrita, `CP A,A`/`SUB A,A` com o mesmo `Z`) vieram pré-anunciadas letra por letra pelo handoff da 0022, então a nota 41 vale de novo: mede o aviso, não uma descoberta. **O achado foi processual, e repetiu o da 0022:** o controle negativo `decoded_elsewhere`/`previously_decoded`, duplicado em cada arquivo de sub-item, teve de ganhar `(0x90..=0x9F)`/`(0xB8..=0xBF)` em **nove** arquivos — os mesmos oito da 0022 mais um nono que quase escapou do `grep` por usar uma variável local (`previously_decoded`) em vez da função-padrão (`decoded_elsewhere`). `alu.rs` reprovou o R7 de primeira de novo (mesma causa da 0022: arquivo curto, três blocos de prosa já estouram 12%). O risco de instante do `(HL)` (o M16 da 0022) não se repetiu: `$96`/`$9E`/`$BE` reusam a mesma `alu_from_hl` já corrigida, sem decisão nova a errar.
**Duas iterações atrás:** 0022 — `ADD a,r8` e `ADC a,r8`, os dezesseis `$80`–`$8F` ([doc](docs/iterations/0022-cpu-add-adc-r8.md)). **As primeiras flags calculadas do projeto:** até aqui as 254 linhas implementadas tinham `-` nas quatro colunas. Abriu o 1.6, que foi **quebrado em cinco** — e a quebra divergiu da do 1.4/1.5 de propósito: 88 opcodes em quatro blocos, mas só **três** formas de M-cycle no grupo inteiro, então o corte é por **semântica de flag** e não por regra de decodificação. **Erro de hardware na conta: nenhum** — e a nota 41 vale integralmente, porque o handoff da 0021 pré-anunciou seis armadilhas de flag e o caso `A=$0F`, operando `$00`, `C=1` virou teste antes de a ALU existir. **O achado é o M16, e ele é a nota 43 numa terceira forma:** ler `(HL)` **dentro do fetch** e gastar o M2 aplicando dá o mesmo `A`, as mesmas flags, os mesmos 2 M-cycles e os mesmos 8 T-cycles — **passou verde nos 251 testes**, inclusive no que fora escrito para medir o instante. O operando de uma ALU não tem destino observável entre os passos (não é como o `HL` do 1.4c, o `SP` do 1.5b/c ou o `PC` do 1.5d), então **quem observa o instante é o estímulo, não a asserção**: troca-se o conteúdo de `(HL)` entre os dois `step`. Bateria: 16 mutantes, 15 mortos de primeira + o M16 com **um** algoz depois do teste novo, 3 controles verdes. Ver notas 44 e 45.
**Próxima tarefa:** ROADMAP 1.6d — `alu a,imm8` (`$C6 $CE $D6 $DE $E6 $EE $F6 $FE`), o bloco `11 ooo 110`, 8 opcodes, 2 M-cycles (`fetch → read(u8)`). As mesmas oito operações dos quatro sub-itens do 1.6 (`Add`/`AddWithCarry`/`Subtract`/`SubtractWithCarry`/`Compare`/`And`/`Xor`/`Or` já existem em `AluOp`), com o operando vindo do `PC` em vez de `r8` — forma de M-cycle nova só na origem do byte, não na ALU. Nota 43: o operando é lido num passo que o estado final não distingue (mesmo risco do M16, agora com o `PC` em vez da memória). **O controle negativo `decoded_elsewhere`/`previously_decoded` tem de ganhar `(0xA0..=0xB7)` nos arquivos que ainda não o têm, e `(0xC6|0xCE|0xD6|0xDE|0xE6|0xEE|0xF6|0xFE)` — ou uma faixa que os capture — depois que este PR sair; procure por `0x00..=0xFF`/`for opcode in`.**

**Tarefa concluída na 0024 (referência):** ROADMAP 1.6c — os 24 `AND`/`XOR`/`OR A,r8`. `alu::apply` ganhou uma quarta função interna, `logic`, que recebe o resultado já combinado e o valor literal de `H` — sem calcular carry nenhum, ao contrário de `add`/`subtract`. Nenhuma abstração de M-cycle nova: os seis `(HL)` (`$A6 $AE $B6`) reusam `State::AluFromHl(AluOp)`, que a 0022 generalizou prevendo esta terceira chegada.
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

Testes do workspace: **277** (eram **266** antes da 0024). Este
número não é o placar — ele mede o que o projeto afirma sobre si mesmo, não o
que o hardware cobra.

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
- **`decoded_elsewhere` é duplicada de propósito em cada arquivo de sub-item.**
  A lista de "opcodes que outro item já decodifica" cresce a cada PR, e extraí-la
  para um lugar só faria a atualização acontecer sozinha. O controle negativo
  existe justamente para obrigar quem acrescenta opcode a vir declarar o que
  acrescentou; um ponto de verdade compartilhado tiraria isso dele.
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

    **Quarto ponto de dado na 0013:** **142/142** em `1.85`. A previsão acima
    acertou o conteúdo — `const fn` com `match` sobre enum e `matches!` em
    contexto const — e nada disso é mais novo que a MSRV. Quatro acertos
    seguidos por lembrança continuam não sendo uma guarda, e o 1.4 multiplica
    esse tipo de código por 245 opcodes. Continua **aberta**.

    **Quinto ponto de dado na 0014:** **156/156** em `1.85`, com `const fn`
    sobre enum novo e padrão de faixa com extremos `const`. Cinco acertos
    seguidos, todos por alguém ter lembrado de rodar o comando. Continua
    **aberta** — e a essa altura o custo de fechar (um job de CI em `1.85`) já é
    menor que o de escrever esta nota mais uma vez.

    **Oitavo ponto de dado na 0020:** **225/225** em `rustc 1.85.1`, com mais dois
    `const fn` sobre enum (`write_r16_stk_low`/`_high`). O item **existe** desde a
    0016 — é o 7.4 — e a dívida segue aberta pelo motivo que o próprio 7.4 nomeia:
    ele está em M7, e o protocolo de iteração executa em ordem. Diagnóstico da
    0015 confirmado por mais quatro pontos: o que fecha uma dívida não é o custo
    baixo, é a posição na fila.

    **Sexto ponto de dado na 0015:** **166/166** em `1.85`, com `const fn` que
    devolve `State` e braço de `match` com guarda. Seis acertos seguidos. A
    frase acima segue verdadeira e a nota segue **aberta**, o que já é o achado:
    seis iterações é tempo de sobra para fechar, e o que impede não é o custo —
    é que o item não existe no ROADMAP, e o protocolo de iteração só executa o
    que está no ROADMAP. **Dívida que ninguém agenda não é priorizada baixo; é
    invisível.**

    **Oitavo ponto de dado na 0018:** **205/205** em `1.85` (`cargo 1.85.1`),
    com `u16::to_le_bytes`/`swap_bytes` em contexto const e máscara com
    `!0x00FFu16`. A 0017 não registrou a conferência; oito iterações, sete
    anotações — a lacuna é o argumento do 7.4 melhor do que qualquer das sete.

    **Sétimo ponto de dado na 0016:** **177/177** em `1.85` (`cargo 1.85.1`),
    com `const fn` sobre enum de dois bits e `wrapping_add`/`wrapping_sub` em
    `u16` — nada mais novo que a MSRV.

    **E o diagnóstico da 0015 foi acatado: agora existe o item — ROADMAP 7.4.**
    A dívida deixou de ser invisível sem deixar de ser baixa: está em M7 e não em
    M0, de propósito, para não preemptar o M1 na regra de "próxima caixa não
    marcada, em ordem", e é puxável a qualquer momento. Criar o item não é
    fechá-lo, e a nota continua **aberta** — mas a partir daqui ela é uma
    prioridade escolhida, e não uma lacuna do processo. O que a 0016 mediu é que
    o conserto do processo custou duas linhas de ROADMAP, contra sete iterações
    de "passou porque alguém lembrou".

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

    **Sexta repetição na 0013, e a novidade é que foram *dois* esqueletos.** O
    primeiro erro de memória era arquitetural — `step()` instruction-stepped — e
    um esqueleto assim **não tem M3 nem M4 onde os erros de timing possam
    aparecer**: ele esconde os erros seguintes em vez de os revelar. Rodar o
    esqueleto A (instruction-stepped, 3 de 11 pegos), consertar só aquilo, e
    rodar o esqueleto B (cycle-stepped com o desvio no M3 e opcode desconhecido
    virando `NOP`, 2 de 11 pegos) mediu três erros que um esqueleto só teria
    mostrado como um.

    **Regra que sai daí:** quando o erro de memória é de *forma* e não de
    *valor*, um esqueleto não basta — o erro de forma colapsa o espaço onde os
    outros viveriam. Encadeie esqueletos, do erro mais estrutural para o mais
    local, e rode a suíte contra cada um.

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

24. **O quarto modo de falha da R1: a spec local corrompida na conversão.** A
    regra supôs "o agente não leu" (original); a nota 19 achou "a spec é
    omissa"; a nota 21 achou "a spec é ambígua". A 0013 achou o pior de ler:
    **a spec está no repositório, tem 890 linhas, tem tabelas bem formadas, e o
    conteúdo se perdeu no HTML→Markdown.**

    A § CPU Instruction Set do `02-cpu.md` deveria descrever cada instrução.
    O que há são layouts de bits soltos: `nop` aparece como oito linhas
    `| 7 | 0 |`…`| 0 | 0 |` sem uma palavra sobre o que faz, e a tabela de
    placeholders (`r8`, `r16`, `r16stk`, `r16mem`, `cond`) virou **uma** tabela
    com os índices 0–3 repetidos quatro vezes e sem cabeçalho que diga a qual
    grupo cada bloco pertence. Quem for implementar 1.4–1.11 lendo aquele
    arquivo não acha semântica nenhuma.

    Isto é pior que omissão e que ambiguidade porque **nada no arquivo sinaliza
    a perda**. Omissão deixa buraco visível; ambiguidade entrega valor plausível
    dos dois lados; conversão corrompida entrega estrutura convincente e
    conteúdo zero, e "eu li a seção" fica tecnicamente verdadeiro.

    **Onde está a informação de verdade:** `03-opcodes.md` (gbops) para timing,
    flags e passo a passo de M-cycles — que é completo e foi o que sustentou a
    0013. Para prosa, o link que a própria seção dá, `gbz80(7)`
    (rgbds.gbdev.io), que **não** está em `docs/reference/`.

    **Não foi consertado na 0013**, e a decisão é a mesma que a 0008 tomou na
    nota 17: `01-`…`09-` são gerados, mexer à mão contraria o
    `docs/reference/README.md`, e o conserto é no `fetch-reference-docs.sh` —
    decisão de projeto, não contrabando dentro de uma iteração de outra coisa.
    Quem for mexer decide entre (a) melhorar a conversão daquela seção, (b)
    trazer `gbz80(7)` para `docs/reference/` como fonte de prosa, ou (c) marcar
    a seção como inútil no README e apontar todo mundo para o `03-opcodes.md`.
    A (b) é a que o M1 inteiro vai querer.

25. **O controle negativo pega o que os esqueletos não pegam — e às vezes é o
    único que pega.** Na 0013, os dois esqueletos deixaram
    `the_unused_opcodes_are_exactly_the_eleven_the_spec_names` passar sem nunca
    ter tido nada para reprovar: os dois traziam a lista certa. A bateria de
    mutação mostrou que aquele teste era **o único** capaz de pegar o mutante
    mais provável da seção — acrescentar `$D9` à lista de ilegais, que é `EXX`
    no Z80 e `RETI` no Game Boy. O outro teste de opcode ilegal só confere que
    os onze travam, e "os onze travam" continua verdade quando são doze.

    **A forma geral:** teste que afirma *pertinência* ("estes N estão na lista")
    não pega *excesso* ("e mais um"). Onde a spec dá uma lista fechada, o teste
    tem de varrer o complemento — na 0013, os 256 opcodes, afirmando dos dois
    lados. Isso já tinha sido escrito na 0011 como "controle negativo por
    coluna"; a leitura nova é que ele **não** é redundância defensiva, é o
    único instrumento para uma classe inteira de erro.

    **E a bateria mediu um buraco que ninguém tinha visto:** o mutante
    `wrapping_add` → `saturating_add` no `PC` foi classificado como controle
    (deveria ficar verde) e ficou — mas por falta de teste, não por
    equivalência. A volta do `PC` em `$FFFF` é comportamento real e **não está
    coberto**. Não é bug; é cobertura ausente, medida e datada. Quem fecha é o
    1.4, o primeiro item com operando que pode atravessar `$FFFF`.

    **O 1.4a não fechou** — os 63 opcodes do bloco `01 ddd sss` têm 1 byte e
    não leem operando nenhum do fluxo de instruções. Passou para o 1.4b.

    ~~**Aberta.**~~ **Fechada na 0015.**
    `an_immediate_load_reads_its_operand_across_the_program_counter_wrap`:
    opcode em `$FFFF` — que é o `IE`, o único byte gravável naquele endereço —
    e operando em `$0000`, que é ROM. Com `saturating_add` o `PC` empaca e o M2
    lê o próprio opcode de volta. **Da medição ao conserto foram duas
    iterações**, e o que sobreviveu no meio foi a nota, não a lembrança.

26. **Correção anterior virada em regra geral é um jeito novo de errar.** Os
    modos de falha da R1 catalogados até aqui são sobre a spec: não lida
    (original), omissa (nota 19), ambígua (nota 21), corrompida na conversão
    (nota 24). A 0014 achou um que não é sobre a spec — é sobre o **histórico do
    próprio projeto**.

    A 0013 descobriu que `JP u16` desvia no M4 e não no M3, e escreveu isso como
    invariante. Na 0014 eu implementei `LD r,(HL)` em três M-cycles — `fetch →
    read((HL)) → internal(escreve o registrador)` — generalizando aquilo para
    "o efeito acontece depois do que a intuição diz". Não existe essa regra. O
    `JP` tem quatro passos na coluna e o quarto é `internal`; o `LD r,(HL)` tem
    dois, e os 8 T-cycles não deixam onde pôr um terceiro. A coluna é **por
    instrução**.

    Isto engana melhor que o erro original por dois motivos. Primeiro, vem com a
    sensação de estar aplicando uma lição, que é o oposto do alarme que a R1
    tenta disparar. Segundo, o `STATUS.md` **ajuda a errar**: a invariante da
    0013 está escrita ali em prosa forte ("escrever o `PC` junto com o byte alto
    desloca o desvio em um — invisível em teste de instrução isolada"), e prosa
    forte sobre um caso lê-se como princípio.

    O que funcionou é o de sempre, e por isso vale insistir: o esqueleto. Quatro
    testes reprovaram, todos nomeando M-cycle. Nenhuma quantidade de releitura
    do `STATUS.md` teria pego — o erro **vinha** do `STATUS.md`.

    **Corolário para escrever invariante:** quando a invariante for sobre uma
    instrução, diga de qual instrução ela é e o que a produz (o número de passos
    da coluna), não só o que ela afirma. A invariante do `LD r,(HL)` acima está
    escrita assim de propósito.

27. **Rode a suíte nova contra o código velho antes de implementar.** Custa
    0,00s e devolve três coisas que nenhum outro momento devolve: o RED com o
    motivo certo, a lista de guardas que passam por vacuidade, e — na 0014 — um
    teste de verdade quebrado.

    O teste era `storing_l_to_hl_writes_the_low_byte_of_the_address_it_wrote_to`
    com `SCRATCH = $C000`. Byte baixo `$00`, WRAM começa zerada: ele afirmava
    `$00 == $00` sem nada ter sido escrito, e teria fechado verde contra a
    implementação certa, contra o esqueleto errado e contra a bateria de
    mutação inteira. **Em teste de memória, endereço e valor não podem
    compartilhar o zero com o estado inicial** — `$C000` é justamente o
    endereço que se escreve sem pensar.

    Isto é anterior ao esqueleto da nota 20 e não o substitui: o esqueleto mede
    o que eu erraria, este passo mede se o **teste** mede alguma coisa.

28. **O esqueleto e a bateria de mutação não exercitam os mesmos testes.**
    `no_load_in_the_block_touches_the_flags` passou contra a implementação
    anterior *e* contra o esqueleto A — nunca tinha tido nada para reprovar.
    Quem o exercitou foi o mutante que zera `F` de lambuja num `LD r,r'`.

    A razão é estrutural: o esqueleto contém os erros que **eu** cometeria, e
    eu não ia escrever um `LD` que mexe em flag. A bateria contém os erros que
    qualquer um pode introduzir depois. Guarda de *ausência* de comportamento
    tende a cair no segundo grupo e não no primeiro — então rodar só o
    esqueleto deixa exatamente essa classe sem medição.

    **Um teste que sobrevive aos dois foi exercitado; um que só passa nos dois
    pode não ter sido exercitado por nenhum.** Rode os dois.

29. **Controle negativo que sobrevive por redundância não é controle — é
    achado.** Na 0014 um dos dois controles foi "remover o braço `HALT` do
    decodificador", escrito na expectativa de que derrubasse a suíte. Ficou
    verde: `load_r8_r8` tem um braço `(MemoryAtHl, MemoryAtHl)` que devolve o
    mesmo veredito, posto ali só para o `match` ser total sem `_ =>`.

    Os dois caminhos garantem o mesmo comportamento e **nenhum teste os
    distingue**. Não é bug e nenhum dos dois saiu (um carrega a citação da spec,
    o outro é a invariante de `match` do projeto), mas a diferença entre
    "redundante de propósito" e "redundante sem ninguém ter notado" é toda a
    diferença no dia em que alguém simplificar um deles. Ficou escrito na
    invariante do `$76`.

30. **A nota 26 corre nas duas direções, e a 0015 correu na outra.** A 0014
    errou acrescentando um M-cycle (`LD r,(HL)` em três, generalizando o `JP
    u16`). A 0015 errou **adiantando** um: `LD (HL),u8` lendo o imediato e
    escrevendo em `(HL)` no mesmo M2, com um `internal` no M3 para fechar os 12
    T-cycles. A fonte é a mesma — a invariante que a 0014 escreveu em prosa
    forte ("não há terceiro M-cycle onde a escrita aconteça de verdade") lida
    como princípio em vez de como fato sobre uma instrução.

    Duas iterações seguidas, dois erros opostos, uma causa. **A correção de
    ontem é o material de que o erro de hoje é feito**, e o `STATUS.md` é o
    veículo: quanto melhor escrita a invariante, mais ela se parece com regra.
    O corolário da nota 26 (dizer de qual instrução a invariante é, e o que a
    produz) foi seguido e **não bastou** — a invariante do `LD r,(HL)` está
    escrita exatamente assim e ainda assim gerou o erro. O que pegou foi o
    esqueleto, outra vez.

    **A leitura nova é sobre o custo de detecção.** O erro deixa o total de
    T-cycles certo e o estado final certo; só o instante da escrita no
    barramento muda. **Nove dos dez testes do 1.4b passam contra ele.** O único
    que o reprova é o que lê a memória *entre* dois `step`. Isto é a R2 sendo
    cobrada: uma suíte que só compare estado antes/depois é cega para a classe
    inteira de erro que a Mooneye mede — e "a instrução dá o resultado certo"
    não é evidência nenhuma sobre timing.

    **Regra prática para os sub-itens que faltam:** toda instrução com mais de
    um acesso ao barramento precisa de um teste que observe o estado *entre* os
    acessos, e não só no fim. O 1.4c e o 1.4d têm oito e seis dessas.

31. **Teste que afirma a fronteira do que existe envelhece quando a fronteira
    anda — e isso é o guarda funcionando.** A 0015 quebrou dois testes de
    iterações anteriores, os dois porque `$06` deixou de ser "não implementado":
    o controle negativo dos 256 do 1.4a, e
    `an_opcode_this_emulator_has_not_reached_is_not_an_illegal_one`, que usava
    `$06` como exemplo.

    Não é atrito acidental, é o preço de ter fronteira testada, e é barato —
    dois `Edit`. O que vale registrar é a **tentação** que aparece junto:
    extrair uma lista compartilhada de "opcodes já decodificados" para que a
    atualização acontecesse sozinha. Isso teria matado a única propriedade que
    justifica o controle negativo — obrigar quem acrescenta opcode a vir
    declarar o que acrescentou. **Guarda que se atualiza sozinho não guarda.**
    Ver a invariante de `decoded_elsewhere`.

    O segundo teste passou a usar `$04` (`INC B`), com a troca documentada
    dentro dele, e ganhou uma função nova de quebra: `INC B` é `00 ddd 100`,
    vizinho de bit do bloco do 1.4b, então ele também reprova uma máscara
    frouxa. Cada iteração do 1.4 vai movê-lo de novo; o dia em que não houver
    opcode não implementado é o dia em que ele sai.

    **Reincidiu na 0016, e o `$04` aguentou:** os dois controles negativos dos
    256 quebraram (o do 1.4a e o do 1.4b, ambos porque `$02` deixou de ser "não
    implementado"), dois `Edit`, e o teste que usa `$04` como exemplo continuou
    verde — `INC B` não é deste sub-item. A tentação de extrair a lista para um
    lugar só apareceu de novo e foi recusada de novo.

32. **A previsão certa não gera, sozinha, a suíte que a honra.** A nota 30 deixou
    uma regra prática explícita para o 1.4c e o 1.4d: *"toda instrução com mais
    de um acesso ao barramento precisa de um teste que observe o estado entre os
    acessos, e não só no fim"*. A 0016 leu a regra, escreveu **onze** testes com
    ela em mente, cometeu exatamente o erro previsto — adiantar o `HL±` para o
    fetch — e **dez dos onze passaram**.

    Isto é diferente das notas 8, 26 e 30, que são sobre erro de leitura da spec
    ou de generalização. Aqui a lição anterior estava correta, disponível, citada
    no cabeçalho do arquivo de teste, e ainda assim a suíte quase não a
    implementou. Escrever "vou testar o meio da instrução" e escrever a asserção
    que lê o estado no meio da instrução são atos diferentes, e o primeiro dá a
    sensação do segundo.

    **Duas medições agora, e concordam:** 9 de 10 na 0015 (`LD (HL),u8` com a
    escrita adiantada), 10 de 11 aqui (`HL±` adiantado). A proporção não mede
    fraqueza da suíte — mede que o erro **não tem efeito observável fora do
    instante do acesso**: estado final igual, T-cycles iguais, flags iguais,
    varredura dos 256 igual. É a classe inteira que a Mooneye mede e que uma
    suíte de antes/depois é cega para.

    **Procedimento, não intenção:** para cada instrução de N > 1 M-cycles, antes
    de implementar, escreva primeiro o teste que faz `step` N vezes com asserção
    **depois de cada uma**. Se a suíte tem uma asserção por M-cycle, ela mede
    timing; se tem asserções antes e depois, ela mede resultado — e a diferença
    não aparece na contagem de testes nem na cobertura.

33. **Troca de motorista: a partir da 0018, quem itera é o Kimi K3 (OpenCode),
    e a 0017 foi a transição medida ao vivo.** O `scripts/loop.sh` chamava
    `claude -p`; passou a chamar `opencode run -m opencode-go/kimi-k3`. O que
    **não** mudou: `CLAUDE.md` (o OpenCode o lê como fallback nativo — **não
    criar `AGENTS.md`**, que o sobrescreveria), a skill `.claude/skills/iterate/`
    (descoberta nativa pelo OpenCode), `gh`, CI, proteção de branch, e cada
    iteração continua sendo um processo novo com contexto zerado. O que mudou:
    as métricas vêm de `--format json` (eventos `step_finish`: `cost` é
    **estimativa** de tabela models.dev × tokens; os **tokens** são a medição
    real — a janela de 5h do plano é server-side e opaca, o loop detecta o
    esgotamento pela falha do provider e para), e `logs/metrics.csv` ganhou a
    coluna `model` para demarcar o regime — **misturar custos claude/kimi sem
    rótulo envenenaria o gráfico 8.2**. O freio `--max-turns 150` não existe no
    OpenCode; quem freia é `timeout` por iteração.

    **A 0017 virou o caso de estudo da transição sem ninguém ter planejado:** a
    sessão de Claude morreu no meio do RED→GREEN, e o que sobreviveu foram os
    artefatos — código, testes e os erros de memória documentados **nos
    comentários**. O que morreu: duração, custo, contagem de tentativas e
    qualquer erro corrigido em silêncio antes dos comentários. É a tese do
    projeto vista do avesso: o que não vai para o artefato, vai para o ralo.
    O erro #3 da 0017 é também a **quarta categoria de erro** do projeto —
    depois de flags, timing e endereçamento de memória, o **erro de medição**:
    o arnês que reprova o código certo (laço de passo fixo para instruções de
    durações diferentes; conserto: `m_cycles_of(opcode)`, e o corolário é que
    laço "genérico" de teste é onde o erro de medição gosta de morar).

    **Revisão cruzada invertida:** o `review.sh` nasceu com o OpenCode de
    revisor e o Claude de autor. Invertidos os papéis, o padrão passou a ser
    `opencode run -m opencode-go/deepseek-v4-pro` (outro modelo, mesma cota);
    `REVIEWER_CMD="claude -p"` devolve a revisão ao Claude se houver cota.
    Continua desligada no loop (`REVIEW=0`) por decisão do operador. E o
    motivo histórico registrado no `loop.sh` para não revisar em lote —
    "revisão suja a árvore" — **estava errado**: `docs/reviews/` está no
    `.gitignore` desde a 0004 e a guarda de árvore limpa não a vê. O custo de
    tempo/crédito por iteração é o motivo que resta, e ele é real.

34. **A nota 26/30 tem três direções, e a 0018 fechou o trio.** O erro é sempre
    o mesmo: uma instrução vizinha, correta, lida como regra geral. O que muda é
    o sentido em que ela desloca o efeito.

    - 0014 — **acrescentou** um M-cycle (`LD r,(HL)` em três, generalizando o
      `internal` do `JP u16`).
    - 0015 — **adiantou** um acesso (`LD (HL),u8` escrevendo no M2,
      generalizando a correção da 0014).
    - 0018 — **atrasou** dois efeitos (as metades de `LD r16,u16` para o fim do
      M3, generalizando o latch do `JP u16`).

    **A novidade da 0018 é a fonte.** Nas duas anteriores a regra falsa vinha do
    `STATUS.md` — prosa forte sobre um caso, lida como princípio. Aqui veio do
    **código**: `JumpImmediate` está 40 linhas acima no mesmo arquivo, tem
    operando do mesmo tamanho, e latcha os dois bytes. Não é lembrança nem
    invariante mal escrita; é o vizinho mais próximo do cursor. O corolário da
    nota 26 (dizer de qual instrução a invariante é) não alcança isso: não havia
    invariante no meio, havia `self.latch |= ... << 8` na tela.

    **O que separa os dois casos está na coluna, e é contável:** o `JP u16` tem
    quatro passos e o quarto é `internal`; o `LD r16,u16` tem três, todos acesso.
    Latchar exige um passo onde o par possa ser escrito, e esse passo só existe
    quando a coluna o dá. **Antes de copiar a forma da instrução vizinha, conte
    os passos das duas.**

    **E a seta é o outro sinal, mais barato de ler:** `read(u16:lower->C)` diz
    onde o byte para; `read(u16:lower)` (que é o que o `$FA` tem) diz que ele
    fica no latch. As duas notações convivem na mesma coluna, na mesma tabela, e
    a diferença entre elas é exatamente a diferença entre as duas implementações.

    **Proporção medida pela terceira vez:** 9/10 na 0015, 10/11 na 0016, 7/8
    aqui. Não mede fraqueza da suíte — mede que a classe inteira de erro é
    invisível fora do instante do acesso. E pela primeira vez a bateria de
    mutação concorda numericamente: dos nove mutantes, **dois** são pegos por um
    único teste, e é o mesmo teste nos dois casos.

35. **Guarda de ausência não entra na conta do esqueleto, por construção.** A
    nota 28 disse isso na 0014; a 0018 mediu o caso extremo.
    `no_16_bit_immediate_load_touches_the_flags` não reprovou o código anterior
    (a CPU travava, e CPU travada não mexe em `F`), não reprovou o esqueleto (eu
    não ia escrever um `LD` que mexe em flag) e não reprovou nenhum dos **oito**
    mutantes de comportamento. Foi preciso escrever um nono mutante só para ele.

    **Procedimento que sai daí:** ao montar a bateria, separe os testes em
    "afirmam presença" e "afirmam ausência", e garanta pelo menos um mutante por
    teste do segundo grupo. Sem isso, um `PEGOS: 8/8` convive com um teste que
    nunca foi exercitado — e a conta não denuncia, porque ela conta mutantes,
    não testes.

36. **A nota 26/30/34 tem uma quarta direção, e a fonte da regra falsa saiu do
    projeto.** As três anteriores vinham de dentro: prosa forte do `STATUS.md`
    lida como princípio (0014, 0015) ou o vizinho mais próximo do cursor no mesmo
    arquivo (0018). A 0019 veio do **Z80** — `PUSH qq` são 11 T-cycles,
    `M1(5) + M2(3) + M3(3)`, e o T-cycle extra do M1 é onde o `SP` é
    decrementado. Daí sai, inteirinho, o desenho errado: decrementar no
    `internal` do M2 e pós-decrementar nas duas escritas.

    A R1 avisa exatamente isso, com estas palavras: *"você conhece Z80 melhor do
    que conhece o SM83"*. **O aviso não impediu nada**, e vale entender por quê:
    a intuição não se apresenta como uma citação do Z80 que dê para desconfiar e
    checar. Ela se apresenta como **um M-cycle vazio parecendo um bug**. Um braço
    de `match` que só troca de estado tem cara de código incompleto, e a pressão
    para dar-lhe serviço é estética, não factual — não há nada para verificar
    porque não há afirmação nenhuma, só desconforto.

    **O que funcionou foi a notação, não a memória.** A invariante do 1.4c
    ("a coluna escreve o efeito **dentro** do passo do acesso") cobre `(--SP)`
    sem uma palavra de mudança, porque `write(A->(HL++))` e
    `write(B->(--SP))` são a mesma construção. Ler a coluna caractere por
    caractere é o único procedimento que pegou as quatro direções; ler o vizinho
    não pegou nenhuma.

    **Corolário prático para o resto do M1:** `internal` no meio da instrução é
    o M-cycle mais fácil de estragar, porque estragá-lo parece consertá-lo.
    O 1.5d (`LD SP,HL`, `fetch → internal`) e o 1.10 (`CALL`, `RET`, `RST` — que
    têm `internal` no meio *e* pilha) são os próximos.

    **Proporção medida pela quarta vez:** 9/10 na 0015, 10/11 na 0016, 7/8 na
    0018, **8/10** aqui. Quatro medições, mesma conclusão: a classe de erro é
    invisível fora do instante do acesso, e a fração de testes que a pega é
    pequena e não cresce com o tamanho da suíte — cresce com quantas asserções
    leem o estado *entre* os M-cycles.

37. **Guarda de ausência ganha valor quando o erro é "algo inerte fez algo".**
    A nota 35 mediu, na 0018, que guarda de ausência não reprova mutante de
    comportamento — foi preciso um nono mutante só para exercitá-la. A 0019 é o
    contraexemplo, e ele não contradiz a nota 35: **refina o critério**.

    `the_internal_m_cycle_of_a_push_changes_nothing_but_the_program_counter` é um
    dos **dois** algozes do erro mais caro da iteração, num empate com o teste que
    lê o `SP` M-cycle por M-cycle. Funcionou porque a forma do erro e a forma da
    guarda coincidem: o erro *é* o `internal` deixando de ser inerte.

    **O critério que sai daí:** guarda de ausência sobre um **efeito colateral
    que ninguém ia escrever** (o `LD` que mexe em flag, o `PUSH` que zera `F`) é
    guarda de regressão futura e precisa de mutante próprio — a nota 35 continua
    valendo, e o mutante #10 da 0019 existe por isso. Guarda de ausência sobre um
    **passo que a spec manda existir vazio** é medição do código de hoje e paga
    imediatamente. Na hora de montar a bateria, o corte não é
    "presença × ausência": é se existe pressão para preencher o vazio.

38. **`grep` pelo mnemônico em `02-cpu.md` pode devolver zero para instrução que
    está lá.** A conversão para Markdown fundiu instruções consecutivas sob o
    **primeiro** cabeçalho de cada grupo. O layout de bits de `push r16stk`
    (`11 rr 0101`) mora sob o cabeçalho **`pop r16stk`**, junto com o do
    `11 rr 0001`; a string `push` não aparece no arquivo. É o mesmo defeito de
    conversão da nota 24 (as quatro tabelas de placeholder fundidas numa, sem
    cabeçalho), agora nas tabelas de codificação — então **vale para o arquivo
    inteiro, não só para os placeholders**.

    A nota 15 mandou `grep` pelo opcode no arquivo todo antes de implementar.
    Isso continua certo e **não** é suficiente: aqui o `grep` acerta o alvo e o
    rótulo em cima dele está errado. O procedimento que resta é o que funcionou:
    a codificação se confirma pelo `03-opcodes.md`, que tem uma linha por opcode
    com nome, tamanho, ciclos e coluna de M-cycles, e as tabelas de bits do
    `02-cpu.md` se leem **pelos bits**, contando a partir do cabeçalho anterior.
    Quem confiar no título conclui que `11 rr 0101` é variante de `POP`.

39. **`cargo test` sem `--no-fail-fast` esconde o estrago de um opcode novo.**
    Os cinco controles negativos dos 256 quebram a cada sub-item do 1.4/1.5
    (nota 31), e o cargo aborta no primeiro binário vermelho: a primeira execução
    da 0019 reportou **um** arquivo quebrado quando eram **cinco**. Susto barato e
    reincidente — a partir daqui, `cargo test --all --no-fail-fast` desde a
    primeira medição, e o passo 6 do protocolo continua com o comando simples só
    porque na CI o veredito é binário.

40. **O erro de instante é assimétrico: o lado que escreve o tolera em silêncio,
    o lado que lê grita.** A 0019 e a 0020 são o mesmo erro de forma — deslocar o
    `±SP` em um passo — nos dois lados da mesma pilha, e as duas medições não se
    parecem:

    - **`PUSH`** (0019): decrementar no `internal` do M2 escreve nos **mesmos dois
      endereços, na mesma ordem**, e deixa o mesmo `SP` final. Só o instante muda.
      **8 dos 10 testes passam.**
    - **`POP`** (0020): pré-incrementar lê em `SP+1` e `SP+2` em vez de `SP` e
      `SP+1`. Par errado, `SP` final errado, topo da pilha ignorado. **7 dos 10
      reprovam.**

    A causa não é a suíte: é onde o endereço é decidido. O `PUSH` o decide
    **antes** do acesso (o `SP` já andou; o valor a escrever é o mesmo), então
    adiantar o decremento não muda *qual* endereço recebe o quê. O `POP` o decide
    **no** acesso, então adiantar o incremento muda o endereço lido — e o dado
    lido vira o estado final.

    **Corolário prático para o 1.10, que é onde isso vai doer:** `CALL`, `RET`,
    `RETI` e `RST` fazem as duas coisas. Uma suíte de `RET` verde **não** autoriza
    concluir que o instante do `CALL` está certo — são regimes de detecção
    diferentes, e o `CALL` está do lado silencioso. Mesma leitura para o 1.5d: o
    `$08` (`LD (u16),SP`) é escrita pura, então a bateria dele deve esperar
    proporções de `PUSH`, não de `POP`, e a asserção que vale é a que lê a memória
    **entre** as duas escritas.

41. **O handoff que descreve o erro *seguinte* funciona — e enfraquece a própria
    medição.** Da 0014 à 0019 o campo `Próxima tarefa` do `STATUS.md` descrevia o
    erro **anterior**; a 0020 foi a primeira em que ele nomeou os três do item que
    vinha, com o mecanismo de cada um. Nenhum dos três virou código, e a bateria
    de mutação teve de **construí-los à mão** para medir se doíam.

    Os dois lados disso são reais e vale escrever os dois. O campo pagou: a classe
    de erro mais cara do projeto (notas 26/30/34/36, seis reincidências) não
    reincidiu. E o dado ficou pior: um `nenhum` obtido com o aviso na tela mede o
    aviso, não o código nem o agente. **O log continua útil porque a bateria
    substitui a medição perdida** — M1, M2 e M5 são exatamente os três avisos,
    escritos como mutantes, e é deles que sai o achado da nota 40. Sem a bateria,
    a 0020 teria produzido uma linha vazia num campo que é a tese do projeto.

    **Procedimento:** quando o handoff pré-anunciar armadilhas, a bateria deixa de
    ser opcional — ela é o que resta de medição.

42. **Sessão interrompida no meio deixa um buraco no dado, e o buraco tem forma
    conhecida.** A 0021 foi escrita por duas sessões: a primeira morreu sem
    commit e sem relato, a segunda leu a spec, revisou o que estava em disco e
    terminou. O campo `Erros de primeira tentativa` passou a medir **o que
    sobreviveu ao revisor**, que é coisa diferente do que ele mede normalmente
    (o que o autor percebeu que errou). Erro cometido e consertado dentro da
    primeira sessão não deixou rastro em lugar nenhum.

    **Onde a sessão morreu é recuperável do artefato, e isso vale o registro:**
    os 14 testes passavam e `cargo clippy -- -D warnings` reprovava com dois
    erros (`enum_variant_names`, `unused_imports`). Ou seja, ela parou **entre o
    passo 6 e o passo 7** — o portão de qualidade nunca rodou. Como diagnóstico
    de sessão morta, `clippy` vermelho + testes verdes é assinatura confiável.

    **O lado bom, que ninguém planejou:** a segunda sessão foi a coisa mais
    próxima da revisão cruzada que o projeto já teve (nota 5, `scripts/review.sh`
    nunca configurado). Os dois achados da 0021 — a justificativa falsa do `$F9`
    e o buraco de cobertura do M11 — são exatamente os que quem escreveu não
    acharia: um é uma citação que só destoa quando alguém volta à tabela, e o
    outro é o ponto cego que a nota 8(b) prevê (mutante escrito por quem escreveu
    o teste herda o mesmo ponto cego). **Não** conclua daí que a interrupção foi
    boa; conclua que revisor sem o raciocínio do autor acha classe de coisa que
    o autor não acha, e que isso é comprável de propósito.

43. **A doutrina da nota 32 tem dois lados, e até a 0021 só um estava em uso.**
    "Asserção depois de **cada** M-cycle" vinha sendo aplicada à **memória** — é
    o que pega o mutante que junta dois acessos num passo só. O outro lado é o
    **operando**: o `PC` observado entre os passos.

    O M11 da 0021 é o caso puro. Ler os dois bytes do endereço do `$08` no M2 e
    gastar o M3 num `internal` dá o mesmo endereço, a mesma memória, o mesmo `PC`
    no fim e os mesmos 20 T-cycles. O que ele quebra é a invariante do 1.3 —
    *uma chamada de `Cpu::step` faz no máximo um acesso ao barramento* —, e
    nenhum teste de estado final a enxerga: o mutante passou verde nos **239**
    testes do workspace. Quem o pega é `the_two_operand_bytes_are_read_one_per_
    m_cycle`, que lê o `PC` depois de cada um dos cinco passos, e é o **único**
    algoz dele.

    **Procedimento, para toda instrução com operando de mais de um byte:** duas
    baterias de asserção entre M-cycles, não uma — a memória (ou o registrador
    de destino) e o `PC`. O 1.6 tem `alu a,imm8` e o 1.10 tem `CALL u16` e os
    saltos condicionais; nos dois o operando é lido em passos que o estado final
    não distingue.

44. **O erro de instante tem dois regimes, e só um deles é a classe que este
    projeto vem medindo.** Cinco iterações mediram a mesma proporção (9/10 na
    0015, 10/11 na 0016, 7/8 na 0018, 8/10 na 0019, 14/15 na 0021): o erro de
    instante deixa quase toda a suíte verde. A 0022 partiu a classe em dois, com
    detecções opostas, e a linha divisória é a **coluna de T-cycles**:

    - **Barulhento** — o erro gasta um M-cycle a mais. O `$86` em três passos
      (latcha no M2, aplica num `internal`) muda o total de 8 para 12 T-cycles.
      **4 algozes**, morre em qualquer suíte.
    - **Silencioso** — o erro cabe nos passos que já existem. O `$86` lendo
      `(HL)` dentro do fetch e aplicando no M2 preserva tudo o que é observável.
      **0 algozes entre 251 testes.**

    A classe silenciosa exige **um passo sobrando** onde o efeito se esconda.
    `ADD A,(HL)` tem dois passos e os dois são acesso, então o único lugar que
    sobra é *dentro* do M1, empilhado com o fetch — o que quebra a invariante do
    1.3 (no máximo um acesso por `step`) sem tocar em nada que um teste de estado
    final veja.

    **Procedimento:** quando um mutante de instante morrer com muitos algozes,
    desconfie de que ele não é o mutante da classe. A classe preserva o total de
    T-cycles. Se o mutante escrito não preserva, ele ainda não foi escrito.
    **Corolário para o 1.6e:** `INC (HL)`/`DEC (HL)` têm três passos, com um read
    e um write no mesmo endereço — lá cabem as duas formas.

45. **A nota 43 tem uma terceira forma: quando o valor não para em lugar nenhum,
    quem observa o instante é o estímulo, não a asserção.** A nota 32 nasceu
    dizendo "asserção depois de cada M-cycle" sobre a **memória**; a 0021
    acrescentou o **`PC`**. As duas pressupõem uma testemunha — um lugar onde o
    valor fica e que dá para ler entre os passos. O `HL` do 1.4c, o `SP` do
    1.5b/c e o `PC` do 1.5d são testemunhas.

    O operando de uma ALU não é. Ele é lido, entra na conta e some; o que fica é
    o resultado, que é igual nas duas versões. Nenhuma asserção entre `step`
    distingue "leu no M1" de "leu no M2" — porque não há o que olhar.

    **O que funciona é mexer na fonte no meio da instrução:** `step`, escrever
    outro valor em `(HL)`, `step`, e ver qual dos dois o resultado denuncia. Isso
    não é uma asserção a mais, é um estímulo a mais, e nenhuma das notas
    anteriores o pedia.

    **Onde isto volta a valer:** toda instrução cujo acesso alimenta um cálculo
    em vez de um registrador — o resto do 1.6, o `BIT` do 1.9, e os desvios
    condicionais do 1.10, onde o byte lido decide o `PC` e não fica em campo
    nenhum.
