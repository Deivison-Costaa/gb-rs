# Iteração 0026 — o controle negativo de decodificação vira ponto único

- **Data:** 2026-07-26
- **Item do roadmap:** 0.6
- **PR:** #32
- **Duração:** n/d — sessão interrompida no meio e retomada por outra sessão
- **Custo reportado:** n/d — sessão interativa de Claude Code
- **Turnos:** n/d (primeira metade) + 1 (retomada)

## Objetivo

Extrair `decoded_elsewhere`/`previously_decoded` — duplicada em 12 arquivos de
teste, uma cópia por sub-item de opcode — para um único helper compartilhado
(`tests/support/mod.rs`), de modo que opcode novo se declare num lugar só.

## Contexto: sessão interrompida

Esta iteração começou numa sessão anterior que morreu sem escrever este
documento. O que sobrou em disco, não versionado: `tests/support/mod.rs`
inexistente, e um teste RED (`decoded_elsewhere_single_source.rs`) já escrito.
Os 12 arquivos originais estavam intactos — a consolidação em si não havia
começado. **Não há relato da primeira metade**, então a tabela abaixo cobre só
o que a sessão de retomada encontrou e fez; qualquer atrito da primeira metade
está perdido (nota 42 do `STATUS.md` já previa essa forma de buraco).

## Spec consultada

Não aplicável — R1 é sobre comportamento de hardware, e este item é
refatoração de teste. O "passo 4" do protocolo se resolveu lendo o código que
ia mudar (os 12 arquivos e o teste RED), não `docs/reference/`.

## Erros de primeira tentativa

> Este item não tem comportamento de hardware, então a categoria de erro é
> outra: cobertura que existia por acidente, e um teste RED cuja asserção
> não provava o que alegava provar.

| # | Categoria | O que eu assumi | O que era verdade | Como foi pego |
|---|---|---|---|---|
| 1 | teste-do-teste | O teste RED encontrado em disco (herdado da sessão morta) estava correto: contar arquivos que definem `decoded_elsewhere`/`previously_decoded` e exigir exatamente 1. | A busca usa `text.contains("fn decoded_elsewhere(")` — e o próprio arquivo do teste contém esse texto literal (é o argumento do `.contains`). Ele sempre se auto-contava. Rodado como estava, ele já falhava com 12 (11 cópias reais + ele mesmo), e depois da consolidação continuaria falhando com 2 (o helper + ele mesmo) em vez de 1. | Rodar o RED antes de confiar nele (regra explícita da retomada) e ler a lista de arquivos no `assert_eq!` — `decoded_elsewhere_single_source.rs` aparecia na própria lista. |
| 2 | cobertura | A busca por `"fn decoded_elsewhere("` / `"fn previously_decoded("` era suficiente para achar toda duplicação. | `cpu_ld_r8_u8.rs` (o item mais antigo, 1.4b) tem a mesma lógica, mas como `let previously_decoded = <opcode == 0x00 || ...>;` local dentro do teste — nunca um `fn` de nível superior. A busca original não o via. | Grep manual por `previously_decoded` fora do padrão `fn` (pedido explícito da retomada) achou o `let` em `cpu_ld_r8_u8.rs`; sem isso, a consolidação teria deixado 1 dos 12 lugares de fora e o teste teria passado do mesmo jeito. |
| 3 | cobertura | A bateria de mutação obrigatória do passo 6 ia pegar qualquer mutação plausível no helper consolidado. | Mutar "acrescentar um opcode que não devia estar" (marcar `0x04`, ainda não decodificado por ninguém, como `decoded_elsewhere`) **não derrubava teste nenhum** — os 12 `sweep`s só tinham `else if !decoded_elsewhere(opcode) { assert Undecoded }`; quando `decoded_elsewhere` mentia dizendo `true`, o `else if` era pulado em silêncio, sem checar a alegação positiva. Antes da consolidação, 12 cópias independentes escondiam esse buraco por redundância (a nota 29 do `STATUS.md` já batizou esse padrão de "não é controle"); depois de consolidar, um erro no único lugar cega os 12 testes ao mesmo tempo. | Rodar a mutação prevista pela própria instrução da retomada ("acrescentar um que não devia estar") e ver `0` testes falhando — sinal de que a consolidação tinha apagado uma proteção que a redundância dava de graça. Corrigido trocando `else if !decoded_elsewhere(opcode) { assert Undecoded }` por um `if/else if/else` de três ramos nos 12 arquivos, com uma asserção positiva (`assert None`) no ramo `decoded_elsewhere(opcode)`. |

