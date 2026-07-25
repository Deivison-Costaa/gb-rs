#!/usr/bin/env bash
# Publica o scoreboard.csv acumulado numa branch de dados — ROADMAP 0.2c.
#
#   ./scripts/publish-scoreboard.sh
#   DATA_BRANCH=outra ./scripts/publish-scoreboard.sh
#
# ------------------------------------------------------------------------
# POR QUE NÃO É `main`
#
# O item 0.2c foi escrito como "commit-back do scoreboard.csv no push para
# main". Não dá, e o motivo é da configuração do repositório, não do script:
#
#   $ gh api repos/<owner>/gb-rs/branches/main/protection
#     required_pull_request_reviews: { required_approving_review_count: 0 }
#     enforce_admins:                { enabled: false }
#
# `required_pull_request_reviews` **ligado** — mesmo exigindo 0 aprovações —
# bloqueia push direto em `main` para quem não tem bypass. `enforce_admins=false`
# dá esse bypass ao dono humano, não ao `github-actions[bot]`, que não é admin.
# E `bypass_pull_request_allowances` só existe em repositório de organização;
# este é de usuário. Abrir PR pela CI também não fecha o ciclo: PR criado com o
# `GITHUB_TOKEN` não dispara workflow, então `check` e `scoreboard` — que a
# proteção exige — nunca rodariam nele.
#
# Restariam afrouxar a proteção de `main` (invariante do projeto) ou provisionar
# um PAT de admin à mão (fora do alcance de um agente). A série vai para uma
# branch própria, que não é protegida e que ninguém faz checkout no dia a dia.
#
# Efeito colateral bem-vindo: `main` fica só com commits de iteração, e o CSV
# não vira campo de conflito entre a CI e a iteração seguinte.
# ------------------------------------------------------------------------
#
# COMO PUBLICA
#
# A branch de dados guarda **um** arquivo, `scoreboard.csv`, com a união do que
# já estava publicado e do que este runner mediu. União, e não substituição, é
# o ponto todo: o runner faz checkout de um commit e mede em cima do CSV
# daquele commit, então o CSV local é sempre um recorte da série, nunca a série
# inteira. Empurrar o recorte por cima apagaria o que outra execução publicou.
#
# A chave da união é a **linha inteira**. Uma linha carrega timestamp (segundo),
# commit, suíte, ROM, status e ciclos: duas medições distintas só colidem se
# caírem no mesmo segundo, no mesmo commit, para a mesma ROM. Em compensação,
# republicar o mesmo CSV é idempotente — e é o que faz "não há nada novo"
# terminar sem commit em vez de empilhar commit vazio a cada push.
#
# O commit é montado com plumbing (`hash-object`/`mktree`/`commit-tree`) em vez
# de worktree: não mexe no índice nem no diretório de trabalho do runner, que
# ainda tem o checkout de `main` e o `scoreboard.csv` que o passo do artefato
# vai ler depois.

set -euo pipefail

die() {
  printf 'ERRO: %s\n' "$*" >&2
  exit 1
}

REPO="$(git rev-parse --show-toplevel 2>/dev/null)" ||
  die "não estou dentro de um repositório git"
readonly REPO

readonly CSV="${SCOREBOARD_CSV:-$REPO/scoreboard.csv}"
readonly CSV_HEADER="timestamp,commit,suite,rom,status,ciclos"

readonly DATA_BRANCH="${DATA_BRANCH:-scoreboard-data}"
readonly DATA_REMOTE="${DATA_REMOTE:-origin}"
readonly DATA_FILE="scoreboard.csv"
readonly PUSH_ATTEMPTS="${PUSH_ATTEMPTS:-3}"

# `commit-tree` exige autor; no runner o ambiente do Actions não define nenhum.
export GIT_AUTHOR_NAME="${GIT_AUTHOR_NAME:-github-actions[bot]}"
export GIT_AUTHOR_EMAIL="${GIT_AUTHOR_EMAIL:-41898282+github-actions[bot]@users.noreply.github.com}"
export GIT_COMMITTER_NAME="${GIT_COMMITTER_NAME:-$GIT_AUTHOR_NAME}"
export GIT_COMMITTER_EMAIL="${GIT_COMMITTER_EMAIL:-$GIT_AUTHOR_EMAIL}"

