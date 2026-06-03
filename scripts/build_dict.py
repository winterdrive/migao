#!/usr/bin/env python3
"""
Build data/bopomofo.tsv from RIME terra-pinyin + pypinyin.

Sources:
  terra_pinyin.dict.yaml  - character pronunciations + frequencies (Apache 2.0)
  rime-essay/essay.txt    - word frequency list (Apache 2.0)

Both from https://github.com/rime/

Requirements:
    pip install pypinyin

Usage:
    python scripts/build_dict.py [--output data/bopomofo.tsv] [--clear-cache]
"""

import argparse
import sys
import urllib.request
from pathlib import Path

try:
    from pypinyin import pinyin, Style
    from pypinyin.contrib.tone_convert import to_tone as _to_tone
    try:
        from pypinyin.phonetic_symbol import phonetic_symbol as _PS
        _PY_TO_BOPO = {v: k for k, v in _PS.items()}
    except ImportError:
        _PY_TO_BOPO = {}
except ImportError:
    print("ERROR: pypinyin not installed. Run: pip install pypinyin", file=sys.stderr)
    sys.exit(1)


def terra_py_to_bopomofo(py_str: str) -> str:
    """Convert a terra_pinyin string like 'wei2' to Bopomofo 'ㄨㄟˊ'."""
    return _PY_TO_BOPO.get(_to_tone(py_str), "")

CACHE_DIR = Path(__file__).parent / "cache"

TERRA_URL = (
    "https://raw.githubusercontent.com/rime/rime-terra-pinyin/master/terra_pinyin.dict.yaml"
)
ESSAY_URL = (
    "https://raw.githubusercontent.com/rime/rime-essay/master/essay.txt"
)


# ── Fetch / cache ────────────────────────────────────────────────────────────

def fetch(url: str, cache_name: str) -> str:
    cache_file = CACHE_DIR / cache_name
    if cache_file.exists():
        print(f"cache hit: {cache_name}", file=sys.stderr)
        return cache_file.read_text(encoding="utf-8")
    print(f"downloading {url} ...", file=sys.stderr)
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    with urllib.request.urlopen(url, timeout=30) as r:
        data = r.read().decode("utf-8")
    cache_file.write_text(data, encoding="utf-8")
    return data


# ── Parse terra_pinyin: char → {pinyin: weight} ──────────────────────────────

def parse_terra(data: str) -> dict[str, tuple[str, int]]:
    """Return {char: (best_pinyin, weight)} for single-character entries."""
    in_body = False
    chars: dict[str, tuple[str, int]] = {}

    for line in data.splitlines():
        if not in_body:
            if line.strip() == "...":
                in_body = True
            continue
        line = line.rstrip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) < 2:
            continue
        word, py = parts[0], parts[1]
        # skip multi-char entries
        if len(word) != 1:
            continue
        weight_str = parts[2].rstrip("%") if len(parts) >= 3 else "100"
        try:
            weight = int(weight_str)
        except ValueError:
            weight = 1
        # keep entry with highest weight
        existing = chars.get(word)
        if existing is None or weight > existing[1]:
            chars[word] = (py, weight)
    return chars


# ── Parse essay.txt: word → frequency ────────────────────────────────────────

def parse_essay(data: str) -> dict[str, int]:
    """Return {word: frequency} from rime-essay."""
    words: dict[str, int] = {}
    for line in data.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        word = parts[0]
        freq = int(parts[1]) if len(parts) >= 2 and parts[1].isdigit() else 1
        words[word] = freq
    return words


# ── Pinyin → Bopomofo via pypinyin ───────────────────────────────────────────

def pinyin_to_bopomofo(char: str) -> str:
    """Convert a single Chinese character to its primary Bopomofo reading via pypinyin."""
    parts = pinyin(char, style=Style.BOPOMOFO, heteronym=False)
    if parts and parts[0]:
        return parts[0][0]
    return ""


def all_bopomofo_readings(char: str) -> list[str]:
    """Return all Bopomofo readings for a character (heteronyms included)."""
    parts = pinyin(char, style=Style.BOPOMOFO, heteronym=True)
    if parts and parts[0]:
        return [r for r in parts[0] if r]
    return []


def word_to_bopomofo(word: str) -> str:
    """Convert a multi-character Chinese word to its Bopomofo string."""
    parts = pinyin(word, style=Style.BOPOMOFO, heteronym=False)
    return "".join(p[0] for p in parts if p)


