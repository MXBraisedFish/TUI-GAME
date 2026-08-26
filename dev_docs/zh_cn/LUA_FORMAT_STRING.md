# Lua 二进制格式字符串教程

## 1. 格式字符串是什么

`serialization.binary_pack`、`serialization.binary_unpack` 和 `serialization.binary_packsize` 使用格式字符串描述二进制数据中每个字段的类型、字节数、字节序与对齐方式。

它适合读取或生成固定协议、存档片段、文件头等二进制数据。Lua 字符串可以保存任意字节，因此打包结果可能包含 `\0`，不能把它当作普通 UTF-8 文本处理。

```lua
local data = serialization.binary_pack {
  fmt = "<I2I4",
  values = { 7, 1000 }
}

local result = serialization.binary_unpack {
  fmt = "<I2I4",
  data = data
}

debug.print {
  message = tostring(result.values[1]) .. ", " .. tostring(result.values[2])
}
```

这里的 `<` 表示小端序，`I2` 表示 2 字节无符号整数，`I4` 表示 4 字节无符号整数。

## 2. 三个方法

### `binary_pack`

按照 `fmt` 依次读取 `values`，返回二进制字符串。

```lua
local data = serialization.binary_pack {
  fmt = ">I2z",
  values = { 513, "TUI" }
}
```

格式中的填充和对齐操作不消耗 `values`。值过少或过多都会报错。

### `binary_unpack`

从 `data` 的 `pos` 位置开始解包。`pos` 为一基字节位置，默认值为 `1`。

```lua
local result = serialization.binary_unpack {
  fmt = ">I2z",
  data = data,
  pos = 1
}

local number = result.values[1]
local text = result.values[2]
local next_pos = result.next_pos
```

返回表结构固定为：

```lua
{
  values = { ... },
  next_pos = 读取结束后的下一字节位置
}
```

`next_pos` 可以直接用于连续读取：

```lua
local first = serialization.binary_unpack {
  fmt = "<I2",
  data = data
}

local second = serialization.binary_unpack {
  fmt = "<I2",
  data = data,
  pos = first.next_pos
}
```

### `binary_packsize`

计算固定长度格式最终占用的字节数。

```lua
local size = serialization.binary_packsize("<I2I4x")
-- 2 + 4 + 1 = 7
```

`z` 和 `s[n]` 的长度取决于实际字符串，不能用 `binary_packsize` 预先计算，使用时会报错。

## 3. 字节序

| 标记 | 含义 |
| ---- | ---- |
| `<` | 后续多字节字段使用小端序 |
| `>` | 后续多字节字段使用大端序 |
| `=` | 后续多字节字段使用宿主原生字节序 |

未指定时默认使用宿主原生字节序。字节序标记可以在同一个格式字符串中多次出现，只影响它之后的字段。

为了让文件和网络协议在不同设备上保持一致，建议明确使用 `<` 或 `>`，不要依赖 `=`。

## 4. 整数类型

| 标记 | 字节数 | 类型 |
| ---- | ------ | ---- |
| `b` | 1 | 有符号整数 |
| `B` | 1 | 无符号整数 |
| `h` | 2 | 有符号整数 |
| `H` | 2 | 无符号整数 |
| `l`、`j` | 8 | 有符号整数 |
| `L`、`J`、`T` | 8 | 无符号整数 |
| `i[n]` | 默认 4，可指定 1–16 | 有符号整数 |
| `I[n]` | 默认 4，可指定 1–16 | 无符号整数 |

方括号表示文档中的可选部分，实际格式不写方括号。例如：

```lua
local data = serialization.binary_pack {
  fmt = "<i2I4",
  values = { -20, 4000 }
}
```

传给整数字段的值必须是 Lua `integer`。无符号字段不能接收负数，数值也必须能放入目标字节数。

虽然 `i[n]` 和 `I[n]` 允许 9–16 字节字段，但 Lua 整数仍是 64 位。额外字节只能是合法的符号扩展或零扩展；解包结果若超出 Lua 整数范围会报错。

## 5. 浮点类型

| 标记 | 字节数 | 类型 |
| ---- | ------ | ---- |
| `f` | 4 | IEEE 754 单精度浮点数 |
| `d`、`n` | 8 | IEEE 754 双精度浮点数 |

```lua
local data = serialization.binary_pack {
  fmt = "<fd",
  values = { 1.5, 3.1415926 }
}
```

整数可以作为浮点字段的输入。`NaN`、正无穷和负无穷不能被打包。

## 6. 字符串类型

| 标记 | 含义 |
| ---- | ---- |
| `cN` | 固定占用 `N` 字节；不足部分补零，输入超过 `N` 字节时报错 |
| `z` | 以零字节结尾；输入自身不能包含零字节 |
| `s[n]` | 先写入长度，再写字符串；长度字段默认使用宿主 `usize` 字节数，可指定 1–8 |

`N` 和 `n` 都按字节计算，不按 Unicode 字符数计算。

```lua
local fixed = serialization.binary_pack {
  fmt = "c8",
  values = { "TUI" }
}

local variable = serialization.binary_pack {
  fmt = "<s2",
  values = { "你好" }
}
```

第二个示例的文本有 2 个字符，但 UTF-8 数据长度为 6 字节，因此长度前缀记录的是 `6`。

解包 `cN` 会完整返回 `N` 个字节，末尾补入的零字节不会自动删除。

## 7. 填充与对齐

### `x`：显式填充

每个 `x` 写入或跳过一个字节，不消耗值。

```lua
local data = serialization.binary_pack {
  fmt = "BxI2",
  values = { 1, 2 }
}
```

### `!n`：最大对齐

设置后续字段的最大对齐值。`n` 必须是 `1..16` 范围内的 2 的幂；省略时使用宿主 `usize` 的字节数。

```lua
local size = serialization.binary_packsize("!4BI4")
```

在这个例子中，`B` 占 1 字节，随后的 `I4` 会先补齐到 4 字节边界，因此总大小为 8 字节。

默认最大对齐为 `1`，也就是默认不会自动插入对齐字节。

### `Xop`：按指定字段对齐

`X` 后面必须紧跟一个固定长度格式项。它只按照该格式项的对齐要求补齐当前位置，不读取或写入该字段，也不消耗值。

```lua
local data = serialization.binary_pack {
  fmt = "!4BXI4I4",
  values = { 1, 100 }
}
```

`XI4` 负责把位置对齐到 `I4` 的边界，真正的 `I4` 才会消费数值 `100`。

## 8. 空白与组合

空格、制表符和换行会被忽略，可以用来提高可读性：

```lua
local fmt = [[
  <
  I2 I2
  c4
]]
```

格式字符串中的控制标记按顺序生效，不能使用括号、重复次数或自定义类型名。

## 9. 错误处理

格式错误、数据不足、范围溢出或字符串不符合要求时，方法会抛出普通 Lua 错误，可以用 `debug.pcall` 捕获：

```lua
local result = debug.pcall {
  func = function()
    return serialization.binary_unpack {
      fmt = ">I4",
      data = "short"
    }
  end
}

if not result.ok then
  debug.warn(result.error_message)
end
```

## 10. 安全限制

- 格式字符串最大 8 KiB。
- 一个格式字符串最多包含 8192 个格式操作。
- 输入及打包输出最大 1 MiB。
- 整数大小 `i[n]`、`I[n]` 限制为 1–16 字节。
- 长度前缀 `s[n]` 限制为 1–8 字节。
- 对齐值限制为 `1、2、4、8、16`。
- 所有尺寸和位置计算都会检查溢出。

这些限制用于避免恶意或错误格式串产生过量内存占用。
