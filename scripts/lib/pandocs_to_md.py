#!/usr/bin/env python3
"""Converte fontes do Pan Docs (mdBook) em um markdown único por tema.

Uso:
    pandocs_to_md.py --title "Timers" --out docs/reference/04-timers.md \
                     --sha <commit> src/Timer_and_Divider_Registers.md ...

O Pan Docs é escrito para o mdBook e usa construções que não sobrevivem a um
`cat` ingênuo:

  {{#bits 8 > "TAC" 2:"Enable" ... }}   macro de diagrama de bits
  {{#include imgs/src/foo.svg:2:}}      SVG embutido
  [texto](<#Alguma Seção>)              link para outra página do livro
  [^nota]                               notas de rodapé, que colidem ao juntar
                                        arquivos diferentes no mesmo documento

Este script resolve cada uma delas. O objetivo é que o arquivo em
docs/reference/ seja legível offline e *sem armadilhas*: se um link não puder
ser resolvido, ele vira texto visível em vez de âncora quebrada silenciosa.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# Versão de página única do Pan Docs: é lá que todas as âncoras internas do
# livro resolvem, então é para lá que os links relativos são reapontados.
SINGLE_PAGE = "https://gbdev.io/pandocs/single.html"


def slugify(text: str) -> str:
    """Reproduz a geração de âncoras do mdBook.

    Minúsculas, descarta tudo que não for [a-z0-9 _-], espaço vira hífen.
    Conferido contra os ids reais de single.html:
        "FF00 — P1/JOYP: Joypad"   -> ff00--p1joyp-joypad
        "FEA0–FEFF range"          -> fea0feff-range
        "INT $40 — VBlank interrupt" -> int-40--vblank-interrupt
    """
    out = []
    for ch in text.lower():
        if ch.isalnum() or ch in "_- ":
            out.append("-" if ch == " " else ch)
    return "".join(out)


def render_bits(raw: str) -> str:
    """`{{#bits 8 > "TAC" 2:"Enable" 1-0:"Clock select" }}` -> tabela."""
    width_m = re.search(r"\{\{#bits\s+(\d+)", raw)
    width = width_m.group(1) if width_m else "8"

    body = re.sub(r"^\{\{#bits\s+\d+\s*[<>]?", "", raw).rstrip("}").strip()

    # O primeiro literal entre aspas, sem `N:` na frente, é o nome do registrador.
    name_m = re.match(r'\s*"([^"]*)"', body)
    name = name_m.group(1) if name_m else ""
    if name_m:
        body = body[name_m.end():]

    fields = re.findall(r'(\d+(?:-\d+)?)\s*:\s*"([^"]*)"', body)
    if not fields:
        return ""

    head = f"**`{name}`** — layout de bits ({width} bits):" if name \
        else f"Layout de bits ({width} bits):"
    rows = "\n".join(f"| {bits} | {label} |" for bits, label in fields)
    return f"{head}\n\n| Bits | Campo |\n|---|---|\n{rows}\n"


def strip_macros(text: str) -> str:
    """Resolve `{{#bits}}`, `{{#include}}` e qualquer outra macro restante."""

    def bits_sub(m: re.Match) -> str:
        return render_bits(m.group(0))

    text = re.sub(r"\{\{#bits\b.*?\}\}", bits_sub, text, flags=re.S)

    def include_sub(m: re.Match) -> str:
        target = m.group(1).split(":")[0]
        return (f"> _Diagrama `{target}` omitido nesta cópia offline "
                f"(era um SVG do Pan Docs). Ver {SINGLE_PAGE}_")

    text = re.sub(r"\{\{#include\s+([^\}]+?)\}\}", include_sub, text)

    # Qualquer macro que sobre vira aviso visível, nunca lixo silencioso.
    text = re.sub(r"\{\{#(\w+)[^\}]*\}\}",
                  lambda m: f"> _(macro mdBook `{m.group(1)}` não convertida)_",
                  text, flags=re.S)
    return text


def rewrite_links(text: str) -> str:
    """Reaponta âncoras internas do livro para a página única online."""
    # Forma [txt](<#Seção Com Espaços>)
    text = re.sub(r"\]\(<#([^>]+)>\)",
                  lambda m: f"]({SINGLE_PAGE}#{slugify(m.group(1))})", text)
    # Forma [txt](#secao)
    text = re.sub(r"\]\(#([^)\s]+)\)",
                  lambda m: f"]({SINGLE_PAGE}#{slugify(m.group(1))})", text)
    return text


def namespace_footnotes(text: str, stem: str) -> str:
    """Prefixa notas de rodapé com o arquivo de origem.

    Sem isso, juntar Audio.md e Audio_details.md no mesmo documento faz duas
    notas `[^1]` diferentes colidirem e uma sobrescrever a outra.
    """
    def ref(m: re.Match) -> str:
        return f"[^{stem}_{m.group(1)}]"
    return re.sub(r"\[\^([^\]]+)\]", ref, text)


def demote_headings(text: str) -> str:
    """Rebaixa todos os títulos um nível: o H1 do tema é do arquivo gerado."""
    out, in_fence = [], False
    for line in text.split("\n"):
        if line.lstrip().startswith("```"):
            in_fence = not in_fence
        if not in_fence and re.match(r"^#{1,5}\s", line):
            line = "#" + line
        out.append(line)
    return "\n".join(out)


def transform(path: Path, sha: str) -> str:
    text = path.read_text(encoding="utf-8")
    stem = path.stem

    # O pré-processador do Pan Docs deixa escapes no fonte. `$` não tem
    # significado em markdown, então `\$8000` é puro ruído visual — some.
    # `\[` e `\]` ficam: ali o escape É significativo e removê-lo criaria
    # referências de link falsas.
    text = text.replace('\\"', '"').replace("\\$", "$")

    text = strip_macros(text)
    text = rewrite_links(text)
    text = namespace_footnotes(text, stem)
    text = re.sub(r'<img[^>]*src="imgs/[^"]*"[^>]*>',
                  "> _(imagem omitida nesta cópia offline)_", text)
    text = demote_headings(text)

    url = f"https://github.com/gbdev/pandocs/blob/{sha}/src/{path.name}"
    return f"<!-- fonte: src/{path.name} @ {sha[:12]} -->\n\n{text.strip()}\n\n" \
           f"_Fonte desta seção: [`src/{path.name}`]({url})_\n"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--title", required=True)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--sha", required=True)
    ap.add_argument("--intro", default="")
    ap.add_argument("sources", nargs="+", type=Path)
    args = ap.parse_args()

    missing = [p for p in args.sources if not p.is_file()]
    if missing:
        print(f"ERRO: fontes ausentes: {', '.join(str(p) for p in missing)}",
              file=sys.stderr)
        return 1

    parts = [
        f"# {args.title}\n",
        "> **Fonte de verdade deste projeto (regra R1 do CLAUDE.md).**",
        "> Cópia do [Pan Docs](https://gbdev.io/pandocs/) (domínio público, CC0),",
        f"> fixada no commit [`{args.sha[:12]}`](https://github.com/gbdev/pandocs/tree/{args.sha}).",
        "> Não editar à mão: regenerado por `scripts/fetch-reference-docs.sh`.\n",
    ]
    if args.intro:
        parts.append(f"{args.intro}\n")

    parts.append("**Nesta página:**\n")
    parts += [f"- {p.stem.replace('_', ' ')}" for p in args.sources]
    parts.append("\n---\n")

    for p in args.sources:
        parts.append(transform(p, args.sha))
        parts.append("\n---\n")

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text("\n".join(parts), encoding="utf-8")
    print(f"  {args.out.name:44} {len(args.sources)} seções, "
          f"{args.out.stat().st_size // 1024} KiB")
    return 0


if __name__ == "__main__":
    sys.exit(main())
