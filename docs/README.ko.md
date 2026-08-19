# 翻譯米糕 Migao

[繁體中文](README.zh-TW.md) | **[English](../README.md)** | [日本語](README.ja.md) | 한국어 | [简体中文](README.zh-CN.md)

<div align="center">
  <img src="assets/migao-banner.png" alt="翻譯米糕 Migao" width="100%">
  <br>
  <strong>한국어(두벌식) 지원 개발 중 — 기여자를 찾습니다!</strong>
</div>

<br>

> **현재 상태:** 한국어 두벌식 IME 오타 복구는 아직 구현되지 않았습니다. 현재는 대만 번체 중국어(주음부호 / 보포모포)와 병음만 지원합니다.
>
> **왜 한국어가 다음 목표인가?** 두벌식은 각 키가 자음 또는 모음에 1:1로 대응하는 결정론적 매핑 구조로, 주음부호와 동일한 방식으로 역변환이 가능합니다. `gksrnrdl` 같은 오타도 구조적으로 영어와 구분되어 신뢰도 높은 탐지가 가능합니다.
>
> **기여 방법:** `src/rules/korean_dubeolsik.rs` 추가 (키 테이블 ~40개 + 유니코드 한글 조합 알고리즘) · [ROADMAP](https://github.com/winterdrive/migao/blob/main/ROADMAP.md) · [이슈 참여](https://github.com/winterdrive/migao/issues)

**Traditional Chinese IME 입력 실수를 복구하는 도구입니다.**

입력기를 바꾸지 않은 상태에서 `su3cl3` 같은 문자열을 입력했다면, Migao는 선택한 텍스트를 그 자리에서 `你好`로 되돌립니다.

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

`migao` CLI, `migao-watch` tray app, PATH, Start Menu shortcut을 설치하고 시작 시 자동 실행 여부를 묻습니다.

Uninstall:

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/winterdrive/migao/main/install.ps1))) -Uninstall
```

## Windows Daily Use *(Windows only)*

`migao-watch` is currently Windows-only. macOS / Linux contributions are welcome — open an [Issue](https://github.com/winterdrive/migao/issues) to discuss.
Alternatives: **CLI pipe** `pbpaste | migao fix | pbcopy` (macOS) / `xclip -o | migao fix | xclip -i` (Linux); **AI agent** see [`skills/migao/SKILL.md`](../skills/migao/SKILL.md)

1. Start Menu에서 **Migao Watch**를 실행합니다. 자동 실행을 켜 둔 경우에는 따로 실행할 필요가 없습니다.
2. 입력기를 바꾸지 않아 잘못 입력된 텍스트를 선택합니다. 많은 에디터에서는 **Ctrl+A**로 입력 칸 전체를 선택할 수 있습니다.
3. **Ctrl+Alt+R**을 누르면 선택한 텍스트가 올바른 중국어로 바로 바뀝니다.
4. 3초 안에 **Ctrl+Alt+R**을 다시 누르면 후보를 순환할 수 있습니다.

수동으로 실행해도 PowerShell window를 계속 열어 둘 필요는 없습니다.

### 트레이 아이콘

작업 표시줄 알림 영역에 픽셀 아트 떡 아이콘이 나타납니다:

| 아이콘 | 의미 |
|------|------|
| 컬러 | 활성 — 단축키 대기 중 |
| 회색 | 일시정지 — 단축키 비활성화 |

우클릭하면 컨텍스트 메뉴가 나타납니다:

- **Pause / Resume** — 종료하지 않고 단축키를 일시정지·재개
- **Launch at Login** — Windows 자동 시작 전환（`HKCU\...\Run`에 기록）
- **Open Migao Command** — `migao status`를 실행하는 터미널 열기
- **Check for Updates** — GitHub에서 새 릴리스를 확인하고 있으면 바로 설치할지 묻는다（확인 자체가 실패하면 릴리스 페이지를 연다）. 시작 시에도 한 번 조용히 확인하며 업데이트가 있을 때만 알린다
- **Report Issue** — GitHub 이슈 양식 열기
- **Exit** — 단축키를 해제하고 트레이 아이콘을 깔끔하게 제거

### 트레이 툴팁 피드백

수정할 때마다 툴팁에 변경 내용이 잠시 표시됩니다:

```text
✓  su3cl3 → 你好
```

후보를 순환하는 동안:

```text
Candidate 2/3: 你好嗎
Reverted to original
```

## CLI

```sh
migao fix "su3cl3"           # 你好
migao fix "5j/ eji6"         # 中國
migao fix --top 3 "su3cl3"   # multiple candidates
migao list                   # supported IMEs
migao status                 # version, config path, watcher status
migao config                 # current config
migao config set hotkey Ctrl+Alt+K
migao watch autostart on     # enable Launch at Login
migao watch autostart off    # disable Launch at Login
migao report "1p4w1" "版本"  # record a correction Migao got wrong
```

Hotkey configuration currently supports `Ctrl+Alt+<A-Z>` and is stored in `%APPDATA%\Migao\config.toml` on Windows.

## Supported IMEs

| Identifier | Alias | Layout |
|------------|-------|--------|
| `bopomofo-daqian` | `zhuyin`, `注音` | Standard Daqian Bopomofo keyboard |
| `pinyin` | `拼音` | Full pinyin QWERTY |

## License

MIT AND Apache-2.0

The Apache 2.0 component covers the RIME dictionary data embedded in the binary.
