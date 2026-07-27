#!/usr/bin/env bash
# Regenera docs/metricas.csv a partir dos logs brutos de orquestração.
#
#   ./scripts/metricas.sh          # regenera e mostra o resumo
#
# Por que existe: a medição de custo, duração e passos é feita pelo processo que
# hospeda o agente, não pelo agente. Ela cai em logs/, que é gitignorado — e o
# .gitignore sempre disse "métricas consolidadas vão em docs/". Este script é a
# ponte, e sem ele o docs/metricas.csv congela no dia em que foi commitado (foi
# o que aconteceu entre o PR #46 e o #72: 23 execuções fora do repositório).
#
# Rode com a árvore limpa, entre séries. O arquivo é regenerado inteiro a partir
# das fontes, então rodar duas vezes não duplica linha.
#
# Fontes, ambas em logs/:
#   metrics.csv      — do scripts/loop.sh (claude -p). Sem cabeçalho, sem modelo,
#                      sem par de commits. A coluna 2 é o índice dentro da série,
#                      não um resultado: linha existir já significa que passou,
#                      porque o loop.sh grava depois do teste de código de saída.
#   metrics-orq.csv  — do orquestrador. Tem cabeçalho, modelo, par de commits, e
#                      **grava também as falhas** — que é como os nove abortos de
#                      26/07 ficaram visíveis.
#
# Onde as duas se sobrepõem (mesmo ts e mesmo custo), a linha do orquestrador
# vence, por ser a mais rica.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

[[ -f logs/metrics-orq.csv ]] || { echo "sem logs/metrics-orq.csv — nada a fazer" >&2; exit 1; }

python3 - <<'PY'
import csv, pathlib

orq = list(csv.DictReader(open("logs/metrics-orq.csv")))
vistos = {(r["ts"], round(float(r["custo_usd"]), 6)) for r in orq}

linhas = []
loop = pathlib.Path("logs/metrics.csv")
if loop.exists():
    for ts, _indice, custo, turnos, dur in csv.reader(open(loop)):
        if (ts, round(float(custo), 6)) not in vistos:
            linhas.append([ts, "ok", custo, turnos, dur, "", "", "claude-opus-5[1m]", "loop.sh"])

for r in orq:
    linhas.append([r["ts"], r["resultado"], r["custo_usd"], r["turnos"], r["duracao_ms"],
                   r["head_antes"], r["head_depois"], r["modelo"], "orquestrador"])

linhas.sort(key=lambda l: l[0])

with open("docs/metricas.csv", "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["ts", "resultado", "custo_usd", "turnos", "duracao_ms",
                "head_antes", "head_depois", "modelo", "fonte"])
    w.writerows(linhas)

falhas = [l for l in linhas if l[1] != "ok" and not l[1].startswith("ok")]
print(f"{len(linhas)} execuções  US$ {sum(float(l[2]) for l in linhas):.2f}  "
      f"{sum(int(l[4]) for l in linhas)/3600000:.1f} h  ({len(falhas)} falhas preservadas)")
PY
