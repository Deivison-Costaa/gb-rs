# Iteração 0006 — `gb-cli info <rom>`

- **Data:** 2026-07-25
- **Item do roadmap:** 0.3b (fecha o 0.3)
- **PR:** #8
- **Duração:** ~40min
- **Custo reportado:** não medido — sessão interativa, sem `--output-format json`.
  Sexta iteração seguida com essa dívida; ver nota 10 do `STATUS.md`.
- **Turnos:** 1

## Objetivo

A casca de I/O do parser da 0005: ler o arquivo, interpretar os argumentos,
imprimir o cabeçalho e escolher o código de saída. Com isso o 0.3 fecha, e o
parser vê ROM escrita por outra pessoa pela primeira vez.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | The Cartridge Header — `0134` título, `0147` tipo, `0148` ROM, `0149` RAM, `014D` checksum | `docs/reference/08-cartridges-mbc.md` |
| `sysexits.h` (BSD) | `EX_USAGE` 64, `EX_DATAERR` 65, `EX_NOINPUT` 66 | — (convenção de sistema, não de hardware) |

Releitura, não leitura nova: as tabelas são as mesmas da 0005. Nenhum
comportamento de hardware foi decidido aqui — este item só dá nome ao que o
`gb-core` já interpretou. A R1 continua valendo para o 0.4.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec/realidade diz | Como foi pego |
|---|---|---|---|---|
| 1 | `teste` | Que `!stdout.contains("2 KiB")` era uma guarda segura da invariante da RAM `$01` (erro #1 da 0005) — varrer a saída inteira atrás do número inventado. | A linha da ROM diz `32 KiB`, e `"32 KiB".contains("2 KiB")` é **verdadeiro**. A guarda reprovava a implementação **correta**. | `cargo test`: 15 verdes e essa vermelha, contra código certo. Escopada ao campo `RAM:`. |
| 2 | `API-Rust` | `bytes.is_multiple_of(MIB)` — foi o que escrevi de primeira, por ser o idioma que o clippy recomenda. | O método é do Rust **1.87**; o workspace declara `rust-version = "1.85"`. **Nenhum passo da CI pegaria:** `dtolnay/rust-toolchain@stable` compila com o stable do dia, onde o método existe. Verde, e a MSRV quebrada em silêncio. | Eu mesmo, antes de compilar. Não há gate — virou nota 13 do `STATUS.md`. |
| 3 | `teste` | Escrevi `"$E7 (calculado $E7) — confere"` como resultado esperado do fixture TETRIS, com a naturalidade de quem calculou. | O checksum daquele cabeçalho é `$08`. `$E7` não veio de lugar nenhum. | Antes de rodar, ao trocar o literal por um valor derivado do próprio fixture. Se tivesse rodado com o literal, o vermelho apontaria para a implementação. |
| 4 | `ferramental` | Que o `awk` que compara tamanho declarado × tamanho do arquivo estava medindo isso. Reportou `DIVERGE` para **121 de 121** ROMs. | O campo comparado era `$3`, que na linha `ROM: $00 32 KiB (2 bancos)` é a string `KiB`, não o número. Zero ROMs divergem. | A implausibilidade do resultado: 121 de 121 divergirem exigiria que as ROMs de teste do ecossistema inteiro tivessem cabeçalho errado. |
| 5 | `ferramental` | Que `./target/debug/gb-cli` era o binário do código-fonte que eu acabara de reverter. | Era o binário da **última mutação** da bateria (`KiB` → `MiB`), que a reversão do `.rs` não desfaz. O fixture de 64 KiB saiu como `64 MiB`. | Implausibilidade de novo: 64 MiB não cabe num cartucho de Game Boy — o teto é 8 MiB. `touch` no fonte, rebuild, e a varredura das 121 ROMs refeita com binário limpo. |
| 6 | `ferramental` | Que a bateria de mutação, tendo reportado `ERRO-SED` no caso que falhou, estava honesta sobre todos os outros. | Quando a substituição falha, o arquivo volta com **mtime antigo**, o cargo considera o binário fresco e não recompila — então o caso seguinte roda contra o **mutante anterior**. A mutação do checksum recebeu, literalmente, o veredito da mutação do código de saída (`missing_file_...`, que não tem nada com checksum). | O conjunto de testes reprovados não fazia sentido para a mutação: inverter o veredito do checksum não pode quebrar `directory_instead_of_rom_exits_with_no_input`. Bateria reescrita em Python, com substituição literal, conferência de ocorrência única e `os.utime` após cada escrita. |

**Sobre o #1 — a guarda que protege a invariante errada de jeito errado.** O
teste existia para impedir que os `2 KiB` da RAM `$01` ressuscitassem no texto
impresso. A intenção estava certa e o instrumento, não: `contains` sobre a saída
inteira não sabe de qual campo veio o casamento. É a forma mais barata de falso
positivo — e a mais cara de diagnosticar em quem herda o teste, porque a
mensagem de falha acusa a implementação.

**Sobre o #2 — a MSRV é promessa que ninguém verifica.** Os outros erros deste
documento foram pegos por alguma coisa. Este não: se eu não tivesse notado ao
escrever, `cargo fmt`, `clippy -D warnings`, `cargo test` e a CI inteira teriam
passado, e `rust-version = "1.85"` seria uma declaração falsa no `Cargo.toml`
até alguém tentar compilar com 1.85. O clippy tem lint de MSRV, mas ele opera
sobre o que você escreveu — e `is_multiple_of` é justamente o que ele
*recomenda*. Nenhum item do ROADMAP cobre isso hoje.

**Sobre o #4, #5 e #6 — a nota 7 do `STATUS.md`, três vezes na mesma sessão.**
Os três são instrumento mentindo antes do experimento. Nos três, o que salvou
foi o resultado ser absurdo demais: 121 de 121 divergirem, 64 MiB num Game Boy,
uma mutação do checksum quebrando o teste de arquivo inexistente.

O #5 e o #6 têm a **mesma causa**: o cargo decide recompilar por mtime, e tanto
`mv arquivo.bak arquivo` quanto uma substituição que falha devolvem o fonte com
mtime anterior ao do artefato. O binário testado deixa de ser o binário do
código. Um teste de mutação sem `touch` explícito não está medindo o mutante que
diz medir — e o modo de falha é silencioso, porque o veredito **existe** e é
plausível, só pertence a outro experimento.

Vale o registro de que a bateria **tinha melhorado** desde a 0005: o caminho
`ERRO-SED` existe justamente por causa do erro #4 daquela iteração, e funcionou
— o `perl` falhou e o script disse que falhou, em vez de classificar como
`NÃO COMPILA`. Não bastou: reportar a própria falha e continuar rodando os
casos seguintes num estado contaminado são coisas diferentes. A versão em
Python (`str.replace` literal, conferência de ocorrência única, `os.utime` a
cada escrita) refez as 13 mutações do zero, e são os números dela que estão na
tabela abaixo.

**O que não deu errado, para o registro não ficar enviesado:** a regra do
título da 0005 — a que a nota 12 do `STATUS.md` apontou como mais provável de
destoar contra ROM real — **não destoou**. Ver abaixo.

## A validação empírica: 121 ROMs de verdade

Primeira vez que o parser encontra cabeçalho que outra pessoa escreveu. Todas
as 121 ROMs de `tests/roms/`, via `gb-cli info`:

| Medida | Resultado |
|---|---|
| Cabeçalho lido sem erro | 121 / 121 |
| Checksum `$014D` confere | 121 / 121 |
| Campo com código fora das tabelas | **0** |
| Tamanho declarado em `$0148` = tamanho do arquivo | 121 / 121 |

Tipos encontrados: 75 `ROM ONLY`, 26 `MBC1+RAM+BATTERY`, 17 `MBC1`, 2
`MBC1+RAM`, 1 `MBC5+RAM+BATTERY`. RAM: 94 sem RAM, 27 com 8 KiB.

**A regra do título passou no caso que a discrimina.** Em
`blargg/mem_timing-2/rom_singles/03-modify_timing.gb`, os 16 bytes de
`$0134`–`$0143` são:

```
30 33 2d 4d 4f 44 49 46 59 5f 54 49 4d 49 4e 80   >03-MODIFY_TIMIN.<
```

Título de 15 caracteres seguido do CGB flag `$80` em `$0143` — exatamente o
cenário da decisão 3 da 0005. Parar no primeiro byte não imprimível dá
`03-MODIFY_TIMIN`; não parar daria `03-MODIFY_TIMIN\u{80}` impresso no terminal
de quem for diagnosticar a ROM. A regra veio de raciocínio sobre a estrutura do
cabeçalho e agora tem uma ROM real por trás.

As 36 ROMs de título `(vazio)` são as individuais do blargg, com os 15 primeiros
bytes zerados e `$80` no décimo sexto. Cabeçalho válido, título ausente — daí o
`(vazio)` explícito em vez de linha em branco.

**A ressalva honesta:** nenhuma das 121 exercita os caminhos de código
desconhecido. `desconhecido` aparece **zero** vezes na varredura. Os ramos de
`$01` na RAM, `$52` na ROM e tipo fora da tabela continuam cobertos só por ROM
sintética. O corpus de teste é homogêneo — é toolchain de homebrew moderna,
não é o mercado de cartuchos.

## Bateria de mutação

13 mutações mais um controle negativo. Nenhuma guarda foi aceita sem ter
falhado uma vez. Números da segunda versão da bateria — os da primeira estão
contaminados pelo erro #6.

| Mutação | Testes que reprovaram |
|---|---|
| arquivo ausente sai `65` em vez de `66` | `missing_file_...`, `directory_instead_of_rom_...` |
| ROM truncada sai `66` em vez de `65` | `truncated_rom_exits_with_data_error` |
| relatório vai para `stderr` | 8 testes, incl. `info_writes_the_report_to_stdout_not_stderr` |
| RAM `$01` vira os 2 KiB inventados | `info_never_invents_two_kib_for_ram_code_one` |
| ROM sem tamanho atestado vira 32 KiB | `info_reports_an_unattested_rom_size_as_unknown` |
| título vazio imprime linha em branco | `info_says_the_title_is_empty_...` |
| veredito do checksum invertido | `info_reports_a_broken_checksum_...`, `info_prints_every_header_field_...` |
| `human()` troca KiB por MiB | `info_prints_every_header_field_of_a_well_formed_rom` |
| bancos de ROM somem do relatório | `info_prints_every_header_field_of_a_well_formed_rom` |
| imprime o checksum calculado no lugar do armazenado | `info_reports_a_broken_checksum_and_still_succeeds` |
| argumento sobrando é ignorado | `info_with_extra_arguments_exits_with_usage` |
| uso sai `2` em vez de `64` | os 4 testes de uso |
| `run` passa a sair `64` | `run_is_still_unimplemented_and_exits_two` |
| **controle:** `LABEL_WIDTH` 10 → 14 | **nenhum** — cosmético, como projetado |

13 de 13 pegas, contra 11 de 12 na 0005. Não é mérito: a suíte foi escrita
depois de a 0005 mostrar exatamente onde um teste passa por vacuidade, e os
campos aqui são strings comparadas por igualdade, que é bem mais fácil de
cobrir do que uma tabela de tamanhos.

A mutação "imprime o calculado no lugar do armazenado" entrou depois das outras,
e é a que mais interessa: ela deixa o relatório de **toda ROM íntegra**
idêntico, e só uma ROM de checksum quebrado a distingue. É a versão local da
observação da 0005 de que fixture zerada testa menos do que parece.

O controle negativo continua sendo o que dá sentido às outras 12 linhas: o
`field()` dos testes afirma o **conteúdo** do campo e ignora o alinhamento, de
propósito. Mudar a largura da coluna é cosmético e não deve pintar nada de
vermelho.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |

Sem regressão e sem avanço: `info` lê cabeçalho, não executa código. A primeira
ROM só pode passar depois do 1.12. `scoreboard.csv`: 726 → 847 linhas de dado,
todas `crash`.

Testes do workspace: 51 → 67.

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum.
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

Segue sem `REVIEWER_CMD` configurado (nota 5 do `STATUS.md`). Campo vazio por
ausência de ferramenta, não por esquecimento.

## Decisões de arquitetura

1. **Códigos de saída do `sysexits.h`, e não `1`.** `64` uso, `65` dado
   inválido, `66` entrada ilegível. O reflexo seria sair `1` em qualquer erro —
   e `1` pertence ao veredito da ROM no contrato do `scoreboard.sh`. Um `info`
   que sai `1` ao não achar o arquivo está afirmando "a ROM reprovou", que é
   falso e acabaria no `scoreboard.csv` como medição. Os códigos ≥ 64 do BSD
   existem justamente para não colidir com códigos de aplicação; a convenção é
   emprestada, não inventada. Moram em `crates/gb-cli/src/exit.rs`, num arquivo
   só, porque são contrato.

2. **`65` e `66` são códigos diferentes de propósito.** "Errei o caminho" e "a
   ROM está corrompida" são diagnósticos distintos, e quem chama precisa
   separá-los sem parsear mensagem de erro.

3. **Cabeçalho esquisito não é erro de execução.** Tipo fora da tabela, RAM sem
   tamanho atestado, checksum que não bate: tudo isso sai `0` e vira texto. É a
   decisão 1 da 0005 chegando à ponta que o usuário lê — a ROM que trava o boot
   ROM é exatamente a que alguém quer diagnosticar. O único erro de conteúdo é
   estrutural: ROM que acaba antes de `$014F`.

4. **`std::env::args_os()`, não `args()`.** `args()` **entra em pânico** com
   argumento que não seja UTF-8 válido, e caminho de arquivo não tem obrigação
   de ser. Pânico sairia por sinal, que é uma terceira categoria de saída que
   nada no projeto trata.

5. **Erro em `stderr`, relatório em `stdout`, e nunca os dois.** Com erro,
   `stdout` fica vazio: relatório pela metade de ROM inválida é pior que
   relatório nenhum, porque tem cara de dado bom. Há teste para os dois lados.

6. **Sem `--help`.** O uso é impresso junto do erro que o provocou. Uma flag de
   ajuda é generalização que o ROADMAP não pediu (passo 5 do protocolo); entra
   quando algum item precisar.

7. **O relatório imprime o tamanho do arquivo, e não o compara com `$0148`.**
   A comparação seria uma verificação nova, com veredito próprio, e o item pede
   impressão. Os dois números ficam lado a lado e quem lê compara. (Na varredura
   das 121, batem em todas — mas isso é dado do documento, não código.)

8. **O `run` continua saindo `2`.** Reescrever o despacho de argumentos é
   exatamente onde esse contrato se perderia sem ninguém notar: as 121 linhas do
   CSV mudariam de categoria sem nada ter mudado no emulador.
   `run_is_still_unimplemented_and_exits_two` guarda.

## Notas

**O 0.3 fecha aqui.** Cabeçalho parseado (0.3a) e legível pela linha de comando
(0.3b). Próximo é o 0.4: `Cartridge` trait + `NoMbc`, que é a primeira vez que o
`gb-core` vai *usar* o campo `$0147` em vez de só nomeá-lo.

**A varredura das 121 ROMs não virou teste, e isso é decisão.** `tests/roms/` é
gitignored e baixado por script; um teste que dependesse dele passaria vazio na
máquina de quem não rodou o download — o modo de falha da nota 8 do `STATUS.md`.
O dado empírico está neste documento, com data e números. Quando o job
`scoreboard` (que tem as ROMs) precisar disso como gate, o lugar é lá.

**Três das seis falhas desta iteração foram do instrumento, não do código.**
Proporção pior que a da 0005, e as três foram pegas pelo mesmo mecanismo: o
resultado ser absurdo demais para ser verdade. Isso **não escala** — funciona
para "121 de 121 divergem", para "64 MiB num Game Boy" e para "a mutação do
checksum quebrou o teste de arquivo inexistente"; não funciona para um número
plausível e errado, que é o caso comum. Vale como aviso, não como método.

Se houver um encaminhamento, é este: os erros #5 e #6 seriam impossíveis se a
bateria de mutação rodasse contra um alvo com `cargo test` forçado a
recompilar. Como ela não é código do projeto — mora no scratchpad da sessão e
morre com ela — o aprendizado só sobrevive se estiver escrito. Está na nota 14
do `STATUS.md`.
