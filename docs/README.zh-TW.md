# 翻譯米糕 Migao

繁體中文 | **[English](../README.md)** | [日本語](README.ja.md) | [한국어](README.ko.md) | [简体中文](README.zh-CN.md)

<div align="center">
  <img src="assets/migao-banner.png" alt="翻譯米糕 Migao" width="100%">
  <br>
  <strong>忘了切換注音？翻譯米糕 Ctrl+Alt+R 一秒還原。</strong>
</div>

<br>

**注音 / 拼音 IME 亂碼自動修正工具。**

忘記切換輸入法，打出一串看不懂的英文字？`su3cl3` 其實是「你好」，`rup wu0 wu0 fu4cp3cl3` 是「今天天氣很好」。翻譯米糕把它們還原回來——一個快捷鍵，就地替換，不需要重打。

```
$ migao fix "su3cl3"
你好

$ migao fix "rup wu0 wu0 fu4cp3cl3"
今天天氣很好
```

---

## 安裝

### Windows — 一鍵安裝（推薦）

在 PowerShell 中執行：

```powershell
irm https://raw.githubusercontent.com/winterdrive/migao/main/install.ps1 | iex
```

同時安裝 `migao`（CLI 工具）與 `migao-watch`（背景常駐程式），加入 PATH，建立開始選單捷徑，並詢問是否要開機自動啟動。

解除安裝：

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/winterdrive/migao/main/install.ps1))) -Uninstall
```

### Cargo

```sh
cargo install migao
```

### 從原始碼編譯

```sh
git clone https://github.com/winterdrive/migao.git
cd Migao
cargo build --release
# 二進位檔：target/release/migao  target/release/migao-watch
```

---

## Windows 常駐程式 — `migao-watch`

`migao-watch` 靜默執行於系統匣，在任何 Windows 應用程式中，按一個快捷鍵即可就地修正亂碼。

### 使用流程

1. 從開始選單啟動 **Migao Watch**（或設定開機自動啟動）
2. 照常打字——如果忘記切換輸入法，打出了亂碼
3. **Ctrl+A** 選取全文
4. **Ctrl+Alt+R** ── 亂碼立刻替換為正確中文
5. **3 秒內再按一次** ── 循環切換備選答案

手動啟動不需要保持 PowerShell 視窗開啟。

### 系統匣圖示

工作列右下角出現米糕像素圖示代表正在運行：

| 圖示 | 狀態 |
|------|------|
| 彩色米糕 | 運行中，快捷鍵已啟用 |
| 灰色米糕 | 已暫停，快捷鍵停用中 |

右鍵選單：

- **Pause / Resume** — 暫停或恢復快捷鍵，不需要退出程式
- **Launch at Login** — 設定開機自動啟動（寫入 Windows 登錄檔）
- **Exit** — 清理資源後完全退出

### 系統匣 Tooltip 回饋

每次修正後，Tooltip 短暫顯示修改結果：

```text
✓  su3cl3 → 你好
```

循環切換備選時：

```
Candidate 2/3: 你好嗎
Reverted to original
```

---

## CLI 工具 — `migao`

### 直接修正亂碼

```sh
migao fix "su3cl3"           # 你好
migao fix "5j/ eji6"         # 中國
migao fix "ji3vu j;4su3dk3"  # 我希望你可以
```

### 從剪貼簿或 stdin 輸入

```sh
echo "su3cl3" | migao fix
```

### 顯示多個候選

```sh
migao fix --top 3 "su3cl3"
# 1  你好
# 2  你號
# 3  你好嗎
# Pick [1-3] (default 1):
```

### 指定輸入法

```sh
migao fix --ime zhuyin "su3cl3"
migao fix --ime pinyin "nihao"
```

### 列出支援的輸入法

```sh
migao list
```

```text
Supported IME identifiers:
  bopomofo-daqian  (aliases: zhuyin, 注音)  — 大千標準注音鍵盤
  pinyin           (alias: 拼音)             — 全拼（標準 QWERTY）
```

---

## 運作原理

```
原始按鍵序列
      │
      ▼
  分段（Segment）    依大千注音規則拆分音節候選（聲母→介音→韻母→聲調）
      │               非注音字符原樣輸出
      ▼
  解碼（Decode）     每個音節 → 注音 Unicode（su3 → ㄋㄧˇ）
      │
      ▼
  Viterbi 解碼       動態規劃找出機率最高的詞序列
      │               使用 474,000 詞條字典 + Bigram 語言模型
      ▼
  輸出繁體中文
```

**信心分數**（結構合法注音音節的比例）作為門檻過濾——英文文字分數接近 0.0，自動拒絕。

---

## 支援的輸入法

| 識別碼 | 別名 | 鍵盤配置 |
|--------|------|---------|
| `bopomofo-daqian` | `zhuyin`、`注音` | 大千標準注音鍵盤 |
| `pinyin` | `拼音` | 全拼 QWERTY |

---

## 字典來源

| 來源 | 授權 |
|------|------|
| [RIME terra_pinyin](https://github.com/rime/rime-terra-pinyin) | Apache 2.0 |
| [rime-essay](https://github.com/rime/rime-essay) | Apache 2.0 |
| [pypinyin](https://github.com/mozillazg/python-pinyin) | MIT |

---

## 授權

MIT AND Apache-2.0

Apache 2.0 部分涵蓋嵌入 binary 中的 RIME 字典資料（`data/bopomofo.tsv`）。
