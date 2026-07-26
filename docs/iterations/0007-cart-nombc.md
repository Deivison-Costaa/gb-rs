# Iteração 0007 — `Cartridge` + `NoMbc`

- **Data:** 2026-07-25
- **Item do roadmap:** 0.4
- **PR:** #9
- **Duração:** ~35min
- **Custo reportado:** não medido — sessão interativa, sem `--output-format json`.
  Sétima iteração seguida com essa dívida; ver nota 10 do `STATUS.md`.
- **Turnos:** 1

## Objetivo

O trait `Cartridge` (o que o barramento enxerga do cartucho) e o cartucho sem
mapeador, com o despacho por `$0147` — a primeira vez que o core **usa** o tipo
do cartucho em vez de só nomeá-lo.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | § No MBC — "not more than 32 KiB", "directly mapped to memory at $0000-7FFF", RAM opcional em `$A000`–`$BFFF` | `docs/reference/08-cartridges-mbc.md` (l. 523) |
| Pan Docs | § 0147 — Cartridge type, **e a nota de rodapé de `$08`/`$09`** | `docs/reference/08-cartridges-mbc.md` (l. 163) |
| Pan Docs | § MBC1, `$A000`–`$BFFF` — "reads return open bus values (often $FF, but not guaranteed)" | `docs/reference/08-cartridges-mbc.md` (l. 572) |
| Pan Docs | Memory Map — `$0000`–`$3FFF`, `$4000`–`$7FFF`, `$A000`–`$BFFF` | `docs/reference/01-memory-map.md` |

A § No MBC tem **quatro linhas**. Foi a nota de rodapé de outra seção que
decidiu o escopo do item — ver o erro #1.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | `hardware` | Que o 0.4 entregaria `NoMbc` **com** os 8 KiB opcionais de RAM em `$A000`–`$BFFF`, e que `load` aceitaria `$00`, `$08` e `$09` — a § No MBC descreve a RAM opcional em pé de igualdade com a ROM. | Os únicos tipos que declaram essa RAM são `$08` e `$09`, e a tabela de `$0147` os marca: *"No licensed cartridge makes use of this option. The exact behavior is unknown."* Não existe comportamento documentado para copiar. | Ler a § 0147 **depois** da § No MBC, antes de escrever (R1). As duas seções isoladas dizem coisas diferentes da que dizem juntas. |
| 2 | `endereçamento` | Que leitura acima do fim de uma ROM menor que 32 KiB espelha o começo — `self.rom[addr as usize % self.rom.len()]`, que é o idioma comum. | A spec descreve o cartucho de 32 KiB e **não diz nada** sobre chip menor. Espelhar é inventar como o chip foi fiado. E o idioma traz um segundo defeito de brinde: `% 0` entra em pânico com ROM vazia. | Escrever o teste primeiro e ter de declarar o valor esperado: não havia fonte para "espelha". Virou a mutação #4 da bateria. |
| 3 | `teste` | Que o `cargo test` vermelho da primeira rodada era o RED do ciclo. | Era `error[E0432]: unresolved imports` — o módulo não existia. Erro de compilação não diz se alguma asserção mede alguma coisa; é a nota 8 do `STATUS.md` outra vez. | Esqueleto descartável (tudo devolve `OPEN_BUS`, `load` nunca despacha) para ver o vermelho por asserção. **4 dos 13 testes passaram contra o esqueleto** — ver abaixo. |

**Sobre o #1 — a seção certa não é a única seção.** A R1 manda ler o arquivo
correspondente antes de implementar, e eu li: a § No MBC, inteira, com a frase
sobre a RAM opcional. O que quase passou foi que a informação que **desqualifica**
aquela frase mora 360 linhas acima, numa nota de rodapé da tabela de `$0147`.
Lidas juntas, as duas dizem: o hardware é possível, ninguém licenciado o usou, e
o comportamento não está documentado. Implementar 8 KiB de RAM ali seria inventar
hardware com aparência de spec — exatamente o que a invariante do `cart` proíbe
desde a 0005, agora em outro campo.

O item ficou menor do que eu tinha planejado, e ficou **certo**: `load` recusa
`$08`/`$09` com erro nomeado em vez de entregar um cartucho cuja RAM lê `$FF` e
engole escrita, que perderia o save do jogo sem um erro sequer.

**Sobre o #3 — três testes passam sem que exista implementação.** Contra o
esqueleto (que responde `OPEN_BUS` a tudo e nunca despacha), passaram:

