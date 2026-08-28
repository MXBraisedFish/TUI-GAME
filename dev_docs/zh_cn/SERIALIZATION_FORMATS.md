# 序列化规范

## 前言

国际化语言旨在保证多语言的适配，让不同国家和地区的玩家在阅读文本时无障碍。Tui Game 提供了一套完整的国际化语言规范，本文档将详细介绍其资源结构、文件格式及 API 使用方法。

---

## 目录

| 章节       | 说明                 | 索引                      |
| ---------- | -------------------- | ------------------------- |
| 通用规则   | 所有序列化的通用规则 | [通用规则](#通用规则)     |
| JSON       | JSON 格式            | [JSON](#json)             |
| TOML       | TOML 格式            | [TOML](#toml)             |
| YAML       | YAML 格式            | [YAML](#yaml)             |
| CSV        | CSV 格式             | [CSV](#csv)               |
| XML        | XML 格式             | [XML](#xml)               |
| INI        | INI 格式             | [INI](#ini)               |
| 二进制数据 | 二进制数据           | [二进制数据](#二进制数据) |

## 链接

| 说明                            | 链接                                  |
| ------------------------------- | ------------------------------------- |
| `lifecycle` 库 API 使用文档     | [lifecycle](api/lifecycle.md)         |
| `serialization` 库 API 使用文档 | [serialization](api/serialization.md) |

---

## 通用规则

- 所有反序列化中 `null` 会被转换为 Lua 值 `nil`，但 Lua 解析器本身会忽略值 `nil`。

**示例**

```json
{
  "test": 1,
  "is_null": null // Lua 为 { test = 1}
}
```

---

## JSON

### 根结构要求

JSON 的根值**可为任意可序列化的值**。

```lua
data = any...
```

### 数据结构映射

**双向映射**

> 该部分值的映射可逆

| Lua       | JSON      |
| --------- | --------- |
| `nil`     | `null`    |
| `boolean` | `boolean` |
| `integer` | `integer` |
| `number`  | `float`   |
| `string`  | `string`  |
| 数组表    | 数组      |
| 对象表    | 对象      |

### 示例

**正确结构**

序列化：

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

反序列化：

```lua
json = '{"name":"TUI GAME","values":[1,2,3]}'
data = serialization.json_decode(json)
debug.print { message = tostring(data.name) }  -- TUI GAME
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

> 数组表不连续，存在数据空洞

```lua
{
  [1] = "A",
  [3] = "C"
}
```

> `null` 被当做可显式的 Lua 值 `nil`

```json
{
  "is_null": null
}
```

```lua
{}
```

---

## TOML

### 根结构要求

TOML 的根值**必须是对象表**。

```lua
data = {
  key = value,
  ...
}
```

### 数据结构映射

**双向映射**

> 该部分值的映射可逆

| Lua       | TOML      |
| --------- | --------- |
| `boolean` | `boolean` |
| `integer` | `integer` |
| `number`  | `float`   |
| `string`  | `string`  |
| 数组表    | 数组      |
| 对象表    | 表        |

**单向映射**

> 该部分值的映射不可逆

| Lua      | 方向 | TOML   |
| -------- | :--: | ------ |
| `string` | $←$  | `date` |

### 示例

**正确示例**

序列化：

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

反序列化：：

```lua
toml = 'title = "TUI GAME"\n[window]\nwidth = 120'
data = serialization.toml_decode(toml)
debug.print { message = tostring(data.window.width) }
```

输出：

```text
120
```

**错误示例**

> 根值类型错误

```lua
{ 1, 2, 3 }
```

> 对象键为非字符串

```lua
{ [2] = "value" }
```

> 误认为单向映射值可逆

```toml
date = 2026-10-01T15:20:45
```

```lua
"2026-10-01T15:20:45" -- 被转换为字符串，且不可再转回日期类型
```

---

## YAML

### 根结构要求

YAML 的根值**必须是对象表**。

```lua
data = {
  key = value,
  ...
}
```

### 数据结构映射

**双向映射**

> 该部分值的映射可逆

| Lua       | YAML      |
| --------- | --------- |
| `nil`     | `null`    |
| `boolean` | `boolean` |
| `integer` | `integer` |
| `number`  | `float`   |
| `string`  | `string`  |
| 数组表    | 序列      |
| 对象表    | 映射      |

### 示例

**正确示例**

序列化：

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

反序列化：：

```lua
yaml = "name: TUI GAME\ntags:\n- tui\n- lua"
data = serialization.yaml_decode(yaml)
debug.print { message = tostring(data.tags[1]) }
```

输出：

```text
tui
```

**错误示例**

> 原始数据中包含自定义标签

```yaml
name: !person TUI # 反序列化不支持自定义标签
```

> 映射键不是字符串

```yaml
1: one
```

> `null` 被当做可显式的 Lua 值 `nil`

```yaml
is_null: null # Lua 为 {}
```

---

## CSV

### 根结构要求

CSV 的根值**必须是二维数组表**。

```lua
data = {
  {string...},
  ...
}
```

### 数据结构映射

**双向映射**

> 该部分值的映射可逆

| Lua      | YAML     |
| -------- | -------- |
| `string` | `string` |

**单向映射**

> 该部分值的映射不可逆

| Lua       | 方向 | CSV      |
| --------- | :--: | -------- |
| `boolean` | $→$  | `string` |
| `integer` | $→$  | `string` |
| `number`  | $→$  | `string` |
| `nil`     | $→$  | 空文本   |
| `string`  | $←$  | 空文本   |

### 示例

**正确示例**

序列化：

```lua
rows = {
  { "name", "age", "work" },
  { "Alice", 12, false },
  { "Bob", 30, true }
}

csv = serialization.csv_encode(rows)
debug.print { message = csv }
```

输出：

```csv
name,age,work
Alice,12,false
Bob,30,true
```

反序列化：：

```lua
csv = "name,age,work\nAlice,12,false\nBob,30,true"
rows = serialization.csv_decode(csv)
debug.print { message = tostring(rows[2][1]) }
debug.print { message = tostring(type(rows[2][2])) }
```

输出：

```text
Alice
string
```

**错误示例**

> 各行列数不一致（表头和表内列数需一致）

```lua
{
  { "name", "age" },
  { "Alice" }
}
```

> 单元格中包含表（不可嵌套）

```lua
{
  { "name", { "nested" } }
}
```

> 误认为单向映射值可逆（转换后均为字符串）

```csv
true -- Lua 为 "true"
1    -- Lua 为 "1"
```

---

## XML

### 根结构要求

XML 的根值**必须遵循特定的表结构**。

```lua
data = {
  root = {            -- 根标签
    _attr = {         -- 属性
      key = value
      ...
    },
    _text = value,    -- 子元素（与子标签冲突，见下文）
    element = { ... } -- 子标签（与子元素冲突，见下文）
  }
}
```

XML 结构：

```xml
<root key = value>
  value                  <!-- 与子标签冲突，见下文 -->
  <element>...</element> <!-- 与子元素冲突，见下文 -->
</root>
```

### 属性

属性保存在对象键 `_attr` 表中。

**示例**

序列化：

```lua
data = {
  root = {
    _attr = {
      version = "1.0",
      enabled = true
    },
    _text = "Hello"
  }
}

xml = serialization.xml_encode(data)
debug.print { message = xml }
```

输出：

```xml
<root enabled="true" version="1.0">Hello</root>
```

### 子元素

子元素保存在对象键 `_text` 中，或直接保存在根标签中。

> 子标签与子元素冲突，当存在子元素时，根标签不可包含子标签。

**示例**

序列化：

```lua
data1 = {
  root = {
    _text = "Hello"
  }
}

data2 = {
  root = "Hello"
}

xml1 = serialization.xml_encode(data1)
debug.print { message = xml1 }

xml2 = serialization.xml_encode(data2)
debug.print { message = xml2 }
```

输出：

```xml
<root>Hello</root>
<root>Hello</root>
```

#### 数据结构映射

**双向映射**

> 该部分值的映射可逆

| Lua      | XML      |
| -------- | -------- |
| `string` | `string` |

**单向映射**

> 该部分值的映射不可逆

| Lua       | 方向 | CSV      |
| --------- | :--: | -------- |
| `boolean` | $→$  | `string` |
| `integer` | $→$  | `string` |
| `number`  | $→$  | `string` |
| `nil`     | $→$  | 空文本   |
| `string`  | $←$  | 空文本   |

### 子标签

其他任何键作为子标签。

> 子标签与子元素冲突，当存在子标签时，根标签不可包含子元素。

**示例**

序列化：

```lua
{
  player = {
    name = "Alice",
    score = 95
  }
}

{
  players = {
    player = {
      { name = "Alice" }, -- 同名使用连续数组表示
      { name = "Bob" }
    }
  }
}
```

苏处：

```xml
<player>
  <name>Alice</name>
  <score>95</score>
</player>

<players>
  <player><name>Alice</name></player>
  <player><name>Bob</name></player>
</players>
```

### 转义规则

| 原字符 | 转义符   |
| ------ | -------- |
| `&`    | `&amp;`  |
| `<`    | `&lt;`   |
| `>`    | `&gt;`   |
| `"`    | `&quot;` |
| `'`    | `&apos;` |

### 正确示例

序列化：

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

反序列化：：

```lua
xml = '<root version="1.0"><item>A</item><item>B</item></root>'
data = serialization.xml_decode(xml)
debug.print { message = data.root.item[1] }  -- A
```

### 错误示例

> **错误 1：根表包含多个子元素**
>
> ```lua
> data = {
>   root1 = "value1",
>   root2 = "value2"
> }
> ```
>
> 根表只能有一个键。

> **错误 2：子元素同时包含文本和子子元素**
>
> ```lua
> {
>   root = {
>     _text = "before",
>     child = "value"
>   }
> }
> ```

> **错误 3：子元素名包含冒号**
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
- `boolean` → `boolean`
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

反序列化：：

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
