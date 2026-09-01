# string 库

## 基本库说明

`string` 提供字符串处理：大小写、反转、截取、查找、匹配、替换、格式化，以及独立的**正则**（regex）变体方法。游戏与屏保会话均可使用。库表为只读。

> 模式（pattern）分为两套：**Lua 模式**（`find`/`match`/`gmatch`/`gsub`）与 **Rust 正则**（`regex_find`/`regex_match`/`regex_gmatch`/`regex_gsub`/`regex_test`/`regex_split`）。正则替换串中 `$0..$9` 引用捕获组，`$$` 转义字面 `$`。

## 目录

### 常量

| 常量名                 | 说明       |
| ------------------- | -------- |
| `string.AUTO`       | 文本模式：自动  |
| `string.PLAIN_TEXT` | 文本模式：纯文本 |
| `string.RICH_TEXT`  | 文本模式：富文本 |

### 方法

| 方法名 | 说明 |
| ------ | ---- |
| `string.lower(text)` | 转小写 |
| `string.upper(text)` | 转大写 |
| `string.reverse(text)` | 反转字符串 |
| `string.regex_escape(text)` | 转义正则特殊字符 |
| `string.sub{...}` | 按字符截取子串 |
| `string.rep{...}` | 重复拼接 |
| `string.find{...}` | 查找首个匹配 |
| `string.match{...}` | 提取匹配捕获 |
| `string.gmatch{...}` | 迭代全部匹配 |
| `string.gsub{...}` | 全局替换 |
| `string.regex_find{...}` | 正则查找 |
| `string.regex_match{...}` | 正则提取 |
| `string.regex_gmatch{...}` | 正则迭代 |
| `string.regex_gsub{...}` | 正则替换 |
| `string.regex_test{...}` | 是否匹配 |
| `string.regex_split{...}` | 正则分割 |
| `string.format{...}` | 格式化字符串 |
| `string.rich_text_to_plain_text{...}` | 富文本转纯文本 |

---

## 常量

## `AUTO`

自动检查检查文本类型。

**可用于**

- 参数 `text_mode`

### 调用

```lua
text.AUTO
```

### 示例

```lua
rect = align.resolve_rect { width = 6, height = 1, horizontal_align = align.CENTER, vertical_align = align.CENTER, offset_x = 0, offset_y = 0 }
draw.text { x = rect.x, y = rect.y, text = "CENTER", fg = color.BRIGHT_RED }
```

输出：

---

## `PLAIN_TEXT`

强制按纯文本解析；`f%`、富文本标签和参数占位符均作为普通字符保留。

**可用于**

- 文本参数 `text_mode`

### 调用

```lua
text.PLAIN_TEXT
```

### 额外补充

- 值为字符串 `"plain_text"`。

---

## `RICH_TEXT`

强制按富文本语法解析，但不识别或移除 `f%` 前缀；无需添加该前缀。

**可用于**

- 文本参数 `text_mode`

### 调用

```lua
text.RICH_TEXT
```

### 额外补充

- 值为字符串 `"rich_text"`。

---

## 方法

## `lower`

将字符串全部转为小写。

### 调用

```lua
-- 单参数
text.lower()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `result` | string | 小写结果 |

### 示例

```lua

```

---

## `upper`

将字符串全部转为大写。

### 调用

```lua
-- 单参数
text.upper()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `result` | string | 大写结果 |

### 示例

```lua

```

---

## `reverse`

按字符反转字符串。

### 调用

```lua
-- 单参数
text.reverse()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `result` | string | 反转结果 |

### 示例

```lua

```

---

## `regex_escape`

转义字符串中的正则特殊字符，使其可作为普通文本参与正则匹配。

### 调用

```lua
-- 单参数
text.regex_escape()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `result` | string | 转义后的字符串 |

### 示例

```lua

```

---

## `sub`

按字符位置截取子串。

### 调用

```lua
-- 单参数
text.sub()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |
| `start` | integer | 是 | - | 起始字符位置 |
| `finish` | integer | 否 | 字符串长度 | 结束字符位置 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `result` | string | 截取结果 |

### 示例

```lua

```

---

## `rep`

将字符串重复 `times` 次，可指定分隔符拼接。

### 调用

```lua
-- 单参数
text.rep()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 要重复的字符串 |
| `times` | integer | 是 | - | 重复次数 |
| `sep` | string | 否 | `""` | 相邻副本间的分隔符 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `result` | string | 重复拼接结果 |

### 示例

```lua

```

---

## `find`

