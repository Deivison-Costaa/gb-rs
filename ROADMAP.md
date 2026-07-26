# ROADMAP

Cada item é **uma iteração = um PR**. Ordem é obrigatória: cada marco depende
do anterior estar verde. Marque `[x]` só depois do merge em `main`.

---

## M0 — Fundação

- [x] 0.1 Workspace Cargo: `gb-core`, `gb-cli`, `gb-desktop`. `#![forbid(unsafe_code)]` no core.
- [x] 0.2 CI: fmt, clippy `-D warnings`, test. Artefato `scoreboard.csv`.
  - [x] 0.2a Job `check`: fmt, clippy `-D warnings` e test rodando **incondicionalmente** (remover a guarda morta do 0.1) + teste que reprova a regressão do workflow.
  - [x] 0.2b Job `scoreboard`: falhar quando `scripts/scoreboard.sh` morre ou o CSV não cresce (`STATUS.md`, nota 7).
  - [x] 0.2c Persistir a série gerada pela CI: publicar o `scoreboard.csv` acumulado numa branch de dados (`scoreboard-data`) no push para `main` (`STATUS.md`, nota 2). **Não é `main`:** a proteção de `main` exige PR, e o `GITHUB_TOKEN` não tem como contorná-la — ver [doc da 0004](docs/iterations/0004-ci-serie-persistida.md).
- [x] 0.3 Parser do header do cartucho (0x0100–0x014F) + `gb-cli info <rom>`: título, tipo de MBC, tamanho ROM/RAM, checksum.
  - [x] 0.3a `CartridgeHeader::parse(&[u8])` em `gb-core`: título, tipo de cartucho, tamanho de ROM/RAM, checksum do header (armazenado × calculado). Puro, sem I/O.
  - [x] 0.3b `gb-cli info <rom>`: leitura do arquivo, parsing de argumentos, impressão e códigos de saída.
- [x] 0.4 `Cartridge` trait + `NoMbc` (ROM-only, 32KB). A RAM opcional da § No MBC **não** entrou: os tipos que a declaram (`$08`/`$09`) são os que o Pan Docs marca como comportamento desconhecido — ver [doc da 0007](docs/iterations/0007-cart-nombc.md).
- [x] 0.5 `scripts/fetch-test-roms.sh`: baixa blargg, mooneye, dmg-acid2 para `tests/roms/`. Entregue pelo scaffold; a [0008](docs/iterations/0008-fetch-test-roms-guard.md) verificou (121 ROMs, três suítes) e cobriu com teste hermético. O fallback ainda entrega menos do que promete — `STATUS.md`, nota 17.
- [x] 0.6 Consolidar o controle negativo de decodificação. `decoded_elsewhere` mora em `tests/support/mod.rs`; os 12 arquivos consumidores usam `mod support; use support::decoded_elsewhere;`. [0026](docs/iterations/0026-decoded-elsewhere-single-source.md).

## M1 — CPU (sem gráficos)

- [x] 1.1 Registradores AF/BC/DE/HL/SP/PC, flags Z/N/H/C, pares de 8/16 bits. **Sem máscara no nibble baixo de `F`**: o Pan Docs no commit fixado não descreve os bits 3–0 nem menciona `POP AF` — ver [doc da 0009](docs/iterations/0009-cpu-registers.md). Se a máscara for necessária, quem cobra é o 1.13.
- [x] 1.2 `Bus` + MMU: WRAM, HRAM, echo RAM, região proibida. Estado pós-boot (pular boot ROM).
  - [x] 1.2a Decodificação de endereço e RAM interna: o mapa de memória inteiro em regiões, WRAM, echo RAM, HRAM, região proibida `$FEA0`–`$FEFF`, e o roteamento das duas janelas do cartucho. Sem valores iniciais — ver [doc da 0010](docs/iterations/0010-bus-memory-map.md). **`Bus` é `struct`, não `trait`** — o item dizia "trait", mas `CLAUDE.md` § Arquitetura diz que o `Bus` é o dono de tudo e que os componentes recebem `&mut Bus`; um trait com um único implementador poria vtable no caminho mais quente do emulador sem comprar nada. Extrair depois é mudança local.
  - [x] 1.2b Estado pós-boot: registradores da CPU e registradores de hardware (`$FF00`–`$FF7F`, `IE`) no hand-off da boot ROM, que este emulador pula. **Quebrado em dois na 0011:** são duas tabelas distintas da § Console state after boot ROM hand-off, e a segunda exige ligar uma região nova ao `Bus` — o que derruba `the_regions_without_an_owner_are_open_bus_and_swallow_writes` e não cabe no mesmo PR pequeno que a primeira.
    - [x] 1.2b-i Registradores da CPU no hand-off: a coluna **DMG** da tabela § CPU registers. `F` não é constante — `H` e `C` saem do checksum do cabeçalho, e é o **gravado em `$014D`**, não o calculado; ver [doc da 0011](docs/iterations/0011-cpu-boot-state.md). Puro, sem tocar no `Bus`.
    - [x] 1.2b-ii Registradores de hardware no hand-off: `$FF00`–`$FF7F` e `IE`, a partir da coluna **DMG / MGB** da tabela § Hardware registers. A tabela dá valor a **41** dos 128 endereços, marca **15** como `---` (só CGB) e **não menciona 72** — os 87 últimos continuam sem dono, e a wave RAM sai dessa lista com a APU (6.4). `OBP0`/`OBP1` são `??` na spec e `$00` por escolha; ver [doc da 0012](docs/iterations/0012-bus-io-boot-state.md). Valor inicial, **não** semântica: sem máscara, sem read-only, sem efeito colateral — isso vem com o componente dono.
