# STATUS

> Este arquivo é a **memória do projeto entre iterações**. O contexto do agente
> é descartado a cada iteração; este arquivo não. Mantenha-o curto e verdadeiro.

**Última iteração concluída:** 0025 — `alu a,imm8`, os 8 opcodes `$C6 $CE $D6 $DE $E6 $EE $F6 $FE` ([doc](docs/iterations/0025-cpu-alu-a-imm8.md)). As mesmas oito operações do 1.6a/b/c, operando vindo do `PC` em vez de `r8`. **Erro de hardware: nenhum** — `AluOp`/`alu::apply` não mudaram, e o risco de instante (`PC` como testemunha, nota 43) já vinha antecipado no handoff da 0024. **O achado foi de cobertura de teste, pego pela própria bateria de mutação:** um operando de teste (`0x0C`) que não compartilhava bit nenhum com `SEED_A` deixava `XOR`/`OR` indistinguíveis (mesmo resultado por coincidência), e faltava o controle "`ADD`/`SUB` ignoram o carry de entrada" ao lado do que já testava `ADC`/`SBC` consumindo-o — nenhum dos dois apareceu lendo o teste, só depois de mutantes sobreviverem. Nota 46. Bateria: **10/10 pegos, 2/2 controles verdes**. `State::AluImmediate`/`Cpu::alu_immediate` espelham `AluFromHl`/`alu_from_hl`; os oito opcodes são casados por literal, não por máscara — não há campo `r8` a isolar em `11 ooo 110`. `decoded_elsewhere`/`previously_decoded` ganhou os oito opcodes deste item nos dez arquivos que já tinham a função.
**Iteração anterior:** 0024 — `AND a,r8`, `XOR a,r8` e `OR a,r8`, os 24 opcodes `$A0`–`$B7` ([doc](docs/iterations/0024-cpu-and-xor-or-r8.md)). A armadilha muda de natureza em relação ao 1.6a/1.6b: `H`/`C` são **constantes na coluna**, não carry nem empréstimo — `AND` tem `H=1`/`C=0`, `XOR`/`OR` têm `H=0`/`C=0`. **Erro de conta: nenhum** — a tabela de flags veio pronta do handoff da 0023 e `alu::logic` recebe `H` como parâmetro literal por chamada em vez de calculá-lo, que era exatamente a forma que a nota da 0023 pedia para evitar. **O atrito foi de ferramenta, não de spec:** `clippy::type_complexity` (tipo de função inline sem `type` nomeado) e `clippy::identity_op` (`0xFF & AFTER` no teste do M2, `0xFF` sendo o neutro do `&`) reprovaram antes do `cargo test --all`. O controle negativo `decoded_elsewhere`/`previously_decoded` ganhou `(0xA0..=0xB7)` em **dez** arquivos — os nove da 0023 mais o próprio `cpu_sub_sbc_cp_r8.rs`, que passou a precisar dele por não ser mais o sub-item mais recente.
**Duas iterações atrás:** 0023 — `SUB a,r8`, `SBC a,r8` e `CP a,r8`, os 24 opcodes `$90`–`$9F` e `$B8`–`$BF` ([doc](docs/iterations/0023-cpu-sub-sbc-cp-r8.md)). Mesma letra do 1.6a, grandeza invertida: `N` é `1` literal, `H`/`C` são empréstimo em vez de carry, e `CP` é a primeira das oito operações da ALU que **não escreve** em `A`. **Erro de conta: nenhum** — as cinco armadilhas vieram pré-anunciadas letra por letra pelo handoff da 0022. **O achado foi processual:** o controle negativo `decoded_elsewhere`/`previously_decoded` teve de ganhar `(0x90..=0x9F)`/`(0xB8..=0xBF)` em **nove** arquivos, um deles com nome de variável diferente do padrão.
**Próxima tarefa:** ROADMAP 1.6e — `INC r8` e `DEC r8` (`00 ddd 100`/`00 ddd 101`), 16 opcodes. **Não tocam `C`** — a coluna é `-`, a primeira vez que uma operação da ALU deixa uma flag intocada em vez de calculá-la ou usar um literal; `alu.rs` precisa de uma forma nova (as três existentes — `add`/`subtract`/`logic` — sempre escrevem as quatro flags). `$34`/`$35` (`INC (HL)`/`DEC (HL)`) são **3 M-cycles**, `fetch → read((HL)) → write((HL))` — read-modify-write no mesmo endereço em passos diferentes: juntar os dois num M-cycle só é o erro #1 da 0015 (`LD (HL),u8`) numa forma nova, e é o primeiro risco de instante do 1.6 que envolve **escrita**, não só leitura. Fecha o item 1.6 inteiro (88 = 16+24+24+8+16). Nota 46 (0025): escolher o operando/par de operandos que **distingue** os casos que o teste alega cobrir é parte de escrever o teste — vale de novo aqui, com quatro operações (`INC`/`DEC` de `r8` e de `(HL)`) e valores de fronteira (`$0F→$10`, `$FF→$00`, `$00→$FF`) fáceis de colapsar num operando só.
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

