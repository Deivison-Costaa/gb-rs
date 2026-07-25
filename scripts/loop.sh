#!/usr/bin/env bash
# Driver do loop de iterações do gb-rs.
#
#   ./scripts/loop.sh 3        # roda 3 iterações e para
#
# Cada invocação de `claude -p` é uma sessão NOVA, com contexto zerado.
# A continuidade vem de STATUS.md + ROADMAP.md + git, não da memória do agente.
#
# Comece com N=1 e supervisionado. Só aumente depois de ver 3 iterações
# saudáveis seguidas.

set -euo pipefail

N="${1:-1}"
mkdir -p logs

for i in $(seq 1 "$N"); do
  echo "════════ iteração $i/$N — $(date -Is) ════════"

  # Guarda: só começa se a árvore estiver limpa e em main.
  if [[ -n "$(git status --porcelain)" ]]; then
    echo "ABORTADO: árvore de trabalho suja. Resolva na mão." >&2
    exit 1
  fi
  git checkout main --quiet && git pull --quiet

  # Guarda: roadmap acabou?
  if ! grep -q '^- \[ \]' ROADMAP.md; then
    echo "ROADMAP concluído. Nada a fazer."
    exit 0
  fi

  LOG="logs/iter-$(date +%Y%m%d-%H%M%S).json"

  set +e
  claude -p "Use a skill 'iterate' para executar exatamente UMA iteração do projeto, do passo 0 ao passo 11. Pare ao final." \
    --permission-mode acceptEdits \
    --allowedTools "Read,Write,Edit,Glob,Grep,Bash,WebFetch,WebSearch" \
    --max-turns 150 \
    --output-format json \
    > "$LOG"
  RC=$?
  set -e

  if [[ $RC -ne 0 ]]; then
    echo "Iteração falhou (exit $RC). Log: $LOG" >&2
    echo "Parando o loop — inspecione STATUS.md → Bloqueios." >&2
    exit $RC
  fi

  # Métricas para a apresentação.
  if command -v jq >/dev/null; then
    jq -r '[.total_cost_usd, .num_turns, .duration_ms] | @csv' "$LOG" \
      | sed "s/^/$(date -Is),$i,/" >> logs/metrics.csv
    echo "custo/turnos/ms: $(jq -r '[.total_cost_usd,.num_turns,.duration_ms]|@csv' "$LOG")"
  fi

  # Revisão cruzada com o segundo modelo (não bloqueia, só registra).
  ./scripts/review.sh || echo "(revisão cruzada falhou — seguindo)"
done

echo "Loop concluído: $N iterações."
