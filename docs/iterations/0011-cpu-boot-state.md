# Iteração 0011 — estado da CPU no hand-off da boot ROM

- **Data:** 2026-07-25
- **Item do roadmap:** 1.2b-i (o 1.2b foi quebrado em dois nesta iteração)
- **PR:** #13
- **Duração:** ~35min
- **Custo reportado:** —  <!-- sessão interativa, sem --output-format json; STATUS.md nota 10 -->
- **Turnos:** 1

## Objetivo

`Registers::after_boot_rom(HeaderChecksum)` — os dez registradores da CPU no
estado em que a boot ROM do DMG entregaria o controle ao cartucho, para que o
emulador possa pular a boot ROM.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs `fe246067b695` | Power-Up Sequence § Console state after boot ROM hand-off → *CPU registers* | `docs/reference/01-memory-map.md:555–602` |
| Pan Docs `fe246067b695` | § 014D — Header checksum | `docs/reference/08-cartridges-mbc.md:456–470` |

A segunda entrou por causa da nota 15: a nota de rodapé do `F` diz "if the
header checksum is $00" e liga a palavra a **outra seção, em outro arquivo**.
Ler só a seção que o `docs/reference/README.md` aponta não bastava para saber
*qual* dos dois checksums a frase quer.

## Quebra do item

O 1.2b pedia "registradores da CPU **e** registradores de hardware". São duas
tabelas distintas da mesma seção, e a segunda tem 40+ entradas e exige ligar
`$FF00`–`$FF7F` e `IE` ao `Bus` — o que derruba
`the_regions_without_an_owner_are_open_bus_and_swallow_writes` e é um segundo
conceito. Virou 1.2b-i (esta) e 1.2b-ii, commitado à parte (`57a270f`) antes de
qualquer código.

Detalhe de processo: o passo 1 da skill manda commitar a quebra e o passo 2
manda criar a branch, mas `main` é protegida e não aceita commit direto. A
branch veio primeiro; a quebra é o primeiro commit dela.

## Erros de primeira tentativa

Procedimento da nota 20: antes de implementar, um **esqueleto descartável com a
versão de memória**, com a assinatura já correta (senão o vermelho é
`error[E0432]` e não mede asserção nenhuma), e a suíte rodada contra ele.
Resultado: **8 dos 11 testes passaram, 3 pegaram o erro** — e o erro foi um só,
exatamente onde o `STATUS.md` avisava que estaria.

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | `f: 0xB0` — constante. `AF = $01B0` é o número que se repete em todo lugar, e eu o escrevi como literal, com o parâmetro `checksum` ignorado (`_checksum`) | A coluna DMG dá `Z=1 N=0 H=? C=?`, e a nota de rodapé: *"If the header checksum is $00, then the carry and half-carry flags are clear; otherwise, they are both set."* `$B0` é o caso comum, não o valor | `half_carry_and_carry_are_clear_when_the_header_checksum_is_zero`, `f_is_b0_on_a_valid_cartridge_and_80_on_the_zero_checksum`, `the_flags_follow_the_stored_byte_and_not_the_computed_one` |
| 2 | endereçamento | — (não chegou a virar código: a spec foi lida antes) | "the header checksum" é ambíguo entre o byte gravado em `$014D` e o valor calculado de `$0134`–`$014C`. A § 014D abre com *"This byte contains an 8-bit checksum"* e a fórmula em C chama o calculado de `checksum`, não de "header checksum" → é o **gravado** | Decisão fixada por teste antes de existir implementação; ver abaixo |

**O que a memória acertou, e vale registrar:** as duas outras armadilhas que o
`STATUS.md` listou não se materializaram. A coluna copiada foi a DMG, não a
DMG0 (`this_is_the_dmg_column_and_not_one_of_the_other_four` passou contra o
esqueleto), e `OBP0`/`OBP1` nem apareceram — são da tabela de hardware, que a
quebra do item mandou para o 1.2b-ii. O erro ficou concentrado na única linha
da tabela que **não é literal**, que é também a única que a memória tinha como
número redondo pronto.

**Leitura nova sobre o esqueleto (nota 20, quarta reincidência da nota 8):**
`the_low_nibble_of_f_has_no_value_in_the_spec_and_comes_out_zero` passou
**verde contra o esqueleto errado**, porque `$B0` também tem o nibble baixo
zerado. Contra o mutante `f: 0x0F` ele pega. Ou seja: é guarda de regressão
futura, não medição do código de hoje — a mesma distinção que a 0007 tinha
achado, e que só aparece quando se roda a suíte contra o esqueleto em vez de se
ler os testes com atenção.

## A ambiguidade do checksum, e por que ela importa aqui

Em hardware real ela não existe: se os dois checksums divergem, o boot ROM
trava e o jogo nunca roda, então todo cartucho que chega ao hand-off tem
gravado == calculado. Este emulador **pula** o boot ROM, e `cart::load` não
julga o cabeçalho de propósito (invariante da 0007) — a ROM corrompida é
justamente a que alguém quer diagnosticar. Então a distinção existe aqui e em
nenhum Game Boy.