# ── Build entries ─────────────────────────────────────────────────────────────

def build_entries(char_map: dict, essay_words: dict) -> list[tuple[str, str, int]]:
    entries: list[tuple[str, str, int]] = []

    # Single characters: primary reading gets full essay boost;
    # secondary readings (heteronyms) get only base freq.
    #
    # Why: a common character like 還 (ㄏㄞˊ, very high freq) also has a
    # secondary reading ㄏㄨㄢˊ. Without this split, 還 would overpower 環
    # under ㄏㄨㄢˊ despite 環 being the intended character for that reading.
    for char, (py, weight) in char_map.items():
        base = max(1, weight * 10)
        essay_freq = essay_words.get(char, 0)
        full_freq = base + essay_freq * 100
        primary_bopo = pinyin_to_bopomofo(char)  # terra_pinyin's primary reading

        # terra_py_to_bopomofo uses terra_pinyin's own py string (e.g. 'wei2')
        # which is more accurate than pypinyin's context-free guess for heteronyms
        # like 為(ㄨㄟˊ primary in terra, but ㄨㄟˋ in pypinyin default).
        terra_primary = terra_py_to_bopomofo(py)
        effective_primary = terra_primary if terra_primary else primary_bopo

        for bopo in all_bopomofo_readings(char):
            if not bopo:
                continue
            if bopo == effective_primary:
                entries.append((bopo, char, full_freq))
            else:
                entries.append((bopo, char, base))  # secondary: no essay boost

    # 一 tone-sandhi alias: in spoken Mandarin, 一 becomes ㄧˋ before
    # 1st/2nd/3rd tone syllables. The IME user types the sandhi form,
    # so we add ㄧˋ → 一 with the same high frequency as ㄧ → 一.
    yi_freq = essay_words.get('一', 0)
    if yi_freq > 0:
        entries.append(('ㄧˋ', '一', yi_freq * 100))

    # Multi-character words from essay.
    # Multiply by 100 to normalize compounds to the same frequency scale as single
    # characters (which receive base + essay_freq * 100). Without this, Viterbi
    # log-prob scoring would always prefer individual high-frequency chars over
    # any compound, breaking words like 今天, 知道, 中國.
    for word, freq in essay_words.items():
        if len(word) < 2:
            continue
        try:
            bopo = word_to_bopomofo(word)
        except Exception:
            continue
        if bopo:
            entries.append((bopo, word, freq * 100))

    return entries


# ── Output ────────────────────────────────────────────────────────────────────

HEADER = """\
# 注音 → 繁體中文字典
# Sources:
#   terra_pinyin  (https://github.com/rime/rime-terra-pinyin) Apache 2.0
#   rime-essay    (https://github.com/rime/rime-essay)        Apache 2.0
# Phonetic conversion: pypinyin (MIT)
# Format: Bopomofo<TAB>word<TAB>frequency
# Generated by scripts/build_dict.py — do not edit manually.
"""


def write_tsv(entries: list, output_path: str) -> None:
    lines = [HEADER]
    for bopo, word, freq in sorted(entries, key=lambda e: -e[2]):
        lines.append(f"{bopo}\t{word}\t{freq}")
    Path(output_path).write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {len(entries):,} entries → {output_path}", file=sys.stderr)


# ── CLI ───────────────────────────────────────────────────────────────────────

def main() -> None:
    parser = argparse.ArgumentParser(description="Build Kotori Bopomofo dictionary")
    parser.add_argument(
        "--output",
        default=str(Path(__file__).parent.parent / "data" / "bopomofo.tsv"),
    )
    parser.add_argument("--clear-cache", action="store_true")
    args = parser.parse_args()

    if args.clear_cache:
        for f in CACHE_DIR.glob("*"):
            f.unlink()
        print("cache cleared", file=sys.stderr)

    terra_data = fetch(TERRA_URL, "terra_pinyin.dict.yaml")
    essay_data = fetch(ESSAY_URL, "essay.txt")

    char_map = parse_terra(terra_data)
    essay_words = parse_essay(essay_data)

    print(f"chars: {len(char_map):,}  essay words: {len(essay_words):,}", file=sys.stderr)

    entries = build_entries(char_map, essay_words)

    if not entries:
        print("ERROR: no entries generated", file=sys.stderr)
        sys.exit(1)

    write_tsv(entries, args.output)


if __name__ == "__main__":
    main()
