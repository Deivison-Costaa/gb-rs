//! Os códigos de saída do `gb-cli`, num lugar só.
//!
//! Isto é contrato, não detalhe: `scripts/scoreboard.sh` lê o código de saída
//! deste binário e o transcreve como veredito para o `scoreboard.csv`, que é a
//! série temporal da apresentação (ROADMAP 8.2). Um código escolhido por
//! conveniência vira um `pass` ou um `fail` falso no dado.
//!
//! Reservado ao veredito da ROM, e **proibido** para qualquer outra coisa:
//!
//! | Código | Significado |
//! |---|---|
//! | `0` | a ROM reportou sucesso |
//! | `1` | a ROM reportou falha |
//! | `124` | o `timeout(1)` matou a execução |
//!
//! O resto é erro do emulador, e o `scoreboard.sh` agrupa tudo como `crash`.
//! Para os erros do próprio `gb-cli` valem os códigos do `sysexits.h` do BSD
//! (`/usr/include/sysexits.h`), que começam em `64` justamente para não
//! colidirem com códigos de aplicação. A convenção é emprestada, não inventada:
//! quem já leu um `EX_USAGE` em outro programa lê este igual.
//!
//! `0` é `ExitCode::SUCCESS` e não aparece aqui — o caminho feliz não precisa
//! de constante para ser lido.

/// Erro do emulador: o subcomando existe no ROADMAP mas ainda não foi escrito.
///
/// Hoje é só o `run` (ROADMAP 1.12). Cai no balde `crash` do scoreboard, que é
/// exatamente a verdade: nenhuma ROM roda ainda.
pub const NOT_IMPLEMENTED: u8 = 2;

/// `EX_USAGE` — a linha de comando está errada.
///
/// Subcomando desconhecido, ROM faltando, argumento sobrando. É engano de quem
/// chamou, e nenhuma ROM foi sequer aberta.
pub const USAGE: u8 = 64;

/// `EX_DATAERR` — o arquivo foi lido e o conteúdo não serve.
///
/// Hoje só acontece com ROM que acaba antes de `$014F`. Separado do
/// [`NO_INPUT`] de propósito: "errei o caminho" e "a ROM está corrompida" são
/// diagnósticos diferentes e quem chamou precisa distinguir os dois.
pub const DATA_ERROR: u8 = 65;

/// `EX_NOINPUT` — não deu para ler o arquivo.
///
/// Inexistente, sem permissão, é um diretório. Qualquer falha do sistema de
/// arquivos cai aqui; a mensagem em `stderr` é que diz qual foi.
pub const NO_INPUT: u8 = 66;
