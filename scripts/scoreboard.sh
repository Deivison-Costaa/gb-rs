#!/usr/bin/env bash
# Roda todas as ROMs de tests/roms/ pelo gb-cli headless e ANEXA o resultado a
# scoreboard.csv. O CSV é acumulativo: cada execução vira um ponto na série
# temporal que gera os gráficos da apresentação (ROADMAP 8.2). Nunca truncar.
#
#   ./scripts/scoreboard.sh            # roda tudo, resumo por suíte
#   ./scripts/scoreboard.sh -v         # imprime uma linha por ROM
#   ROM_TIMEOUT=60 ./scripts/scoreboard.sh
#
# Formato do CSV:
#   timestamp,commit,suite,rom,status,ciclos
#
# `suite` e `rom` saem entre aspas porque há ROM com vírgula no nome
# (`03-op sp,hl.gb`). Sem as aspas o CSV do trabalho inteiro nasce corrompido.
#
# Status possíveis:
#   pass     gb-cli saiu 0 — a ROM reportou sucesso
#   fail     gb-cli saiu 1 — a ROM reportou falha
#   timeout  estourou ROM_TIMEOUT
#   crash    gb-cli saiu com qualquer outro código
#   skip     gb-cli ainda não existe (ou não há ROMs) — contabiliza como 0
#
# Código de saída do próprio script: 0 se anexou ao menos uma linha, != 0 se
# não anexou nenhuma (ROADMAP 0.2b). "Rodou sem medir nada" é erro, não
# sucesso — senão a CI fica verde com a série temporal congelada.
#
# ------------------------------------------------------------------------
# CONTRATO COM O gb-cli
#
# Este script é escrito ANTES do gb-cli existir (regra R5: teste antes de
# implementar). Ele define o contrato que os itens 0.3 e 1.12 do ROADMAP têm
# de cumprir:
#
#   gb-cli run <rom> --headless --max-cycles <n>
#
#     exit 0    a ROM reportou sucesso
#     exit 1    a ROM reportou falha
#     exit 124  reservado ao timeout(1), não usar
#     outro     erro do emulador (ROM inválida, pânico, opcode não implementado)
#
#     stdout    saída da porta serial (é por onde blargg e mooneye falam)
#     saída     deve conter, em algum ponto, o token `cycles=<n>` com o total
#               de M-cycles executados — é a coluna `ciclos` do CSV
#
# Quem decide passou/falhou é o gb-cli, não este script: cada suíte sinaliza de
# um jeito (blargg escreve texto na serial; mooneye usa `LD B,B` com valores
# mágicos de Fibonacci nos registradores; dmg-acid2 exige hash do framebuffer).
# Essa lógica pertence ao emulador, que conhece a convenção de cada uma.
# ------------------------------------------------------------------------

set -euo pipefail

readonly ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly ROMS_DIR="${ROMS_DIR:-$ROOT/tests/roms}"
readonly CSV="${SCOREBOARD_CSV:-$ROOT/scoreboard.csv}"
readonly CSV_HEADER="timestamp,commit,suite,rom,status,ciclos"

readonly ROM_TIMEOUT="${ROM_TIMEOUT:-30}"
readonly MAX_CYCLES="${MAX_CYCLES:-250000000}"

# SHA-256 do framebuffer esperado do dmg-acid2 — extraído de
# tests/roms/dmg-acid2/dmg-acid2-dmg.png (160×144, modo L, 4 tons).
readonly DMG_ACID2_HASH='f844ea760a6f1fe137f7f992c7ab1c72d34c7fcd3a807b4174a78eb04a32a458'

# Sufixos de modelo da mooneye que NÃO são DMG-B (o alvo deste emulador).
# Elas continuam sendo executadas — o enunciado pede "todas as ROMs baixadas" —
# mas vão para uma suíte própria, para não parecerem regressão no gráfico.
# `-GS` (Game Boy + Super Game Boy) e `-dmgABC*` valem para DMG e ficam de fora
# desta lista de propósito.
readonly NONDMG_SUFFIXES='-(dmg0|mgb|sgb|sgb2|S|A)\.gb$'

VERBOSE=0
[[ "${1:-}" == "-v" || "${1:-}" == "--verbose" ]] && VERBOSE=1