- [x] 1.3 Laço M-cycle: `step()` avança 1 M-cycle. Fetch/decode/execute como máquina de estados. **`JP u16` (`C3`) entrou junto, e o item não pedia:** com só `NOP` decodificado a máquina não tem estado — instruction-stepped e cycle-stepped dão o mesmo resultado, e a R2 fica sem teste que a separe do que ela proíbe. Medido, não suposto: contra o esqueleto instruction-stepped, o teste de `NOP` **passou**. Ver [doc da 0013](docs/iterations/0013-cpu-mcycle-loop.md). Os onze opcodes inexistentes travam a CPU (`02-cpu.md` § Moved, Removed, and Added Opcodes); os 243 ainda não decodificados param com `Lockup::UndecodedOpcode`, que é rótulo diferente de propósito.
- [x] 1.4 Opcodes: loads 8-bit. **Quebrado em quatro na 0014:** o grupo `x8/lsm`
  da tabela de gbops tem **85** opcodes e cinco modos de endereçamento
  distintos — passa de longe do "um PR pequeno, um conceito só" do protocolo de
  iteração. A quebra é por **regra de decodificação**, não por quantidade: cada
  sub-item é um bloco contíguo da tabela com uma forma de M-cycle própria.
  85 = 63 + 8 + 8 + 6.
  - [x] 1.4a O bloco `LD r8,r8` — `$40`–`$7F` **sem** `$76`: 63 opcodes, uma
    regra só (`01 ddd sss`, § Block 1 do `02-cpu.md`) e três formas de M-cycle:
    `LD r,r'` em 1, `LD r,(HL)` e `LD (HL),r` em 2. `$76` é `HALT` — a spec o
    chama de **exceção** à codificação, e ele é o 2.3, não este item.
    `LD r,(HL)` faz a leitura no barramento e a escrita no registrador **no
    mesmo** M2: não há terceiro M-cycle, e supor que houvesse foi a lição do
    `JP u16` aplicada onde ela não vale — ver
    [doc da 0014](docs/iterations/0014-cpu-ld-r8-block.md).
  - [x] 1.4b Imediatos de 8 bits: `LD r8,u8` (`$06 $0E $16 $1E $26 $2E $3E`) e
    `LD (HL),u8` (`$36`) — o bloco `00 ddd 110`, em 2 e 3 M-cycles. Ao
    contrário da § Block 1, **não há exceção**: o índice 6 do campo de destino
    dá o `$36`, que é load como os outros sete. O `$36` é a primeira instrução
    do projeto com **dois** acessos ao barramento, e a coluna os põe em
    M-cycles diferentes (`fetch → read(u8) → write((HL))`); juntá-los no M2 com
    um `internal` no M3 dá o mesmo total e adianta a escrita em um — ver
    [doc da 0015](docs/iterations/0015-cpu-ld-r8-u8.md).
  - [x] 1.4c Indireto por par de registradores: `LD (BC),A`, `LD A,(BC)`,
    `LD (DE),A`, `LD A,(DE)` (`$02 $0A $12 $1A`) e as quatro formas com `HL+`/
    `HL-` (`$22 $2A $32 $3A`) — 8 opcodes, e o efeito colateral sobre `HL`.
    **Nenhuma forma de M-cycle nova:** as oito linhas são `fetch` mais um
    acesso, como o 1.4a. O novo é o `++`/`--` escrito **dentro** do passo do
    acesso — `HL` muda no M2, e resolver o endereço no fetch adianta o efeito em
    um M-cycle com o mesmo estado final. **10 dos 11 testes passam contra essa
    versão errada**; ver [doc da 0016](docs/iterations/0016-cpu-ld-r16mem.md).
    Que o endereço é o valor de antes é confirmado fora da § Block 0, na
    § OAM Corruption Bug do `06-ppu.md` ("before the operation").
  - [x] 1.4d Endereço absoluto e a página `$FF00`: `LD (u16),A` / `LD A,(u16)`
    (`$EA $FA`), `LD (FF00+u8),A` / `LD A,(FF00+u8)` (`$E0 $F0`) e
    `LD (FF00+C),A` / `LD A,(FF00+C)` (`$E2 $F2`) — 6 opcodes.
    **Era aqui que a tabela de micro-operações se decidia, e ela se decidiu
    contra si mesma:** quatro sub-itens esperando dados, e o desenho escolhido
    é `State` por variante de forma + **uma** função compartilhada
    (`Cpu::access`), o último passo das três formas — a generalização nasceu
    onde a repetição existia, não onde se apostava. `$E2`/`$F2` têm **1** byte
    (`C` é o operando; as tabelas antigas erram), e os seis são reconhecidos um
    a um, sem máscara — qualquer uma frouxa leva `$E8`/`$F8` (o 1.7). Iteração
    da **transição de motorista**: começada em sessão de Claude Code que morreu
    no RED→GREEN, retomada e concluída por Kimi K3/OpenCode — ver
    [doc da 0017](docs/iterations/0017-cpu-ld-absolute-ff00.md) e `STATUS.md`,
    nota 33.
