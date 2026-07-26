# Iteração 0005 — Parser do cabeçalho do cartucho

- **Data:** 2026-07-25
- **Item do roadmap:** 0.3a (o 0.3 foi quebrado em 0.3a/0.3b nesta iteração)
- **PR:** #7
- **Duração:** ~35min
- **Custo reportado:** não medido — sessão interativa, sem `--output-format json`.
  Quinta iteração seguida com essa dívida; ver nota 10 do `STATUS.md`.
- **Turnos:** 1

## Objetivo

`gb_core::cart::CartridgeHeader::parse(&[u8])`: título, tipo de cartucho,
tamanho de ROM/RAM e checksum do cabeçalho, a partir dos bytes de `$0100`–`$014F`.
Puro, sem I/O — o subcomando `info` é o 0.3b.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | The Cartridge Header (`$0134`–`$014F`) | `docs/reference/08-cartridges-mbc.md` |

Primeiro item do projeto com spec de hardware de verdade — as 0001 a 0004 eram
CI e bash. A R1 funcionou: dois dos quatro erros abaixo foram pegos **antes** de
virar código, só por ler antes de escrever.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | `tabela` | Que o código `$01` de `$0149` (tamanho da RAM) são **2 KiB**. Ia escrever `0x01 => Some(2 * 1024)` sem hesitar — é assim que meio mundo documenta. | "Unused". O Pan Docs registra explicitamente que **um chip de 2 KiB nunca foi usado em cartucho** e que a origem do valor é desconhecida; ROMs homebrew antigas gravam `$01` por engano e em geral nem usam RAM. | Ler `08-cartridges-mbc.md` § 0149 antes de implementar (R1). Virou o teste `ram_size_code_one_is_unattested_not_two_kib`. |
| 2 | `tabela` | Que `$52`, `$53` e `$54` de `$0148` são 1.1, 1.2 e 1.5 MiB (72, 80 e 96 bancos) — número que eu teria escrito como fato. | Aparecem "only in unofficial docs"; nenhum cartucho ou arquivo conhecido os usa; sendo todos os outros potências de 2, a própria spec diz que provavelmente estão errados. | A nota de rodapé, na mesma leitura do #1. `RomSize::bytes` devolve `None` para eles. |
| 3 | `teste` | Que a suíte escrita antes da implementação (19 testes, R5) media a regra do título. Ela passou verde de primeira, e a regra "pare no primeiro byte não imprimível" parecia coberta por `title_stops_at_the_cgb_flag`. | Nenhum dos 19 distinguia `take_while` de `filter`: o CGB flag (`$80`) some nos dois. O caso que separa é título **curto** seguido do código do fabricante em `$013F`–`$0142`, que é ASCII imprimível — ali `filter` produz `"MARIOAXYE"`. | Mutação `.take_while` → `.filter`, que ficou verde. Teste `title_stops_before_the_manufacturer_code` escrito **depois**, e só então a mutação foi pega. |
| 4 | `ferramental` | Que o script de mutação estava medindo o que eu queria. Ele reportou `NÃO COMPILA` para **12 de 12** mutações. | Nada disso: `cargo test` imprime `error: test failed, to rerun pass...` no começo da linha quando um teste falha, e o `grep '^error'` do script vinha antes da checagem de teste reprovado. Toda mutação corretamente pega estava sendo classificada como erro de compilação. | 12/12 não compilarem é implausível — `0x01 => Some(2 * 1024)` é Rust trivialmente válido. Foi a implausibilidade do resultado, não o script, que denunciou. |

**Sobre o #1 e o #2 — o que a R1 comprou.** Os dois são a mesma forma de erro:
um valor que circula amplamente e que eu reproduziria com confiança total, sem
nenhum sinal interno de dúvida. Não seriam pegos por teste, porque o teste
teria sido escrito com o mesmo número errado — é exatamente o modo de falha
contra o qual "escreva o teste primeiro" não protege. Só ler a fonte pega.

Vale notar o que **não** deu errado, senão o registro fica enviesado: a fórmula
do checksum (`x = x - byte - 1` sobre `$0134`–`$014C`), a fórmula do tamanho da
ROM (`32 KiB × (1 << code)`) e o intervalo do título saíram da memória iguais à
spec. O erro está na periferia das tabelas, não no miolo delas.

**Sobre o #3 — a nota 8 do `STATUS.md`, terceira reincidência.** "Passou de
primeira" de novo, e de novo era vacuidade. A diferença desta vez é que a
mutação mediu isso em vez de eu confiar na leitura: das 12 mutações, 11 foram
pegas por testes que já existiam e **uma passou verde**, apontando o único
buraco da suíte. O teste que fechou o buraco não veio de reler os testes com
mais atenção; veio de a máquina dizer qual deles não existia.

**Sobre o #4 — o instrumento mentiu antes do experimento.** É a nota 7 do
`STATUS.md` em outra roupa: inferência anotada como medição. Se eu tivesse
aceitado a primeira saída, a conclusão registrada aqui seria "a bateria de
mutação não roda" — e o buraco do erro #3 continuaria de pé, escondido atrás de
um problema inventado. O detalhe que mais engana: a linha do **controle
negativo** saiu `VERDE`, correta por acidente, o que dava ao relatório inteiro
uma aparência de coerência interna.

### A bateria de mutação

Nenhuma guarda foi aceita sem falhar uma vez. O RED do passo 4 foi só erro de
compilação (`could not find cart in gb_core`), que é RED fraco: prova que o
módulo não existe, não que os testes medem alguma coisa.