查找首个匹配的位置与捕获；找不到返回 `nil`。

### 调用

```lua
-- 单参数
text.find()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |
| `pattern` | string | 是 | - | Lua 模式 |
| `init` | integer | 否 | `1` | 起始搜索位置 |
| `plain` | boolean | 否 | `false` | 是否按纯文本查找 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `start` | integer | 匹配起点（字符位置，1 起始） |
| `finish` | integer | 匹配终点（字符位置） |
| `captures...` | 若干值 | 各捕获组内容 |

### 示例

```lua

```

---

## `match`

提取首个匹配的捕获组；无捕获组时返回整个匹配。

### 调用

```lua
-- 单参数
text.match()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |
| `pattern` | string | 是 | - | Lua 模式 |
| `init` | integer | 否 | `1` | 起始搜索位置 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `captures...` | 若干值 | 各捕获组内容 |

### 示例

```lua

```

---

## `gmatch`

返回迭代全部匹配的迭代函数，每次产出捕获组（无捕获组时产出整个匹配）。

### 调用

```lua
-- 单参数
text.gmatch()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |
| `pattern` | string | 是 | - | Lua 模式 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `iterator` | function | 迭代函数 |

### 示例

```lua

```

---

## `gsub`

全局替换匹配内容，返回结果与替换次数。

### 调用

```lua
-- 单参数
text.gsub()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |
| `pattern` | string | 是 | - | Lua 模式 |
| `repl` | string / table / function | 是 | - | 替换内容 |
| `limit` | integer | 否 | 全部 | 最大替换次数 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `result` | string | 替换结果 |
| `count` | integer | 实际替换次数 |

### 示例

```lua

```

---

## `regex_find`

用正则查找首个匹配，返回位置与捕获表。

### 调用

```lua
-- 单参数
text.regex_find()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |
| `pattern` | string | 是 | - | 正则表达式 |
| `init` | integer | 否 | `1` | 起始搜索位置 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `start` | integer | 匹配起点（字符位置，1 起始） |
| `finish` | integer | 匹配终点（字符位置） |
| `capture_table` | table | 捕获组表 |

### 示例

```lua

```

---

## `regex_match`

用正则提取首个匹配的捕获组；无捕获组时返回整个匹配。

### 调用

```lua
-- 单参数
text.regex_match()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |
| `pattern` | string | 是 | - | 正则表达式 |
| `init` | integer | 否 | `1` | 起始搜索位置 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `captures...` | 若干值 | 各捕获组内容 |

### 示例

```lua

```

---

## `regex_gmatch`

返回用正则迭代全部匹配的迭代函数。

### 调用

```lua
-- 单参数
text.regex_gmatch()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |
| `pattern` | string | 是 | - | 正则表达式 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `iterator` | function | 迭代函数 |

### 示例

```lua

```

---

## `regex_gsub`

用正则全局替换，返回结果与替换次数。

### 调用

```lua
-- 单参数
text.regex_gsub()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |
| `pattern` | string | 是 | - | 正则表达式 |
| `repl` | string / table / function | 是 | - | 替换内容 |
| `limit` | integer | 否 | 全部 | 最大替换次数 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `result` | string | 替换结果 |
| `count` | integer | 实际替换次数 |

### 示例

```lua

```

---

## `regex_test`

判断文本是否匹配给定正则。

### 调用

```lua
-- 单参数
text.regex_test()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |
| `pattern` | string | 是 | - | 正则表达式 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `matched` | boolean | 是否存在匹配 |

### 示例

```lua

```

---

## `regex_split`

按正则分割字符串，返回数组表。

### 调用

```lua
-- 单参数
text.regex_split()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 目标字符串 |
| `pattern` | string | 是 | - | 正则表达式 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `parts` | table | 分割结果数组 |

### 示例

```lua

```

---

## `format`

按格式串格式化值列表。

### 调用

```lua
-- 单参数
text.format()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `format_string` | string | 是 | - | 格式串 |
| `values` | table | 否 | `nil` | 参数值数组 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `result` | string | 格式化结果 |

### 示例

```lua

```

---

## `rich_text_to_plain_text`

将富文本转换为可见的纯文本。

### 调用

```lua
-- 单参数
text.rich_text_to_plain_text()
```

### 参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `text` | string | 是 | - | 富文本字符串 |
| `rich_params` | table | 否 | `nil` | 富文本参数表 |
| `strip_header` | boolean | 否 | `true` | 是否剥离 `f%` 头 |

### 返回

| 返回值名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 可见纯文本 |

### 示例

```lua

```