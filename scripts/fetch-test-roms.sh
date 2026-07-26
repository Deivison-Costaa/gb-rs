#!/usr/bin/env bash
# Baixa as ROMs de teste para tests/roms/ (que é gitignored — ROM não entra no git).
#
#   ./scripts/fetch-test-roms.sh          # baixa se faltar, no-op se já estiver lá
#   ./scripts/fetch-test-roms.sh --force  # rebaixa tudo do zero
#
# Suítes:
#   - blargg: cpu_instrs, instr_timing, mem_timing, mem_timing-2, halt_bug,
#             oam_bug, interrupt_time, dmg_sound
#   - mooneye-test-suite: acceptance
#   - dmg-acid2
#
# FONTE: o bundle `c-sp/game-boy-test-roms`, que empacota as três suítes já
# montadas. A alternativa seria montar a mooneye a partir do fonte, o que exige
# RGBDS instalado — dependência pesada e desnecessária na CI. O bundle é
# fixado por tag E por sha256: se o conteúdo mudar, o script falha em vez de
# trocar o chão do scoreboard por baixo dos pés silenciosamente.
#
# Se o bundle estiver fora do ar, há fallback por suíte para blargg e dmg-acid2
# (fontes oficiais). A mooneye não tem release pré-montada upstream: nesse caso
# o script avisa e segue sem ela, em vez de derrubar a CI inteira.

set -euo pipefail

readonly BUNDLE_VERSION="v7.0"

# TEST_ROMS_BUNDLE_URL/_SHA256 são costura de teste, não configuração: existem
# para `crates/gb-cli/tests/fetch_test_roms.rs` apontar o download para um zip
# local (`file://`) e medir este script sem rede. Andam em par de propósito —
# quem trocar só a URL esbarra no sha256 fixado e o script morre, que é
# exatamente o comportamento que a fixação existe para dar. A CI não os define,
# e `ci_does_not_override_the_pinned_bundle` reprova o dia em que definir.
readonly BUNDLE_URL="${TEST_ROMS_BUNDLE_URL:-https://github.com/c-sp/game-boy-test-roms/releases/download/${BUNDLE_VERSION}/game-boy-test-roms-${BUNDLE_VERSION}.zip}"
readonly BUNDLE_SHA256="${TEST_ROMS_BUNDLE_SHA256:-b9a9d7a1075aa35a3d07c07c34974048672d8520dca9e07a50178f5860c3832c}"

readonly BLARGG_FALLBACK_URL="https://codeload.github.com/retrio/gb-test-roms/tar.gz/refs/heads/master"
readonly ACID2_FALLBACK_URL="https://github.com/mattcurrie/dmg-acid2/releases/download/v1.0/dmg-acid2.gb"

readonly ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly ROMS_DIR="${ROMS_DIR:-$ROOT/tests/roms}"
readonly CACHE_DIR="${ROM_CACHE_DIR:-$ROOT/.cache/roms}"
readonly STAMP="$ROMS_DIR/.bundle-version"

# Suítes blargg exigidas: sem estas o marco M1/M2 não tem como ser medido.
readonly REQUIRED_BLARGG=(
  cpu_instrs instr_timing mem_timing mem_timing-2
  halt_bug oam_bug interrupt_time dmg_sound
)

FORCE=0
[[ "${1:-}" == "--force" ]] && FORCE=1

log()  { printf '  %s\n' "$*"; }
warn() { printf '  ATENÇÃO: %s\n' "$*" >&2; }
die()  { printf 'ERRO: %s\n' "$*" >&2; exit 1; }

# `trap ... RETURN` não é local à função em bash: ele sobreviveria e dispararia
# de novo no return de main, com $tmp já fora de escopo. Um único trap EXIT
# sobre uma lista é o que de fato limpa.
# O `return 0` no fim não é decorativo: o status da última instrução do trap de
# EXIT vira o status de saída do script. Sem ele o script saía 1 mesmo tendo
# feito tudo certo, e a CI quebrava sem nenhuma mensagem de erro.
TMPDIRS=()
cleanup() {
  local d
  for d in "${TMPDIRS[@]+"${TMPDIRS[@]}"}"; do
    rm -rf "$d"
  done
  return 0
}
trap cleanup EXIT

new_tmpdir() {
  local d
  d="$(mktemp -d)"
  TMPDIRS+=("$d")
  printf '%s' "$d"
}

