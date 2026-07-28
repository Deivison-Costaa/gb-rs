//! O passo 9 do protocolo manda fechar a caixa-pai quando o último sub-item
//! fecha, e usar dois espaços de indentação. Está escrito lá desde que o 1.4
//! ficou dez iterações aberto com tudo pronto. Falhou de novo em 27/07 no 6.8,
//! com 6.8c/d/e escritos com três espaços — a armadilha que o próprio passo 9
//! descreve. Regra lembrada quatro vezes e esquecida quatro é regra que precisa
//! de teste, como a R7 precisou (ver docs/orquestracao.md).

use std::path::{Path, PathBuf};

fn roadmap() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("gb-core mora dois níveis abaixo da raiz do workspace")
        .join("ROADMAP.md")
}

struct Caixa {
    linha: usize,
    indentacao: usize,
    marcada: bool,
    texto: String,
}

fn caixas(fonte: &str) -> Vec<Caixa> {
    fonte
        .lines()
        .enumerate()
        .filter_map(|(i, linha)| {
            let sem_espaco = linha.trim_start();
            let marcada = match sem_espaco.get(..6) {
                Some("- [x] ") => true,
                Some("- [ ] ") => false,
                _ => return None,
            };
            Some(Caixa {
                linha: i + 1,
                indentacao: linha.len() - sem_espaco.len(),
                marcada,
                texto: sem_espaco[6..].chars().take(60).collect(),
            })
        })
        .collect()
}

#[test]
fn sub_itens_usam_exatamente_dois_espacos_de_indentacao() {
    let fonte = std::fs::read_to_string(roadmap()).expect("ROADMAP.md é UTF-8");

    let torto: Vec<String> = caixas(&fonte)
        .iter()
        .filter(|c| c.indentacao % 2 != 0 || c.indentacao > 4)
        .map(|c| {
            format!(
                "  linha {}: {} espaços — {}",
                c.linha, c.indentacao, c.texto
            )
        })
        .collect();

    assert!(
        torto.is_empty(),
        "indentação fora do padrão de dois espaços esconde sub-item de varredura \
         (passo 9 do SKILL.md):\n{}",
        torto.join("\n")
    );
}

#[test]
fn caixa_pai_com_todos_os_filhos_fechados_esta_fechada() {
    let fonte = std::fs::read_to_string(roadmap()).expect("ROADMAP.md é UTF-8");
    let todas = caixas(&fonte);

    let mut orfas = Vec::new();
    for (i, pai) in todas.iter().enumerate() {
        if pai.marcada {
            continue;
        }
        let filhos: Vec<&Caixa> = todas[i + 1..]
            .iter()
            .take_while(|c| c.indentacao > pai.indentacao)
            .filter(|c| c.indentacao == pai.indentacao + 2)
            .collect();

        if !filhos.is_empty() && filhos.iter().all(|f| f.marcada) {
            orfas.push(format!(
                "  linha {}: {} — {} sub-itens, todos fechados",
                pai.linha,
                pai.texto,
                filhos.len()
            ));
        }
    }

    assert!(
        orfas.is_empty(),
        "caixa-pai aberta com todo o trabalho feito faz o passo 1 apontar para \
         trabalho já concluído:\n{}",
        orfas.join("\n")
    );
}

fn status() -> PathBuf {
    roadmap().with_file_name("STATUS.md")
}

/// O primeiro item **acionável**: caixa aberta sem sub-item aberto embaixo dela.
/// Caixa-pai com filho pendente não é tarefa — a tarefa é o filho.
fn primeira_acionavel(todas: &[Caixa]) -> Option<&Caixa> {
    todas.iter().enumerate().find_map(|(i, c)| {
        if c.marcada || c.texto.contains("BLOQUEADO") {
            return None;
        }
        let tem_filho_aberto = todas[i + 1..]
            .iter()
            .take_while(|f| f.indentacao > c.indentacao)
            .any(|f| !f.marcada);
        (!tem_filho_aberto).then_some(c)
    })
}

fn alvo_do_status(texto: &str) -> Option<String> {
    let linha = texto.lines().find(|l| l.contains("Próxima tarefa"))?;
    let depois = &linha[linha.find("ROADMAP")? + "ROADMAP".len()..];
    let inicio = depois.find("**")? + 2;
    let fim = depois[inicio..].find("**")? + inicio;
    Some(depois[inicio..fim].trim().to_string())
}

#[test]
fn status_aponta_a_primeira_caixa_acionavel_do_roadmap() {
    let fonte = std::fs::read_to_string(roadmap()).expect("ROADMAP.md é UTF-8");
    let status_txt = std::fs::read_to_string(status()).expect("STATUS.md é UTF-8");

    let todas = caixas(&fonte);
    let Some(esperado) = primeira_acionavel(&todas) else {
        return;
    };
    let id_esperado = esperado.texto.split_whitespace().next().unwrap_or("");

    let alvo = alvo_do_status(&status_txt)
        .expect("STATUS.md precisa de `Próxima tarefa: ROADMAP **X.Y**`");

    assert_eq!(
        alvo, id_esperado,
        "o parágrafo `Próxima tarefa` é o que decide a fila de verdade — medido \
         sete vezes. Apontar para outro item faz caixa envelhecer: o 2.4b levou 21 \
         iterações e o 3.8 levou 7. Aponte para a primeira acionável (linha {} do \
         ROADMAP) ou marque a caixa como BLOQUEADO, com a razão.",
        esperado.linha
    );
}
