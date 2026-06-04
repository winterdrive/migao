#!/usr/bin/env bash
# smoke.sh — representative CLI smoke test for migao
# Run from repo root: bash .agents/skills/test-smoke/smoke.sh [binary]
# Exit 0 = all pass, 1 = any failure.
set -uo pipefail

BIN="${1:-./target/debug/migao}"
pass=0; fail=0

check() {
    local label="$1" expected="$2"; shift 2
    local got
    got=$("$BIN" "$@" 2>&1) || true
    if [ "$got" = "$expected" ]; then
        echo "  PASS  $label"; ((pass++))
    else
        echo "  FAIL  $label"
        echo "        expected: [$expected]"
        echo "        got:      [$got]"
        ((fail++))
    fi
}

echo ""
echo "migao CLI smoke test  ($BIN)"
echo "================================="

# Bopomofo correction
check "su3cl3 → 你好"              "你好"       fix "su3cl3"
check "sentence → 今天天氣很好"    "今天天氣很好" fix "rup wu0 wu0 fu4cp3cl3"

# Pipe / stdin mode
got=$(echo "su3cl3" | "$BIN" fix 2>&1) || true
[ "$got" = "你好" ] && { echo "  PASS  pipe su3cl3 → 你好"; ((pass++)); } \
                    || { echo "  FAIL  pipe: got [$got]"; ((fail++)); }

# Top-N (first candidate)
got=$("$BIN" fix --top 3 "su3cl3" 2>&1 | head -1) || true
[ "$got" = "你好" ] && { echo "  PASS  --top 3 first = 你好"; ((pass++)); } \
                     || { echo "  FAIL  --top 3 first: got [$got]"; ((fail++)); }

# Explicit IME flags
check "--ime pinyin nihao → 你好"  "你好"  fix --ime pinyin "nihao"
check "--ime zhuyin su3cl3 → 你好" "你好"  fix --ime zhuyin "su3cl3"

# Unrecognised input → exit 1
"$BIN" fix "hello world" >/dev/null 2>&1; code=$?
[ "$code" = "1" ] && { echo "  PASS  plain english → exit 1"; ((pass++)); } \
                  || { echo "  FAIL  plain english: exit $code (want 1)"; ((fail++)); }

# list command
"$BIN" list 2>&1 | grep -q "bopomofo-daqian" \
    && { echo "  PASS  list shows bopomofo-daqian"; ((pass++)); } \
    || { echo "  FAIL  list missing bopomofo-daqian"; ((fail++)); }

echo ""
echo "================================="
total=$((pass + fail))
if [ "$fail" -eq 0 ]; then
    echo "PASS  $pass/$total"; exit 0
else
    echo "FAIL  $pass/$total passed, $fail failed"; exit 1
fi
