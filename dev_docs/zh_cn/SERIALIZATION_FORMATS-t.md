# 文本序列化格式规范

## 1. 文档用途

本文整理 `serialization` 库当前支持的六种文本序列化格式：

- JSON
- TOML
- YAML
- CSV
- XML
- INI

内容以当前 Rust 宿主实现为准，描述 Lua 值如何编码、文本如何解码、允许的数据结构、安全限制及不支持的格式。

API 的调用参数请查看 [`api/serialization.md`](api/serialization.md)。二进制格式字符串不属于本文范围，请查看 [`LUA_FORMAT_STRING.md`](LUA_FORMAT_STRING.md)。

---

## 2. 通用规则

### 2.1 文本编码

- 所有格式的输入和输出均为 UTF-8 字符串。
- 非 UTF-8 Lua 字符串不能传给这些文本格式方法。
- 单个输入字符串和序列化后的输出字符串最大为 1 MiB。
- 二进制数据不能直接写入这些文本格式；需要先使用 `encoding.base64_encode` 或 `encoding.hex_encode` 转换为文本。

### 2.2 可序列化的 Lua 值

通用编码器支持以下 Lua 值：

| Lua 类型 | 编码规则 |
| -------- | -------- |
| `nil` | 转换为空值；目标格式不支持空值时编码失败 |
| `boolean` | 转换为布尔值 |
| `integer` | 保留为整数 |
| `number` | 保留为浮点数，但必须是有限数值 |
| `string` | 必须是有效 UTF-8 文本 |
| `table` | 根据键结构转换为数组或对象 |

以下值不能序列化：

- `function`
- `thread`
- `userdata`
- `lightuserdata`
- error 对象
- 包含循环引用的表
- `NaN`
- 正无穷和负无穷

### 2.3 Lua 表的数组与对象判定

Lua 表按键结构分为两种：

#### 数组表

数组表只能包含从 `1` 开始、连续且无空洞的正整数键：

```lua
local array = { "A", "B", "C" }
```

以下表是稀疏数组，不能序列化：

```lua
local sparse = {
  [1] = "A",
  [3] = "C"
}
```

#### 对象表

对象表只能使用字符串键：

```lua
local object = {
  name = "TUI GAME",
  version = 1
}
```

#### 禁止混合

同一张表不能同时包含数组键和字符串键：

```lua
local invalid = {
  "A",
  name = "mixed"
}
```

空表没有数组元素，因此默认按空对象处理。若目标格式要求数组，空表可能无法表达空数组。

### 2.4 通用安全限制

- 表最大嵌套深度为 32 层。
- 单次转换最多处理 16384 个值节点或表项。
- 循环表会立即被拒绝。
- JSON、TOML 和 YAML 的通用值转换同样受深度和节点数量限制；CSV、XML 和 INI 还需遵守各自章节列出的专用限制。
- 不保证对象键或 XML 子元素保持 Lua 表的遍历顺序；需要顺序时应使用数组。

---

## 3. JSON

### 3.1 支持的数据结构

JSON 支持完整的通用 Lua 数据模型：

| Lua 值 | JSON 值 |
| ------ | ------- |
| `nil` | `null` |
| `boolean` | `true` / `false` |
| `integer` / `number` | number |
| `string` | string |
| 数组表 | array |
| 对象表 | object |

编码结果为紧凑 JSON，不自动缩进：

```lua
local text = serialization.json_encode {
  value = {
    name = "TUI GAME",
    enabled = true,
    values = { 1, 2, 3 }
  }
}
```

结果：

```json
{"enabled":true,"name":"TUI GAME","values":[1,2,3]}
```

### 3.2 解码规则

- JSON object 解码为字符串键 Lua 表。
- JSON array 解码为从 `1` 开始的连续 Lua 数组表。
- JSON integer 在 Lua 整数范围内时解码为 `integer`。
- 其他 JSON number 解码为 `number`。
- JSON `null` 解码为 Lua `nil`。

需要注意：Lua 表不能实际保存 `nil`。如果 JSON array 中含有 `null`，对应索引会成为空洞；如果 JSON object 的字段值为 `null`，对应字段不会存在。不要依赖 JSON `null` 与“字段不存在”之间的区别。

### 3.3 格式要求

- 输入必须是单个合法 JSON 值。
- 对象键必须是字符串。
- 不允许注释、尾随逗号、`NaN` 或无穷值。
- 字符串转义遵循 JSON 语法。

---

## 4. TOML

### 4.1 根结构

TOML 根值必须是对象表：

```lua
local data = {
  title = "TUI GAME",
  window = {
    width = 120,
    height = 40
  }
}
```

以下根值均不能编码为 TOML：

```lua
serialization.toml_encode { value = "text" }
serialization.toml_encode { value = { 1, 2, 3 } }
```

### 4.2 支持的数据结构

- 字符串键对象表映射为 TOML table。
- 连续数组表映射为 TOML array。
- 字符串、布尔值、整数和有限浮点数映射为对应 TOML 标量。
- TOML 不支持 `null`，因此任何位置出现 `nil` 空值都可能导致编码失败。
- 数组中的元素必须符合 TOML 自身的类型要求。