Nenhum erro de hardware — não havia o que errar, já que `alu::apply`,
`AluOp` e o resto do `gb-core` não mudaram uma linha.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |
| testes do workspace | 288 | 289 |

Bateria de mutação: **5/5 mutações pegas, 2/2 controles verdes.**

- Tirar `opcode == 0xC3` da lista → 12 sweeps falham.
- Tirar `(0xA0..=0xB7)` (AND/XOR/OR) da lista → 11 sweeps falham (o próprio
  item dono do range não passa por essa asserção).
- Inverter a máscara de `LD r16,u16` (`==` → `!=`) → 12 sweeps falham.
- Acrescentar `0x04` (opcode ainda não decodificado por ninguém, 1.6e) →
  **0 falhas antes da correção do achado #3, 12 depois.**
- Controle 1 (reordenar duas cláusulas, sem mudança semântica) → 0 falhas.
- Controle 2 (duplicar `opcode == 0x00`, sem mudança semântica) → 0 falhas.

## Revisão cruzada (segundo modelo)

Não houve. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(`STATUS.md`, nota 5).

## Decisões de arquitetura

`tests/support/mod.rs` (subdiretório, não `tests/support.rs` no primeiro
nível) é o único lugar que define `decoded_elsewhere`. Cada um dos 12
consumidores ganhou `mod support; use support::decoded_elsewhere;` e perdeu a
cópia local — inclusive `cpu_ld_r8_u8.rs`, cujo `let previously_decoded = ...`
virou uma chamada direta (`decoded_elsewhere(opcode)`), unificando o nome nos
12 lugares (a maioria já usava `decoded_elsewhere`; `previously_decoded`
desapareceu).

O corpo de `decoded_elsewhere` é a união de todas as cláusulas que existiam
espalhadas pelas 12 cópias — inclusive a cláusula referente ao bloco de cada
arquivo específico (por exemplo, o helper inclui `(0x40..=0x7F)` mesmo sendo
usado por `cpu_ld_r8_block.rs`, dono desse bloco). Isso é seguro porque cada
`sweep` testa `in_block` **antes** de `decoded_elsewhere` num `if/else if`: o
próprio bloco do arquivo nunca chega a cair no ramo `decoded_elsewhere`, então
incluí-lo ali não muda nenhum resultado — e evita 12 versões private do
mesmo helper.

O teste RED (`decoded_elsewhere_single_source.rs`) ganhou duas correções além
da consolidação em si: exclui a si mesmo da varredura por nome de arquivo
(`file!()`), e a asserção final compara o caminho exato
(`tests/support/mod.rs`) em vez de só contar `1` — um `1` que apontasse para
um arquivo errado teria passado no teste antigo.

## Notas

### Redundância acidental não é proteção, mas às vezes é a única que existe

O achado #3 é o mais caro desta iteração: a bateria de mutação existe
justamente para expor esse tipo de coisa, e expôs. Antes da consolidação, se
alguém tivesse (por engano) marcado um opcode não decodificado como
`decoded_elsewhere` numa única cópia, as outras onze — intocadas — ainda
pegariam esse opcode em seus próprios `sweep`s. Não era um controle
desenhado; era sorte estrutural de ter 12 cópias. Consolidar removeu essa
sorte sem substituí-la por nada, até a correção do `if/else if/else` de três
ramos. A nota 29 do `STATUS.md` ("controle negativo que sobrevive por
redundância não é controle") previu exatamente essa classe de problema, só
que para outro contexto — aqui ela se aplicou ao próprio ato de eliminar a
redundância.

### O teste que guarda o item pode estar quebrado por dentro

O achado #1 (auto-contagem) é um lembrete específico da instrução de
retomada: "não confie que o teste em disco está certo". Um teste RED escrito
numa sessão anterior, nunca visto passar em GREEN, carrega o mesmo risco de
qualquer código não revisado — e neste caso o risco se materializou: mesmo
depois da consolidação perfeita, o teste original teria ficado vermelho para
sempre (contando a si mesmo como um segundo "definidor").
