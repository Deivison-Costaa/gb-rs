#!/usr/bin/env bash
# Mede a densidade de comentários por arquivo .rs (R7 do CLAUDE.md: teto de 5%).
# Somente leitura. A guarda que reprova é crates/gb-core/tests/comment_density.rs;
# as duas usam a mesma fórmula — mexeu numa, mexa na outra.

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

printf "%-42s %6s %6s %6s %6s %7s\n" "ARQUIVO" "TOTAL" "CODIGO" "DOC" "COMENT" "%COMENT"
printf "%-42s %6s %6s %6s %6s %7s\n" "------" "-----" "------" "---" "------" "-------"

TT=0; TC=0; TD=0; TI=0

for f in $(git ls-files '*.rs' | sort); do
  total=$(wc -l < "$f")
  # `grep -c` sai 1 quando conta zero — o `|| true` é o que permite `set -e`.
  blank=$(grep -c '^[[:space:]]*$' "$f" || true)
  doc=$(grep -cE '^[[:space:]]*(///|//!)' "$f" || true)
  inner=$(grep -cE '^[[:space:]]*(//[^/!]|//$|/\*|\*)' "$f" || true)
  code=$(( total - blank - doc - inner ))
  cmt=$(( doc + inner ))
  pct=0
  [[ $(( code + cmt )) -gt 0 ]] && pct=$(( cmt * 100 / (code + cmt) ))
  printf "%-42s %6d %6d %6d %6d %6d%%\n" "$f" "$total" "$code" "$doc" "$inner" "$pct"
  TT=$((TT+total)); TC=$((TC+code)); TD=$((TD+doc)); TI=$((TI+inner))
done

TCM=$(( TD + TI ))
echo
printf "%-42s %6d %6d %6d %6d %6d%%\n" "TOTAL" "$TT" "$TC" "$TD" "$TI" "$(( TCM * 100 / (TC + TCM) ))"
