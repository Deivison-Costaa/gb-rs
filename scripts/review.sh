#!/usr/bin/env bash
# Revisão cruzada da última iteração com um SEGUNDO modelo.
#
# O ponto: quem escreveu o código é péssimo juiz do próprio código, e um modelo
# diferente erra em lugares diferentes. Os achados viram dado para a
# apresentação (quantos procedem? em que categoria?).
#
# Ajuste REVIEWER_CMD para a ferramenta que você tem (opencode, ollama, etc).

set -euo pipefail

REVIEWER_CMD="${REVIEWER_CMD:-opencode run}"
OUT_DIR="docs/reviews"
mkdir -p "$OUT_DIR"

DIFF=$(git diff HEAD~1 HEAD -- '*.rs' || true)
if [[ -z "$DIFF" ]]; then
  echo "Sem mudanças em .rs no último commit — nada a revisar."
  exit 0
fi

STAMP=$(date +%Y%m%d-%H%M%S)
PROMPT_FILE=$(mktemp)

{
  cat docs/prompts/review.md
  echo
  echo "## Diff sob revisão"
  echo '```diff'
  echo "$DIFF"
  echo '```'
} > "$PROMPT_FILE"

$REVIEWER_CMD < "$PROMPT_FILE" > "$OUT_DIR/$STAMP.md"
rm -f "$PROMPT_FILE"

echo "Revisão salva em $OUT_DIR/$STAMP.md"
echo "→ Leia, classifique os achados como procedentes ou não, e registre no doc da iteração."