Por isso o parâmetro é `HeaderChecksum` e não `u8`. O tipo carrega os dois
valores, a escolha entre eles fica escrita num lugar só (com o porquê), e
nenhum chamador pode passar o byte errado sem querer. `after_boot_rom(0x00)`
seria uma chamada perfeitamente plausível com o significado inteiramente
perdido.

## Bateria de mutação

Nota 14 (mtime carimbado com `os.utime`) e nota 18 (padrão que casa exatamente
uma vez, conferido antes de aplicar) aplicadas no harness.

**14 mutantes, 14 pegos. 2 controles negativos, 2 verdes.**

Os mutantes que mais interessam, porque são os erros que a memória produz:

- `checksum.stored()` → `checksum.computed()` — pego só por
  `the_flags_follow_the_stored_byte_and_not_the_computed_one`. Nenhum outro
  teste distingue os dois, porque `after_boot_valid` monta cartucho íntegro.
- `e: 0xD8` → `0xC1`, `b: 0x00` → `0xFF`, `l: 0x4D` → `0x03` (coluna DMG0),
  `a: 0x01` → `0xFF` (MGB), `c: 0x13` → `0x14` (SGB) — cada um é uma célula
  vizinha da tabela, e todos morrem em
  `this_is_the_dmg_column_and_not_one_of_the_other_four`.
- `f: 0` → `f: 0x0F` — pego pelo teste que passou verde contra o esqueleto.

Controles: remover o `set_flag(Flag::N, false)` redundante (é no-op sobre
`f: 0`) e trocar `!= 0` por `> 0` continuaram verdes, como tinham de continuar.
Sem eles, "14/14 pegos" não distinguiria suíte boa de suíte que quebra com
qualquer mudança.

Vale o mesmo alerta da 0007: os mutantes foram escritos por quem escreveu os
testes, na mesma sessão. 14/14 autoriza dizer que os catorze modos de falha
imaginados doem, e nada além disso.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas (121 ROMs) | 0/121 | 0/121 |
| testes do workspace | 111 | 122 |

Sem movimento no placar de ROMs, como esperado: não há laço de execução até o
1.3, então nenhuma ROM chega a rodar um opcode.

`cargo +1.85 test --all` também deu 122/122 — segundo ponto de dado da nota 13,
que continua aberta porque continua dependendo de alguém lembrar de rodar.

## Revisão cruzada (segundo modelo)

Não executada. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(`STATUS.md`, nota 5) — não há segundo modelo disponível no ambiente.

## Decisões de arquitetura

- **O estado pós-boot é construtor, não `Default`.** Ele depende da ROM, então
  não *pode* ser `Default` — e essa impossibilidade é a melhor justificativa
  possível para a fronteira que a 0009 tinha desenhado por convenção.
- **`cpu` passa a depender de `cart`** (`use crate::cart::HeaderChecksum`).
  Dentro do mesmo crate, e é a dependência que o hardware tem: o estado da CPU
  no hand-off é função do cabeçalho do cartucho. Inverter isso (o `cart`
  produzindo registradores) seria pior.
- **`F` é montado a partir dos quatro flags nomeados, não escrito como byte.**
  Assim o nibble baixo sair zero é *consequência* de a tabela nomear quatro
  bits, e não uma máscara — que é o que a 0009 decidiu não ter.

## Notas

O achado da iteração é que a R1 tem um terceiro modo de falha, depois de "não
leu a spec" (a regra original) e "a spec é omissa" (nota 19): **a spec é
ambígua, e a leitura errada é indistinguível da certa em todo caso que roda em
hardware real**. Se eu tivesse usado o checksum calculado, nenhuma ROM comercial,
nenhuma ROM de teste e nenhum jogo jamais mostraria diferença — só ROM
corrompida mostra, e ROM corrompida é o caso que este projeto decidiu tratar
como cidadão de primeira classe lá na 0007.

O que resolve não é ler com mais cuidado, é o mesmo movimento da nota 19:
transformar a dúvida em **pergunta com endereço** — *qual byte, em que
endereço?* — e ir atrás da frase que responde. A resposta estava em outro
arquivo do `docs/reference/`, na primeira frase da seção que o link apontava.

Previsão registrada, a conferir e não retroajustar: se a escolha estiver
errada, quem cobra é a Mooneye `acceptance/boot_regs-dmgABC` no ROADMAP 7.1 —
mas provavelmente **não vai cobrar**, porque a ROM dela tem checksum válido
como qualquer outra. Isto é, esta decisão pode nunca ser falsificada por teste
nenhum do projeto. É por isso que ela está escrita aqui e fixada por
`the_flags_follow_the_stored_byte_and_not_the_computed_one`.
