# Lua 二进制格式字符串教程

## 前言

二进制格式化字符串作为二进制数据包核心参数，用于精确描述如何在 Lua 值与原始字节流之间进行双向转换。本文档旨在为开发者提供一份语法速查手册。

> 如果需要更完整的规范要求，可见 Lua 语言官方文档，Tui Game 仅在 API 的使用上稍作修改，语法规则与原版无异。

---

## 目录

1. 字节序控制
2. 对齐控制
3. 原生整数类型（C 语言映射）
4. 固定尺寸整数类型
5. 浮点数类型
6. 字符串类型
7. 特殊填充与空白符

---

## 字节序控制

用于指定多字节数值在二进制数据中的字节排列顺序。

### 语法

| 符号 | 说明               |
| ---- | ------------------ |
| `<`  | 使用小端序         |
| `>`  | 使用大端序         |
| `=`  | 使用系统默认字节序 |

### 示例

```lua
little = serialization.binary_pack {
  fmt = "<I4",
  values = { 0x12345678 }
}

big = serialization.binary_pack {
  fmt = ">I4",
  values = { 0x12345678 }
}

debug.print { message = tostring(#little) }
debug.print { message = tostring(#big) }
```

输出：

```text
4
4
```

---

## 对齐控制

用于控制数据项在二进制结构中的对齐方式。

### 语法

| 符号  | 说明                                             |
| ----- | ------------------------------------------------ |
| `!n`  | 设置最大对齐值                                   |
| `Xop` | 按 `op` 对应的数据类型执行一次对齐（上限取`!n`） |

### 额外补充

- `!n` 中的 `n` 为对齐字节数。
- `n` 必须为 $2^x$，且范围为 $[2, 16]$。
- 对齐产生的空位由格式处理自动完成，不需要额外提供数据。
- `Xop` 本身不读取或写入数据，仅使用 `op` 的对齐规则。

### 示例

```lua
size1 = serialization.binary_packsize("c1 i4")
size2 = serialization.binary_packsize("!4 c1 i4")

debug.print { message = tostring(size1) }
debug.print { message = tostring(size2) }

bytes1 = serialization.binary_pack {
  fmt = "c1 Xi4 i4",
  values = { "A", 100 }
}
bytes2 = serialization.binary_pack {
  fmt = "!4 c1 Xi4 i4", -- Xop 需要 !n 开头
  values = { "A", 100 }
}

debug.print { message = tostring(#bytes1) }
debug.print { message = tostring(#bytes2) }
```

输出：

```text
5
8
5
8
```

---

## 原生整数类型

用于按照运行环境对应的原生整数类型保存数值。

### 语法

| 符号 | 说明             | 类型        | 有符号 |
| ---- | ---------------- | ----------- | ------ |
| `b`  | 1字节有符号整数  | char        | 是     |
| `B`  | 1字节无符号整数  | char        | 否     |
| `h`  | 短字节有符号整数 | short       | 是     |
| `H`  | 短字节无符号整数 | short       | 否     |
| `i`  | 有符号整数       | int         | 是     |
| `I`  | 无符号整数       | int         | 否     |
| `l`  | 长字节有符号整数 | long        | 是     |
| `L`  | 长字节无符号整数 | long        | 否     |
| `j`  | Lua 无符号整数   | Lua Integer | 是     |
| `J`  | Lua 无符号整数   | Lua Integer | 否     |

### 额外补充

- 小写符号表示有符号整数，大写符号表示无符号整数。
- `h`/`H`/`i`/`I`/`l`/`L` 的实际大小由运行环境决定。

### 示例

```lua
size1 = serialization.binary_packsize("b B")
size2 = serialization.binary_packsize("h H")
size3 = serialization.binary_packsize("i I")
size4 = serialization.binary_packsize("l L")
size5 = serialization.binary_packsize("j J")

debug.print { message = tostring(size1) }
debug.print { message = tostring(size2) }
debug.print { message = tostring(size3) }
debug.print { message = tostring(size4) }
debug.print { message = tostring(size5) }
```

输出：

```lua
2  -- 每个数据为 1 字节
4  -- 每个数据为 2 字节
8  -- 每个数据为 4 字节
16 -- 每个数据为 8 字节
16 -- 每个数据为 8 字节
```

---

## 固定尺寸整数类型

用于显式指定整数占用的字节数。

### 语法

| 格式   | 说明               |
| ------ | ------------------ |
| `i[n]` | `n` 字节有符号整数 |
| `I[n]` | `n` 字节无符号整数 |

其中 `[n]` 表示实际使用时填写整数，例如：

```text
i1
I2
i4
I8
```

### 额外补充

- `n` 用于指定整数占用的字节数。
- 小写 `i` 表示有符号整数。
- 大写 `I` 表示无符号整数。
- 固定尺寸不会随运行平台变化，适合用于明确的二进制数据结构。
- 数值必须能够由指定的整数尺寸表示。

### 示例

```lua
bytes = serialization.binary_pack {
    fmt = "<i1 I2 i4",
    values = {
        -1,
        65535,
        123456
    }
}

result = serialization.binary_unpack {
    fmt = "<i1 I2 i4",
    data = bytes
}

debug.print {
    message =
        tostring(result.values[1]) .. ", " ..
        tostring(result.values[2]) .. ", " ..
        tostring(result.values[3])
}
```

