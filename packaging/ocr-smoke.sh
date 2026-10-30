#!/usr/bin/env bash
# OCR smoke test of a packaged pdfcraft-cli: proves the release finds its bundled models (no
# PDFCRAFT_MODELS, no source tree) and reads text. Turns a .txt into a PDF and renders it to a
# picture, so the scan has no text layer and every word below comes from OCR. Then:
#   1. OCRs the whole scan and checks the recognised lines;
#   2. exports an evidence package with one crop per line (and report.docx) and checks that each
#      crop's ocr_text is exactly its own line, in order, with three evidence PNGs.
#
# Usage: packaging/ocr-smoke.sh <path to the packaged pdfcraft-cli or pdfcraft-cli.exe>
# Runs on macOS and on Windows (Git Bash). Exits non-zero when OCR is missing or wrong.
set -euo pipefail

cli="$1"
expected=("PUMP SERIAL 4471" "RATED FLOW 250 M3H" "WITNESSED BY COSTA")
unset PDFCRAFT_MODELS
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cd "$work"

fail() { echo "error: $*" >&2; exit 1; }

printf '%s\n\n\n' "${expected[@]}" >sample.txt
"$cli" batch sample.txt --steps convert --out-dir text >/dev/null
# 100 dpi: the scan is an 850 x 1100 pt page with the lines at about 94, 174 and 220 pt from the top.
"$cli" render text/sample.pdf --page 1 --dpi 100 --out page.png
"$cli" batch page.png --steps convert --out-dir scan >/dev/null
"$cli" batch scan/page.pdf --steps ocr --out-dir ocr | tee batch.json
grep -q '"failed": 0' batch.json || fail "OCR batch failed"
"$cli" text ocr/page.pdf | tr -d '\r' | tee recognised.txt
for line in "${expected[@]}"; do
  grep -qF "$line" recognised.txt || fail "OCR did not read \"$line\""
done

cat >steps.json <<'EOF'
[
  {"tool": "doc_open", "args": {"path": "scan/page.pdf"}},
  {"tool": "doc_export_snapshot_package", "args": {"doc": 1, "out": "package", "word": true, "items": [
    {"page": 1, "rect": [100, 70, 600, 120], "caption": "Serial"},
    {"page": 1, "rect": [100, 150, 600, 200], "caption": "Flow"},
    {"page": 1, "rect": [100, 195, 600, 245], "caption": "Witness"}
  ]}}
]
EOF
"$cli" run --script steps.json --root . >export.log 2>&1 || { cat export.log >&2; fail "package export failed"; }
for f in manifest.json report.tex report.docx report.marker.json source.pdf \
  evidence/item_1.png evidence/item_2.png evidence/item_3.png; do
  [[ -s "package/$f" ]] || fail "package has no $f"
done
tr -d '\r' <package/manifest.json >manifest.json
# The manifest lists item_1..item_3 in order, each followed by its own crop's OCR text.
for n in 1 2 3; do
  want="${expected[$((n - 1))]}"
  item_line="$(grep -n "\"id\": \"item_$n\"" manifest.json | cut -d: -f1 || true)"
  text_line="$(grep -n "\"ocr_text\": \"$want\"" manifest.json | cut -d: -f1 || true)"
  [[ -n "$item_line" && -n "$text_line" ]] || fail "item_$n has no ocr_text \"$want\""
  [[ "$text_line" -gt "$item_line" ]] || fail "item_$n's ocr_text is not its own"
  [[ -z "${last:-}" || "$item_line" -gt "$last" ]] || fail "items are not in order"
  last="$text_line"
done
echo "OCR smoke test passed: $cli"