- [x] 1.5 Opcodes: loads 16-bit + stack (PUSH/POP). **Quebrado em quatro na
  0018:** o grupo `x16/lsm` da tabela de gbops tem **14** opcodes
  (`01 08 11 21 31 C1 C5 D1 D5 E1 E5 F1 F5 F9`) e **cinco** formas de M-cycle
  distintas — 2, 3, 4 e 5 M-cycles. `LD HL,SP+i8` (`$F8`) **não** é deste grupo:
  gbops o classifica em `x16/alu`, e ele é o 1.7. A quebra é por **regra de
  decodificação**, como a do 1.4: cada sub-item é um bloco de codificação com
  uma forma própria, e sobram dois avulsos. 14 = 4 + 4 + 4 + 2.
  - [x] 1.5a `LD r16,u16` (`$01 $11 $21 $31`) — o bloco `00 rr 0001`
    (§ Block 0 do `02-cpu.md`), 3 M-cycles. Primeiro uso do placeholder `r16`
    (`bc de hl sp`, e não `af` — esse é o `r16stk` do 1.5b/1.5c). A coluna
    escreve **metade do par por M-cycle** (`read(u16:lower->C)` →
    `read(u16:upper->B)`): latchar os dois bytes e escrever o par no fim dá o
    mesmo estado final e os mesmos 12 T-cycles. **É o que eu escrevi de
    memória**, copiando o `JP u16` do mesmo arquivo — que latcha porque tem um
    quarto M-cycle `internal`, e este não tem. 7 dos 8 testes passam contra essa
    versão; ver [doc da 0018](docs/iterations/0018-cpu-ld-r16-u16.md) e
    `STATUS.md`, nota 34. A seta da coluna faz parte do passo: `read(u16:lower)`
    sem seta (o `$FA`) é que é latch.
  - [x] 1.5b `PUSH r16stk` (`$C5 $D5 $E5 $F5`) — o bloco `11 rr 0101`,
    4 M-cycles: `fetch → internal → write(upper->(--SP)) → write(lower->(--SP))`.
    Primeiro `internal` do projeto que não é o último passo, e primeiro operando
    que é registrador e endereço ao mesmo tempo com o `SP` mudando **entre** os
    dois acessos. `r16stk` tem `af` no índice 3. O `--SP` é **pré**-decremento
    escrito **dentro** do passo da escrita, como o `HL++` do 1.4c: decrementar no
    `internal` do M2 dá o mesmo estado final e os mesmos 16 T-cycles, e **8 dos
    10 testes passam** contra essa versão. **É o que eu escrevi de memória**,
    copiando o Z80 — onde o decremento mora no T-cycle extra do M1; ver
    [doc da 0019](docs/iterations/0019-cpu-push-r16stk.md) e `STATUS.md`,
    nota 36. O layout de bits está sob o cabeçalho **`pop r16stk`** do
    `02-cpu.md`: a string `push` não existe no arquivo (nota 38).
  - [x] 1.5c `POP r16stk` (`$C1 $D1 $E1 $F1`) — o bloco `11 rr 0001`,
    3 M-cycles: `fetch → read((SP++)->lower) → read((SP++)->upper)`. É onde
    `POP AF` esbarra na decisão do 1.1 de **não** mascarar o nibble baixo de `F`
    — a previsão registrada (quem cobra é a blargg `cpu_instrs/01-special` no
    1.13) continua de pé e **não** foi retroajustada. As três armadilhas
    anunciadas no handoff (pós-incremento, meia metade por M-cycle, `F` sem
    máscara) **não viraram código** — primeira vez que o `STATUS.md` descreve o
    erro seguinte em vez do anterior. O achado ficou na comparação com a 0019:
    é o mesmo erro de forma nos dois lados da pilha, e **8 de 10 testes passam**
    contra ele no `PUSH` contra **3 de 10** no `POP` — o lado que escreve tolera
    o erro de instante em silêncio, o que lê grita. Ver
    [doc da 0020](docs/iterations/0020-cpu-pop-r16stk.md) e `STATUS.md`, nota 40.
  - [x] 1.5d Os dois avulsos: `LD SP,HL` (`$F9`, 2 M-cycles,
    `fetch → internal`) e `LD (u16),SP` (`$08`, **5** M-cycles e dois bytes
    escritos em endereços consecutivos — a instrução mais longa do projeto até
    aqui). Nenhum dos dois cabe num bloco `rr`. **A iteração foi interrompida no
    meio e retomada por outra sessão**, e a primeira metade não deixou relato —
    o campo `Erros de primeira tentativa` mede aqui o que sobreviveu ao revisor,
    não o que o autor percebeu. O `$F9` é o primeiro caso em que a spec local
    **não decide** o instante (nota 21): a coluna não tem seta nem anotação em
    passo nenhum, e as duas únicas linhas do arquivo que dizem quando o `SP`
    recebe (`$33`, `$E8`) partem o par em duas metades com `Probably` — apontando
    para o **contrário** do que ficou implementado. A escolha (par inteiro no
    `internal`) ficou; a justificativa via `$F8`, que tem `internal` pelado e não
    sustenta nada, foi o erro #2. O achado é o **M11**: ler os dois bytes do
    endereço do `$08` no M2 dá a mesma memória, o mesmo `PC` e os mesmos 20
    T-cycles, e passou verde nos 239 testes anteriores — quem o pega é uma
    asserção de `PC` **entre** os M-cycles (nota 32 aplicada ao operando, não só
    à memória). Ver [doc da 0021](docs/iterations/0021-cpu-ld-stack-pointer.md)
    e `STATUS.md`, notas 42 e 43.
