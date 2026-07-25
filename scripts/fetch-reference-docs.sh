#!/usr/bin/env bash
# Regenera docs/reference/ — a fonte de verdade do projeto (regra R1).
#
#   ./scripts/fetch-reference-docs.sh
#
# Baixa as seções relevantes do Pan Docs e a tabela de opcodes do gbops, e as
# converte para um markdown por tema. Ambas as fontes são FIXADAS por commit:
# a spec que o agente lê hoje tem de ser a mesma que ele leu na iteração
# passada, senão "consultei a spec" não quer dizer nada.
#
# Para atualizar as specs, mude PANDOCS_SHA/GBOPS_SHA aqui, rode o script, e
# commite o diff de docs/reference/ separado de qualquer mudança de código —
# assim dá para ver o que a spec mudou.
#
# docs/reference/ É COMMITADO. Não é cache: é o que torna o projeto auditável
# e o que permite trabalhar offline.

set -euo pipefail

readonly PANDOCS_SHA="fe246067b695b5404a4a6a47efb4fd6d921ececb"
readonly GBOPS_SHA="90b9bf296aed373335a0abbef0a0794a919b9f2c"

readonly PANDOCS_RAW="https://raw.githubusercontent.com/gbdev/pandocs/${PANDOCS_SHA}/src"
readonly GBOPS_RAW="https://raw.githubusercontent.com/izik1/gbops/${GBOPS_SHA}/dmgops.json"

readonly ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly OUT_DIR="$ROOT/docs/reference"
readonly LIB="$ROOT/scripts/lib"

die() { printf 'ERRO: %s\n' "$*" >&2; exit 1; }

TMP=""
cleanup() { [[ -n "$TMP" ]] && rm -rf "$TMP"; return 0; }
trap cleanup EXIT

# --- mapa tema -> seções do Pan Docs -------------------------------------
# A ordem dentro de cada tema é a ordem de leitura, não a alfabética.

readonly T1_MEMORY="Memory_Map Hardware_Reg_List Power_Up_Sequence"
readonly T2_CPU="CPU_Registers_and_Flags CPU_Instruction_Set CPU_Comparison_with_Z80"
readonly T4_TIMERS="Timer_and_Divider_Registers Timer_Obscure_Behaviour"
readonly T5_INTERRUPTS="Interrupts Interrupt_Sources halt"
readonly T6_PPU="Graphics Tile_Data Tile_Maps OAM OAM_DMA_Transfer Window LCDC STAT Scrolling Palettes Rendering pixel_fifo Accessing_VRAM_and_OAM OAM_Corruption_Bug"
readonly T7_APU="Audio Audio_Registers Audio_details"
readonly T8_CART="The_Cartridge_Header MBCs nombc MBC1 MBC2 MBC3 MBC5"
readonly T9_IO="Joypad_Input Serial_Data_Transfer_(Link_Cable)"

readonly ALL_SECTIONS="$T1_MEMORY $T2_CPU $T4_TIMERS $T5_INTERRUPTS $T6_PPU $T7_APU $T8_CART $T9_IO"

fetch_sections() {
  local n=0 f
  for f in $ALL_SECTIONS; do
    curl -sSfL --retry 3 --retry-delay 2 --connect-timeout 20 \
      -o "$TMP/$f.md" "$PANDOCS_RAW/$f.md" \
      || die "falhou ao baixar $f.md do Pan Docs (sha $PANDOCS_SHA)"
    n=$(( n + 1 ))
  done
  echo "  $n seções do Pan Docs baixadas"
}

# Recebe: título, arquivo de saída, intro, e os stems das seções.
build_theme() {
  local title="$1" out="$2" intro="$3"; shift 3
  local srcs=() s
  for s in "$@"; do srcs+=("$TMP/$s.md"); done
  python3 "$LIB/pandocs_to_md.py" \
    --title "$title" --out "$OUT_DIR/$out" --sha "$PANDOCS_SHA" \
    --intro "$intro" "${srcs[@]}"
}

main() {
  command -v python3 >/dev/null || die "python3 é necessário"
  command -v curl    >/dev/null || die "curl é necessário"
  [[ -f "$LIB/pandocs_to_md.py" ]] || die "$LIB/pandocs_to_md.py não encontrado"

  TMP="$(mktemp -d)"
  mkdir -p "$OUT_DIR"

  echo "Baixando specs (Pan Docs @ ${PANDOCS_SHA:0:12}, gbops @ $GBOPS_SHA)…"
  fetch_sections
  curl -sSfL --retry 3 --connect-timeout 20 -o "$TMP/dmgops.json" "$GBOPS_RAW" \
    || die "falhou ao baixar dmgops.json do gbops"
  echo "  tabela de opcodes do gbops baixada"

  echo
  echo "Gerando docs/reference/:"

  build_theme "Mapa de memória e registradores de I/O" "01-memory-map.md" \
    "Cobre o ROADMAP 1.2 (MMU, WRAM, HRAM, echo RAM, região proibida) e o estado pós-boot dos registradores, necessário para pular a boot ROM." \
    $T1_MEMORY

  build_theme "CPU SM83 — registradores, flags e instruções" "02-cpu.md" \
    "Cobre o ROADMAP 1.1 e 1.4–1.11. A seção de comparação com o Z80 é leitura obrigatória antes de qualquer opcode: é o catálogo das divergências que a regra R1 existe para evitar." \
    $T2_CPU

  python3 "$LIB/gbops_to_md.py" \
    --json "$TMP/dmgops.json" --out "$OUT_DIR/03-opcodes.md" --sha "$GBOPS_SHA"

  build_theme "Timer e divisor" "04-timers.md" \
    "Cobre o ROADMAP 2.1. A seção de comportamento obscuro documenta o atraso de 4 ciclos no overflow de TIMA, que é o que a blargg instr_timing cobra." \
    $T4_TIMERS

  build_theme "Interrupções e HALT" "05-interrupts.md" \
    "Cobre o ROADMAP 2.2 e 2.3, incluindo o bug do HALT e o atraso de uma instrução do EI." \
    $T5_INTERRUPTS

  build_theme "PPU — registradores, modos e renderização" "06-ppu.md" \
    "Cobre todo o marco M3. Inclui o pixel FIFO por completude: este projeto usa renderizador por scanline (invariante do STATUS.md), mas o FIFO explica os efeitos que o scanline não reproduz." \
    $T6_PPU

  build_theme "APU — canais, frame sequencer e mixagem" "07-apu.md" \
    "Cobre todo o marco M6." \
    $T7_APU

  build_theme "Cartucho — cabeçalho e MBCs" "08-cartridges-mbc.md" \
    "Cobre o ROADMAP 0.3, 0.4 e todo o marco M5. Inclui MBC1, MBC2, MBC3 e MBC5." \
    $T8_CART

  build_theme "Joypad e porta serial" "09-joypad-serial.md" \
    "Cobre o ROADMAP 1.12 (stub serial, por onde as ROMs blargg reportam resultado) e 4.1 (joypad)." \
    $T9_IO

  echo
  echo "OK. $(find "$OUT_DIR" -name '*.md' | wc -l) arquivos em docs/reference/."
  echo "Lembre: docs/reference/ é commitado. Regenerou? Commite o diff separado."
}

main "$@"