die() { printf 'ERRO: %s\n' "$*" >&2; exit 1; }

(( BASH_VERSINFO[0] >= 4 )) || die "precisa de bash 4+ (arrays associativos)"

# --- localizar o gb-cli -------------------------------------------------

find_gb_cli() {
  if [[ -n "${GB_CLI:-}" && -x "${GB_CLI:-}" ]]; then
    printf '%s' "$GB_CLI"; return 0
  fi
  local c
  for c in "$ROOT/target/release/gb-cli" "$ROOT/target/debug/gb-cli"; do
    [[ -x "$c" ]] && { printf '%s' "$c"; return 0; }
  done
  if command -v gb-cli >/dev/null 2>&1; then
    command -v gb-cli; return 0
  fi

  # Existe workspace mas o binário não foi construído? Tenta construir.
  # Best-effort: se falhar, o script entra em modo skip em vez de quebrar a CI.
  if [[ -f "$ROOT/Cargo.toml" ]]; then
    echo "gb-cli não encontrado — tentando cargo build --release -p gb-cli…" >&2
    if (cd "$ROOT" && cargo build --release -p gb-cli >&2); then
      [[ -x "$ROOT/target/release/gb-cli" ]] && { printf '%s' "$ROOT/target/release/gb-cli"; return 0; }
    fi
  fi
  return 1
}

# --- classificação --------------------------------------------------------

