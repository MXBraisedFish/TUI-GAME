# Lua 模式与正则表达式语法规范

## 前言

Lua 模式与正则表达式是字符串匹配操作的核心参数，用于精确描述如何在 Lua 字符串中执行文本检索与捕获。本文档旨在为开发者提供一份语法速查手册。

> 模式串语法规则与 Lua 官方原版完全一致，Tui Game 仅在签名的 API 使用上有修改。如需完整的规范说明，请参阅 Lua 官方文档。

---

## 目录

| 章节                 | 说明                          | 索引                                        |
| -------------------- | ----------------------------- | ------------------------------------------- |
| 第一部分：Lua 模式   | Lua 原生模式匹配语法          | [第一部分：Lua 模式](#第一部分lua-模式)     |
| 第二部分：正则表达式 | 基于 Rust 的 Unicode 正则语法 | [第二部分：正则表达式](#第二部分正则表达式) |

---

## Lua 模式

Lua 模式是 Lua 语言内置的轻量级匹配语法，相较于正则表达式语法更简洁。

---

## 魔法字符

### 语法

| 语法 | 说明                                                      |
| ---- | --------------------------------------------------------- |
| `%`  | 转义字符                                                  |
| `.`  | 匹配除换行符外的任意字符                                  |
| `+`  | 匹配前一字符 1 次或多次                                   |
| `*`  | 匹配前一字符 0 次或多次                                   |
| `-`  | 匹配前一字符 0 次或多次（非贪婪）；在字符集内取连字符范围 |
| `?`  | 匹配前一字符 0 次或 1 次                                  |
| `^`  | 匹配字符串开头；在字符集内取反                            |
| `$`  | 匹配字符串结尾                                            |
| `()` | 用于捕获分组                                              |
| `[]` | 定义字符集                                                |

### 示例

```lua
s1 = "abc123"
r1 = string.find { text = s1, pattern = "%d+" }
debug.print { message = r1.start .. " " .. r1.finish }

s2 = "abc"
r2 = string.find { text = s2, pattern = "%d*" }
debug.print { message = r2.start .. " " .. r2.finish }
```

输出：

```text
4 6
1 0
```

---

## 转义符号

### 语法

| 语法 | 说明     |
| ---- | -------- |
| `%`  | 转义符号 |

### 示例

```lua
s1 = "100% complete"
r1 = string.find { text = s1, pattern = "%%" }
debug.print { message = r1.start .. " " .. r1.finish .. " " .. r1.captures[1] }

s2 = "file.txt"
r2 = string.find { text = s2, pattern = "%." }
debug.print { message = r2.start .. " " .. r2.finish .. " " .. r2.captures[1] }
```

输出：

```text
4 4 %
5 5 .
```

---

## 字符类

### 语法

| 语法 | 说明                                       |
| ---- | ------------------------------------------ |
| `.`  | 匹配除换行符外的任意字符                   |
| `%a` | 匹配任意字母（A-Z 和 a-z）                 |
| `%c` | 匹配任意控制字符                           |
| `%d` | 匹配任意数字（0-9）                        |
| `%l` | 匹配任意小写字母                           |
| `%p` | 匹配任意标点符号                           |
| `%s` | 匹配任意空白字符（空格、制表符、换行等）   |
| `%u` | 匹配任意大写字母                           |
| `%w` | 匹配任意字母数字（A-Z、a-z、0-9）          |
| `%x` | 匹配任意十六进制数字                       |
| `%z` | 匹配任意 ASCII NUL 字符（`\0`）            |
| `%A` | 匹配任意非字母（A-Z 和 a-z）               |
| `%C` | 匹配任意非控制字符                         |
| `%D` | 匹配任意非数字（0-9）                      |
| `%L` | 匹配任意非小写字母                         |
| `%P` | 匹配任意非标点符号                         |
| `%S` | 匹配任意非空白字符（空格、制表符、换行等） |
| `%U` | 匹配任意非大写字母                         |
| `%W` | 匹配任意非字母数字（A-Z、a-z、0-9）        |
| `%X` | 匹配任意非十六进制数字                     |
| `%Z` | 匹配任意非 ASCII NUL 字符（`\0`）          |

### 示例

```lua
s1 = "abc123"
r1 = string.find { text = s1, pattern = "%d+" }
debug.print { message = r1.start .. " " .. r1.finish .. " " .. r1.captures[1] }

s2 = "123abc"
r2 = string.find { text = s2, pattern = "%D+" }
debug.print { message = r2.start .. " " .. r2.finish .. " " .. r2.captures[1] }
```

输出：

```text
4 6 123
4 6 abc
```

---

## 量词

| 语法 | 说明                              |
| ---- | --------------------------------- |
| `+`  | 匹配前一字符 1 次或多次           |
| `*`  | 匹配前一字符 0 次或多次           |
| `-`  | 匹配前一字符 0 次或多次（非贪婪） |
| `?`  | 匹配前一字符 0 次或 1 次          |

### 示例

```lua
s1 = "abc123def"
r1 = string.find { text = s1, pattern = "%d+" }
debug.print { message = r1.start .. " " .. r1.finish .. " " .. r1.captures[1] }

s2 = "abc"
r2 = string.find { text = s2, pattern = "%d*" }
debug.print { message = r2.start .. " " .. r2.finish .. " " .. r2.captures[1] }

s3 = "a<b>c<d>e"
r3 = string.find { text = s3, pattern = "<.->" }
debug.print { message = r3.start .. " " .. r3.finish .. " " .. r3.captures[1] }

s4 = "color"
r4 = string.find { text = s4, pattern = "colou?r" }
debug.print { message = r4.start .. " " .. r4.finish .. " " .. r4.captures[1] }
```

输出：

```text
4 6 123
1 0
2 4 <b>
1 5 color
```

---

## 字符集

### 语法

| 语法 | 说明               |
| ---- | ------------------ |
| `[]` | 匹配所定义字符集   |
| `-`  | 连字符范围         |
| `^`  | 取反字符集（补集） |

### 示例

```lua
s1 = "abc123"
r1 = string.find { text = s1, pattern = "[abc]+" }
debug.print { message = r1.start .. " " .. r1.finish .. " " .. r1.captures[1] }

s2 = "hello123"
r2 = string.find { text = s2, pattern = "[a-z]+" }
debug.print { message = r2.start .. " " .. r2.finish .. " " .. r2.captures[1] }

s3 = "abc123"
r3 = string.find { text = s3, pattern = "[^a-z]+" }
debug.print { message = r3.start .. " " .. r3.finish .. " " .. r3.captures[1] }
```

输出：

```text
1 3 abc
1 5 hello
4 6 123
```

### 额外补充

- `-` 两侧字符范围为 ASCII 字符表。
- `^` 仅放在字符集开头时代表取反字符集，若需要开头使用普通字符需转义 `%^`。

---

## 捕获

### 语法

| 语法 | 说明           |
| ---- | -------------- |
| `()` | 捕获匹配的内容 |

### 示例

```lua
s1 = "Name: Alice"
r1 = string.find { text = s1, pattern = "Name: (%w+)" }
debug.print { message = r1.captures[1] .. " " .. r1.captures.n }

s2 = "Alice, 30"
r2 = string.find { text = s2, pattern = "(%w+), (%d+)" }
debug.print { message = r2.captures[1] .. " " .. r2.captures[2] .. " " .. r2.captures.n }

s3 = "Today is 2024-12-25"
r3 = string.find { text = s3, pattern = "(%d+)-(%d+)-(%d+)" }
debug.print { message = r3.captures[1] .. " " .. r3.captures[2] .. " " .. r3.captures[3] .. " " .. r3.captures.n }

s4 = "hello"
r4 = string.find { text = s4, pattern = "()hello()" }
debug.print { message = r4.captures[1] .. " " .. r4.captures[2] .. " " .. r4.captures.n }
```

输出：

```text
Alice 1
Alice 30 2
2024 12 25 3
1 6 2
```

### 额外补充

- `()` 空捕获返回捕获起始位置。

---

## 锚点

| 语法 | 说明         |
| ---- | ------------ |
| `^`  | 匹配输入开头 |
| `$`  | 匹配输入结尾 |

### 示例

```lua
s1 = "hello world"
r1 = string.find { text = s1, pattern = "^hello" }
debug.print { message = r1.start .. " " .. r1.finish .. " " .. r1.captures[1] }

s2 = "hello world"
r2 = string.find { text = s2, pattern = "world$" }
debug.print { message = r2.start .. " " .. r2.finish .. " " .. r2.captures[1] }
```

输出：

```text
1 5 hello
7 11 world
```

---

## 平衡匹配

### 语法

| 语法   | 说明                                   |
| ------ | -------------------------------------- |
| `%bxy` | 匹配从 `x` 开始到 `y` 结束的平衡字符串 |

### 示例

```lua
s1 = "a(b(c)d)e"
r1 = string.find { text = s1, pattern = "%b()" }
debug.print { message = r1.start .. " " .. r1.finish .. " " .. r1.captures[1] }

s2 = "x[1,2,[3,4]]y"
r2 = string.find { text = s2, pattern = "%b[]" }
debug.print { message = r2.start .. " " .. r2.finish .. " " .. r2.captures[1] }
```

输出：

```text
2 8 (b(c)d)
2 12 [1,2,[3,4]]
2 8 {b{c}d}
2 8 <b<c>d>
```

---

## 捕获引用

### 语法

| 语法      | 说明                 |
| --------- | -------------------- |
| `%0`      | 完整匹配             |
| `%1`-`%9` | 第 1 至第 9 个捕获组 |

### 示例

```lua
s1 = "hello world"
r1 = string.gsub { text = s1, pattern = "%a+", repl = "[%0]" }
debug.print { message = r1.result .. " " .. r1.count }

s2 = "2024-12-25"
r2 = string.gsub { text = s2, pattern = "(%d+)-(%d+)-(%d+)", repl = "%1/%2/%3" }
debug.print { message = r2.result .. " " .. r2.count }
```

```text
[hello] [world] 2
2024/12/25 1
```

---

## 正则表达式

正则表达式是描述字符模式的形式语言，相较于 Lua 模式语法功能更加全面。

---

## Lua 长字符串

> 在正则表达式中可能需要频繁的使用 `\`做转义字符，为了避免 Lua 中的 `\\` 导致的转义地狱，可以使用 Lua 长字符串语法

### 语法

| 语法   | 说明     |
| ------ | -------- |
| `[[]]` | 长字符串 |

### 示例

```lua
str1 = [[string \d \" \[]]
debug.print { message = str1 }

str2 = [=[string [[]] string]=]
debug.print { message = str2 }
```

输出：

```text
string \d \" \[
string [[]] string
```

### 额外补充

- 带 `=` 符号的是长字符串的特殊格式，当文本中出现 `[[` 和 `]]` 时可以使用该格式，`=` 不限数量，但必须左右两侧数量相等。

---

## 元字符

### 语法

| 语法 | 含义                           |
| ---- | ------------------------------ | --- |
| `\`  | 转义字符                       |
| `.`  | 匹配除换行符外的任意字符       |     |
| `*`  | 匹配前一字符 0 次或多次        |
| `+`  | 匹配前一字符 1 次或多次        |
| `?`  | 匹配前一字符 0 次或 1 次       |
| `-`  | 在字符集内取连字符范围         |
| `^`  | 匹配字符串开头；在字符集内取反 |
| `$`  | 匹配输入字符串的结尾           |
| `\|` | 逻辑或                         |
| `()` | 用于捕获分组                   |
| `[]` | 定义字符集                     |
| `{}` | 精确指定重复次数               |

### 示例

```lua
```

输出：

```text
```

### 3.2 任意字符

| 语法     | 含义                                      |
| -------- | ----------------------------------------- |
| `.`      | 匹配除 `\n` 以外的任意一个 Unicode 字符   |
| `(?s:.)` | 匹配包括 `\n` 在内的任意一个 Unicode 字符 |

`.` 匹配一个 Unicode 标量值，不是一个 UTF-8 字节，也不保证匹配一个完整的用户可见字形。例如组合字符和部分 Emoji 可能由多个 Unicode 标量值组成。

### 3.3 连接与选择

| 语法    | 含义                   |
| ------- | ---------------------- | --------------------------------------- |
| `xy`    | 先匹配 `x`，再匹配 `y` |
| `x      | y`                     | 匹配 `x` 或 `y`；相同起点下优先左侧分支 |
| `(?:abc | def)123`               | 对选择分组后再连接 `123`                |

正则会先寻找最靠左的可匹配位置。在同一个起点有多个分支都能匹配时，分支书写顺序会影响结果：

```text
samwise|sam   优先得到 samwise
sam|samwise   优先得到 sam
```

## 4. 字符类

### 4.1 基本字符类

| 语法           | 含义                               |
| -------------- | ---------------------------------- |
| `[xyz]`        | 匹配 `x`、`y` 或 `z`               |
| `[^xyz]`       | 匹配除 `x`、`y`、`z` 外的字符      |
| `[a-z]`        | 匹配范围 `a` 至 `z`                |
| `[a-zA-Z0-9_]` | 多个范围和字符的并集               |
| `[\[\]]`       | 匹配 `[` 或 `]`                    |
| `[-a]`         | 位于适当位置时，`-` 可作为普通字符 |

范围按 Unicode 码点顺序计算。范围两端必须是可作为范围边界的单个字符。

### 4.2 嵌套与集合运算

字符类支持嵌套和集合运算：

| 语法          | 分类       | 含义                       |
| ------------- | ---------- | -------------------------- |
| `[x[^xyz]]`   | 嵌套、并集 | 匹配除 `y`、`z` 外的字符   |
| `[a-y&&xyz]`  | 交集       | 匹配 `x` 或 `y`            |
| `[0-9&&[^4]]` | 交集与取反 | 匹配除 `4` 外的 ASCII 数字 |
| `[0-9--4]`    | 差集       | 匹配除 `4` 外的 ASCII 数字 |
| `[a-g~~b-h]`  | 对称差     | 匹配 `a` 或 `h`            |
| `[a&&b]`      | 空集合     | 永远不匹配                 |

字符类运算符：

| 运算符   | 含义                                   |
| -------- | -------------------------------------- |
| 相邻书写 | 并集                                   |
| `&&`     | 交集                                   |
| `--`     | 差集                                   |
| `~~`     | 对称差                                 |
| `^`      | 对整个字符类取反；必须位于该字符类开头 |

字符类优先级由高到低为：

1. 范围，例如 `[a-cd]` 等同于 `[[a-c]d]`。
2. 并集，例如 `[ab&&bc]` 等同于 `[[ab]&&[bc]]`。
3. 交集、差集和对称差，三者优先级相同，从左到右计算。
4. 取反，例如 `[^a-z&&b]` 等同于 `[^[a-z&&b]]`。

### 4.3 Perl 风格字符类

以下字符类默认按 Unicode 定义工作：

| 语法 | 含义                                       |
| ---- | ------------------------------------------ |
| `\d` | Unicode 十进制数字，等同于 `\p{Nd}`        |
| `\D` | 非 Unicode 十进制数字                      |
| `\s` | Unicode 空白字符，等同于 `\p{White_Space}` |
| `\S` | 非 Unicode 空白字符                        |
| `\w` | Unicode 单词字符                           |
| `\W` | 非 Unicode 单词字符                        |

Unicode `\w` 包括字母、组合标记、十进制数字、连接标点和 Join Control，不只包括 ASCII 的 `[A-Za-z0-9_]`。

需要 ASCII 语义时，可以使用 POSIX 字符类，或在安全的局部分组中关闭 Unicode：

```text
[[:digit:]]   ASCII 数字
[[:word:]]    ASCII 单词字符
(?-u:\w)      ASCII 单词字符
```

### 4.4 ASCII/POSIX 字符类

| 语法           | 等价范围或含义             |
| -------------- | -------------------------- |
| `[[:alnum:]]`  | `[0-9A-Za-z]`              |
| `[[:alpha:]]`  | `[A-Za-z]`                 |
| `[[:ascii:]]`  | `[\x00-\x7F]`              |
| `[[:blank:]]`  | 制表符和空格               |
| `[[:cntrl:]]`  | ASCII 控制字符             |
| `[[:digit:]]`  | `[0-9]`                    |
| `[[:graph:]]`  | ASCII 可见非空格字符       |
| `[[:lower:]]`  | `[a-z]`                    |
| `[[:print:]]`  | ASCII 可打印字符，包括空格 |
| `[[:punct:]]`  | ASCII 标点符号             |
| `[[:space:]]`  | ASCII 空白字符             |
| `[[:upper:]]`  | `[A-Z]`                    |
| `[[:word:]]`   | `[0-9A-Za-z_]`             |
| `[[:xdigit:]]` | `[0-9A-Fa-f]`              |

取反写法是在类名开头添加 `^`：

```text
[[:^digit:]]
```

POSIX 字符类必须写在外层字符类中，例如 `[[:digit:]]`，不能单独写成 `[:digit:]`。

## 5. Unicode 属性

Unicode 模式默认开启。支持以下属性写法：

| 语法               | 含义                       |
| ------------------ | -------------------------- |
| `\pL`              | Unicode Letter 类别        |
| `\p{Letter}`       | Unicode Letter 类别        |
| `\P{Letter}`       | Unicode Letter 类别的补集  |
| `\p{Greek}`        | Greek Script               |
| `\p{sc:Greek}`     | 明确指定 Script 为 Greek   |
| `\p{Script=Greek}` | 同上                       |
| `\p{scx:Greek}`    | Script Extensions 为 Greek |
| `\p{age:3.2}`      | Unicode 3.2 已分配的码点   |
| `\p{Alphabetic}`   | Alphabetic 二元属性        |

属性名和值忽略 ASCII 大小写、空格和下划线，并接受标准简称。例如 `gc` 表示 `General_Category`，`sc` 表示 `Script`，`scx` 表示 `Script_Extensions`。

支持的属性类别为：

- `General_Category`，包括 `Any`、`ASCII` 和 `Assigned`。
- `Script`。
- `Script_Extensions`。
- `Age`。
- 二元及枚举属性：`ASCII_Hex_Digit`、`Alphabetic`、`Bidi_Control`、`Case_Ignorable`、`Cased`、`Changes_When_Casefolded`、`Changes_When_Casemapped`、`Changes_When_Lowercased`、`Changes_When_Titlecased`、`Changes_When_Uppercased`、`Dash`、`Default_Ignorable_Code_Point`、`Deprecated`、`Diacritic`、`Emoji`、`Emoji_Presentation`、`Emoji_Modifier`、`Emoji_Modifier_Base`、`Emoji_Component`、`Extended_Pictographic`、`Extender`、`Grapheme_Base`、`Grapheme_Cluster_Break`、`Grapheme_Extend`、`Hex_Digit`、`IDS_Binary_Operator`、`IDS_Trinary_Operator`、`ID_Continue`、`ID_Start`、`Join_Control`、`Logical_Order_Exception`、`Lowercase`、`Math`、`Noncharacter_Code_Point`、`Pattern_Syntax`、`Pattern_White_Space`、`Prepended_Concatenation_Mark`、`Quotation_Mark`、`Radical`、`Regional_Indicator`、`Sentence_Break`、`Sentence_Terminal`、`Soft_Dotted`、`Terminal_Punctuation`、`Unified_Ideograph`、`Uppercase`、`Variation_Selector`、`White_Space`、`Word_Break`、`XID_Continue`、`XID_Start`。

常用 General Category 简称：

| 总类 | 子类                                     | 含义                                 |
| ---- | ---------------------------------------- | ------------------------------------ |
| `L`  | `Lu`, `Ll`, `Lt`, `Lm`, `Lo`             | 字母：大写、小写、标题、修饰、其他   |
| `M`  | `Mn`, `Mc`, `Me`                         | 组合标记：非间距、间距组合、包围     |
| `N`  | `Nd`, `Nl`, `No`                         | 数字：十进制、字母数字、其他数字     |
| `P`  | `Pc`, `Pd`, `Ps`, `Pe`, `Pi`, `Pf`, `Po` | 标点符号                             |
| `S`  | `Sm`, `Sc`, `Sk`, `So`                   | 符号：数学、货币、修饰、其他         |
| `Z`  | `Zs`, `Zl`, `Zp`                         | 分隔符：空格、行、段落               |
| `C`  | `Cc`, `Cf`, `Cs`, `Co`, `Cn`             | 其他：控制、格式、代理、私用、未分配 |

例如 `\pL` 匹配所有字母，`\p{Lu}` 只匹配大写字母，`\P{Nd}` 匹配非十进制数字字符。Script 和 Script Extensions 接受依赖所支持的标准 Unicode Script 名称，例如 `Latin`、`Greek`、`Han`、`Hiragana`。

当前依赖所带的 Unicode 属性表版本为 Unicode 16.0.0。

注意：正则不会自动执行 Unicode 规范化。视觉上相同但规范化形式不同的文本，可能不会互相匹配。需要时应由数据来源统一规范化形式。

## 6. 重复与数量限定

### 6.1 贪婪重复

| 语法     | 含义                   |
| -------- | ---------------------- |
| `x*`     | 匹配零个或更多 `x`     |
| `x+`     | 匹配一个或更多 `x`     |
| `x?`     | 匹配零个或一个 `x`     |
| `x{n}`   | 恰好匹配 `n` 个 `x`    |
| `x{n,}`  | 至少匹配 `n` 个 `x`    |
| `x{n,m}` | 匹配 `n` 至 `m` 个 `x` |

默认重复是贪婪的，会在整体正则仍然能够匹配的前提下尽可能多地匹配。

### 6.2 懒惰重复

在重复语法后增加 `?`，改为尽可能少地匹配：

| 语法      | 含义                             |
| --------- | -------------------------------- |
| `x*?`     | 零个或更多，懒惰                 |
| `x+?`     | 一个或更多，懒惰                 |
| `x??`     | 零个或一个，懒惰                 |
| `x{n,m}?` | `n` 至 `m` 个，懒惰              |
| `x{n,}?`  | 至少 `n` 个，懒惰                |
| `x{n}?`   | 恰好 `n` 个；添加 `?` 不改变数量 |

`U` 标志可以交换贪婪与懒惰的默认含义。

## 7. 分组与捕获

| 语法            | 分类       | 含义                           |
| --------------- | ---------- | ------------------------------ |
| `(exp)`         | 捕获组     | 创建按左括号顺序编号的捕获组   |
| `(?:exp)`       | 非捕获组   | 只分组，不进入捕获结果         |
| `(?P<name>exp)` | 命名捕获组 | 创建同时具有名称和编号的捕获组 |
| `(?<name>exp)`  | 命名捕获组 | 上一种写法的简写               |

捕获编号从 `1` 开始，`0` 表示完整匹配。

捕获组名称：

- 必须以 `_` 或 Unicode 字母开头。
- 后续可以包含 Unicode 字母和数字，以及 `.`, `_`, `[` 和 `]`。
- 命名捕获组仍然占用一个数字编号。

本项目当前的 Lua 返回结构只按数字编号返回捕获，不会把命名捕获额外写入同名键。因此，命名主要用于增强表达式可读性；Lua 侧仍按捕获顺序读取。

未参与匹配的可选捕获组返回 `nil`。Lua 数组中的 `nil` 会形成空洞，不能依赖 `#table` 得到捕获组的准确数量。

## 8. 位置、边界与零宽匹配

| 语法                | 含义                              |
| ------------------- | --------------------------------- |
| `^`                 | 输入开头；`m` 模式下也匹配行开头  |
| `$`                 | 输入结尾；`m` 模式下也匹配行结尾  |
| `\A`                | 仅匹配整个输入开头，不受 `m` 影响 |
| `\z`                | 仅匹配整个输入结尾，不受 `m` 影响 |
| `\b`                | Unicode 单词边界                  |
| `\B`                | 非 Unicode 单词边界               |
| `\b{start}` 或 `\<` | Unicode 单词起始边界              |
| `\b{end}` 或 `\>`   | Unicode 单词结束边界              |
| `\b{start-half}`    | 单词起始边界的左半条件            |
| `\b{end-half}`      | 单词结束边界的右半条件            |

这些表达式不消耗字符，因此属于零宽匹配。

空正则是合法的，可以在字符串开头、字符之间和结尾匹配空内容。引擎保证不会在一个 UTF-8 字符的内部返回空匹配位置。

空正则与空字符类不同：

- 空正则 `""` 匹配空内容。
- 空集合 `[a&&b]` 永远不匹配任何内容。

默认只有 `\n` 被识别为换行。启用 `mR` 后，`^` 和 `$` 会安全识别 `\r\n`，不会匹配在 `\r` 与 `\n` 中间。

## 9. 转义序列

| 语法             | 含义                          |
| ---------------- | ----------------------------- |
| `\a`             | Bell，U+0007                  |
| `\f`             | Form Feed，U+000C             |
| `\t`             | 水平制表符                    |
| `\n`             | 换行符                        |
| `\r`             | 回车符                        |
| `\v`             | 垂直制表符，U+000B            |
| `\x7F`           | 恰好两位十六进制字符码        |
| `\x{10FFFF}`     | 可变长度 Unicode 码点         |
| `\u007F`         | 恰好四位十六进制 Unicode 码点 |
| `\u{7F}`         | 可变长度 Unicode 码点         |
| `\U0000007F`     | 恰好八位十六进制 Unicode 码点 |
| `\U{7F}`         | 可变长度 Unicode 码点         |
| `\p{...}`        | Unicode 属性类                |
| `\P{...}`        | Unicode 属性类补集            |
| `\d`, `\s`, `\w` | Perl 风格字符类               |
| `\D`, `\S`, `\W` | Perl 风格字符类补集           |

ASCII 标点通常可以在前面添加 `\` 以匹配字面量。`<` 和 `>` 具有边界转义含义；需要明确匹配它们时，可以使用 `[<>]`、`\x3C` 和 `\x3E`。

八进制转义 `\123` 在当前实现中未启用。由于反向引用同样不受支持，使用 `\1`、`\2` 等写法会产生正则编译错误，而不是引用捕获组。

## 10. 模式标志

### 10.1 标志语法

| 语法         | 含义                           |
| ------------ | ------------------------------ |
| `(?i)`       | 从当前位置开始启用 `i`         |
| `(?-i)`      | 从当前位置开始关闭 `i`         |
| `(?im)`      | 同时启用 `i` 和 `m`            |
| `(?i-m)`     | 启用 `i`，关闭 `m`             |
| `(?i:exp)`   | 只在该非捕获分组内启用 `i`     |
| `(?i-m:exp)` | 只在该分组内启用或关闭指定标志 |

### 10.2 可用标志

| 标志 | 作用                                          |
| ---- | --------------------------------------------- |
| `i`  | Unicode 大小写不敏感匹配                      |
| `m`  | 多行模式，使 `^` 和 `$` 同时匹配行边界        |
| `s`  | 允许 `.` 匹配 `\n`                            |
| `R`  | CRLF 模式；配合 `m` 识别 `\r\n` 行边界        |
| `U`  | 交换贪婪和懒惰重复的含义                      |
| `u`  | Unicode 模式；默认开启                        |
| `x`  | 扩展模式：忽略空白，并允许以 `#` 开始的行注释 |

扩展模式 `x` 会忽略字符类内部的空白。需要匹配空格时，应使用 `\ `、`\x20` 或在局部分组中关闭 `x`：

```text
(?x)
hello       # 匹配 hello
\x20world  # 再匹配一个空格和 world
```

可以局部关闭 Unicode 模式来获得 ASCII `\w`、`\d`、`\s` 和单词边界：

```text
(?-u:\w+)
```

因为本项目处理的是有效 UTF-8 字符串，关闭 Unicode 后会导致表达式可能匹配无效 UTF-8 字节的写法将被拒绝。例如不要使用 `(?-u:.)` 匹配任意字节。

## 11. 不支持的语法

为了保证可预测的运行时间和避免灾难性回溯，当前引擎不支持以下常见功能：

| 不支持的功能     | 常见写法                |
| ---------------- | ----------------------- | ---- |
| 正向先行断言     | `(?=exp)`               |
| 负向先行断言     | `(?!exp)`               |
| 正向后行断言     | `(?<=exp)`              |
| 负向后行断言     | `(?<!exp)`              |
| 模式内反向引用   | `\1`, `\2`, `\k<name>`  |
| 递归和子程序调用 | `(?R)`, `(?1)`          |
| 条件表达式       | `(?(condition)yes       | no)` |
| 原子分组         | `(?>exp)`               |
| 占有量词         | `x*+`, `x++`, `x?+`     |
| PCRE 控制动词    | `(*SKIP)`, `(*FAIL)` 等 |
| 任意字形簇类     | `\X`                    |
| 引号式字面量     | `\Q...\E`               |
| 其他引擎的锚点   | `\G`, `\Z`              |
| 内联注释组       | `(?# comment)`          |

注意区分：模式内的反向引用不支持，但 `string.regex_gsub` 的替换字符串支持 `$0` 至 `$9` 引用本次匹配的捕获内容。

## 12. 替换语法

### 12.1 字符串替换

`string.regex_gsub` 的 `repl` 为字符串时支持：

| 语法         | 含义                 |
| ------------ | -------------------- |
| `$0`         | 完整匹配             |
| `$1` 至 `$9` | 第 1 至第 9 个捕获组 |
| `$$`         | 字面量 `$`           |

示例：

```lua
local result, count = string.regex_gsub {
  text = "Ada:42",
  pattern = [[(\w+):(\d+)]],
  repl = "$2 / $1",
}

-- result == "42 / Ada"
-- count == 1
```

当前替换解析器每次只读取 `$` 后的一个数字：

- `$10` 表示 `$1` 后跟字面量 `0`，不是第 10 个捕获组。
- 不支持 `$name` 或 `${name}` 命名替换。
- 对不存在或未参与匹配的捕获组，不插入任何内容。

### 12.2 表替换

`repl` 为表时：

1. 有捕获组时，以第一个捕获组作为表键。
2. 没有捕获组时，以完整匹配作为表键。
3. 表值为字符串、整数或浮点数时，转换为字符串并替换。
4. 表值缺失、为 `false` 或为其他不支持类型时，保留原匹配文本。

### 12.3 函数替换

`repl` 为函数时：

1. 有捕获组时，按顺序传入所有捕获组。
2. 没有捕获组时，传入完整匹配。
3. 返回字符串、整数或浮点数时，转换为替换文本。
4. 返回 `nil`、`false` 或其他不支持类型时，保留原匹配文本。

替换函数运行在当前 Lua 回调预算内，不能借此重置执行时间或指令预算。

## 13. 各 API 的匹配结果

### 13.1 `regex_find`

```lua
local start_pos, finish_pos, captures = string.regex_find {
  text = "项目-123",
  pattern = [[(\p{Han}+)-(\d+)]],
  init = 1,
}
```

- `start_pos` 和 `finish_pos` 是从 `1` 开始的 Unicode 字符位置，不是 UTF-8 字节位置。
- `finish_pos` 是包含式终点。
- 空匹配的终点会比起点小 `1`；例如输入开头的空匹配返回 `start_pos = 1`, `finish_pos = 0`。
- `captures[1]`、`captures[2]` 依次保存捕获组。
- 没有匹配时，第一个返回值为 `nil`。
- 没有捕获组时，`captures` 是空表。

`init` 也是从 `1` 开始的 Unicode 字符位置；负数从文本末尾计算，`0` 非法。

### 13.2 `regex_match`

```lua
local name, score = string.regex_match {
  text = "Ada:42",
  pattern = [[(\w+):(\d+)]],
}
```

- 有捕获组时，按捕获组顺序返回若干值。
- 没有捕获组时，返回完整匹配。
- 没有匹配时返回 `nil`。

### 13.3 `regex_gmatch`

```lua
local iterator = string.regex_gmatch {
  text = "A1 B2 C3",
  pattern = [[([A-Z])(\d)]],
}

for letter, number in iterator do
  debug.print { message = letter .. ":" .. number }
end
```

- 返回非重叠匹配。
- 有捕获组时，每轮返回捕获组。
- 没有捕获组时，每轮返回完整匹配。
- 迭代结束时不返回值。

### 13.4 `regex_test`

```lua
local contains_number = string.regex_test {
  text = "abc123",
  pattern = [[\d+]],
}
```

`regex_test` 判断任意位置是否存在匹配。需要验证整个字符串时，应使用 `\A` 和 `\z`：

```lua
local valid = string.regex_test {
  text = "123",
  pattern = [[\A\d+\z]],
}
```

### 13.5 `regex_split`

```lua
local parts = string.regex_split {
  text = "one, two;three",
  pattern = [[\s*[,;]\s*]],
}

-- parts[1] == "one"
-- parts[2] == "two"
-- parts[3] == "three"
```

开头、结尾或相邻的分隔匹配会产生空字符串项。捕获组不会被额外插入分割结果。

## 14. 安全限制

| 项目                  | 限制                                   |
| --------------------- | -------------------------------------- |
| 正则表达式源码        | 最大 8 KiB                             |
| 编译后的正则          | 最大 1 MiB                             |
| 普通 API 输入字符串   | 最大 1 MiB                             |
| `regex_gmatch` 结果   | 最多 10,000 项，捕获文本合计最大 1 MiB |
| `regex_split` 结果    | 最多 10,000 项，文本合计最大 1 MiB     |
| `regex_gsub` 替换次数 | 最多 10,000 次                         |
| `regex_gsub` 输出     | 最大 1 MiB                             |

每次匹配、分割或替换 API 调用都会编译本次传入的正则，目前不提供脚本可见的已编译正则对象或正则缓存。高频回调中应避免反复构造复杂表达式。

该引擎不使用无上限回溯，因此不会出现典型 PCRE 灾难性回溯，但大型输入、复杂 Unicode 字符类和大量捕获仍会消耗时间与内存，并继续受 Lua 回调时间、指令和内存预算约束。

所有输入和输出都是有效 UTF-8 字符串。该 API 不用于任意二进制字节匹配。

## 15. 注意事项与建议

1. 需要匹配整个输入时使用 `\A...\z`，不要只依赖 `regex_test`。
2. 处理用户输入时先调用 `string.regex_escape`。
3. 推荐使用 Lua 长字符串编写正则，减少双重反斜杠转义。
4. 位置返回值按 Unicode 字符计算，不是字节偏移，也不是终端显示宽度。
5. `.` 匹配 Unicode 标量值，不等于一个终端字符格或一个完整 Emoji。
6. `\d`、`\s`、`\w` 默认是 Unicode 语义；只需要 ASCII 时明确使用 POSIX 类。
7. 命名捕获不会在 Lua 捕获表中生成名称键。
8. 替换引用只支持 `$0..$9` 和 `$$`。
9. 正则不执行 Unicode 规范化，也不支持大小写折叠之外的语言特定文本规则。
10. 空匹配可能产生大量结果；使用 `regex_gmatch`、`regex_split` 和 `regex_gsub` 时应特别注意表达式是否能够匹配空字符串。

## 16. 综合示例

```lua
local source = "用户: Alice, 分数: 120\n用户: Bob, 分数: 98"
local pattern = [[(?m)^用户:\s*(\p{Alphabetic}+),\s*分数:\s*(\d+)$]]

local iterator = string.regex_gmatch {
  text = source,
  pattern = pattern,
}

for name, score in iterator do
  debug.print {
    message = name .. " => " .. score,
  }
end

local normalized, count = string.regex_gsub {
  text = source,
  pattern = pattern,
  repl = "$1=$2",
}

-- normalized:
-- Alice=120
-- Bob=98
-- count == 2
```

---

# 附录：Lua 模式与正则表达式对比速查

| 特性               | Lua 模式      | 正则表达式（本实现） |
| ------------------ | ------------- | -------------------- | --- |
| 转义字符           | `%`           | `\`                  |
| 任意字符（除换行） | `.`           | `.`                  |
| 数字字符类         | `%d`          | `\d`（默认 Unicode） |
| 空白字符类         | `%s`          | `\s`（默认 Unicode） |
| 字母数字字符类     | `%w`          | `\w`（默认 Unicode） |
| 字符集合           | `[abc]`       | `[abc]`              |
| 补集               | `[^abc]`      | `[^abc]`             |
| 集合运算           | 不支持        | `&&`、`--`、`~~`     |
| Unicode 属性       | 不支持        | `\p{L}` 等           |
| 贪婪重复           | `*`、`+`、`?` | `*`、`+`、`?`        |
| 非贪婪重复         | `-`           | `*?`、`+?`、`??`     |
| 分支选择           | 不支持        | `                    | `   |
| 捕获组             | `(pattern)`   | `(pattern)`          |
| 非捕获组           | 不支持        | `(?:pattern)`        |
| 命名捕获组         | 不支持        | `(?<name>pattern)`   |
| 锚点               | `^`、`$`      | `^`、`$`、`\A`、`\z` |
| 单词边界           | 不支持        | `\b`、`\B`           |
| 平衡匹配           | `%b()`        | 不支持               |
| 反向引用           | 不支持        | 不支持               |
| 先行/后行断言      | 不支持        | 不支持               |
| 替换引用（`gsub`） | `$0`..`$9`    | `$0`..`$9`           |

> **特别注意**：本框架的 `string.gsub` 替换引用统一使用 `$` 前缀（标准 Lua 为 `%`），与正则表达式系列 API 保持一致。
