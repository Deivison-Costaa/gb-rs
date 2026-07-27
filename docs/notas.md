# Notas de iteração

> A numeração é citada nos docs de iteração e em comentários do código.
> **Nunca renumere.** Nota nova entra com o próximo número livre.


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

46. **O operando de teste tem de distinguir os casos que o teste alega
    cobrir — e isso é achado pela bateria de mutação, não pela leitura do
    teste.** A 0025 (`alu a,imm8`) escreveu um teste genérico, um `for` sobre
    as oito operações, com um único operando fixo (`0x0C`) contra
    `SEED_A = 0x21`. Os dois não compartilham bit nenhum, então `A & operando`
    e `A ^ operando` colapsam nos mesmos zero bits que sobram: `XOR` e `OR`
    deram o mesmo resultado (`0x2D`) por coincidência de dado, não porque o
    código estivesse certo. Mutar `XOR_A_IMM8` para despachar `AluOp::Or` (ou
    o inverso) não quebrava teste nenhum — o teste lia como se cobrisse as
    três operações bit a bit, mas só cobria duas.

    O mesmo padrão apareceu do outro lado: o teste de `ADC`/`SBC` consumindo o
    carry de entrada existia, mas faltava o controle inverso (`ADD`/`SUB`
    **ignorando** o carry) — sem ele, uma ALU que sempre soma/subtrai o carry
    de entrada passa no teste que existe.

    A nota 25 já dizia isto sobre o controle negativo (`decoded_elsewhere`);
    esta é a mesma doutrina aplicada ao **dado de entrada** de um teste
    positivo. Escolher o operando (ou o par de operandos) que separa os casos
    é parte de escrever o teste — verificado, nas duas vezes, só depois que um
    mutante sobreviveu à bateria.


47. **Redundância acidental (12 cópias do mesmo controle) escondia um buraco
    que a bateria de mutação só achou depois de consolidar num lugar só.**
    Antes da 0026, `decoded_elsewhere` existia 12 vezes, uma cópia por
    arquivo de sub-item. Nenhum `sweep` verificava a alegação positiva da
    função (`decoded_elsewhere(opcode) == true` ⇒ o opcode está de fato
    decodificado) — só a negativa (`== false` ⇒ tem de estar `Undecoded`).
    Se uma cópia mentisse dizendo `true` para um opcode que ninguém decodifica
    ainda, as outras onze cópias — intocadas, cada uma com seu próprio
    `sweep` — continuariam corretas, e o erro passaria batido só naquele
    arquivo. Doze testemunhas independentes, mesmo sem ninguém desenhar isso
    como controle cruzado.

    A 0026 consolidou as 12 cópias num único `tests/support/mod.rs`. A
    bateria de mutação obrigatória (passo 6) mutou o helper acrescentando um
    opcode que nenhum item decodifica (`0x04`, ainda não implementado — é o
    1.6e) e **nenhum dos 289 testes falhou**. As 12 testemunhas viraram uma
    só, e a lacuna que a redundância escondia ficou visível: todas as 12
    verificações passaram a confiar na mesma alegação não verificada, ao
    mesmo tempo.

    A correção não foi no helper — foi nos 12 consumidores. Trocar
    `else if !decoded_elsewhere(opcode) { assert Undecoded }` (que pula em
    silêncio quando a função diz `true`) por três ramos —
    `if in_block {…} else if ILLEGAL {…} else if decoded_elsewhere(opcode)
    { assert None } else { assert Undecoded }` — fecha o buraco: agora toda
    alegação de `decoded_elsewhere`, positiva ou negativa, tem uma asserção
    em cima.

    A nota 29 já batizou "controle negativo que sobrevive por redundância"
    de não-controle, num contexto diferente (esqueleto vs. bateria de
    mutação). Esta é a mesma doutrina aplicada ao próprio ato de eliminar a
    redundância: consolidar duplicatas é certo — o ROADMAP 0.6 pedia
    exatamente isso — mas a redundância que se está eliminando pode estar
    escondendo, sem querer, uma proteção que ninguém desenhou de propósito.
    Rodar a bateria de mutação **depois** de consolidar é o que torna essa
    perda visível antes do merge, não depois.