输出：

```text
-1, 65535, 123456
```

这里同时使用了 1 字节、2 字节和 4 字节整数。

---

## 浮点数类型

用于保存带小数的数值。

### 语法

| 符号 | 说明          |
| ---- | ------------- |
| `f`  | float 浮点数  |
| `d`  | double 浮点数 |
| `n`  | Lua Number    |

### 额外补充

- `f` 通常用于单精度浮点数。
- `d` 通常用于双精度浮点数。
- `n` 使用 Lua 当前的 Number 类型。
- 浮点数同样会受到格式字符串中字节序设置的影响。

### 示例

```lua
bytes = serialization.binary_pack {
    fmt = "<f d",
    values = {
        1.25,
        100.5
    }
}

result = serialization.binary_unpack {
    fmt = "<f d",
    data = bytes
}

debug.print {
    message =
        tostring(result.values[1]) .. ", " ..
        tostring(result.values[2])
}
```

输出类似：

```text
1.25, 100.5
```

---

## 字符串类型

二进制格式字符串提供固定长度、空字符结尾和长度前缀三种字符串格式。

### 语法

| 格式   | 说明                        |
| ------ | --------------------------- |
| `c[n]` | 固定 `n` 字节字符串         |
| `z`    | 以 `\0` 结尾的字符串        |
| `s[n]` | 带 `n` 字节长度前缀的字符串 |

### 额外补充

#### `c[n]`

按照指定的固定字节数保存字符串。

例如：

```text
c4
```

表示该字段固定占用 4 个字节。

#### `z`

保存以空字符 `\0` 作为结束标记的字符串。

打包时会写入结束标记，解包时读取到 `\0` 为止。

#### `s[n]`

首先保存字符串长度，再保存字符串本身。

例如：

```text
s1
```

表示先使用 1 字节无符号整数保存字符串长度，随后保存字符串内容。

适合长度不固定的字符串字段。

### 示例

固定长度字符串：

```lua
bytes = serialization.binary_pack {
    fmt = "c3",
    values = { "ABC" }
}

result = serialization.binary_unpack {
    fmt = "c3",
    data = bytes
}

debug.print { message = result.values[1] }
```

输出：

```text
ABC
```

空字符结尾字符串：

```lua
bytes = serialization.binary_pack {
    fmt = "z",
    values = { "Tui Game" }
}

result = serialization.binary_unpack {
    fmt = "z",
    data = bytes
}

debug.print { message = result.values[1] }
```

输出：

```text
Tui Game
```

长度前缀字符串：

```lua
bytes = serialization.binary_pack {
    fmt = "s1",
    values = { "Hello" }
}

debug.print { message = tostring(#bytes) }
```

输出：

```text
6
```

其中：

```text
1 字节字符串长度
+
5 字节字符串内容
=
6 字节
```

---

## 填充与空白符

用于添加固定填充字节，或提高格式字符串本身的可读性。

### 语法

| 符号 | 说明                       |
| ---- | -------------------------- |
| `x`  | 插入或跳过一个空字节       |
| 空格 | 无实际作用，仅用于分隔格式 |

### 额外补充

- `x` 在打包时插入一个 `0x00` 字节。
- 解包时，`x` 会跳过对应位置的一个字节。
- `x` 不需要对应 `values` 中的数据。
- 格式字符串中的空格会被忽略，可以自由用于分隔不同字段。

例如：

```text
<I2s2I4
```

和：

```text
< I2 s2 I4
```

表示相同的格式。

### 示例

在两个整数之间加入一个填充字节：

```lua
bytes = serialization.binary_pack {
    fmt = "<I2 x I2",
    values = {
        100,
        200
    }
}

debug.print { message = tostring(#bytes) }
```

输出：

```text
5
```

其中两个 `I2` 分别占用 2 字节，中间的 `x` 占用 1 字节：

```text
2 + 1 + 2 = 5
```

解包时不需要为 `x` 接收任何值：

```lua
result = serialization.binary_unpack {
    fmt = "<I2 x I2",
    data = bytes
}

debug.print {
    message =
        tostring(result.values[1]) ..
        ", " ..
        tostring(result.values[2])
}
```

输出：

```text
100, 200
```

---

## 格式组合

实际使用时，不同格式可以直接组合。

例如：

```text
< I4 s1 f
```

可以理解为：

| 格式 | 说明                     |
| ---- | ------------------------ |
| `<`  | 后续多字节数据使用小端序 |
| `I4` | 4 字节无符号整数         |
| `s1` | 1 字节长度前缀字符串     |
| `f`  | 浮点数                   |

使用：

```lua
bytes = serialization.binary_pack {
    fmt = "< I4 s1 f",
    values = {
        100,
        "Player",
        12.5
    }
}

result = serialization.binary_unpack {
    fmt = "< I4 s1 f",
    data = bytes
}

debug.print {
    message =
        tostring(result.values[1]) .. ", " ..
        result.values[2] .. ", " ..
        tostring(result.values[3])
}
```

输出：

```text
100, Player, 12.5
```

对于普通脚本开发，通常只需要根据数据结构选择对应的格式符号，并保证打包与解包使用相同的格式字符串即可。
