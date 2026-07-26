# Iteração 0012 — registradores de hardware no hand-off da boot ROM

- **Data:** 2026-07-26
- **Item do roadmap:** 1.2b-ii
- **PR:** #14
- **Duração:** ~35min
- **Custo reportado:** — <!-- sessão interativa, sem --output-format json (STATUS.md, nota 10) -->
- **Turnos:** 1

## Objetivo

Ligar `$FF00`–`$FF7F` e `$FFFF` ao `Bus`, com os valores da coluna **DMG / MGB**
da tabela § Hardware registers — e não ligar o que a spec não descreve.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | § Console state after boot ROM hand-off → *Hardware registers* | `docs/reference/01-memory-map.md` (linhas 604–686) |
| Pan Docs | notas de rodapé `_obp`, `_cgb_only`, `_unk` da mesma tabela | idem |
| Pan Docs | § Common remarks (WRAM/HRAM aleatórias) | idem, linhas 543–553 |
| Pan Docs | § FF30–FF3F — Wave pattern RAM (para confirmar que a tabela **não** a cobre) | `docs/reference/07-apu.md` |

## Erros de primeira tentativa

Procedimento da nota 20: os testes foram escritos lendo a spec, e só então um
**esqueleto descartável com a versão de memória** foi posto no lugar da
implementação e a suíte rodada contra ele. O que segue não é lembrança do que eu
teria escrito — é o que o `cargo test` reprovou.

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | `DMA` (`$FF46`) = `$00` | A coluna DMG / MGB dá **`$FF`**. `$00` é a coluna **CGB / AGB** | `every_named_register_holds_the_dmg_column_value_at_hand_off` **e** `this_is_the_dmg_mgb_column_and_not_one_of_the_other_three` |
| 2 | endereçamento | `OBP0`/`OBP1` = `$FF` — paleta de objeto "branca", que é como todo mundo inicializa | `??`, e a nota de rodapé é explícita: *"These registers are left entirely uninitialized. Their value tends to be most often $00 or $FF, but the value is especially not reliable"*. Tendência observada, não fiação | `obp0_and_obp1_are_uninitialized_in_the_spec_and_zero_by_choice_here` |
| 3 | endereçamento | `BANK` (`$FF50`) = `$01`, "porque a boot ROM já se desmapeou" | `---` nas **quatro** colunas. A tabela é tirada a `PC = $0100`, quando isso já aconteceu, e ela mesmo assim não atribui valor ao registrador | `the_cgb_only_registers_do_not_exist_on_this_console` |
| 4 | endereçamento | `$FF00`–`$FF7F` é um array plano de 128 bytes; o que a tabela não menciona vale `$00` | A tabela dá valor a **41** endereços e marca **15** como `---`. Sobre os outros **72** — inclusive a wave RAM inteira, `$FF30`–`$FF3F` — ela não diz nada | `the_addresses_the_table_never_mentions_have_no_owner_yet` |

**Placar do esqueleto: 5 dos 9 testes o reprovaram.** Erros #1 e #4 doeram em dois
testes cada.

**O que a memória acertou, e vale registrar porque era a armadilha esperada:**
`DIV = $AB`, `STAT = $85` e `LY = $00`. São exatamente as três células que
separam a coluna DMG / MGB da coluna vizinha DMG0 (`$18`, `$81`, `$91`) — a
previsão da 0011 era que copiar a coluna errada fosse o risco principal, e não
foi. O risco real era outro: **três valores plausíveis vindos de folclore de
emulador**, não de uma coluna errada.

O erro #1 é o caso interessante dos dois lados. O controle negativo
`this_is_the_dmg_mgb_column_and_not_one_of_the_other_three` foi escrito para
pegar "copiou a coluna errada inteira"; ele pegou "escreveu de memória um valor
que por acaso é o de outra coluna". Não é o mesmo erro, mas o sintoma é idêntico
— e a mensagem que ele imprimiu (`$FF46 (DMA) saiu com $00, que é o valor da
coluna CGB / AGB`) diz de onde o número provavelmente veio. Controle negativo
por coluna paga mais do que o motivo pelo qual foi escrito.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |

Sem mudança, e não podia haver: ainda não existe CPU que execute. Testes do
workspace: **122 → 131**.

## Revisão cruzada (segundo modelo)

Não executada. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(`STATUS.md`, nota 5), e a revisão cruzada está desligada por padrão desde
`b8e3a8f`.

## Decisões de arquitetura

**1. `$FF00`–`$FF7F` passa a ter dono por endereço, não por região.** Até aqui
toda região era inteira: ou tinha componente, ou não tinha. A faixa de I/O não
é assim — 41 endereços ganharam célula e valor, 87 continuam respondendo
`OPEN_BUS`. Um array `IO_HAS_OWNER: [bool; 0x80]`, construído em tempo de
compilação a partir da própria tabela, decide quem é quem. Só há uma lista.