48. **Uma flag que fica intocada não aparece lendo o `diff` — só testando os
    dois lados do estouro.** `INC`/`DEC r8` (1.6e) é a primeira operação da
    ALU cuja coluna `C` é `-`: nenhuma linha em `alu::increment`/`decrement`
    menciona `Flag::C`. Essa ausência não deixa rastro no código — um
    `set_flag(Flag::C, ...)` esquecido e um `set_flag(Flag::C, ...)` que
    nunca deveria existir são, ao ler o arquivo, exatamente a mesma coisa:
    nada escrito.

    A bateria de mutação tornou isso visível na direção oposta: acrescentar
    `registers.set_flag(Flag::C, result < value)` (um carry aritmético
    "razoável" para quem está pensando em `ADD`, não em `INC`) só quebra
    teste se o caso escolhido **estourar o byte inteiro** (`0xFF` → `0x00`)
    com `C` começando limpo — um carry calculado ligaria aqui, o correto é
    ficar limpo. O caso espelhado (sem estouro nenhum, `C` começando ligado)
    pega o mutante oposto, o que zera `C` incondicionalmente. Precisa dos
    dois: testar só um lado deixa passar um mutante que acerta por acaso
    nesse lado e erra no outro — a mesma forma da nota 46, agora sobre uma
    flag que devia ficar parada em vez de um operando que devia distinguir
    operações.


49. **Um valor de "F sujo" único pode coincidir, por acidente, com o que uma
    mutação plausível produziria — e aí o controle não controla nada.**
    `INC`/`DEC r16` (1.7a) chegou como código órfão de duas sessões mortas sem
    relato; o teste que confere "nenhuma flag muda" sujava `F` com um valor
    só, `DIRTY_F = 0b1010_0101`, antes de rodar o opcode. Esse byte já tem o
    bit `Z` (bit 7) ligado. A bateria de mutação obrigatória do passo 6
    acrescentou `set_flag(Flag::Z, true)` quando o resultado do `INC`/`DEC`
    dava `0x0000` — o engano mais plausível que existe, porque é exatamente a
    regra de `Z` que `INC r8` (1.6e) usa, só que ela não vale para o par de
    16 bits. O mutante **não** derrubou nenhum teste: forçar um bit que já
    estava em `1` não muda o byte observado, e o `assert_eq!` comparando `F`
    inteiro contra `DIRTY_F` não tem como perceber a diferença.

    A mesma classe de não-controle da nota 29/47, agora num valor de teste em
    vez de numa lista de opcodes: um único "sujo" cobre só a metade das
    mutações possíveis por bit — a metade que força o bit para o lado que ele
    já não estava. A correção foi trocar o valor único por dois extremos,
    `ALL_FLAGS_SET = 0b1111_0000` e `ALL_FLAGS_CLEAR = 0b0000_1111`, e rodar
    o teste inteiro (as quatro flags, INC e DEC, os quatro pares) sob as duas
    polaridades. Qualquer mutação que force um bit de flag para `0` ou para
    `1`, em qualquer direção, agora tem uma das duas rodadas partindo do lado
     oposto — e não tem como se esconder.

51. **ROM blargg imprime o opcode em hex, não o índice do teste.** Quando uma
    ROM de teste encontra um checksum que não bate, ela imprime o byte do opcode
    via `print_a` (formato hex), não o número sequencial do teste. A 0048 perdeu
    metade da iteração revisando BIT 1,(HL) porque o STATUS anterior descrevia
    o erro "27" como índice (possivelmente BIT 1,(HL) ou LD (HL+),A). Na verdade,
    "27" é o opcode de DAA ($27), e tanto a ROM 11 quanto a ROM 01 falhavam pelo
    mesmo motivo — DAA estava errado desde a 0044.

    Lição: antes de interpretar um número de erro do blargg, abra o fonte da ROM
    e veja o que `print_a` imprime — é o byte em `(instr)`, não um contador.

