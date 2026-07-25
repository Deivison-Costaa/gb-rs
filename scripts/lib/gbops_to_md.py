#!/usr/bin/env python3
"""Converte o dmgops.json do gbops na tabela de opcodes do SM83, em markdown.

Uso:
    gbops_to_md.py --json dmgops.json --out docs/reference/03-opcodes.md --sha <commit>

O que importa aqui, e é o motivo de a tabela ser gerada em vez de copiada:

  - **Flags por opcode.** A coluna Z/N/H/C diz o que cada instrução faz com cada
    flag. É onde a intuição de Z80 mais erra (R1). `RLCA` zera Z no SM83, ao
    contrário do `RLC A` prefixado por CB, que o calcula.
  - **Timing por M-cycle.** O gbops registra o que acontece em *cada* ciclo da
    instrução (fetch/read/write/internal). É exatamente a granularidade que a
    R2 exige do `Cpu::step()`, e não dá para derivar de um total em T-cycles.
  - **Timing condicional.** Saltos condicionais têm duração diferente conforme
    tomem ou não o desvio; as duas aparecem separadas.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

FLAG_MEANING = """
| Símbolo | Significado |
|---|---|
| `-` | flag não é afetada |
| `0` | flag é sempre zerada |
| `1` | flag é sempre setada |
| `Z` / `N` / `H` / `C` | flag é calculada a partir do resultado |
"""


def fmt_timing(steps: list[dict]) -> str:
    """Lista de M-cycles -> `fetch → read(i8) → internal` legível numa célula."""
    if not steps:
        return "—"
    out = []
    for s in steps:
        t = s.get("Type", "?")
        c = (s.get("Comment") or "").strip()
        out.append(f"{t}({c})" if c else t)
    return " → ".join(out)


def render_table(ops: list[dict], prefixed: bool) -> str:
    lines = [
        "| Opcode | Instrução | Grupo | Bytes | T-cycles | Z | N | H | C | M-cycles (passo a passo) |",
        "|---|---|---|---|---|---|---|---|---|---|",
    ]
    for i, op in enumerate(ops):
        name = op.get("Name") or "—"
        if name in ("unused", "UNUSED", ""):
            name = "*(inválido)*"

        code = f"`{'CB ' if prefixed else ''}{i:02X}`"
        group = op.get("Group") or "—"
        length = op.get("Length", "—")

        nb, br = op.get("TCyclesNoBranch"), op.get("TCyclesBranch")
        if nb is not None and br is not None and nb != br:
            # Condicional: sem desvio / com desvio.
            tcycles = f"{nb} / {br}"
        else:
            tcycles = str(nb if nb is not None else "—")

        f = op.get("Flags") or {}
        z, n, h, c = (f.get(k, "-") for k in ("Z", "N", "H", "C"))

        timing = fmt_timing(op.get("TimingNoBranch") or [])
        if op.get("TimingBranch"):
            timing += f" &nbsp;/&nbsp; **com desvio:** {fmt_timing(op['TimingBranch'])}"

        # `|` dentro de célula quebraria a tabela.
        name = name.replace("|", "\\|")
        lines.append(
            f"| {code} | `{name}` | {group} | {length} | {tcycles} "
            f"| `{z}` | `{n}` | `{h}` | `{c}` | {timing} |"
        )
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--sha", required=True)
    args = ap.parse_args()

    if not args.json.is_file():
        print(f"ERRO: {args.json} não encontrado", file=sys.stderr)
        return 1

    data = json.loads(args.json.read_text(encoding="utf-8"))
    unpref = data.get("Unprefixed") or []
    cbpref = data.get("CBPrefixed") or []
    if len(unpref) != 256 or len(cbpref) != 256:
        print(f"ERRO: esperava 256+256 opcodes, veio {len(unpref)}+{len(cbpref)}",
              file=sys.stderr)
        return 1

    doc = f"""# Tabela de opcodes do SM83

> **Fonte de verdade deste projeto (regra R1 do CLAUDE.md).**
> Gerada de [gbops](https://izik1.github.io/gbops/) (`dmgops.json`), fixado no
> commit [`{args.sha[:12]}`](https://github.com/izik1/gbops/tree/{args.sha}).
> Não editar à mão: regenerado por `scripts/fetch-reference-docs.sh`.

## Como ler esta tabela

**Flags (colunas Z, N, H, C):**
{FLAG_MEANING}
**T-cycles:** 1 M-cycle = 4 T-cycles. Onde aparecem dois valores (`8 / 12`), o
primeiro é sem tomar o desvio e o segundo é tomando — timing condicional.

**M-cycles (passo a passo):** o que o barramento faz em cada M-cycle da
instrução. É esta coluna que a regra R2 exige: `Cpu::step()` avança **um**
M-cycle, então cada passo aqui é uma parada da máquina de estados, não um
somatório aplicado no fim.

## Armadilhas que esta tabela resolve (R1)

O SM83 não é um Z80. Os pontos onde a intuição de Z80 erra e a tabela desmente:

- **`RLCA`/`RRCA`/`RLA`/`RRA` (opcodes `07`, `0F`, `17`, `1F`)** zeram `Z`
  incondicionalmente. Os equivalentes prefixados por CB (`CB 07` = `RLC A` etc.)
  **calculam** `Z`. Mesmo nome, flag diferente.
- **`ADD SP,i8` (`E8`) e `LD HL,SP+i8` (`F8`)** zeram `Z` e `N`, e calculam
  `H`/`C` sobre o **byte baixo** — carry de bit 3 e bit 7, não de bit 11/15
  como um `ADD HL,rr` faria.
- **`DAA` (`27`)** depende de `N` e `H` deixados pela operação anterior.
- **`LD (a16),SP` (`08`)** tem 5 M-cycles e escreve dois bytes.
- Opcodes `D3 DB DD E3 E4 EB EC ED F4 FC FD` **não existem** no SM83 e travam a
  CPU. Não são NOPs.

---

## Opcodes sem prefixo

{render_table(unpref, prefixed=False)}

---

## Opcodes com prefixo `CB`

Todos têm 2 bytes (o `CB` mais o opcode) e operam sobre registrador ou `(HL)`.

{render_table(cbpref, prefixed=True)}
"""

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(doc, encoding="utf-8")
    print(f"  {args.out.name:44} 512 opcodes, {args.out.stat().st_size // 1024} KiB")
    return 0


if __name__ == "__main__":
    sys.exit(main())