Os 87 se dividem em dois casos que o código **não** distingue e a prosa sim:
15 são `---` (a spec afirma que o registrador não existe no DMG) e 72 são
silêncio (a spec não fala deles). Hoje os dois dão `OPEN_BUS`, e isso não é
afirmação sobre o hardware — é ausência de quem responda, a mesma decisão que
VRAM e OAM levaram no 1.2a. Quando a APU chegar (6.4), a wave RAM sai do segundo
grupo; o primeiro grupo não sai nunca, porque este emulador não vira CGB.

**2. `Bus::new` é o estado de hand-off; não há `Bus::after_boot_rom`.** A
assimetria com `Registers::after_boot_rom` (1.2b-i) é deliberada e vale
explicar: lá o estado **depende da ROM** (o `F` sai do checksum gravado), então
havia o que um construtor parametrizado carregar, e `Registers::default()`
continuava fazendo sentido como "ausência de decisão". Aqui a coluna é literal e
este emulador não tem outro estado em que estar — ele nunca roda a boot ROM. Dois
construtores seriam duas coisas a manter em sincronia para nenhum ganho.

O que sai de `Bus::new` mistura **spec** (os registradores de hardware) com
**escolha deste emulador** (RAM interna zerada, `OBP0`/`OBP1` zerados). A
distinção está no doc de cada constante, e cada escolha tem um teste que a nomeia.

**3. Valor inicial não é semântica, e a fronteira está fixada por teste.** O que
esta iteração entrega é o retrato dos 41 registradores a `PC = $0100`. O que ela
deliberadamente **não** entrega: máscara de bits não usados (`TAC` lê `$F8`
porque a boot ROM o deixou assim, não porque haja máscara), bit read-only (`LY`
aceita escrita), efeito colateral (escrever em `DIV` devia zerá-lo e não zera).
`the_named_registers_have_storage_and_no_read_semantics_yet` fixa isso como
divergência conhecida, e a mensagem dele diz *"se o componente dono chegou, este
teste é que está velho"*.

**4. `src/bus.rs` → `src/bus/{mod,boot}.rs`.** A transcrição da tabela, com a
justificativa de cada linha estranha, é um arquivo por si.

## Notas

**O RED contra a implementação atual passou 5 dos 9 testes** — todos os que
afirmam *ausência* (`---`, endereços sem dono, o controle negativo de coluna, a
autoconferência da transcrição). Com tudo respondendo `OPEN_BUS`, "não é `$00`"
e "é `$FF`" são verdade de graça. É a armadilha (a) da 0007 outra vez, e é
exatamente por isso que o esqueleto existe: ele foi o único momento em que esses
cinco testes tiveram algo real para reprovar, e quatro deles reprovaram.

**Dois dos nove testes não discriminam nada e sabem disso.**
`the_named_registers_have_storage_and_no_read_semantics_yet` e
`the_hand_off_state_is_what_bus_new_gives_because_the_boot_rom_is_skipped`
passaram contra o esqueleto *e* contra a implementação. Não são medição do
código de hoje — são guarda de regressão para amanhã, no sentido da leitura (a)
da 0007. Vale saber a diferença ao contar cobertura.

**A nota 15 pagou de novo, e por um caminho novo.** A informação que desqualifica
`OBP0`/`OBP1` não está na linha da tabela: está numa nota de rodapé 30 linhas
abaixo, referenciada por um marcador `[^Power_Up_Sequence_obp]` que na renderização
do arquivo é só um `??`. Ler a tabela sem seguir os marcadores daria dois valores
inventados com aparência de spec — o mesmo modo de falha da 0007, agora com o
alvo a 30 linhas em vez de 360.

**O `BANK` (`$FF50`) merece um parágrafo porque a explicação errada é sedutora.**
Ele é `---` em todas as colunas, e o reflexo é raciocinar "a tabela é tirada a
`PC = $0100`, logo a boot ROM já se desmapeou, logo `BANK` vale `$01`". O
raciocínio é bom e a conclusão não está na spec — que é precisamente o caso da
nota 19 (spec omissa preenchida por convicção pronta). O que a tabela diz é
`---`, e `---` é o que virou código.

**O que esta iteração não pode falsificar sozinha.** A tabela vem com o aviso de
que é *"highly volatile […] may contain errors"*, e a fonte dela é a Mooneye
`acceptance/boot_hwio-dmgABCmgb` — que está em `tests/roms/` e mede exatamente
isto. Ela é do ROADMAP 7.1 e hoje dá `crash` como as outras 120. **Previsão
registrada, a conferir e não a retroajustar:** quando o 7.1 chegar, essa ROM é a
que cobra os 41 valores daqui, e é também a que vai dizer o que os 72 endereços
sem dono deviam responder — porque ela os lê. Se ela reprovar por causa deles, a
resposta não é chutar `$FF`: é achar a seção do Pan Docs que descreve I/O não
mapeado, trazê-la para `docs/reference/`, e só então implementar.