- [x] 1.6 Opcodes: ALU 8-bit (ADD/ADC/SUB/SBC/AND/OR/XOR/CP/INC/DEC) — **atenção
  ao half-carry**. **Quebrado em cinco na 0022:** o grupo `x8/alu` da tabela de
  gbops tem **88** opcodes em quatro blocos de codificação (`10 ooo rrr`,
  `11 ooo 110`, `00 ddd 100`, `00 ddd 101`), e a quebra do 1.4 e do 1.5 — por
  regra de decodificação, uma forma de M-cycle por sub-item — **não serve aqui**.
  As formas de M-cycle são só três em 88 linhas (`fetch`; `fetch → read`;
  `fetch → read((HL)) → write((HL))`), e a dimensão que de fato muda é a
  **coluna de flags**: `Z N H C` calculadas, `N` literal `0` × literal `1`,
  `H` carry × empréstimo × literal `1` × literal `0`, `C` calculada × literal `0`
  × **não afetada**. Então o corte é por **semântica de flag**, e os dois últimos
  sub-itens por bloco. 88 = 16 + 24 + 24 + 8 + 16.
  - [x] 1.6a `ADD a,r8` e `ADC a,r8` (`$80`–`$8F`) — os blocos `10 000 rrr` e
    `10 001 rrr`, 16 opcodes. **As primeiras flags calculadas do projeto**: até
    a 0021 as 254 linhas implementadas tinham `-` nas quatro colunas. `H` está
    **definido** na § BCD Flags do `02-cpu.md` (*"carry for the lower 4 bits of
    the result"*) e `C` na § The Carry Flag (*"higher than $FF"*) — as duas
    definições, e não uma deduzida da outra. `ADC` consome o `C` de **entrada**,
    e o `H` dele conta-o: `A=$0F` + `$00` + `C=1` é o único caso que separa as
    duas versões. Duas formas de M-cycle: 1 para registrador, 2 para `(HL)`
    (`fetch → read((HL))`, **sem seta** — e a ausência de seta aqui **não** é
    latch, porque a linha tem 8 T-cycles e não 12: não existe o terceiro passo
    onde o latch aterrissaria, e a nota 34 lida como regra geral erra aqui).
    **O achado é o M16:** ler `(HL)` dentro do fetch e gastar o M2 aplicando
    preserva `A`, as flags, os 2 M-cycles e os 8 T-cycles, e **passou verde nos
    251 testes** — o operando de uma ALU não tem testemunha entre os passos, e
    quem o pega é trocar a memória **entre** os dois `step`. Ver
    [doc da 0022](docs/iterations/0022-cpu-add-adc-r8.md) e `STATUS.md`,
    notas 44 e 45.
  - [x] 1.6b `SUB a,r8`, `SBC a,r8` e `CP a,r8` (`$90`–`$9F` e `$B8`–`$BF`) —
    os blocos `10 010 rrr`, `10 011 rrr` e `10 111 rrr`, 24 opcodes. `N` é `1`
    **literal**, e o `H` é **empréstimo** do bit 4 e não carry: três colunas com
    a mesma letra do 1.6a e o significado invertido. `CP` é a única das oito que
    **não escreve em `A`** — só produz flags. `CP` fica aqui e não com o `AND`
    porque a coluna de flags dele é a do `SUB`, letra por letra; o que o separa
    do bloco `10 010` é só a ausência da escrita. As cinco armadilhas vieram
    pré-anunciadas pelo handoff da 0022 e nenhuma virou código; o achado real
    foi processual — o controle negativo `decoded_elsewhere` precisou de
    atualização em **nove** arquivos, um deles com nome de variável diferente
    do padrão. Ver [doc da 0023](docs/iterations/0023-cpu-sub-sbc-cp-r8.md).
  - [x] 1.6c `AND a,r8`, `XOR a,r8` e `OR a,r8` (`$A0`–`$B7`) — os blocos
    `10 100 rrr`, `10 101 rrr` e `10 110 rrr`, 24 opcodes. Aqui `H` e `C` são
    **constantes na coluna**, não resultado de conta: `AND` tem `H` = `1` e
    `C` = `0`; `XOR` e `OR` têm `H` = `0` e `C` = `0`. Um half-carry calculado
    "genericamente" pelas três erra as três. `alu::logic` recebe `H` como
    parâmetro literal por chamada, sem calcular nada — ver
    [doc da 0024](docs/iterations/0024-cpu-and-xor-or-r8.md).
  - [x] 1.6d `alu a,imm8` (`$C6 $CE $D6 $DE $E6 $EE $F6 $FE`) — o bloco
    `11 ooo 110`, 8 opcodes, 2 M-cycles (`fetch → read(u8)`). As mesmas oito
    operações dos três sub-itens acima, com o operando vindo do `PC` em vez de
    `r8`. Ao contrário do `(HL)` do 1.6a (nota 45, sem testemunha), o `PC`
    **é** testemunha entre os M-cycles (nota 43) — o teste do instante lê o
    `PC` depois do M1 em vez de trocar memória no meio. `State::AluImmediate`
    e `Cpu::alu_immediate` espelham `AluFromHl`/`alu_from_hl`, casando os oito
    opcodes por literal, não por máscara — não há campo `r8` a isolar aqui.
    Achado real foi de cobertura de teste, não de spec: a bateria de mutação
    pegou um operando de teste que não distinguia `XOR` de `OR`, e um par
    `ADD`/`SUB` sem o controle "ignora o carry de entrada" (nota 46). Ver
    [doc da 0025](docs/iterations/0025-cpu-alu-a-imm8.md).
  - [x] 1.6e `INC r8` e `DEC r8` (`00 ddd 100` e `00 ddd 101`) — 16 opcodes.
    **Não tocam `C`**: a coluna é `-`, e é a divergência de flags que mais
    aparece em ROM real. `$34`/`$35` (`INC (HL)`/`DEC (HL)`) são **3 M-cycles**,
    `fetch → read((HL)) → write((HL))` — read-modify-write no **mesmo** endereço,
    em passos **diferentes**: juntar os dois num M-cycle dá a mesma memória e os
    mesmos 12 T-cycles, que é o erro #1 da 0015 numa forma nova.
    `alu::increment`/`decrement` devolvem o resultado em vez de escrever em `A`
    — o operando é qualquer `r8` ou `(HL)`. Ver
    [doc da 0027](docs/iterations/0027-cpu-inc-dec-r8.md) e `STATUS.md`, nota 48.
- [x] 1.7 Opcodes: ALU 16-bit + `ADD SP,e8` / `LD HL,SP+e8` (flags contraintuitivas).
  **Quebrado em quatro na 0028:** o grupo `x16/alu` da tabela de gbops tem **14**
  opcodes e, como o 1.6, a quebra por regra de decodificação não serve — há só
  duas formas de M-cycle (`fetch → internal` para os doze primeiros, `fetch →
  read(i8) → internal [→ write]` para os dois últimos) mas **quatro** semânticas
  de flag distintas. O corte é por coluna de flags. 14 = 8 + 4 + 1 + 1.
  - [x] 1.7a `INC r16` e `DEC r16` (`$03 $13 $23 $33 $0B $1B $2B $3B`) — 8
    opcodes, as quatro colunas de flag em `-`: nenhuma é tocada, nem calculada
    nem literal — a primeira vez que isso vale para um par de 16 bits inteiro
    e não só para `C` (o 1.6e fez isso para `r8`). `fetch(escreve a metade
    baixa) → internal(escreve a metade alta)`, 2 M-cycles. Ver
    [doc da 0028](docs/iterations/0028-cpu-inc-dec-r16.md).
  - [x] 1.7b `ADD HL,r16` (`$09 $19 $29 $39`) — 4 opcodes, `N` = `0` literal,
    `H`/`C` calculados sobre o par de 16 bits inteiro (carry do bit 11 e do
    bit 15) — ao contrário do `ADD SP,e8`/`LD HL,SP+e8` do 1.7c/1.7d, que
    calculam sobre o byte baixo. `Z` não é afetada. Mesma forma de M-cycle do
    1.7a.
  - [x] 1.7c `ADD SP,e8` (`$E8`) — 1 opcode, 4 M-cycles
    (`fetch → read(i8) → internal → write`), o mais longo do grupo. `Z`/`N`
    literais `0`; `H`/`C` calculados sobre o **byte baixo** de `SP` somado ao
    imediato — regra de 8 bits sobre um valor de 16, não o par inteiro do
    1.7b. `02-cpu.md` não tem essa seção; a nota 34/36 do 1.6a (ausência de
    seta não é sempre latch) precisa ser reavaliada aqui antes de supor.
  - [x] 1.7d `LD HL,SP+e8` (`$F8`) — 1 opcode, 3 M-cycles
    (`fetch → read(i8) → internal`), a mesma coluna de flags do 1.7c mas um
    M-cycle a menos (escreve em `HL`, um par de registrador, não em `SP` pelo
    barramento) — não presuma a mesma forma só porque a flag é igual.
- [x] 1.8 Opcodes: rotações e shifts (RLCA/RRCA/RLA/RRA — divergem do prefixo CB no flag Z).
- [x] 1.9 Opcodes: prefixo CB completo (BIT/RES/SET/rot).
  - [x] 1.9a CB decode + RLC (`CB 00`–`CB 07`) — mecanismo de dois M-cycles: o `$CB` no fetch transita para um segundo fetch que lê e decodifica o opcode real. RLC calcula `Z` (result == 0), enquanto o `RLCA` não-prefixado zera incondicionalmente — mesma armadilha da 0032 ao contrário. `(HL)` é read-modify-write em 4 M-cycles (16 T-cycles).
  - [x] 1.9b RRC + RL + RR (`CB 08`–`CB 1F`).
  - [x] 1.9c SLA + SRA + SWAP + SRL (`CB 20`–`CB 3F`).
  - [x] 1.9d BIT (`CB 40`–`CB 7F`) — `Z` = bit testado, `N=0`, `H=1`, `C` intocado. `(HL)` são 12 T-cycles (read sem write).
  - [x] 1.9e RES (`CB 80`–`CB BF`) — sem flags, `(HL)` é read-modify-write.
  - [x] 1.9f SET (`CB C0`–`CB FF`) — sem flags, `(HL)` é read-modify-write.
- [ ] 1.10 Opcodes: jumps, calls, rets, RST — com timing condicional correto. `JP u16` (`C3`) já saiu no 1.3; o que sobra aqui é o difícil — os desvios condicionais duram tempos diferentes conforme tomem ou não o desvio (`8 / 12`, `12 / 24`), e essa é a coluna que a tabela dá em dois valores. **Quebrado em cinco na 0039:** o grupo `control/br` tem **29** opcodes (já feito o `$C3`) em várias formas de M-cycle. A quebra é por **tipo de desvio**: cada sub-item é uma categoria com forma própria, e a ordem é do mais simples (M-cycles e conceitos) para o mais complexo (aninhamento de operações).
  - [x] 1.10a `JR cc,i8` (`$18` `$20` `$28` `$30` `$38`) — 5 opcodes, 2-3 M-cycles, timing condicional (8/12 T). Forma mais simples de desvio condicional: `fetch → read(i8)` sempre, `→ internal(modify PC)` só se a condição bater (o incondicional `$18` sempre toma). O deslocamento é signed, relativo a `PC` pós-leitura do opcode, e o `internal` do M3 é onde `PC += i8`.
  - [x] 1.10b `JP cc,u16` (`$C2` `$CA` `$D2` `$DA`) + `JP HL` (`$E9`) — 5 opcodes, 3-4 / 1 M-cycles, timing condicional (12/16 T). Sem desvio: `fetch → read(low) → read(high)`; com desvio: `→ internal(set PC)`. `JP HL` é incondicional de 1 M-cycle: `fetch` já copia `HL` para `PC`.
  - [x] 1.10c `CALL cc,u16` (`$C4` `$CC` `$CD` `$D4` `$DC`) — 5 opcodes, 3/6 M-cycles, timing condicional (12/24 T). Sem desvio: `fetch → read(low) → read(high)`; com desvio: `→ internal → write(PC:upper→(--SP)) → write(PC:lower→(--SP))`. Primeira instrução com `PUSH` implícito e o primeiro `internal` que decide condição entre a leitura do operando e a escrita na pilha.
     - [x] 1.10d `RET cc` (`$C0` `$C8` `$D0` `$D8`) + `RET` (`$C9`) + `RETI` (`$D9`) — 6 opcodes, 2/4-5 M-cycles, timing condicional (8/20 T). Sem desvio: `fetch → internal`; com desvio: `→ read((SP++)→lower) → read((SP++)→upper) → internal(set PC)`. O `internal` do M2 decide a condição; é o mesmo que o `POP` faz mas sem escrever em registrador — `SP` avança, os bytes lidos vão para `PC`. `RETI` é `RET` + `EI` em hardware; a ativação do IME entra aqui ou fica delegada para o 1.11/2.2.
  - [ ] 1.10e `RST` (`$C7` `$CF` `$D7` `$DF` `$E7` `$EF` `$F7` `$FF`) — 8 opcodes, 4 M-cycles (16 T), `fetch → internal → write(PC:upper→(--SP)) → write(PC:lower→(--SP))`. Essencialmente `CALL` para endereço fixo (`$00 $08 $10 $18 $20 $28 $30 $38`), sem condição e sem leitura de operando do fluxo.
- [ ] 1.11 Opcodes: misc — `DAA`, `CPL`, `SCF`, `CCF`, `DI`, `EI`, `NOP`, `STOP`.
- [ ] 1.12 Stub da porta serial (FF01/FF02) → `gb-cli` imprime em stdout.
- [ ] 1.13 blargg `cpu_instrs/individual/01` a `05`.
- [ ] 1.14 blargg `cpu_instrs/individual/06` a `11` + `cpu_instrs.gb` completo.

**Marco M1: 11/11 cpu_instrs, zero código gráfico escrito.**

## M2 — Timing e interrupções

- [ ] 2.1 Timer: DIV, TIMA, TMA, TAC + comportamento de overflow (delay de 4 ciclos).
- [ ] 2.2 Interrupções: IE/IF/IME, vetores, timing de despacho, `EI` com delay de 1 instrução.
- [ ] 2.3 `HALT` + o bug do HALT.
- [ ] 2.4 blargg `instr_timing`, `mem_timing`, `mem_timing-2`, `halt_bug`.

## M3 — PPU

- [ ] 3.1 Registradores: LCDC, STAT, SCY, SCX, LY, LYC, BGP, OBP0, OBP1, WY, WX. VRAM/OAM.
- [ ] 3.2 Máquina de modos (OAM scan 80 / draw / hblank / vblank) + interrupções STAT e VBlank.
- [ ] 3.3 Background por scanline: tilemap, tiledata, endereçamento signed/unsigned.
- [ ] 3.4 Window (incluindo o contador interno de linha da window).
- [ ] 3.5 Sprites: OAM scan, limite de 10/linha, prioridade, flip X/Y, modo 8x16.
- [ ] 3.6 Bloqueio de acesso a VRAM/OAM por modo.
- [ ] 3.7 `dmg-acid2` passando + comparação de hash do framebuffer na CI.

## M4 — Jogável

- [ ] 4.1 Joypad: P1/JOYP + interrupção.
- [ ] 4.2 MBC1: banking de ROM/RAM, modo 0/1.
- [ ] 4.3 SRAM com bateria: persistir `.sav` ao sair, carregar ao abrir.
- [ ] 4.4 `gb-desktop`: winit + pixels, 60 fps, mapeamento de teclado.

**Marco M4: Tetris e Super Mario Land jogáveis.**

## M5 — Mappers

- [ ] 5.1 MBC2 (RAM embutida de 4 bits).
- [ ] 5.2 MBC3 + RTC.
- [ ] 5.3 MBC5.

**Marco M5: Pokémon Red boota, salva e recarrega o save.**

## M6 — APU

- [ ] 6.1 Frame sequencer 512 Hz (length / envelope / sweep).
- [ ] 6.2 Canal 2: square sem sweep (o mais simples — comece por ele).
- [ ] 6.3 Canal 1: square + sweep de frequência.
- [ ] 6.4 Canal 3: wave RAM.
- [ ] 6.5 Canal 4: noise (LFSR de 15/7 bits).
- [ ] 6.6 Mixer: NR50/NR51/NR52, panning, DAC enable.
- [ ] 6.7 Downsample para 48 kHz + ring buffer + saída via `cpal`.
- [ ] 6.8 blargg `dmg_sound` 01 a 12.

## M7 — Rigor

- [ ] 7.1 Suíte Mooneye acceptance no scoreboard.
- [ ] 7.2 `oam_bug`, `interrupt_time`.
- [ ] 7.3 Savestates (serde) + fast-forward + screenshot.
- [ ] 7.4 Verificar a MSRV na CI: job em `1.85` (ou `cargo-msrv`) para que
  `rust-version = "1.85"` deixe de ser promessa que ninguém checa. A CI usa
  `dtolnay/rust-toolchain@stable`, então API mais nova que a MSRV compila, passa
  no clippy e passa nos testes. **Sete iterações conferiram à mão** (0009, 0011,
  0012, 0013, 0014, 0015, 0016 — a última deu 177/177 em `1.85`), sempre por
  alguém ter lembrado. Item criado na 0016 porque a 0015 diagnosticou que o que
  mantinha a dívida aberta era ela não existir aqui (`STATUS.md`, nota 13);
  está em M7 e não em M0 para não preemptar o M1, e é puxável a qualquer momento.
  Alternativa legítima: apagar a linha do `Cargo.toml` — declaração que ninguém
  checa é pior que declaração nenhuma.

## M8 — Apresentação

- [ ] 8.1 Consolidar `docs/iterations/*` em relatório único.
- [ ] 8.2 Gráficos a partir de `scoreboard.csv`: aprovações por commit, custo por iteração, taxa de erro de primeira tentativa por categoria.
- [ ] 8.3 Roteiro de demo: dmg-acid2 → Tetris → Pokémon Red com save → Prehistorik Man renderizando errado (trade-off do scanline renderer, explicado).
