# 序列化规范

## 前言

国际化语言旨在保证多语言的适配，让不同国家和地区的玩家在阅读文本时无障碍。Tui Game 提供了一套完整的国际化语言规范，本文档将详细介绍其资源结构、文件格式及 API 使用方法。

---

## 目录

| 章节              | 说明                         | 索引                                    |
| ----------------- | ---------------------------- | --------------------------------------- |
| 可序列化的 Lua 值 | 哪些 Lua 值可被序列化        | [可序列化的 Lua 值](#可序列化的-lua-值) |
| JSON              | JSON 格式的编码与解码        | [JSON](#json)                           |
| TOML              | TOML 格式的编码与解码        | [TOML](#toml)                           |
| YAML              | YAML 格式的编码与解码        | [YAML](#yaml)                           |
| CSV               | CSV 格式的编码与解码         | [CSV](#csv)                             |
| XML               | XML 格式的编码与解码         | [XML](#xml)                             |
| INI               | INI 格式的编码与解码         | [INI](#ini)                             |
| 二进制打包        | 按格式串打包与解包二进制数据 | [二进制打包](#二进制打包)               |
| 格式选择建议      | 不同场景下的格式推荐         | [格式选择建议](#格式选择建议)           |

## 链接

| 说明                            | 链接                                  |
| ------------------------------- | ------------------------------------- |
| `lifecycle` 库 API 使用文档     | [lifecycle](api/lifecycle.md)         |
| `serialization` 库 API 使用文档 | [serialization](api/serialization.md) |

---

## 可序列化的 Lua 值

| Lua 类型  | 编码结果             |
| --------- | -------------------- |
| `nil`     | 转换为对应格式的空值 |
| `boolean` | 布尔值               |
| `integer` | 整数                 |
| `number`  | 浮点数               |
| `string`  | 字符串               |
| 数组表    | 数组/序列            |
| 对象表    | 对象/映射            |

---

## JSON

### 数据结构映射

| Lua                | JSON           |
| ------------------ | -------------- |
| `nil`              | `null`         |
| `boolean`          | `true`/`false` |
| `integer`/`number` | number         |
| `string`           | string         |
| 数组表             | array          |
| 对象表             | object         |

### 示例

**正确结构**

```lua
data = {
  name = "TUI GAME",
  enabled = true,
  values = { 1, 2, 3 }
}

json = serialization.json_encode(data)
debug.print { message = json }
```

输出：

```json
{ "enabled": true, "name": "TUI GAME", "values": [1, 2, 3] }
```

解码：

```lua
json = '{"name":"TUI GAME","values":[1,2,3]}'
data = serialization.json_decode(json)
debug.print { message = data.name }  -- TUI GAME
```

输出：

```text
TUI GAME
```

**错误结构**

> Lua 表值混合

```lua
{
  1,
  "A",
  obj = { ... }
}
```

> 数组表不连续

```lua
{
  [1] = "A",
  [3] = "C"
}
```

> Lua 值 `nil` 被当做可显式的 `null`

```lua
-- lua
{
  is_null = nil
}

-- json
{}
```

---

## TOML

### 根结构要求

TOML 的根值**必须是对象表**，不能是数组或基本类型。

```lua
data = {
  key = value
}
```

### 数据结构映射

| Lua                | JSON           |
| ------------------ | -------------- |
| `boolean`          | `true`/`false` |
| `integer`/`number` | number         |
| `string`           | string         |
| 数组表             | array          |
| 对象表             | table          |

### 正确示例

```lua
data = {
  title = "TUI GAME",
  window = { width = 120, height = 40 }
}

toml = serialization.toml_encode(data)
debug.print { message = toml }
```

输出：

```toml
title = "TUI GAME"

[window]
width = 120
height = 40
```

解码：

```lua
toml = 'title = "TUI GAME"\n[window]\nwidth = 120'
data = serialization.toml_decode(toml)
debug.print { message = data.window.width }  -- 120
```

### 错误示例

> **错误 1：根值为数组**
>
> ```lua
> data = { 1, 2, 3 }
> serialization.toml_encode(data)  -- 失败
> ```

> **错误 2：表中包含 `nil`**
>
> TOML 不支持空值，任何位置出现 `nil` 都会导致编码失败。

> **错误 3：对象键为非字符串**
>
> ```lua
> data = { [1] = "value" }  -- 数字键不被允许
> ```

---

## YAML

YAML 适用于人工编辑的复杂层级数据，可读性高。本库仅支持能够安全转换为 JSON 的子集。

### 数据结构映射

- YAML sequence → 数组表
- YAML mapping → 对象表
- YAML null → Lua `nil`

### 支持的值

- 空值、布尔值、有限数字、UTF-8 字符串
- 连续序列、字符串键映射

### 不支持的特性

- 自定义标签（如 `!type`）
- 复杂映射键
- 注释、锚点、缩进风格（解码后不保留）
- 任何 YAML 标签会直接拒绝整个文档

### 方法

| 方法          | 参数         | 返回值   | 说明                  |
| ------------- | ------------ | -------- | --------------------- |
| `yaml_encode` | `value: any` | `string` | 将 Lua 值编码为 YAML  |
| `yaml_decode` | `s: string`  | `any`    | 将 YAML 解码为 Lua 值 |

### 正确示例

```lua
data = {
  name = "TUI GAME",
  enabled = true,
  tags = { "tui", "lua" }
}

yaml = serialization.yaml_encode(data)
debug.print { message = yaml }
```

输出：

```yaml
enabled: true
name: TUI GAME
tags:
  - tui
  - lua
```

解码：

```lua
yaml = "name: TUI GAME\ntags:\n- tui\n- lua"
data = serialization.yaml_decode(yaml)
debug.print { message = data.tags[1] }  -- tui
```

### 错误示例

> **错误 1：YAML 中包含自定义标签**
>
> ```yaml
> name: !person TUI
> ```
>
> 解码时遇到任何标签都会拒绝整个文档。

> **错误 2：映射键不是字符串**
>
> 复杂映射键无法安全转换为 Lua 对象表。

> **错误 3：依赖 YAML 的隐式类型推断**
>
> 跨格式转换时，应明确使用字符串、数字或布尔值，避免依赖解析器的自动推断。

---

## CSV

CSV 适用于规则的二维表格数据，不保存字段类型。

### 数据结构要求

CSV 的输入必须是**二维连续数组**：

```lua
rows = {
  { "name", "score" },
  { "Alice", 95 },
  { "Bob", 87 }
}
```

- 第一层是行数组，第二层是每行的列数组
- 所有行必须具有相同的列数
- 每个单元格只能是：字符串、整数/有限浮点数、布尔值、空值
- 单元格不能是表、函数、线程或 userdata

### 文本格式

- 字段分隔符：逗号 `,`
- 引号字符：双引号 `"`
- 包含逗号、双引号或换行的字段自动加引号
- 字段内部的双引号通过重复双引号转义

### 方法

| 方法         | 参数          | 返回值   | 说明                  |
| ------------ | ------------- | -------- | --------------------- |
| `csv_encode` | `rows: table` | `string` | 将二维数组编码为 CSV  |
| `csv_decode` | `s: string`   | `table`  | 将 CSV 解码为二维数组 |

### 正确示例

```lua
rows = {
  { "name", "description" },
  { "TUI GAME", "terminal, Lua and games" },
  { "quote", 'He said "Hello"' }
}

csv = serialization.csv_encode(rows)
debug.print { message = csv }
```

输出：

```csv
name,description
TUI GAME,"terminal, Lua and games"
quote,"He said ""Hello"""
```

解码：

```lua
csv = "name,score\nAlice,95\nBob,87"
rows = serialization.csv_decode(csv)
debug.print { message = rows[2][1] }  -- Alice
```

### 错误示例

> **错误 1：各行列数不一致**
>
> ```lua
> rows = {
>   { "name", "score" },
>   { "Alice" }  -- 只有1列，与第一行2列不一致
> }
> ```

> **错误 2：单元格中包含表**
>
> ```lua
> rows = {
>   { "name", { "nested" } }  -- 单元格不能是表
> }
> ```

> **错误 3：空表表示空 CSV**
>
> 空 Lua 表 `{}` 被识别为对象而非空数组，不适用于表达空 CSV。

---

## XML

XML 适用于具有明确标签结构的数据，支持元素、属性和重复节点。

XML 使用专门的表映射规则，不同于普通 JSON 表映射。

### 根元素

编码输入必须是**只包含一个命名元素**的表：

```lua
data = {
  root = {
    child = "Hello"
  }
}
```

对应：

```xml
<root><child>Hello</child></root>
```

### 元素文本

标量值直接作为元素文本：

```lua
{ root = "Hello" }
```

也可以使用 `_text` 字段：

```lua
{ root = { _text = "Hello" } }
```

连续数组项会依次拼接为文本（不插入分隔符）：

```lua
{ root = { "Hello", " ", "TUI" } }  -- → <root>Hello TUI</root>
```

### 属性

属性保存在 `_attr` 表中：

```lua
{
  root = {
    _attr = { version = "1.0", enabled = true },
    _text = "Hello"
  }
}
```

对应：

```xml
<root enabled="true" version="1.0">Hello</root>
```

属性值只能是 `nil`、布尔值、有限数字或 UTF-8 字符串。

### 子元素

普通字符串键表示子元素：

```lua
{
  player = {
    name = "Alice",
    score = 95
  }
}
```

同名子元素使用非空连续数组表示：

```lua
{
  players = {
    player = {
      { name = "Alice" },
      { name = "Bob" }
    }
  }
}
```

对应：

```xml
<players>
  <player><name>Alice</name></player>
  <player><name>Bob</name></player>
</players>
```

### 禁止混合内容

一个元素**不能同时包含文本和子元素**：

```lua
-- 不合法
{
  root = {
    _text = "before",
    child = "value"
  }
}
```

### 名称要求

- 第一个字符：Unicode 字母或 `_`
- 后续字符：Unicode 字母、数字、`_`、`-` 或 `.`
- 不允许空名称
- 不支持命名空间前缀（冒号）
- `_attr` 和 `_text` 是保留字段

### 转义规则

编码时自动转义：`&`→`&amp;`、`<`→`&lt;`、`>`→`&gt;`、`"`→`&quot;`、`'`→`&apos;`

### 方法

| 方法         | 参数           | 返回值   | 说明                 |
| ------------ | -------------- | -------- | -------------------- |
| `xml_encode` | `value: table` | `string` | 将 Lua 值编码为 XML  |
| `xml_decode` | `s: string`    | `table`  | 将 XML 解码为 Lua 值 |

### 正确示例

```lua
data = {
  root = {
    _attr = { version = "1.0" },
    item = {
      { name = "Alice" },
      { name = "Bob" }
    }
  }
}

xml = serialization.xml_encode(data)
debug.print { message = xml }
```

输出：

```xml
<root version="1.0"><item><name>Alice</name></item><item><name>Bob</name></item></root>
```

解码：

```lua
xml = '<root version="1.0"><item>A</item><item>B</item></root>'
data = serialization.xml_decode(xml)
debug.print { message = data.root.item[1] }  -- A
```

### 错误示例

> **错误 1：根表包含多个元素**
>
> ```lua
> data = {
>   root1 = "value1",
>   root2 = "value2"
> }
> ```
>
> 根表只能有一个键。

> **错误 2：元素同时包含文本和子元素**
>
> ```lua
> {
>   root = {
>     _text = "before",
>     child = "value"
>   }
> }
> ```

> **错误 3：元素名包含冒号**
>
> ```lua
> { ["ns:root"] = "value" }  -- 命名空间前缀不支持
> ```

---

## INI

INI 适用于简单的键值配置，只支持一层节，不保留类型。

### 数据结构要求

- 根值必须是对象表
- 根表中的标量字段 → 全局键
- 根表中的对象字段 → 节

```lua
data = {
  application = "TUI GAME",
  server = {
    host = "127.0.0.1",
    port = 8080
  }
}
```

对应：

```ini
application=TUI GAME

[server]
host=127.0.0.1
port=8080
```

### 层级限制

- 只支持根级全局键和一层节
- 节内只能包含标量键值
- 不支持节内继续嵌套
- 不支持数组

### 键名和节名要求

- 不能为空
- 不能包含换行
- 不能包含 `[`、`]`、`=`、`;`、`#`

### 支持的值类型

- `nil` → 编码为空文本
- `boolean` → `true`/`false`
- `integer` 或有限 `number`
- UTF-8 字符串（不能包含换行）

### 重要：类型丢失

INI 不保存原始类型，**所有解码值均为 Lua 字符串**：

```ini
enabled=true
count=10
```

解码后：

```lua
{
  enabled = "true",  -- 字符串，不是布尔值
  count = "10"       -- 字符串，不是整数
}
```

### 方法

| 方法         | 参数        | 返回值   | 说明                 |
| ------------ | ----------- | -------- | -------------------- |
| `ini_encode` | `t: table`  | `string` | 将 Lua 表编码为 INI  |
| `ini_decode` | `s: string` | `table`  | 将 INI 解码为 Lua 表 |

### 正确示例

```lua
data = {
  application = "TUI GAME",
  server = {
    host = "127.0.0.1",
    port = 8080
  }
}

ini = serialization.ini_encode(data)
debug.print { message = ini }
```

输出：

```ini
application = TUI GAME

[server]
host = 127.0.0.1
port = 8080
```

解码：

```lua
ini = "[server]\nhost=127.0.0.1\nport=8080"
data = serialization.ini_decode(ini)
debug.print { message = data.server.host }  -- "127.0.0.1"（字符串）
```

### 错误示例

> **错误 1：节内嵌套子表**
>
> ```lua
> data = {
>   server = {
>     network = {
>       host = "127.0.0.1"  -- 不支持二级嵌套
>     }
>   }
> }
> ```

> **错误 2：键名包含 `=`**
>
> 键名不能包含等号，否则解析会出错。

> **错误 3：字符串值包含换行**
>
> INI 不支持多行值。

---

## 二进制打包

用于按格式字符串打包和解包二进制数据。

### 方法

| 方法              | 参数                                         | 返回值                               | 说明                 |
| ----------------- | -------------------------------------------- | ------------------------------------ | -------------------- |
| `binary_pack`     | `{fmt: string, values: table}`               | `string`                             | 按格式串打包数据     |
| `binary_unpack`   | `{fmt: string, data: string, pos?: integer}` | `{values: table, next_pos: integer}` | 从二进制串解包数据   |
| `binary_packsize` | `fmt: string`                                | `integer`                            | 返回打包所需总字节数 |

### 正确示例

打包：

```lua
bytes = serialization.binary_pack {
  fmt = "<I4 I4",
  values = { 100, 200 }
}
debug.print { message = "packed " .. tostring(#bytes) .. " bytes" }
```

输出：

```text
packed 8 bytes
```

解包：

```lua
result = serialization.binary_unpack {
  fmt = "<I4 I4",
  data = bytes
}
debug.print { message = result.values[1] .. ", " .. result.values[2] }
```

输出：

```text
100, 200
```

查询大小：

```lua
size = serialization.binary_packsize("<I4 I4")
debug.print { message = tostring(size) }
```

输出：

```text
8
```

### 错误示例

> **错误 1：`fmt` 和 `values` 数量不匹配**
>
> ```lua
> serialization.binary_pack {
>   fmt = "<I4 I4",
>   values = { 100 }  -- 需要2个值
> }
> ```

> **错误 2：`data` 长度不足**
>
> ```lua
> serialization.binary_unpack {
>   fmt = "<I4",
>   data = "\x01"  -- 需要4字节
> }
> ```

> **错误 3：`pos` 超出数据范围**
>
> 指定的起始位置超出了二进制数据的长度。

---

## 格式选择建议

| 需求                   | 推荐格式 | 原因                               |
| ---------------------- | -------- | ---------------------------------- |
| 通用数据交换           | JSON     | 结构明确，跨语言兼容最好           |
| 人工编辑的程序配置     | TOML     | 配置层级清晰，标量类型明确         |
| 人工编辑的复杂层级数据 | YAML     | 可读性高，但应限制在安全子集       |
| 规则二维表格           | CSV      | 适合表格数据，不保存字段类型       |
| 元素、属性和重复节点   | XML      | 适合具有明确标签结构的数据         |
| 简单键值配置           | INI      | 结构简单，但只有一层节且不保留类型 |