52. **O mapeamento do clock select do timer não é contíguo nem ordenado por
    frequência.** A tabela do Pan Docs mapeia `00 → bit 9 (4096 Hz), 01 → bit 3
    (262144 Hz), 10 → bit 5 (65536 Hz), 11 → bit 7 (16384 Hz)`. A primeira
    intuição (bits 0, 1, 2, 3 em ordem crescente) erra porque o `sys_counter`
    avança 4 por M-cycle, então o período visível é `2^(bit+1)/4` M-cycles. A
    0050 confirmou cada entrada casando `2^(bit+1)/4` com a coluna
    `Increment every` da spec. O default de `const fn clock_bit` retorna 9, mas
    o braço `_` só é atingido para `select ≥ 4` — impossível dado que
    `clock_bit` é chamada com `tac & 0x03`.

53. **`check_interrupt` faz read-modify-write de IF e isso é perigoso com
    periféricos.** A 0051 implementou o dispatch de interrupções com
    `check_interrupt` lendo IF via `bus.read(IF_ADDR)`, limpando o bit do
    vetor escolhido (`if_reg & !(1 << bit)`), e escrevendo de volta via
    `bus.write(IF_ADDR, ...)`. Entre a leitura e a escrita, nenhum
    periférico roda (o timer, único existente, roda em `bus.tick_timer()`
    ANTES de `check_interrupt`). Com PPU (M3) rodando no mesmo `step()`, o
    RMW se torna uma janela de perda: se a PPU setar um bit de IF entre a
    leitura do CPU e a escrita de volta, o bit é sobrescrito com 0. A solução
    é garantir que a PPU escreva em IF antes do `check_interrupt` (como o
    timer faz), ou que o dispatch use uma máscara que não releia IF (e.g.,
    escrever `bus.write(IF_ADDR, if_reg & !(1 << bit))` com `if_reg` lido
    uma única vez — que é o que já está feito, mas a invariante correta é:
    "toda escrita de periférico em IF acontece antes de check_interrupt no
    mesmo M-cycle"). Ver [doc da 0051](docs/iterations/0051-interrupts.md)
    § Decisões de arquitetura, item 3.

54. **O RTC do MBC3 é acionado por oscilador externo de 32.768 kHz que o
    emulador não modela.** O timer do RTC (segundos, minutos, horas, dias) não
    avança sozinho dentro do emulador — não há cristal de quartzo simulado. O
    emulador precisa decidir se usa o relógio do host (via `std::time`) ou se
    oferece uma interface manual para avançar o RTC. O latch (`$6000`: `$00` →
    `$01`) congela uma cópia dos registradores do RTC para leitura consistente;
    escrever `$00` de volta destrava. MBC3 com timer (`$0F`, `$10`) sempre tem
    bateria externa para o RTC; os tipos sem timer (`$11`–`$13`) não têm RTC e
    funcionam como MBC1 simplificado com banking até 2MB. Ver
    [doc da 0068](docs/iterations/0068-mbc2.md) § Notas (handoff para 5.2).

55. **O latch do RTC durante escrita tem comportamento de borda ambíguo.**
    A spec do Pan Docs § MBC3 descreve o latch (`$00` → `$01`: congela cópia;
    `$01` → `$00`: destrava), mas não especifica se escritas nos registradores
    do RTC (`$08`–`$0C`) enquanto o latch está ativo (`$01`) são ignoradas ou
    passam direto para o contador físico. Testar com ROMs MBC3 reais (Pokémon
    Gold/Silver) é o caminho; na ausência de teste, a escolha conservadora é
    permitir escrita mesmo sob latch (o latch só afeta leitura, não escrita —
    o contador continua avançando e aceitando ajustes). Ver
    [doc da 0068](docs/iterations/0068-mbc2.md) § Notas.