# O bundle traz blargg/cgb_sound junto, que é suíte de Game Boy Color e não foi
# pedida — este emulador é DMG. Deixá-la entrar poluiria o scoreboard com 13
# falhas permanentes que não são regressão de nada.
prune_blargg() {
  local d="$ROMS_DIR/blargg" entry base keep
  [[ -d "$d" ]] || return 0
  for entry in "$d"/*; do
    [[ -e "$entry" ]] || continue
    base="$(basename "$entry")"
    [[ "$base" == "halt_bug.gb" ]] && continue
    keep=0
    for s in "${REQUIRED_BLARGG[@]}"; do
      [[ "$base" == "$s" ]] && { keep=1; break; }
    done
    (( keep )) || rm -rf "$entry"
  done
}

require_tools() {
  local missing=()
  for t in curl unzip tar sha256sum; do
    command -v "$t" >/dev/null 2>&1 || missing+=("$t")
  done
  (( ${#missing[@]} == 0 )) || die "faltam ferramentas: ${missing[*]}"
}

# Já baixado nesta versão? Então não faz nada — o script roda em toda CI.
already_current() {
  (( FORCE == 0 )) \
    && [[ -f "$STAMP" ]] \
    && [[ "$(cat "$STAMP")" == "$BUNDLE_VERSION" ]] \
    && [[ -f "$ROMS_DIR/dmg-acid2/dmg-acid2.gb" ]] \
    && [[ -d "$ROMS_DIR/blargg/cpu_instrs/individual" ]]
}

download_bundle() {
  local zip="$CACHE_DIR/game-boy-test-roms-${BUNDLE_VERSION}.zip"
  mkdir -p "$CACHE_DIR"

  if [[ -f "$zip" ]] && echo "$BUNDLE_SHA256  $zip" | sha256sum -c --status 2>/dev/null; then
    log "bundle $BUNDLE_VERSION já em cache"
  else
    log "baixando bundle $BUNDLE_VERSION (~3.6 MiB)…"
    curl -sSfL --retry 3 --retry-delay 2 --connect-timeout 20 \
      -o "$zip.part" "$BUNDLE_URL" || return 1
    mv "$zip.part" "$zip"
    echo "$BUNDLE_SHA256  $zip" | sha256sum -c --status \
      || die "sha256 do bundle não confere — download corrompido ou release alterada. Verifique antes de seguir."
    log "sha256 confere"
  fi

  local tmp
  tmp="$(new_tmpdir)"

  unzip -q "$zip" -d "$tmp" \
    'blargg/*' 'mooneye-test-suite/acceptance/*' 'dmg-acid2/*'

  rm -rf "$ROMS_DIR/blargg" "$ROMS_DIR/mooneye" "$ROMS_DIR/dmg-acid2"
  mkdir -p "$ROMS_DIR/mooneye"
  mv "$tmp/blargg"                         "$ROMS_DIR/blargg"
  mv "$tmp/mooneye-test-suite/acceptance"  "$ROMS_DIR/mooneye/acceptance"
  mv "$tmp/dmg-acid2"                      "$ROMS_DIR/dmg-acid2"

  prune_blargg
  echo "$BUNDLE_VERSION" > "$STAMP"
}

# Plano B: fontes oficiais, uma por suíte. Cobre blargg e dmg-acid2.
download_fallback() {
  warn "bundle indisponível — caindo para as fontes oficiais por suíte"
  local tmp
  tmp="$(new_tmpdir)"

  log "blargg via retrio/gb-test-roms…"
  if curl -sSfL --retry 3 --connect-timeout 20 -o "$tmp/blargg.tar.gz" "$BLARGG_FALLBACK_URL"; then
    mkdir -p "$tmp/blargg"
    tar xzf "$tmp/blargg.tar.gz" -C "$tmp/blargg" --strip-components=1
    rm -rf "$ROMS_DIR/blargg"
    mkdir -p "$ROMS_DIR"
    mv "$tmp/blargg" "$ROMS_DIR/blargg"
  else
    warn "blargg indisponível também"
  fi

  log "dmg-acid2 via release oficial…"
  mkdir -p "$ROMS_DIR/dmg-acid2"
  curl -sSfL --retry 3 --connect-timeout 20 \
    -o "$ROMS_DIR/dmg-acid2/dmg-acid2.gb" "$ACID2_FALLBACK_URL" \
    || warn "dmg-acid2 indisponível também"

  prune_blargg

  warn "mooneye NÃO baixada: não existe release pré-montada upstream, só o fonte (exige RGBDS)."
  warn "O scoreboard vai reportar 0 ROMs de mooneye até o bundle voltar."

  echo "fallback" > "$STAMP"
}

verify() {
  local problems=0

  for suite in "${REQUIRED_BLARGG[@]}"; do
    local n
    # halt_bug é ROM solta na raiz de blargg/; as demais são diretórios.
    if [[ "$suite" == "halt_bug" ]]; then
      n=$(find "$ROMS_DIR/blargg" -maxdepth 1 -name 'halt_bug.gb' 2>/dev/null | wc -l)
    else
      n=$(find "$ROMS_DIR/blargg/$suite" -name '*.gb' 2>/dev/null | wc -l)
    fi
    printf '  %-24s %3d ROMs\n' "blargg/$suite" "$n"
    (( n > 0 )) || { warn "blargg/$suite vazia"; problems=1; }
  done

  local moon acid
  moon=$(find "$ROMS_DIR/mooneye/acceptance" -name '*.gb' 2>/dev/null | wc -l)
  printf '  %-24s %3d ROMs\n' "mooneye/acceptance" "$moon"

  acid=$(find "$ROMS_DIR/dmg-acid2" -name 'dmg-acid2.gb' 2>/dev/null | wc -l)
  printf '  %-24s %3d ROMs\n' "dmg-acid2" "$acid"
  (( acid > 0 )) || { warn "dmg-acid2 ausente"; problems=1; }

  return $problems
}

main() {
  require_tools

  if already_current; then
    echo "ROMs de teste já presentes ($BUNDLE_VERSION). Nada a fazer."
    echo "Use --force para rebaixar."
    exit 0
  fi

  mkdir -p "$ROMS_DIR"
  echo "Baixando ROMs de teste para $ROMS_DIR"

  download_bundle || download_fallback

  echo
  echo "Conferindo:"
  if verify; then
    echo
    echo "OK — ROMs prontas."
  else
    echo
    warn "algumas suítes ficaram incompletas (ver acima)."
    warn "O scoreboard roda mesmo assim, só reporta menos ROMs."
  fi
}

main "$@"