### 4.3 解码规则

- TOML table 解码为对象表。
- TOML array 解码为连续数组表。
- 整数、浮点数、布尔值和字符串解码为相应 Lua 类型。
- TOML 日期和时间值通过中间安全数据模型转换为字符串，不作为宿主日期对象暴露给 Lua。

### 4.4 格式要求

- 输入必须是合法的单个 TOML 文档。
- 不支持在 Lua 中保留 TOML 注释、原始排版或键顺序。
- 编码后重新解码可以保留数据语义，但不保证文本格式与原文件完全一致。
- Lua 对象键必须是字符串，不能使用数字键表示 TOML table 字段。

---

## 5. YAML

### 5.1 支持范围

YAML 仅支持能够安全转换为 JSON 数据模型的子集：

- 空值
- 布尔值
- 有限数字
- UTF-8 字符串
- 连续序列
- 字符串键映射

```lua
local text = serialization.yaml_encode {
  value = {
    name = "TUI GAME",
    enabled = true,
    tags = { "tui", "lua" }
  }
}
```

### 5.2 不支持的 YAML 能力

- 自定义标签，例如 `!type`。
- 不能安全转换成字符串键对象的复杂映射键。
- 用于构造宿主对象或执行代码的类型语义。
- 保留注释、锚点名称、原始缩进或原始标量书写风格。

解码时发现任何 YAML 标签都会拒绝整个文档，而不是忽略标签。

### 5.3 解码规则

- YAML sequence 解码为连续数组表。
- YAML mapping 解码为字符串键对象表。
- YAML null 解码为 Lua `nil`，因此与 JSON `null` 一样不能在 Lua 表中保留明确的空值位置。
- 数字会根据可表示范围解码为 `integer` 或 `number`。

### 5.4 格式要求

- 输入必须是合法的单个 YAML 文档。
- 映射键应使用字符串。
- 不应依赖 YAML 解析器的隐式复杂类型推断；需要稳定跨格式转换时，应明确使用字符串、数字或布尔值。
- 编码和解码只保证数据语义，不保证 YAML 文本样式往返一致。

---

## 6. CSV

### 6.1 Lua 数据结构

CSV 编码输入必须是二维连续数组：

```lua
local rows = {
  { "name", "score" },
  { "Alice", 95 },
  { "Bob", 87 }
}
```

第一层是行数组，第二层是每一行的列数组。

每个单元格只能是：

- 字符串
- 整数或有限浮点数
- 布尔值
- 空值

单元格不能是表、函数、线程或 userdata。

### 6.2 行列要求

- 所有行必须具有相同的列数。
- 行和列都必须使用从 `1` 开始的连续数组。
- 空 Lua 表默认被识别为对象，因此当前接口不适合用 `{}` 表达空 CSV 或零列行。
- CSV 不区分数字、布尔值和字符串的类型信息，编码时都会转换为文本。

### 6.3 文本格式

- 字段分隔符为逗号 `,`。
- 引号字符为双引号 `"`。
- 包含逗号、双引号或换行的字段会自动加引号。
- 字段内部的双引号通过重复双引号转义。
- 编码器使用标准 CSV 行结束方式；解码器接受合法的 CSV 换行。
- 第一行不会被自动当作表头，所有行都作为普通数据返回。

示例：

```csv
name,description
TUI GAME,"terminal, Lua and games"
quote,"He said ""Hello"""
```

### 6.4 解码规则

CSV 解码结果始终是二维数组，并且每个单元格始终为 Lua `string`：

```lua
local rows = serialization.csv_decode("name,score\nAlice,95")

-- rows[2][2] 是字符串 "95"，不是整数 95。
```

如果需要数字或布尔值，脚本必须自行调用 `tonumber` 或执行明确转换。

### 6.5 安全限制

- 输入文本最大 1 MiB。
- 最多解码 16384 行。
- 格式错误、引号未闭合或行列数不一致时解码失败。

---

## 7. XML

XML 使用专门的 Lua 表映射，不使用普通 JSON 表映射规则。

### 7.1 根元素

编码输入必须是只包含一个命名元素的表：

```lua
local data = {
  root = {
    child = "Hello"
  }
}
```

对应：

```xml
<root><child>Hello</child></root>
```

根表为空、包含多个元素或使用整数键都会报错。

### 7.2 元素文本

标量值直接作为元素文本：

```lua
{ root = "Hello" }
```

也可以使用 `_text`：

```lua
{
  root = {
    _text = "Hello"
  }
}
```

连续数组项会依次转换为文本并直接拼接，不插入分隔符：

```lua
{
  root = { "Hello", " ", "TUI GAME" }
}
```

同一元素不能同时使用 `_text` 和连续数组文本。

### 7.3 属性

属性保存在 `_attr` 表中：

```lua
{
  root = {
    _attr = {
      version = "1.0",
      enabled = true
    },
    _text = "Hello"
  }
}
```

对应：

```xml
<root enabled="true" version="1.0">Hello</root>
```