| Mutação | Teste que reprovou |
|---|---|
| checksum começa em `$0133` | `checksum_ignores_bytes_outside_0134_014c` (+2) |
| checksum termina em `$014D` | `checksum_ignores_bytes_outside_0134_014c` (+2) |
| checksum sem o `-1` por byte | `checksum_follows_the_boot_rom_formula` (+2) |
| checksum soma em vez de subtrair | `checksum_follows_the_boot_rom_formula` (+1) |
| RAM `$01` = 2 KiB (o erro #1) | `ram_size_code_one_is_unattested_not_two_kib` |
| RAM `$04` e `$05` trocados | `ram_size_is_a_table_and_it_is_not_monotonic` |
| ROM aceita `$09` | `rom_size_refuses_the_unattested_codes` |
| bancos de ROM de 32 KiB | `rom_size_is_32_kib_shifted_by_the_code` |
| título filtra em vez de parar | **nenhum** → erro #3 → `title_stops_before_the_manufacturer_code` |
| campo do título com 15 bytes | `title_may_fill_all_sixteen_bytes` |
| título sem `trim_end` | `title_drops_trailing_spaces` |
| `MIN_ROM_LEN` off-by-one | `rejects_rom_that_ends_inside_the_header` |
| **controle:** `is_ascii_graphic()` → `(0x20..=0x7E)` | **nenhum** — equivalente, como projetado |

Duas linhas ficaram verdes e elas querem dizer coisas opostas: a do título é
buraco, a do controle é a prova de que a bateria não reprova por qualquer
mudança. Sem o controle, "tudo foi pego" não distinguiria uma suíte boa de uma
suíte que quebra com o vento.

Uma observação de graça: `checksum_of_a_sealed_blank_rom_is_valid` **não** pegou
a mutação "soma em vez de subtrai" — com o cabeçalho todo zerado, somar e
subtrair zero dão no mesmo. Fixture zerada testa menos do que parece.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |

Sem regressão e sem avanço: nenhuma ROM roda até o `gb-cli` executar código
(1.12). `scoreboard.csv`: 605 → 726 linhas de dado, todas `crash`.

Testes do workspace: 31 → 51.

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum.
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

Segue sem `REVIEWER_CMD` configurado (nota 5 do `STATUS.md`). Campo vazio por
ausência de ferramenta, não por esquecimento.

## Decisões de arquitetura

1. **Código desconhecido não é erro de parse.** `CartridgeType`, `RomSize` e
   `RamSize` guardam o byte cru e devolvem `None` no significado. O único erro
   é estrutural (`TooShort`). Um `info` que se recusa a falar sobre a ROM
   esquisita é justamente o que não serve para diagnosticar ROM esquisita — e o
   0.3b vai rodar sobre 121 ROMs de teste, várias delas homebrew.

2. **`None` nunca vira número inventado.** Vale para a RAM `$01` e para as ROMs
   `$52`/`$53`/`$54`. O campo é impresso pelo 0.3b e acaba em texto que alguém
   lê como medição; "desconhecido" é informação, `2048` seria ficção com cara de
   dado.

3. **O título é o trecho inicial de ASCII imprimível, com `trim_end`.** O
   cabeçalho **não** tem bit que diga se o cartucho é antigo (título de 16) ou
   novo (título de 15 ou 11, com código do fabricante e CGB flag no fim). Como
   não dá para decidir pela estrutura, decide-se pelos bytes: para no primeiro
   que não seja imprimível. Não é a única regra defensável, mas é a única que
   não exige adivinhar a época do cartucho.

4. **`parse` recebe a ROM inteira, não uma fatia de 80 bytes.** Assim os índices
   no código são os endereços absolutos da spec (`0x0134..=0x014C`), e conferir
   implementação contra documento é comparação literal, não aritmética de
   deslocamento. Custa uma checagem de tamanho na entrada.

5. **Checksum global (`$014E`–`$014F`) fora de escopo.** Precisaria da ROM
   inteira e nenhum hardware o verifica — só o emulador de GB Tower do Pokémon
   Stadium. Entra quando algum item pedir; hoje seria código sem cliente.

6. **Testes em `tests/cart_header.rs` com ROMs sintéticas.** Integração contra
   a API pública, que é o que o 0.3b vai consumir. Nada de ler `tests/roms/`:
   aquilo é gitignored e baixado por script, e o teste passaria vazio na máquina
   de quem não rodou o download — o modo de falha da nota 8.

## Notas

**O 0.3 foi quebrado antes de começar.** Parser puro em `gb-core` e subcomando
com I/O em `gb-cli` são dois conceitos e passariam de ~300 linhas juntos. Mesmo
critério que quebrou o 0.2 em a/b/c; o corte aqui é o da R3.

**A primeira validação empírica do parser é o 0.3b.** Tudo aqui foi verificado
contra ROMs que eu mesmo montei — o que fecha o argumento sobre a fórmula, mas
não sobre a realidade. As 121 ROMs de `tests/roms/` são o primeiro contato com
cabeçalho que alguém escreveu de verdade, e a regra do título (decisão 3) é a
que tem mais chance de destoar. Se destoar, o dado vai para o doc da 0006.

**Cinco iterações, custo não medido em todas.** Sem novidade e sem
encaminhamento: continua não sendo tarefa de nenhum item do ROADMAP.