git_() { git -C "$REPO" "$@"; }

# Linhas de dado do CSV — cabeçalho e linhas em branco não contam.
data_rows() { awk 'FNR > 1 && $0 != "" { n++ } END { print n + 0 }' "$1"; }

# O topo da branch de dados no remoto, ou vazio se ela ainda não existe.
remote_tip() {
  if git_ fetch --quiet "$DATA_REMOTE" "$DATA_BRANCH" 2>/dev/null; then
    git_ rev-parse FETCH_HEAD
  else
    printf ''
  fi
}

# O CSV já publicado (ou só o cabeçalho, se não há branch/arquivo) em $2.
published_csv() {
  local tip="$1" dest="$2"
  if [[ -n "$tip" ]] && git_ cat-file -e "$tip:$DATA_FILE" 2>/dev/null; then
    git_ show "$tip:$DATA_FILE" >"$dest"
  else
    printf '%s\n' "$CSV_HEADER" >"$dest"
  fi
}

# União de $1 (publicado) e $2 (local), nessa ordem, sem repetir linha.
#
# O `NR == FNR` distingue o primeiro arquivo do segundo. Quando o publicado tem
# só o cabeçalho ele não contribui com nenhum registro, e aí `NR == FNR` passa a
# valer para o local — que é justamente o que se quer: imprimir tudo.
merge_csv() {
  printf '%s\n' "$CSV_HEADER"
  awk '
    FNR == 1   { next }
    $0 == ""   { next }
    NR == FNR  { visto[$0] = 1; print; next }
    !($0 in visto) { print }
  ' "$1" "$2"
}

main() {
  [[ -f "$CSV" ]] || die "não achei $CSV — rode ./scripts/scoreboard.sh antes."

  local rows
  rows="$(data_rows "$CSV")"
  # Mesmo raciocínio da 0.2b: publicar zero linha com sucesso afirmaria que a
  # série foi em frente quando nada foi medido.
  (( rows > 0 )) || die "$CSV não tem nenhuma linha de dado — nada a publicar."

  local tmp
  tmp="$(mktemp -d)"
  # shellcheck disable=SC2064  # expandir $tmp agora é o certo: é o que se apaga.
  trap "rm -rf '$tmp'" EXIT

  local attempt tip base merged added blob tree commit msg
  base="$tmp/publicado.csv"
  merged="$tmp/uniao.csv"

  for (( attempt = 1; attempt <= PUSH_ATTEMPTS; attempt++ )); do
    # Refeito a cada tentativa de propósito: um push rejeitado quer dizer que o
    # topo mudou, e reempurrar o mesmo commit só repetiria a rejeição.
    tip="$(remote_tip)"
    published_csv "$tip" "$base"
    merge_csv "$base" "$CSV" >"$merged"

    if cmp -s "$base" "$merged"; then
      printf '%s já tem as %s linhas de %s — nada novo a publicar.\n' \
        "$DATA_BRANCH" "$rows" "$CSV"
      return 0
    fi

    added=$(( $(data_rows "$merged") - $(data_rows "$base") ))
    msg="chore(scoreboard): +$added linhas de $(git_ rev-parse --short HEAD 2>/dev/null || echo desconhecido)"

    blob="$(git_ hash-object -w --stdin <"$merged")"
    tree="$(printf '100644 blob %s\t%s\n' "$blob" "$DATA_FILE" | git_ mktree)"
    if [[ -n "$tip" ]]; then
      commit="$(git_ commit-tree "$tree" -p "$tip" -m "$msg")"
    else
      commit="$(git_ commit-tree "$tree" -m "$msg")"
    fi

    if git_ push --quiet "$DATA_REMOTE" "$commit:refs/heads/$DATA_BRANCH"; then
      printf 'Publicado em %s/%s: +%s linhas (commit %s).\n' \
        "$DATA_REMOTE" "$DATA_BRANCH" "$added" "${commit:0:12}"
      return 0
    fi

    printf 'push rejeitado (tentativa %s/%s) — refazendo sobre o topo novo.\n' \
      "$attempt" "$PUSH_ATTEMPTS" >&2
  done

  die "não consegui publicar em $DATA_BRANCH após $PUSH_ATTEMPTS tentativas."
}

main "$@"
