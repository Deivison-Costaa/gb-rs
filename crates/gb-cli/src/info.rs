//! `gb-cli info <rom>` — ROADMAP 0.3b.
//!
//! A casca de I/O do parser da 0005. O `gb-core` não pode abrir arquivo nem
//! imprimir (R3), então tudo que sobra do subcomando mora aqui: ler os bytes,
//! chamar [`CartridgeHeader::parse`], formatar e escolher o código de saída.
//!
//! Spec dos campos: `docs/reference/08-cartridges-mbc.md` § The Cartridge
//! Header. Nada de comportamento de hardware é decidido neste arquivo — ele
//! só dá nome ao que o `gb-core` já interpretou.
//!
//! **Código desconhecido não vira erro.** Tipo de cartucho fora da tabela,
//! tamanho de RAM sem tamanho atestado, checksum que não bate: tudo isso é
//! relatado e o processo sai `0`. Um `info` que se recusa a falar sobre a ROM
//! esquisita é justamente o que não serve para diagnosticar ROM esquisita — e
//! o veredito sobre o que fazer com a informação é de quem lê.

use std::path::Path;
use std::process::ExitCode;

use gb_core::cart::{CartridgeHeader, CartridgeType, HeaderChecksum, RamSize, RomSize};

use crate::exit;

/// Largura da coluna dos rótulos, incluindo os dois-pontos e o espaço que
/// separa do valor. `checksum:` é o mais longo, com 9 caracteres.
const LABEL_WIDTH: usize = 10;

/// O que a spec diz quando o byte não está na tabela dela.
///
/// Existe uma constante para isto porque a alternativa — inventar um número
/// plausível — é o erro #1 e #2 da iteração 0005, e o texto impresso aqui é
/// lido como medição por quem não vai conferir o Pan Docs.
const UNKNOWN_SIZE: &str = "tamanho desconhecido";

/// Lê a ROM, imprime o relatório do cabeçalho em `stdout` e devolve o código
/// de saída.
///
/// Mensagem de erro vai para `stderr` e, quando há erro, `stdout` fica vazio:
/// relatório pela metade de ROM inválida é pior do que relatório nenhum, porque
/// tem cara de dado bom.
pub fn report(path: &Path) -> ExitCode {
    // A ROM inteira, e não só os 336 bytes do cabeçalho: `tamanho` é um dos
    // campos do relatório, e o maior cartucho possível tem 8 MiB.
    let rom = match std::fs::read(path) {
        Ok(rom) => rom,
        Err(error) => {
            eprintln!("não consegui ler {}: {error}", path.display());
            return ExitCode::from(exit::NO_INPUT);
        }
    };

    let header = match CartridgeHeader::parse(&rom) {
        Ok(header) => header,
        Err(error) => {
            eprintln!("{}: {error}", path.display());
            return ExitCode::from(exit::DATA_ERROR);
        }
    };

    print!("{}", render(path, rom.len(), &header));
    ExitCode::SUCCESS
}

/// O relatório, como texto. Puro, para que o formato possa ser conferido sem
/// abrir arquivo nenhum.
fn render(path: &Path, rom_len: usize, header: &CartridgeHeader) -> String {
    let title = if header.title().is_empty() {
        // Campo em branco é indistinguível de campo que faltou.
        "(vazio)".to_string()
    } else {
        header.title().to_string()
    };

    [
        line("arquivo", &path.display().to_string()),
        line("tamanho", &format!("{rom_len} bytes")),
        line("título", &title),
        line("tipo", &cartridge_type(header.cartridge_type())),
        line("ROM", &rom_size(header.rom_size())),
        line("RAM", &ram_size(header.ram_size())),
        line("checksum", &checksum(header.checksum())),
    ]
    .concat()
}

/// Uma linha `rótulo: valor`, com a coluna do valor alinhada.
fn line(label: &str, value: &str) -> String {
    let label = format!("{label}:");
    format!("{label:<LABEL_WIDTH$}{value}\n")
}

/// `$0147`. O byte cru vem sempre; o nome, só quando está na tabela.
fn cartridge_type(kind: CartridgeType) -> String {
    let name = kind.name().unwrap_or("desconhecido");
    format!("${:02X} {name}", kind.code())
}

/// `$0148`. O número de bancos acompanha o tamanho porque é ele que o mapeador
/// vai usar a partir do 0.4 — e é onde um cabeçalho mentiroso aparece primeiro.
fn rom_size(size: RomSize) -> String {
    let described = match (size.bytes(), size.banks()) {
        (Some(bytes), Some(banks)) => format!("{} ({banks} bancos)", human(bytes)),
        _ => UNKNOWN_SIZE.to_string(),
    };
    format!("${:02X} {described}", size.code())
}

/// `$0149`. Sem bancos: o cabeçalho não diz quantos, e a tabela do Pan Docs os
/// lista só como comentário.
fn ram_size(size: RamSize) -> String {
    let described = size.bytes().map_or_else(|| UNKNOWN_SIZE.to_string(), human);
    format!("${:02X} {described}", size.code())
}

/// `$014D`, dos dois lados. O boot ROM **trava a máquina** quando eles não
/// batem, então o veredito vem junto — mas os dois bytes também, porque
/// "não confere" sem os números não diagnostica nada.
fn checksum(checksum: HeaderChecksum) -> String {
    let verdict = if checksum.is_valid() {
        "confere"
    } else {
        "NÃO CONFERE"
    };
    format!(
        "${:02X} (calculado ${:02X}) — {verdict}",
        checksum.stored(),
        checksum.computed()
    )
}

/// Tamanho em KiB ou MiB, e só quando a divisão é exata.
///
/// Todos os valores das duas tabelas são potências de 2 a partir de 8 KiB, ou
/// zero — o ramo dos bytes crus existe para o `$00` da RAM e para não mentir
/// caso alguma tabela cresça com um valor quebrado.
fn human(bytes: u32) -> String {
    const KIB: u32 = 1024;
    const MIB: u32 = 1024 * KIB;

    // `%` e não `is_multiple_of`: o método só existe a partir do Rust 1.87 e o
    // workspace declara `rust-version = "1.85"`. A CI compila com o stable do
    // dia e não reprovaria isso — a MSRV é promessa nossa, não do compilador.
    if bytes >= MIB && bytes % MIB == 0 {
        format!("{} MiB", bytes / MIB)
    } else if bytes >= KIB && bytes % KIB == 0 {
        format!("{} KiB", bytes / KIB)
    } else {
        format!("{bytes} B")
    }
}
