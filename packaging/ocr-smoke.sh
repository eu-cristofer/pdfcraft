#!/usr/bin/env bash
# OCR smoke test of a packaged pdfcraft-cli: proves the release finds its bundled models (no
# PDFCRAFT_MODELS, no source tree) and reads text. Turns a .txt into a PDF, renders it to a
# picture, OCRs the picture and checks the recognised words.
#
# Usage: packaging/ocr-smoke.sh <path to the packaged pdfcraft-cli or pdfcraft-cli.exe>
# Runs on macOS and on Windows (Git Bash). Exits non-zero when OCR is missing or wrong.
set -euo pipefail

cli="$1"
expected=("PUMP PERFORMANCE TEST" "COSTA INSPECTION OCR CHECK")
unset PDFCRAFT_MODELS
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cd "$work"

printf '%s\n' "${expected[@]}" >sample.txt
"$cli" batch sample.txt --steps convert --out-dir text >/dev/null
"$cli" render text/sample.pdf --page 1 --dpi 200 --out page.png
# The picture has no text layer, so every word below comes from OCR.
"$cli" batch page.png --steps convert,ocr --out-dir scan | tee batch.json
grep -q '"failed": 0' batch.json || { echo "error: OCR batch failed" >&2; exit 1; }

"$cli" text scan/page.pdf | tr -d '\r' | tee recognised.txt
for line in "${expected[@]}"; do
  grep -qF "$line" recognised.txt || { echo "error: OCR did not read \"$line\"" >&2; exit 1; }
done
echo "OCR smoke test passed: $cli"
