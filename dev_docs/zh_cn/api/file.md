# file 库

## 基本库说明

`file` 提供异步文件读写与目录枚举。

---

## 目录

### 常量

| 常量名 | 说明 | 索引 |
| ----- | ---- | ---- |
| `AUTO` | 自动检测模式 | [AUTO](#AUTO) |
| `ALL` | 全部统一模式 | [ALL](#ALL) |
| `CR` | 回车换行符 | [CR](#CR) |
| `LF` | 换行符 | [LF](#LF) |
| `CRLF` | 回车换行符组合 | [CRLF](#CRLF) |
| `UTF_8` | UTF-8 编码 | [UTF_8](#UTF_8) |
| `UTF_16LE` | UTF-16 小端编码 | [UTF_16LE](#UTF_16LE) |
| `UTF_16BE` | UTF-16 大端编码 | [UTF_16BE](#UTF_16BE) |
| `GBK` | GBK 编码（简体中文） | [GBK](#GBK) |
| `GB18030` | GB18030 编码（简体中文） | [GB18030](#GB18030) |
| `BIG5` | BIG5 编码（繁体中文） | [BIG5](#BIG5) |
| `SHIFT_JIS` | Shift JIS 编码（日文） | [SHIFT_JIS](#SHIFT_JIS) |
| `EUC_JP` | EUC-JP 编码（日文） | [EUC_JP](#EUC_JP) |
| `ISO_2022_JP` | ISO-2022-JP 编码（日文） | [ISO_2022_JP](#ISO_2022_JP) |
| `EUC_KR` | EUC-KR 编码（韩文） | [EUC_KR](#EUC_KR) |
| `WINDOWS_874` | Windows-874 编码（泰文） | [WINDOWS_874](#WINDOWS_874) |
| `WINDOWS_1250` | Windows-1250 编码（中欧） | [WINDOWS_1250](#WINDOWS_1250) |
| `WINDOWS_1251` | Windows-1251 编码（西里尔） | [WINDOWS_1251](#WINDOWS_1251) |
| `WINDOWS_1252` | Windows-1252 编码（西欧） | [WINDOWS_1252](#WINDOWS_1252) |
| `WINDOWS_1253` | Windows-1253 编码（希腊） | [WINDOWS_1253](#WINDOWS_1253) |
| `WINDOWS_1254` | Windows-1254 编码（土耳其） | [WINDOWS_1254](#WINDOWS_1254) |
| `WINDOWS_1255` | Windows-1255 编码（希伯来） | [WINDOWS_1255](#WINDOWS_1255) |
| `WINDOWS_1256` | Windows-1256 编码（阿拉伯） | [WINDOWS_1256](#WINDOWS_1256) |
| `WINDOWS_1257` | Windows-1257 编码（波罗的海） | [WINDOWS_1257](#WINDOWS_1257) |
| `WINDOWS_1258` | Windows-1258 编码（越南） | [WINDOWS_1258](#WINDOWS_1258) |
| `ISO_8859_2` | ISO-8859-2 编码（中欧） | [ISO_8859_2](#ISO_8859_2) |
| `ISO_8859_3` | ISO-8859-3 编码（南欧） | [ISO_8859_3](#ISO_8859_3) |
| `ISO_8859_4` | ISO-8859-4 编码（北欧） | [ISO_8859_4](#ISO_8859_4) |
| `ISO_8859_5` | ISO-8859-5 编码（西里尔） | [ISO_8859_5](#ISO_8859_5) |
| `ISO_8859_6` | ISO-8859-6 编码（阿拉伯） | [ISO_8859_6](#ISO_8859_6) |
| `ISO_8859_7` | ISO-8859-7 编码（希腊） | [ISO_8859_7](#ISO_8859_7) |
| `ISO_8859_8` | ISO-8859-8 编码（希伯来） | [ISO_8859_8](#ISO_8859_8) |
| `ISO_8859_8_I` | ISO-8859-8-I 编码（希伯来，逻辑顺序） | [ISO_8859_8_I](#ISO_8859_8_I) |
| `ISO_8859_9` | ISO-8859-9 编码（土耳其） | [ISO_8859_9](#ISO_8859_9) |
| `ISO_8859_10` | ISO-8859-10 编码（北欧） | [ISO_8859_10](#ISO_8859_10) |
| `ISO_8859_11` | ISO-8859-11 编码（泰文） | [ISO_8859_11](#ISO_8859_11) |
| `ISO_8859_13` | ISO-8859-13 编码（波罗的海） | [ISO_8859_13](#ISO_8859_13) |
| `ISO_8859_14` | ISO-8859-14 编码（凯尔特） | [ISO_8859_14](#ISO_8859_14) |
| `ISO_8859_15` | ISO-8859-15 编码（西欧） | [ISO_8859_15](#ISO_8859_15) |
| `ISO_8859_16` | ISO-8859-16 编码（东南欧） | [ISO_8859_16](#ISO_8859_16) |
| `KOI8_R` | KOI8-R 编码（俄文） | [KOI8_R](#KOI8_R) |
| `KOI8_U` | KOI8-U 编码（乌克兰） | [KOI8_U](#KOI8_U) |
| `IBM866` | IBM866 编码（俄文） | [IBM866](#IBM866) |
| `MACINTOSH` | Macintosh 编码（西欧） | [MACINTOSH](#MACINTOSH) |
| `X_MAC_CYRILLIC` | x-mac-cyrillic 编码（西里尔） | [X_MAC_CYRILLIC](#X_MAC_CYRILLIC) |

### 方法

| 方法名 | 说明 |
| ------ | ---- |
| `read` | 读取文本文件 |
| `write` | 写入文本文件 |
| `list_dir` | 枚举目录 |

---

## 常量

## `AUTO`

自动检测模式。

**可用于**

- 编码/换行参数

### 调用

```lua
file.AUTO
```

---

## `ALL`

全部统一模式。

**可用于**

- 换行参数

### 调用

```lua
file.ALL
```

---

## `CR`

回车换行符。

**可用于**

- 换行参数

### 调用

```lua
file.CR
```

---

## `LF`

换行符。

**可用于**

- 换行参数

### 调用

```lua
file.LF
```

---

## `CRLF`

回车换行符组合。

**可用于**

- 换行参数

### 调用

```lua
file.CRLF
```
---

## `UTF_8`

UTF-8 编码。

**可用于**

- 编码参数

### 调用

```lua
file.UTF_8
```

---

## `UTF_16LE`

UTF-16 小端编码。

**可用于**

- 编码参数

### 调用

```lua
file.UTF_16LE
```

---

## `UTF_16BE`

UTF-16 大端编码。

**可用于**

- 编码参数

### 调用

```lua
file.UTF_16BE
```

---

## `GBK` - `GB18030`

GBK 编码（简体中文）。

**可用于**

- 编码参数

### 调用

```lua
file.GBK
```

---

## `GB18030`

GB18030 编码（简体中文）。

**可用于**

- 编码参数

### 调用

```lua
file.GB18030
```

---

## `BIG5`

BIG5 编码（繁体中文）。

**可用于**

- 编码参数

### 调用

```lua
file.BIG5
```

---

## `SHIFT_JIS`

Shift JIS 编码（日文）。

**可用于**

- 编码参数

### 调用

```lua
file.SHIFT_JIS
```

---

## `EUC_JP`

EUC-JP 编码（日文）。

**可用于**

- 编码参数

### 调用

```lua
file.EUC_JP
```

---

## `ISO_2022_JP`

ISO-2022-JP 编码（日文）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_2022_JP
```

---

## `EUC_KR`

EUC-KR 编码（韩文）。

**可用于**

- 编码参数

### 调用

```lua
file.EUC_KR
```

---

## `WINDOWS_874`

Windows-874 编码（泰文）。

**可用于**

- 编码参数

### 调用

```lua
file.WINDOWS_874
```

---

## `WINDOWS_1250`

Windows-1250 编码（中欧）。

**可用于**

- 编码参数

### 调用

```lua
file.WINDOWS_1250
```

---

## `WINDOWS_1251`

Windows-1251 编码（西里尔）。

**可用于**

- 编码参数

### 调用

```lua
file.WINDOWS_1251
```

---

## `WINDOWS_1252`

Windows-1252 编码（西欧）。

**可用于**

- 编码参数

### 调用

```lua
file.WINDOWS_1252
```

---

## `WINDOWS_1253`

Windows-1253 编码（希腊）。

**可用于**

- 编码参数

### 调用

```lua
file.WINDOWS_1253
```

---

## `WINDOWS_1254`

Windows-1254 编码（土耳其）。

**可用于**

- 编码参数

### 调用

```lua
file.WINDOWS_1254
```

---

## `WINDOWS_1255`

Windows-1255 编码（希伯来）。

**可用于**

- 编码参数

### 调用

```lua
file.WINDOWS_1255
```

---

## `WINDOWS_1256`

Windows-1256 编码（阿拉伯）。

**可用于**

- 编码参数

### 调用

```lua
file.WINDOWS_1256
```

---

## `WINDOWS_1257`

Windows-1257 编码（波罗的海）。

**可用于**

- 编码参数

### 调用

```lua
file.WINDOWS_1257
```

---

## `WINDOWS_1258`

Windows-1258 编码（越南）。

**可用于**

- 编码参数

### 调用

```lua
file.WINDOWS_1258
```

---

## `ISO_8859_2`

ISO-8859-2 编码（中欧）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_2
```

---

## `ISO_8859_3`

ISO-8859-3 编码（南欧）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_3
```

---

## `ISO_8859_4`

ISO-8859-4 编码（北欧）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_4
```

---

## `ISO_8859_5`

ISO-8859-5 编码（西里尔）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_5
```

---

## `ISO_8859_6`

ISO-8859-6 编码（阿拉伯）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_6
```

---

## `ISO_8859_7`

ISO-8859-7 编码（希腊）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_7
```

---

## `ISO_8859_8`

ISO-8859-8 编码（希伯来）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_8
```

---

## `ISO_8859_8_I`

ISO-8859-8-I 编码（希伯来，逻辑顺序）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_8_I
```

---

## `ISO_8859_9`

ISO-8859-9 编码（土耳其）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_9
```

---

## `ISO_8859_10`

ISO-8859-10 编码（北欧）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_10
```

---

## `ISO_8859_11`

ISO-8859-11 编码（泰文）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_11
```

---

## `ISO_8859_13`

ISO-8859-13 编码（波罗的海）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_13
```

---

## `ISO_8859_14`

ISO-8859-14 编码（凯尔特）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_14
```

---

## `ISO_8859_15`

ISO-8859-15 编码（西欧）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_15
```

---

## `ISO_8859_16`

ISO-8859-16 编码（东南欧）。

**可用于**

- 编码参数

### 调用

```lua
file.ISO_8859_16
```

---

## `KOI8_R`

KOI8-R 编码（俄文）。

**可用于**

- 编码参数

### 调用

```lua
file.KOI8_R
```

---

## `KOI8_U`

KOI8-U 编码（乌克兰）。

**可用于**

- 编码参数

### 调用

```lua
file.KOI8_U
```

---

## `IBM866`

IBM866 编码（俄文）。

**可用于**

- 编码参数

### 调用

```lua
file.IBM866
```

---

## `MACINTOSH`

Macintosh 编码（西欧）。

**可用于**

- 编码参数

### 调用

```lua
file.MACINTOSH
```

---

## `X_MAC_CYRILLIC`

x-mac-cyrillic 编码（西里尔）。

**可用于**

- 编码参数

### 调用

```lua
file.X_MAC_CYRILLIC
```

---

## 方法

### `read`

- **方法作用**：异步读取 `assets/` 目录下的文本文件。
- **方法要求**：无
- **方法参数**：

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 额外补充 |
| ------ | ---- | ---- | ------ | ---- | -------- |
| `path` | string | 是 | — | 相对 `assets/` 的文件路径 | 必须为相对路径且存在 |
| `encoding` | const | 否 | `"auto"` | 文本编码 | 使用 `file.*` 编码常量 |
| `end_of_line` | const | 否 | `"auto"` | 换行符规范 | 使用 `file.AUTO/ALL/CR/LF/CRLF` |
| `event_tip` | string | 否 | `nil` | 事件提示文本 | 上限 4 KiB |

- **方法返回**：无直接返回值（异步）。完成后 `HandleEvent` 收到 `file` 事件：

| 字段 | 类型 | 说明 |
| ---- | ---- | ---- |
| `request_id` | integer | 请求标识，用于对应该次调用 |
| `kind` | string | 恒为 `"read_text"` |
| `path` | string | 请求的虚拟路径 |
| `tip` | string / nil | 请求时传入的 `event_tip` |
| `ok` | boolean | 是否成功 |
| `text` | string | 成功时的文本内容（`ok=true`） |
| `error` | table | 失败时含 `code`、`message`（`ok=false`） |

- **方法的使用**：

```lua

```

---

### `write`

- **方法作用**：异步写入文本文件到 `assets/` 目录（允许创建新文件）。
- **方法要求**：仅游戏脚本、关闭安全模式
- **方法参数**：

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 额外补充 |
| ------ | ---- | ---- | ------ | ---- | -------- |
| `path` | string | 是 | — | 相对 `assets/` 的文件路径 | 必须为相对路径；父目录必须存在 |
| `text` | string | 是 | — | 要写入的文本 | 上限 1 MiB，不含 NUL |
| `encoding` | const | 否 | `"auto"` | 文本编码 | 使用 `file.*` 编码常量 |
| `end_of_line` | const | 否 | `"auto"` | 换行符规范 | 使用 `file.AUTO/ALL/CR/LF/CRLF` |
| `event_tip` | string | 否 | `nil` | 事件提示文本 | 上限 4 KiB |

- **方法返回**：无直接返回值（异步）。完成后 `HandleEvent` 收到 `file` 事件：

| 字段 | 类型 | 说明 |
| ---- | ---- | ---- |
| `request_id` | integer | 请求标识 |
| `kind` | string | 恒为 `"write_text"` |
| `path` | string | 请求的虚拟路径 |
| `tip` | string / nil | 请求时传入的 `event_tip` |
| `ok` | boolean | 是否成功 |
| `error` | table | 失败时含 `code`、`message`（`ok=false`） |

- **方法的使用**：

```lua

```

---

### `list_dir`

- **方法作用**：异步枚举 `assets/` 目录下的条目。
- **方法要求**：仅游戏脚本、关闭安全模式
- **方法参数**：

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 额外补充 |
| ------ | ---- | ---- | ------ | ---- | -------- |
| `path` | string | 是 | — | 相对 `assets/` 的目录路径 | 必须为相对路径且为目录 |
| `recursive` | boolean | 否 | `false` | 是否递归子目录 | — |
| `file_type` | string | 否 | `nil` | 仅匹配指定扩展名 | 如 `"rs"`；传 `"all"` 或不传匹配全部 |
| `event_tip` | string | 否 | `nil` | 事件提示文本 | 上限 4 KiB |

- **方法返回**：无直接返回值（异步）。完成后 `HandleEvent` 收到 `file` 事件：

| 字段 | 类型 | 说明 |
| ---- | ---- | ---- |
| `request_id` | integer | 请求标识 |
| `kind` | string | 恒为 `"list_dir"` |
| `path` | string | 请求的虚拟路径 |
| `tip` | string / nil | 请求时传入的 `event_tip` |
| `ok` | boolean | 是否成功 |
| `entries` | table | 成功时的条目数组，每项含 `path`、`file_type`（`ok=true`） |
| `error` | table | 失败时含 `code`、`message`（`ok=false`） |

- **方法的使用**：

```lua

```
