#!/usr/bin/env python3
"""
Accuracy regression gate for migao.

Runs `migao fix` on every entry in tests/accuracy/golden.tsv and checks
that at least THRESHOLD% of inputs produce the expected output exactly.

Usage:
    python scripts/accuracy_gate.py [--bin path/to/migao] [--threshold 90]

Exit codes:
    0  accuracy >= threshold
    1  accuracy below threshold or binary not found
"""

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent
GOLDEN = ROOT / "tests" / "accuracy" / "golden.tsv"
DEFAULT_BIN = ROOT / "target" / "release" / "migao"


def load_golden(path: Path) -> list[tuple[str, str]]:
    cases = []
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t", 1)
        if len(parts) == 2:
            cases.append((parts[0], parts[1]))
    return cases


def run_migao(binary: Path, inp: str) -> str:
    result = subprocess.run(
        [str(binary), "fix", inp],
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    return result.stdout.strip()


def main() -> None:
    parser = argparse.ArgumentParser(description="Kotori accuracy regression gate")
    parser.add_argument(
        "--bin",
        default=str(DEFAULT_BIN),
        help="Path to migao binary (default: target/release/migao)",
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=90.0,
        help="Minimum required accuracy %% (default: 90)",
    )
    args = parser.parse_args()

    binary = Path(args.bin)
    if sys.platform == "win32" and binary.suffix == "":
        binary = binary.with_suffix(".exe")

    if not binary.exists():
        print(f"ERROR: binary not found: {binary}", file=sys.stderr)
        print("Build with: cargo build --release", file=sys.stderr)
        sys.exit(1)

    cases = load_golden(GOLDEN)
    if not cases:
        print("ERROR: golden set is empty", file=sys.stderr)
        sys.exit(1)

    passed: list[str] = []
    failed: list[tuple[str, str, str]] = []

    for inp, expected in cases:
        actual = run_migao(binary, inp)
        if actual == expected:
            passed.append(inp)
        else:
            failed.append((inp, expected, actual))

    total = len(cases)
    n_pass = len(passed)
    pct = 100.0 * n_pass / total

    if failed:
        print(f"FAILURES ({len(failed)}/{total}):")
        for inp, exp, act in failed:
            print(f"  input:    {inp}")
            print(f"  expected: {exp}")
            print(f"  actual:   {act}")
        print()

    status = "PASS" if pct >= args.threshold else "FAIL"
    print(
        f"{status}  {n_pass}/{total} = {pct:.1f}%"
        f"  (threshold: {args.threshold:.0f}%)"
    )

    if pct < args.threshold:
        sys.exit(1)


if __name__ == "__main__":
    main()