Testes do workspace: **288** (eram **279** antes da 0025). Este
número não é o placar — ele mede o que o projeto afirma sobre si mesmo, não o
que o hardware cobra.

## Invariantes

Corpo em [`docs/invariantes.md`](docs/invariantes.md). Abra só a que
importar para o item da vez.

- Identificadores em inglês; comentários, docs e mensagens de teste em
- Workspace:
- Códigos de saída do `gb-cli`
- `gb-cli info <rom>` relata, não julga.
- Os três passos de qualidade do job `check` são incondicionais.
- `main` é protegida:
- `docs/reference/` é a fonte de verdade e é commitado.
- ROMs de teste não entram no git.
- `scoreboard.csv` é acumulativo e versionado.
- A série completa mora na branch `scoreboard-data`, não em `main`.
- O `GITHUB_TOKEN` deste repositório é `read` por padrão.
- `scripts/scoreboard.sh` sai != 0 quando não anexa nenhuma linha.
- O job `scoreboard` não pode engolir o veredito do script.
- Contrato do `gb-cli`
- No `cart`, código desconhecido não é erro; `None` não é número inventado.
- Tabela de RAM não é fórmula.
- O cartucho fala com o barramento por dois métodos, e só.
- `OPEN_BUS = $FF` é constante nomeada.
- `cart::load` despacha por `$0147` e não julga o cabeçalho.
- `NoMbc` mapeia direto e não espelha.
- O título do cartucho é o trecho inicial de ASCII imprimível.
- `F` carrega os 8 bits: o `gb-core` não mascara o nibble baixo.
- Flags: `Z`=7, `N`=6, `H`=5, `C`=4, e o Z80 não vale de guia.
- `Registers`: campos de 8 bits públicos, pares de 16 bits são métodos.
- `Registers::default()` é tudo zero, e isso não é o estado pós-boot.
- `after_boot_rom` copia a coluna DMG, e as cinco colunas são consoles
- O `F` pós-boot sai do checksum 
- O `Bus` é `struct`, não `trait`, e é o dono do estado.
- `Bus::read`/`write` não avançam o tempo.
- `Region` é público e separado do `Bus`.
- A região proibida `$FEA0`–`$FEFF` lê `$00`, não `$FF`.
- A HRAM tem 127 bytes: `$FF80`–`$FFFE`.
- O echo é mais curto que a fonte.
- Região sem dono lê `OPEN_BUS` e engole escrita — e há teste fixando isso.
- A RAM interna começa zerada, e isso é escolha, não hardware.
- A faixa de I/O tem dono por endereço, não por região — 41 / 15 / 72.
- `Bus::new` é o estado de hand-off; não existe `Bus::after_boot_rom`.
- `OBP0`/`OBP1` são `??` na spec, e `$00` aqui por escolha.
- Valor inicial não é semântica, e a fronteira está fixada por teste.
- `DMA` é `$FF` no DMG, não `$00`.
- `Cpu::step(&mut Bus)` avança um M-cycle e não devolve contagem.
- O `fetch` é o primeiro M-cycle da instrução, e ele conta.
- `JP u16` desvia no M4, não no M3.
- `Cpu` não é dono do `Bus`.
- Não há tabela de micro-operações, e a ausência é deliberada.
- `Lockup` distingue "o SM83 não tem esse opcode" de "este emulador ainda não
- Onde a CPU para de travar é escolha, não spec.
- `$76` é `HALT`, e é o único buraco de um bloco perfeitamente regular.
- `LD r,(HL)` faz a leitura e a escrita no registrador no mesmo M2.
- `R8` e `ByteRegister` são dois tipos.
- `LD (HL),u8` (`$36`) lê o imediato no M2 e escreve em `(HL)` no M3.
- `LD r,u8` faz a leitura e a escrita no registrador no mesmo M2.
- O bloco `00 ddd 110` não tem exceção.
- Opcode reconhecido por máscara pede controle negativo dos 256.
- `decoded_elsewhere` é duplicada de propósito em cada arquivo de sub-item.
- Ainda não há tabela de micro-operações, e a data mudou.
- Os oito opcodes `r16mem` são 2 M-cycles, e o `HL±` é do M2.
- `LD r16,u16` escreve meia metade por M-cycle, e não o par no fim.
- A seta da coluna faz parte do passo.
- `r16` é `bc de hl sp`; `af` é do `r16stk`, que é outra tabela.
- `SP` é a única metade de par que não é um campo de 8 bits.
- `LD HL,SP+i8` (`$F8`) não é do `x16/lsm`.
- O endereço é o valor de antes do `HL±`, e quem confirma isso não é a § Block
- `LD (HL+),A` com `HL` apontando para o destino não é caso especial.
- A máscara dos oito `r16mem` tem duas formas, e as duas estão em uso de
- Os seis do 1.4d são reconhecidos um a um, sem máscara — e isso fecha o
- O `--SP` do `PUSH` é do passo da escrita, e o `internal` do M2 não faz nada.
- A metade alta vai primeiro, para o endereço mais alto.
- `Cpu::push_byte` é indivisível de propósito.
- `R16Stk` e `R16` são dois tipos, e a quarta variante é a diferença inteira.
- `PUSH AF` escreve os 8 bits de `F`.
- O `SP` dá a volta abaixo de `$0000`:
- O `(SP++)` do `POP` é pós-incremento, e a metade baixa vem primeiro.
- `Cpu::pop_byte` é a simétrica de `push_byte`, e a simetria é de papel.
- O mesmo erro de instante é barulhento no `POP` e silencioso no `PUSH`, e a
- `write_r16_stk_low`/`_high` não reusam as do `R16`.
- `POP AF` lê os 8 bits de `F`.
- `$F1` é a única linha do bloco com flags, e o teste do `PUSH` não se
- O instante do `LD SP,HL` é escolha deste projeto, e a spec local aponta para
- O `(u16+1)` do `$08` é endereço no mapa inteiro, não índice dentro da
- O `$08` e o `PUSH` guardam o mesmo layout little-endian escrevendo em ordens
- As quatro fases do `$08` são dois valores de 16 bits, não um.
- O `$08` duplica o latch de dois bytes do `Absolute`, e a duplicação é
- `H` é o carry do nibble baixo e `C` o do byte — duas grandezas, dois bits de
- O carry de entrada do `ADC` faz parte do resultado, então conta para `H`
- `ADD A,(HL)` são 2 M-cycles, e a nota 34 não vale aqui.
- O operando de uma ALU não tem testemunha entre os M-cycles, e por isso o
- `alu.rs` é módulo próprio e `apply` é função livre sobre `Registers`.
- `H`/`C` do `SUB`/`SBC`/`CP` são empréstimo, não carry — mesma letra do
- O empréstimo de entrada do `SBC` conta para o `H` também, espelho exato do
- `CP` é a primeira das oito operações da ALU que não escreve em `A`.
- `decoded_elsewhere`/`previously_decoded` precisou de atualização em nove
- `H`/`C` do `AND`/`XOR`/`OR` são constantes na coluna, e `alu::logic`
- `decoded_elsewhere`/`previously_decoded` chegou a dez arquivos.