| Teste | Por quê |
|---|---|
| `the_external_ram_window_is_open_bus` | vacuidade — afirma **ausência** de comportamento |
| `addresses_outside_the_cartridge_are_open_bus` | vacuidade — idem |
| `load_does_not_judge_the_header_checksum` | vacuidade — o esqueleto não julga nada |
| `load_propagates_the_header_error` | de verdade — o esqueleto já chamava `parse` |

Os três primeiros continuam no arquivo de propósito, mas é honesto dizer o que
eles são: **guardas de regressão futura**, não medições do código de hoje. Eles
começam a valer no 1.2, quando o `Bus` rotear de verdade, e no 4.2, quando o
MBC1 chegar e alguém puder fazer `NoMbc` entrar em pânico em endereço não
mapeado. Hoje não distinguem implementação de esqueleto.

### Bateria de mutação

11 mutantes, aplicados por substituição literal com conferência de ocorrência
única e `os.utime` explícito (nota 14 do `STATUS.md`, que custou dois resultados
falsos à 0006).

| Resultado | Contagem |
|---|---|
| Mutações que deviam ser pegas, e foram | 9/9 |
| Controles negativos (equivalentes), que deviam ficar verdes | 2/2 |
| Surpresas | 0 |

Pegas: teto de 32 KiB virando 64; teto exclusivo em vez de inclusivo; janela de
ROM parando em `$3FFF`; espelhamento da ROM curta; `$00` no lugar de barramento
aberto; escrita em ROM pegando; tamanho recusado reportado como zero; despacho
aceitando qualquer `$0147`; `ROM_ONLY` virando `$01`.

**A leitura honesta do 9/9 não é "a suíte está completa".** A 0005 rodou 12
mutações e achou um buraco; esta rodou 11 e não achou nenhum — mas os mutantes
foram escritos por quem escreveu os testes, na mesma sessão. O ponto cego dos
testes é provavelmente o mesmo ponto cego da lista de mutantes. O que o 9/9
autoriza a dizer é que os nove modos de falha **imaginados** doem; não que não
haja um décimo.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas (121 ROMs) | 0/121 | 0/121 |

Sem regressão e sem avanço, como esperado: não há CPU para rodar ROM nenhuma.
O placar volta a se mexer no 1.13.

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum. `scripts/review.sh` continua sem `REVIEWER_CMD`
  configurado (nota 5 do `STATUS.md`) — sétima iteração sem revisão cruzada.
- **Achados:** —
- **Procedentes:** —

## Decisões de arquitetura

**`Cartridge` é trait, e o `Bus` vai carregar `Box<dyn Cartridge>`.** É o que o
ROADMAP pede, e a interface é honesta com o hardware: o MBC fica entre o
barramento e os chips, e o Game Boy não tem como perguntar nada a ele além de
ler e escrever endereço. O custo é despacho dinâmico em **toda** leitura de
memória — o caminho mais quente do emulador inteiro. Fica registrado como coisa
a medir quando houver o que medir (M1), pela mesma razão que o `Cargo.toml` adia
LTO: o custo é certo e o ganho, especulativo. Trocar para `enum` depois é
mudança local, porque só estes dois métodos atravessam a fronteira.

**`load` não julga o cabeçalho.** Checksum errado, título ilegível e tamanho
declarado que não bate com o arquivo montam normalmente. Quem trava a máquina
por checksum é o boot ROM, que este emulador pula (1.2). É a mesma decisão que o
`gb-cli info` tomou na 0006, e pelo mesmo motivo: a ROM quebrada é justamente a
que alguém quer investigar.

**`OPEN_BUS = $FF` é constante nomeada, não literal solto.** O Pan Docs escreve
"often $FF, but not guaranteed", e o nome carrega o "not guaranteed" para o
lugar onde alguém vá depender do valor.

Todas as três viram invariante no `STATUS.md`.

## Notas

**Sobre MSRV (nota 13 do `STATUS.md`):** nada de API pós-1.85 entrou —
`RangeInclusive::contains`, `into_boxed_slice`, `slice::get`, `debug_struct`.
Verificado **por inspeção**, que é exatamente a fraqueza que a nota 13 aponta:
só há `stable` (1.94.1) instalado, e nenhum passo da CI mediria a diferença.

**`fmt::Debug` de `NoMbc` é manual.** `derive` despejaria 32 KiB no terminal de
quem for depurar o `Bus`; a implementação imprime `NoMbc { rom_len: 32768 }`.

**O teste redefine `MIN_ROM_LEN` em vez de importar `gb_core::cart::MIN_ROM_LEN`.**
Deliberado, e a mesma prática de `cart_header.rs`: oráculo que vem da
implementação não testa a implementação.
