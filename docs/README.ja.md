# 翻譯米糕 Migao

[繁體中文](README.zh-TW.md) | **[English](../README.md)** | 日本語 | [한국어](README.ko.md) | [简体中文](README.zh-CN.md)

<div align="center">
  <img src="assets/migao-banner.png" alt="翻譯米糕 Migao" width="100%">
  <br>
  <strong>日本語対応は開発中です。一緒に作りませんか？</strong>
</div>

<br>

> **現在の状態:** 日本語 IME のオタク修正はまだ実装されていません。現在は台湾繁体字中国語（注音符号）とピンインのみ対応しています。
>
> **なぜ日本語が難しいのか？** ローマ字 IME はキー入力がそのまま英字として残るため、逆変換が不可能です（例：`nihao` と入力しても `nihao` のままで、復元する変換がありません）。かな直打ちは技術的には可能ですが、利用者が少ないため優先度が低い状態です。
>
> **貢献の方向性:** かな直打ちルールの実装に興味がある方はぜひ参加ください · [ROADMAP](https://github.com/winterdrive/migao/blob/main/ROADMAP.md) · [Issue で議論](https://github.com/winterdrive/migao/issues)

**Traditional Chinese IME の入力ミスを修正するツールです。**

入力方式を切り替え忘れて `su3cl3` のような文字列を入力してしまった場合、Migao は選択中のテキストをその場で `你好` に戻します。

```
Type:   su3cl3
Select: Ctrl+A
Fix:    Ctrl+Alt+R
Result: 你好
```

## Install

### Windows one-liner

```powershell
irm https://raw.githubusercontent.com/winterdrive/migao/main/install.ps1 | iex
```

`migao` CLI、`migao-watch` tray app、PATH、Start Menu shortcut をインストールし、起動時に自動起動するか確認します。

Uninstall:

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/winterdrive/migao/main/install.ps1))) -Uninstall
```

## Windows Daily Use *(Windows only)*

`migao-watch` is currently Windows-only. macOS / Linux contributions are welcome — open an [Issue](https://github.com/winterdrive/migao/issues) to discuss.
Alternatives: **CLI pipe** `pbpaste | migao fix | pbcopy` (macOS) / `xclip -o | migao fix | xclip -i` (Linux); **AI agent** see [`skills/migao/SKILL.md`](../skills/migao/SKILL.md)

1. Start Menu から **Migao Watch** を起動します。自動起動を有効にしている場合は不要です。
2. 入力方式を切り替え忘れて文字化けしたテキストを選択します。多くのエディタでは **Ctrl+A** で入力欄全体を選択できます。
3. **Ctrl+Alt+R** を押すと、選択したテキストがその場で修正されます。
4. 3 秒以内にもう一度 **Ctrl+Alt+R** を押すと、候補を切り替えられます。

手動起動しても PowerShell window を開いたままにする必要はありません。

## CLI

```sh
migao fix "su3cl3"           # 你好
migao fix "5j/ eji6"         # 中國
migao fix --top 3 "su3cl3"   # multiple candidates
migao list                   # supported IMEs
```

## Supported IMEs

| Identifier | Alias | Layout |
|------------|-------|--------|
| `bopomofo-daqian` | `zhuyin`, `注音` | Standard Daqian Bopomofo keyboard |
| `pinyin` | `拼音` | Full pinyin QWERTY |

## License

MIT AND Apache-2.0

The Apache 2.0 component covers the RIME dictionary data embedded in the binary.
