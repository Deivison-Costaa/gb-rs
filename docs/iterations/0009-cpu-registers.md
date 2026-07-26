# Iteração 0009 — registradores e flags do SM83

- **Data:** 2026-07-25
- **Item do roadmap:** 1.1
- **PR:** #11
- **Duração:** ~35min
- **Custo reportado:** — <!-- sessão interativa, sem --output-format json (STATUS.md nota 10) -->
- **Turnos:** 1

## Objetivo

O banco de registradores do SM83: `AF`/`BC`/`DE`/`HL`/`SP`/`PC`, os pares de
8/16 bits e os flags `Z`/`N`/`H`/`C`. Primeiro código de hardware do M1.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | § CPU registers and flags (lido inteiro, R1) | `docs/reference/02-cpu.md` |
| Pan Docs | § CPU Comparison with Z80 | `docs/reference/02-cpu.md` |
| Pan Docs | § CPU Instruction Set (lista `r8`, `r16stk`) | `docs/reference/02-cpu.md` |
| Pan Docs | § Console state after boot ROM hand-off (para **não** implementar) | `docs/reference/01-memory-map.md` |
| gbops | `F1 POP AF`, `F5 PUSH AF` | `docs/reference/03-opcodes.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | Que `set_af` mascara o nibble baixo de `F` (`self.f = value as u8 & 0xF0`) — eu ia escrever isso como fato, sem hesitar | **Nada.** A tabela do § Flags Register para no bit 4 e não diz uma palavra sobre os bits 3–0. `POP AF` não aparece em **nenhum** dos 75 arquivos do Pan Docs no commit fixado | O `STATUS.md` previu este erro antes de a iteração começar e mandou conferir; a conferência foi um `grep` no repositório inteiro do Pan Docs, não só nas seções importadas |
| 2 | flags | Instinto de Z80: bit 7 = `S` (sinal), e algum flag de paridade no nibble baixo | § CPU Comparison with Z80: *"The sign and parity/overflow flags have been removed"*. Bit 7 é `Z` | Leitura da seção antes de escrever. O teste `each_flag_sits_on_the_bit_the_spec_assigns_it` fixa as quatro posições |
| 3 | API-Rust | Que os testes podiam montar fixtures com `Registers::default()` seguido de atribuição de campo | Não é a spec, é o clippy: `field_reassign_with_default` é erro sob `-D warnings` | `cargo clippy --all-targets` — 5 ocorrências, todas em teste. Custou um helper `with_f` e dois literais de struct |

**Sobre o #1 — é o achado da iteração, e o mecanismo importa mais que o fato.**
A regra R1 supôs que o modo de falha fosse "o agente não leu a spec". Aqui a
spec foi lida e o modo de falha teria sido outro: a spec é **omissa**, e
omissão não parece omissão quando já se tem uma convicção para preencher o
buraco. Ler a seção não teria bastado — `02-cpu.md` não contradiz a máscara,
ele simplesmente não fala dela, e "não contradisse" é fácil de ler como
"confirmou". O que resolveu foi transformar a convicção numa pergunta
falsificável (*"em que arquivo, em que linha?"*) e ir procurar a linha.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas (121 ROMs) | 0/121 | 0/121 |
| testes do workspace | 85 | 98 |

Sem mudança no scoreboard, como esperado: registrador não roda ROM. A primeira
linha que pode mexer é o 1.13.

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum — `scripts/review.sh` continua sem `REVIEWER_CMD`
  configurado (`STATUS.md`, nota 5). Campo vazio por ausência de ferramenta,
  não por esquecimento.

## Decisões de arquitetura

1. **`F` carrega os 8 bits; o 1.1 não mascara nada.** Decisão pré-registrada no
   `STATUS.md` antes de a iteração começar, e cumprida. A previsão falsificável
   que fica: se a máscara for mesmo necessária, quem cobra é a blargg
   `cpu_instrs/01-special` no 1.13. Nesse dia haverá evidência para trazê-la —
   e a fonte entra em `docs/reference/` junto, pela R1. Enquanto isso, o teste
   `f_keeps_the_bits_the_spec_does_not_describe` impede a máscara de entrar por
   hábito, sem alguém decidir que ela entra.

2. **Campos de 8 bits públicos; pares de 16 bits são métodos.** Um banco de
   registradores não tem invariante a proteger — `regs.a = value` é o que a
   instrução faz, e um acessor por registrador só engrossaria o decodificador
   do 1.4. Os pares são métodos porque aí há cálculo.

3. **`Default` é tudo zero e não é o estado pós-boot.** `A=$01`, `SP=$FFFE`,
   `PC=$0100` são do 1.2, e estão em `01-memory-map.md`. Zero aqui é ausência
   de decisão, não decisão errada — e há teste guardando o limite entre os dois
   itens.

## Notas

**A bateria de mutação: 11/11 pegos, 2/2 controles verdes.** O mutante que
importa é o #9, que **aplica a máscara do folclore** — ele é pego. Ou seja: a
decisão de não mascarar está fixada por teste, não por comentário. Se a máscara
voltar, volta vermelho, e alguém tem de justificar. Os dois controles negativos
(`!= 0` → `> 0`; reordenar os braços do `match`) ficaram verdes, o que é a
evidência de que a suíte discrimina em vez de quebrar com qualquer mudança
(nota 8 do `STATUS.md`).

O ressalvo da 0007 continua valendo e não deve ser esquecido: os mutantes foram
escritos por quem escreveu os testes, na mesma sessão. 11/11 autoriza dizer que
os onze modos de falha imaginados doem. Nada além disso.

**Nota 8(a) reincidiu, como previsto.** O primeiro vermelho foi `error[E0432]`
— módulo inexistente —, que não mede asserção nenhuma. Com o esqueleto
descartável (tudo inerte) o RED de verdade foi **9 falhas e 4 passes por
vacuidade**: `sp_and_pc_are_sixteen_bit_and_have_no_halves` (campos simples, sem
lógica minha), `the_default_is_zeroed_and_is_not_the_post_boot_state` e
`the_flags_ignore_the_bits_below_bit_four` (afirmam ausência) e
`setting_a_flag_twice_is_the_same_as_setting_it_once` (no-op é idempotente).
Os quatro são guarda de regressão futura, não medição do código de hoje — vale
saber a diferença ao contar "13 testes".

**A nota 13 (MSRV) ganhou um ponto de dado, e continua aberta.** Esta iteração
introduziu `const fn` com `&mut self` (estável desde 1.83) e atribuição
desestruturante em contexto const — exatamente o tipo de coisa que a 0006 quase
deixou passar. Desta vez foi conferido: `rustup toolchain install 1.85` e
`cargo +1.85 test --all` → **98/98 passando** em `rustc 1.85.1`. Mas isso foi
uma conferência **manual, feita uma vez, por quem sabia procurar** — a CI
continua usando `stable` e a promessa do `Cargo.toml` continua sem guarda
automática. O próximo `const fn` entra sem ninguém olhar.
