#!/usr/bin/env python3
"""Generate valid pinyin syllable list for src/ime/pinyin.rs"""
import pypinyin
import re

syllables = set()
for cp in range(0x4E00, 0x9FFF):
    result = pypinyin.pinyin(chr(cp), style=pypinyin.Style.NORMAL, heteronym=True)
    if result and result[0]:
        for py in result[0]:
            py = re.sub(r"\d", "", py.strip()).lower()
            if py and py.isascii():
                syllables.add(py)

# longest-first so greedy segmenter matches "zhuang" before "zh"
s = sorted(syllables, key=lambda x: (-len(x), x))

print(f"// {len(s)} valid Mandarin pinyin syllables (tone-less, longest-first)")
print("pub const VALID_SYLLABLES: &[&str] = &[")
row = []
for py in s:
    row.append(f'"{py}"')
    if len(row) == 8:
        print("    " + ", ".join(row) + ",")
        row = []
if row:
    print("    " + ", ".join(row) + ",")
print("];")
print(f"\n// generated from pypinyin over U+4E00..U+9FFF", flush=True)