属性值只能是 `nil`、布尔值、有限数字或 UTF-8 字符串，编码后全部表现为 XML 文本。解码后的属性值也始终是 Lua 字符串。

### 7.4 子元素

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
<players><player><name>Alice</name></player><player><name>Bob</name></player></players>
```

空表作为单个空元素编码：

```lua
{ root = { empty = {} } }
```

对应：

```xml
<root><empty/></root>
```

### 7.5 禁止混合内容

一个元素不能同时包含文本和子元素：

```lua
-- 不合法
{
  root = {
    _text = "before",
    child = "value"
  }
}
```

以下 XML 也不能解码：

```xml
<root>before<child/>after</root>
```

### 7.6 名称要求

当前元素名和属性名必须满足：

- 第一个字符是 Unicode 字母或 `_`。
- 后续字符只能是 Unicode 字母、数字、`_`、`-` 或 `.`。
- 不允许空名称。
- 不支持包含冒号的命名空间前缀。
- `_attr` 和 `_text` 是保留字段；其他以 `_` 开头的元素字段会被拒绝。

### 7.7 转义规则

编码时会自动转义：

- `&` → `&amp;`
- `<` → `&lt;`
- `>` → `&gt;`
- `"` → `&quot;`
- `'` → `&apos;`

解码支持 XML 预定义实体，但不支持 DTD 或自定义实体。

### 7.8 解码后的 Lua 结构

- 最外层表以根元素名称为键。
- 无属性、无子元素的叶节点直接解码为字符串。
- 空叶节点解码为空字符串 `""`。
- 属性保存在 `_attr` 表中。
- 有属性或子元素的节点，其文本保存在 `_text` 中。
- 单个子元素直接保存在元素名字段中。
- 重复的同名子元素保存为连续数组。
- XML 声明、注释和处理指令不会进入结果表。
- CDATA 内容作为普通文本返回。
- 文本节点首尾的空白会在解码时被裁剪；需要精确保留边界空白时，不应依赖当前 XML 映射。

例如：

```xml
<root version="1.0"><item>A</item><item>B</item></root>
```

解码为：

```lua
{
  root = {
    _attr = {
      version = "1.0"
    },
    item = { "A", "B" }
  }
}
```

### 7.9 安全限制

- 只能包含一个根元素。
- 最大嵌套深度为 32 层。
- 最多处理 16384 个元素节点。
- 不支持 DTD、实体声明和自定义实体引用。
- 不支持混合内容。
- 标签必须正确闭合且名称匹配。
- 根元素外不能存在非空文本。

---

## 8. INI

### 8.1 Lua 数据结构

INI 根值必须是对象表。根表中的标量字段表示全局键，根表中的对象字段表示节：

```lua
local data = {
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

### 8.2 层级限制

- 只支持根级全局键和一层节。
- 节内只能包含标量键值。
- 不支持节内继续嵌套表。
- 不支持数组。

### 8.3 键名和节名

键名及节名：

- 不能为空。
- 不能包含换行。
- 不能包含 `[`, `]`, `=`, `;`, `#`。

### 8.4 值要求

支持以下值：

- `nil`，编码为空文本。
- `boolean`，编码为 `true` 或 `false`。
- `integer` 或有限 `number`。
- UTF-8 字符串。

字符串值不能包含换行。INI 不保存原始类型，因此所有解码值都为 Lua 字符串：

```ini
enabled=true
count=10
```

解码后：

```lua
{
  enabled = "true",
  count = "10"
}
```

### 8.5 文本解析规则

- 空行会被忽略。
- 去除每行首尾空白。
- 去除键和值在 `=` 两侧的空白。
- 整行第一个非空字符为 `;` 或 `#` 时，该行作为注释忽略。
- 不解析行尾注释；值中的 `;` 或 `#` 会被当作普通文本。
- 节必须写成完整的 `[section]`。
- 普通字段必须包含 `=`。
- 同一作用域内不允许重复键。
- 不应声明重复节；重复节不属于受支持的规范输入。

### 8.6 往返限制

INI 只能保存字符串形式的标量，因此编码后再解码不会保留数字、布尔值和空值的 Lua 类型。注释、空白、字段排列和原始书写格式也不会保留。

---

## 9. 格式选择建议

| 需求 | 推荐格式 | 原因 |
| ---- | -------- | ---- |
| 通用数据交换 | JSON | 结构明确，跨语言兼容最好 |
| 人工编辑的程序配置 | TOML | 配置层级清晰，标量类型明确 |
| 人工编辑的复杂层级数据 | YAML | 可读性高，但应限制在安全子集 |
| 规则二维表格 | CSV | 适合表格数据，不保存字段类型 |
| 元素、属性和重复节点 | XML | 适合具有明确标签结构的数据 |
| 简单键值配置 | INI | 结构简单，但只有一层节且不保留类型 |

如果数据需要在多种格式之间相互转换，建议只使用 JSON 共同数据模型：字符串键对象、连续数组、字符串、布尔值和有限数字。XML、CSV 与 INI 都有专用结构，不能无损表达任意通用 Lua 表。