## Bloqueios

_(nenhum)_

A proteção de branch **funcionou** no plano atual — não foi preciso o
contorno previsto no prompt de bootstrap.

## Notas

Corpo em [`docs/notas.md`](docs/notas.md). O parágrafo **Próxima tarefa**
acima cita por número as que valem para a iteração seguinte — abra essas.
Numeração é estável e citada no código: **nunca renumere**.

1. A guarda na CI some sozinha. — RESOLVIDA
2. As linhas geradas pela CI se perdem. — RESOLVIDA
3. `scoreboard.csv` vai gerar conflito
4. 9 ROMs mooneye são de outros modelos
5. `scripts/review.sh` ainda não foi configurado.
6. `blargg/cgb_sound` foi deliberadamente excluída
7. O `scoreboard.sh` tinha um bug latente e a 0001 o destravou. — RESOLVIDA
8. Escrever teste antes da implementação não torna o teste testado.
9. Bash: `declare -A m` sem atribuição é variável NÃO associada.
10. O custo por iteração não está sendo medido.
11. A publicação da série funciona — observada.
12. O parser do cabeçalho nunca viu ROM de verdade. — RESOLVIDA
13. A MSRV é promessa que ninguém verifica.
14. Bateria de mutação: o cargo decide rebuild por mtime.
15. **A R1 diz "leia a seção correspondente"; a seção correspondente não é a
16. O 0.5 já está feito, mas não marcado. — RESOLVIDA
17. **O fallback do `fetch-test-roms.sh` entrega menos do que promete, e sai
18. Guarda de script bash não sofre a armadilha de mtime da nota 14.
19. A R1 protege contra spec não lida. Não protege contra spec omissa.
20. **Escreva o erro de memória em código, não em prosa — e depois leia qual
21. O terceiro modo de falha da R1: spec ambígua.
22. **A previsão de qual armadilha vai doer erra — e o registro é o que mostra
23. A nota 15, terceira reincidência, agora a 30 linhas de distância.
24. O quarto modo de falha da R1: a spec local corrompida na conversão.
25. **O controle negativo pega o que os esqueletos não pegam — e às vezes é o
26. Correção anterior virada em regra geral é um jeito novo de errar.
27. Rode a suíte nova contra o código velho antes de implementar.
28. O esqueleto e a bateria de mutação não exercitam os mesmos testes.
29. **Controle negativo que sobrevive por redundância não é controle — é
30. A nota 26 corre nas duas direções, e a 0015 correu na outra.
31. **Teste que afirma a fronteira do que existe envelhece quando a fronteira
32. A previsão certa não gera, sozinha, a suíte que a honra.
33. **Troca de motorista: a partir da 0018, quem itera é o Kimi K3 (OpenCode),
34. A nota 26/30 tem três direções, e a 0018 fechou o trio.
35. Guarda de ausência não entra na conta do esqueleto, por construção.
36. **A nota 26/30/34 tem uma quarta direção, e a fonte da regra falsa saiu do
37. Guarda de ausência ganha valor quando o erro é "algo inerte fez algo".
38. **`grep` pelo mnemônico em `02-cpu.md` pode devolver zero para instrução que
39. `cargo test` sem `--no-fail-fast` esconde o estrago de um opcode novo.
40. **O erro de instante é assimétrico: o lado que escreve o tolera em silêncio,
41. **O handoff que descreve o erro *seguinte* funciona — e enfraquece a própria
42. **Sessão interrompida no meio deixa um buraco no dado, e o buraco tem forma
43. A doutrina da nota 32 tem dois lados, e até a 0021 só um estava em uso.
44. **O erro de instante tem dois regimes, e só um deles é a classe que este
45. **A nota 43 tem uma terceira forma: quando o valor não para em lugar nenhum,
46. **O operando de teste tem de distinguir os casos que o teste alega cobrir —