suite_of() {
  local rel="$1" top rest
  top="${rel%%/*}"
  rest="${rel#*/}"
  case "$top" in
    blargg)
      # blargg/cpu_instrs/individual/01.gb -> blargg/cpu_instrs
      # blargg/halt_bug.gb                 -> blargg/halt_bug
      if [[ "$rest" == */* ]]; then printf 'blargg/%s' "${rest%%/*}"
      else printf 'blargg/%s' "${rest%.gb}"; fi ;;
    mooneye)
      if [[ "$rel" =~ $NONDMG_SUFFIXES ]]; then printf 'mooneye/acceptance-nondmg'
      else printf 'mooneye/acceptance'; fi ;;
    dmg-acid2) printf 'dmg-acid2' ;;
    *)         printf '%s' "$top" ;;
  esac
}

csv_escape() { printf '"%s"' "${1//\"/\"\"}"; }

# Linhas de dado do CSV — o cabeçalho não conta. 0 se o arquivo não existe.
csv_data_lines() {
  if [[ ! -f "$CSV" ]]; then printf '0'; return 0; fi
  local n
  n="$(wc -l < "$CSV")"
  if (( n > 0 )); then printf '%s' "$(( n - 1 ))"; else printf '0'; fi
}

# --- execução -------------------------------------------------------------

run_one() {
  local gb_cli="$1" rom="$2"
  local fb_hash="${3:-}"
  local out rc cycles extra_args=()

  if [[ -n "$fb_hash" ]]; then
    extra_args=(--check-fb-hash "$fb_hash")
  fi

  set +e
  out="$(timeout --signal=KILL "$ROM_TIMEOUT" \
         "$gb_cli" run "$rom" --headless --max-cycles "$MAX_CYCLES" \
         "${extra_args[@]}" 2>&1)"
  rc=$?
  set -e

  # O `|| true` não é decorativo: com `set -e` + `pipefail`, um grep que não
  # casa sai 1, derruba a atribuição e mata o script inteiro — antes do
  # fallback da linha seguinte. O caminho só passou a ser exercido quando o
  # gb-cli passou a existir (ROADMAP 0.1) sem ainda emitir `cycles=`.
  cycles="$(printf '%s' "$out" | grep -oE 'cycles=[0-9]+' | tail -1 | cut -d= -f2 || true)"
  [[ -n "$cycles" ]] || cycles=0

  case "$rc" in
    0)   printf 'pass %s'    "$cycles" ;;
    1)   printf 'fail %s'    "$cycles" ;;
    124|137) printf 'timeout %s' "$cycles" ;;
    *)   printf 'crash %s'   "$cycles" ;;
  esac
}

main() {
  local ts commit gb_cli mode rows_before rows_after

  # Medido antes de qualquer escrita — inclusive antes da criação do cabeçalho.
  rows_before="$(csv_data_lines)"

  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  commit="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"

  if gb_cli="$(find_gb_cli)"; then
    mode=run
    echo "gb-cli: $gb_cli"
  else
    gb_cli=""
    mode=skip
    echo "gb-cli ainda não existe (ROADMAP 0.1–1.12). Reportando 0 em tudo."
  fi

  # ROMs em ordem estável: o CSV é uma série temporal, ordem aleatória
  # atrapalharia qualquer diff entre execuções.
  local roms=()
  if [[ -d "$ROMS_DIR" ]]; then
    while IFS= read -r -d '' f; do roms+=("$f"); done \
      < <(find "$ROMS_DIR" -type f -name '*.gb' -print0 | sort -z)
  fi

  if (( ${#roms[@]} == 0 )); then
    echo "Nenhuma ROM em $ROMS_DIR — rode ./scripts/fetch-test-roms.sh primeiro."
  fi

  [[ -f "$CSV" ]] || echo "$CSV_HEADER" > "$CSV"

  # O `=()` não é enfeite: sob `set -u`, um `declare -A total` sem atribuição
  # deixa a variável declarada e **não associada**, e o `${#total[@]}` do resumo
  # abaixo derruba o script — justamente no caso de zero ROMs, que é o caso que
  # o `if` ali existe para tratar. Bash 5.3, testado.
  declare -A total=() pass=()
  local rom rel suite status cycles result

  for rom in "${roms[@]+"${roms[@]}"}"; do
    rel="${rom#"$ROMS_DIR"/}"
    suite="$(suite_of "$rel")"

    if [[ "$mode" == run ]]; then
      if [[ "$suite" == dmg-acid2 ]]; then
        result="$(run_one "$gb_cli" "$rom" "$DMG_ACID2_HASH")"
      else
        result="$(run_one "$gb_cli" "$rom")"
      fi
      status="${result%% *}"
      cycles="${result##* }"
    else
      status=skip
      cycles=0
    fi

    total["$suite"]=$(( ${total["$suite"]:-0} + 1 ))
    [[ "$status" == pass ]] && pass["$suite"]=$(( ${pass["$suite"]:-0} + 1 ))
    : "${pass[$suite]:=0}"

    printf '%s,%s,%s,%s,%s,%s\n' \
      "$ts" "$commit" "$(csv_escape "$suite")" "$(csv_escape "$rel")" \
      "$status" "$cycles" >> "$CSV"

    if (( VERBOSE )) || [[ "$status" == crash ]]; then
      printf '  %-8s %s\n' "$status" "$rel"
    fi
  done

  # --- resumo ---
  echo
  printf '%-32s %s\n' "SUÍTE" "PASSANDO"
  printf '%-32s %s\n' "--------------------------------" "--------"
  local grand_t=0 grand_p=0 s t p
  if (( ${#total[@]} > 0 )); then
    while IFS= read -r s; do
      t="${total["$s"]}"
      p="${pass["$s"]:-0}"
      printf '%-32s %3d/%-3d\n' "$s" "$p" "$t"
      grand_t=$(( grand_t + t ))
      grand_p=$(( grand_p + p ))
    done < <(printf '%s\n' "${!total[@]}" | sort)
  fi
  printf '%-32s %s\n' "--------------------------------" "--------"
  printf '%-32s %3d/%-3d\n' "TOTAL" "$grand_p" "$grand_t"
  echo
  rows_after="$(csv_data_lines)"
  echo "Anexado a $CSV ($rows_after linhas no total, $rows_before antes)."

  # ROADMAP 0.2b — a checagem que faltava.
  #
  # `set -e` cobre o script morrer. Não cobre o script terminar bem e não ter
  # medido nada: ROMs não baixadas, `find` mudo, laço sem nenhuma volta. Nesse
  # caso a CI ficaria verde, o artefato subiria com o CSV de ontem, e a série
  # temporal do ROADMAP 8.2 congelaria sem sinal nenhum. Sair 0 aqui seria
  # afirmar "medi tudo e está tudo bem" tendo medido nada.
  if (( rows_after <= rows_before )); then
    die "não anexou nenhuma linha a $CSV (antes: $rows_before, depois: $rows_after).
     Nenhuma ROM foi medida — rode ./scripts/fetch-test-roms.sh, ou confira
     ROMS_DIR=$ROMS_DIR."
  fi
}

main "$@"
