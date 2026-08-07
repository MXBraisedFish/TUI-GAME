# TUI GAME Lua API 文档

| 项目         | 内容                                                             |
| ---------- | -------------------------------------------------------------- |
| **API 版本** | 1                                                              |
| **最后更新日期** | （待补充）                                                          |
| **更新作者**   | MXFish                                                         |
| **文档作用** | 面向最终上层脚本开发者，提供 TUI GAME 引擎注入到 Lua 沙箱中的全部 API（18 个库）的索引与查询手册入口。 |

## 文档目录

1. [API 使用标准](#1-api-使用标准)
2. [子库文档引导](#2-子库文档引导)
3. [全部库常量与方法总览](#3-全部库常量与方法总览)

---

## 1. API 使用标准

本节约定所有库共用的调用与行为规则，各子库文档不再重复。

### 1.1 库表与只读

- 引擎在 Lua 沙箱中注入 18 个库：`base`、`math`、`utf8`、`table`、`string`、`color`、`char`、`align`、`measurement`、`draw`、`debug`、`game`、`event`、`loader`、`file`、`serialization`、`encoding`、`random`。
- 所有库表均为**只读**：直接赋值、修改或增删字段会抛出错误（可用 `debug.pcall` 捕获）。
- 沙箱不注入任何同名全局函数（如 `type`、`pairs`、`print`、`require` 等），相关能力统一挂载在各库表下。
- 只读库表可由 `base.pairs`、`base.ipairs`、`base.rawlen` 与 `#` 读取，但不能修改。

### 1.2 参数传递约定

- **单参数方法**：可写作 `lib.method(value)`，也可写作 `lib.method{ value = value }`。
- **多参数方法**：必须传一个**命名参数表** `lib.method{ param = value, ... }`。
- **未知参数名**：命名参数表中出现未声明的字段会直接报错。
- **类型严格**：参数类型不符、取值越界或缺少必填参数时抛出可捕获的运行时错误；不使用隐式转换。

### 1.3 通用限制

- 字符串参数与结果上限：**1 MiB**。
- 参数表上限：**16,384 项**，嵌套深度上限 **32 层**。
- 正则/模式匹配结果上限：**10,000 项**；`table.sort` 数组上限 **4,096 项**。
- 单次回调内宿主命令上限：**4,096 条**；单帧绘制命令上限 **4,096 条**、绘制文本总量上限 **1 MiB**。
- 违反资源上限且无法恢复时，回调以致命错误终止（`debug.pcall` 无法捕获）。

### 1.4 方法要求（Method Requirements）

每个方法可能带有以下前置要求，不满足时调用被**静默忽略**（仅首次通过日志提示一次），不抛错：

| 要求 | 含义 |
| ---- | ---- |
| **无** | 无额外限制，任何会话均可调用。 |
| **开启调试** | 需在宿主配置中开启调试模式（`debug_enabled`）。 |
| **仅游戏脚本** | 仅游戏会话可用；屏保会话中调用被忽略。 |
| **关闭安全模式** | 需宿主关闭安全模式（`safe_mode_enabled = false`）。屏保会话始终处于安全模式。 |

### 1.5 错误处理

- 所有可预期错误（参数错误、越界、非法常量等）均为 Lua 运行时错误，可用 `debug.pcall` / `debug.xpcall` 捕获。
- 致命资源超限、内存不足错误会绕过 `pcall` 直接使会话进入故障状态。

### 1.6 运行时阶段约束

- `draw.*` 方法只能在 `Render` 回调阶段调用，其余阶段调用会报错。
- `game.save_game` / `game.save_best` 在 `SaveGame` / `SaveBest` 回调内调用会被忽略（防止递归保存）。
- 完整的回调协议（`Init`、`HandleEvent`、`Update`、`UpdateFrame`、`Render`、`SaveGame`、`SaveBest`）请参阅 [Lua Runtime 协议](LUA_RUNTIME.md)。

---

## 2. 子库文档引导

| 库名 | 基本说明 | 文档 |
| ---- | -------- | ---- |
| `base` | Lua 基础值操作：迭代、类型转换、保护性读取 | [base](api/base.md) |
| `math` | 数学运算：三角函数、对数、取整、组合数等 | [math](api/math.md) |
| `utf8` | UTF-8 字符串处理：字符数、码点、ASCII 互转 | [utf8](api/utf8.md) |
| `table` | 表操作：拼接、插入、移动、排序、打包 | [table](api/table.md) |
| `string` | 字符串处理：大小写、查找、匹配、格式、正则 | [string](api/string.md) |
| `color` | 颜色常量与 RGB/HEX 颜色构造 | [color](api/color.md) |
| `char` | 边框字符集与 ASCII 字符数组常量 | [char](api/char.md) |
| `align` | 基于终端尺寸的对齐坐标计算 | [align](api/align.md) |
| `measurement` | 文本尺寸测量 | [measurement](api/measurement.md) |
| `draw` | 绘制：文本、填充矩形、描边矩形、擦除、请求渲染 | [draw](api/draw.md) |
| `debug` | 调试输出、断言与受保护调用 | [debug](api/debug.md) |
| `game` | 游戏生命周期控制：退出、保存 | [game](api/game.md) |
| `event` | 事件处理：跳过、清空动作队列 | [event](api/event.md) |
| `loader` | 加载并执行 `scripts/` 目录下的 Lua 模块 | [loader](api/loader.md) |
| `file` | 异步文件读写与目录枚举（结果经事件回调返回） | [file](api/file.md) |
| `serialization` | 多格式序列化：JSON/CSV/YAML/TOML/INI/XML 与二进制打包解包 | [serialization](api/serialization.md) |
| `encoding` | 编码转换：Base64、URL 百分号编码、十六进制 | [encoding](api/encoding.md) |
| `random` | 随机数：直接生成与生成器对象管理 | [random](api/random.md) |

---

## 3. 全部库常量与方法总览

### 3.1 常量

| 常量名 | 作用 | 子文档 |
| ------ | ---- | ------ |
| `math.PI` | 圆周率 π | [math](api/math.md) |
| `math.E` | 自然常数 e | [math](api/math.md) |
| `math.POSITIVE_INFINITE` | 正无穷（等价 `INFINITE`） | [math](api/math.md) |
| `math.INFINITE` | 正无穷 | [math](api/math.md) |
| `math.NEGATIVE_INFINITE` | 负无穷 | [math](api/math.md) |
| `math.DEG` | 弧度转角度系数（`180/π`） | [math](api/math.md) |
| `math.RAD` | 角度转弧度系数（`π/180`） | [math](api/math.md) |
| `math.MAX_INTEGER` | 最大整数 `2^63-1` | [math](api/math.md) |
| `math.MIN_INTEGER` | 最小整数 `-2^63` | [math](api/math.md) |
| `string.AUTO` | 文本模式：自动 | [string](api/string.md) |
| `string.PLAIN_TEXT` | 文本模式：纯文本 | [string](api/string.md) |
| `string.RICH_TEXT` | 文本模式：富文本 | [string](api/string.md) |
| `color.BLACK` | 黑色 | [color](api/color.md) |
| `color.RED` | 红色 | [color](api/color.md) |
| `color.GREEN` | 绿色 | [color](api/color.md) |
| `color.YELLOW` | 黄色 | [color](api/color.md) |
| `color.BLUE` | 蓝色 | [color](api/color.md) |
| `color.MAGENTA` | 品红 | [color](api/color.md) |
| `color.CYAN` | 青色 | [color](api/color.md) |
| `color.GRAY` | 灰色（等价 `GREY`） | [color](api/color.md) |
| `color.GREY` | 灰色 | [color](api/color.md) |
| `color.BRIGHT_GRAY` | 亮灰（等价 `BRIGHT_GREY`） | [color](api/color.md) |
| `color.BRIGHT_GREY` | 亮灰 | [color](api/color.md) |
| `color.BRIGHT_RED` | 亮红 | [color](api/color.md) |
| `color.BRIGHT_GREEN` | 亮绿 | [color](api/color.md) |
| `color.BRIGHT_YELLOW` | 亮黄 | [color](api/color.md) |
| `color.BRIGHT_BLUE` | 亮蓝 | [color](api/color.md) |
| `color.BRIGHT_MAGENTA` | 亮品红 | [color](api/color.md) |
| `color.BRIGHT_CYAN` | 亮青 | [color](api/color.md) |
| `color.WHITE` | 白色 | [color](api/color.md) |
| `color.NONE` | 无颜色（等价省略颜色参数） | [color](api/color.md) |
| `color.TRANSPARENT` | 透明（仅背景色） | [color](api/color.md) |
| `char.LINE` | 单线边框字符表 | [char](api/char.md) |
| `char.BOLD_LINE` | 粗线边框字符表 | [char](api/char.md) |
| `char.DOUBLE_LINE` | 双线边框字符表 | [char](api/char.md) |
| `char.ROUNDED_LINE` | 圆角线边框字符表 | [char](api/char.md) |
| `char.ASCII_NUMBER` | `"0"`~`"9"` 字符数组 | [char](api/char.md) |
| `char.ASCII_LOWERCASE` | `"a"`~`"z"` 字符数组 | [char](api/char.md) |
| `char.ASCII_UPPERCASE` | `"A"`~`"Z"` 字符数组 | [char](api/char.md) |
| `char.ASCII_LETTER` | 大小写字母字符数组 | [char](api/char.md) |
| `char.ASCII_CHARACTER` | ASCII 符号字符数组 | [char](api/char.md) |
| `char.ASCII` | 数字+字母+符号全量字符数组 | [char](api/char.md) |
| `align.AUTO` | 自动对齐（水平居中 / 垂直居中） | [align](api/align.md) |
| `align.LEFT` | 左对齐 | [align](api/align.md) |
| `align.HORIZONTAL_CENTER` | 水平居中 | [align](api/align.md) |
| `align.RIGHT` | 右对齐 | [align](api/align.md) |
| `align.TOP` | 顶部对齐 | [align](api/align.md) |
| `align.VERTICAL_CENTER` | 垂直居中 | [align](api/align.md) |
| `align.BOTTOM` | 底部对齐 | [align](api/align.md) |
| `align.CENTER` | 双向居中 | [align](api/align.md) |
| `debug.VERSION` | 运行时版本标识字符串 | [debug](api/debug.md) |
| `file.AUTO` | 编码/换行：自动检测 | [file](api/file.md) |
| `file.ALL` | 换行：全部统一 | [file](api/file.md) |
| `file.CR` | 换行：`\r` | [file](api/file.md) |
| `file.LF` | 换行：`\n` | [file](api/file.md) |
| `file.CRLF` | 换行：`\r\n` | [file](api/file.md) |
| `file.UTF_8` | 编码：UTF-8 | [file](api/file.md) |
| `file.UTF_16LE` | 编码：UTF-16 小端 | [file](api/file.md) |
| `file.UTF_16BE` | 编码：UTF-16 大端 | [file](api/file.md) |
| `file.GBK` | 编码：GBK | [file](api/file.md) |
| `file.GB18030` | 编码：GB18030 | [file](api/file.md) |
| `file.BIG5` | 编码：Big5 | [file](api/file.md) |
| `file.SHIFT_JIS` | 编码：Shift-JIS | [file](api/file.md) |
| `file.EUC_JP` | 编码：EUC-JP | [file](api/file.md) |
| `file.ISO_2022_JP` | 编码：ISO-2022-JP | [file](api/file.md) |
| `file.EUC_KR` | 编码：EUC-KR | [file](api/file.md) |
| `file.WINDOWS_874` | 编码：Windows-874 | [file](api/file.md) |
| `file.WINDOWS_1250` | 编码：Windows-1250 | [file](api/file.md) |
| `file.WINDOWS_1251` | 编码：Windows-1251 | [file](api/file.md) |
| `file.WINDOWS_1252` | 编码：Windows-1252 | [file](api/file.md) |
| `file.WINDOWS_1253` | 编码：Windows-1253 | [file](api/file.md) |
| `file.WINDOWS_1254` | 编码：Windows-1254 | [file](api/file.md) |
| `file.WINDOWS_1255` | 编码：Windows-1255 | [file](api/file.md) |
| `file.WINDOWS_1256` | 编码：Windows-1256 | [file](api/file.md) |
| `file.WINDOWS_1257` | 编码：Windows-1257 | [file](api/file.md) |
| `file.WINDOWS_1258` | 编码：Windows-1258 | [file](api/file.md) |
| `file.ISO_8859_2` | 编码：ISO-8859-2 | [file](api/file.md) |
| `file.ISO_8859_3` | 编码：ISO-8859-3 | [file](api/file.md) |
| `file.ISO_8859_4` | 编码：ISO-8859-4 | [file](api/file.md) |
| `file.ISO_8859_5` | 编码：ISO-8859-5 | [file](api/file.md) |
| `file.ISO_8859_6` | 编码：ISO-8859-6 | [file](api/file.md) |
| `file.ISO_8859_7` | 编码：ISO-8859-7 | [file](api/file.md) |
| `file.ISO_8859_8` | 编码：ISO-8859-8 | [file](api/file.md) |
| `file.ISO_8859_8_I` | 编码：ISO-8859-8-I | [file](api/file.md) |
| `file.ISO_8859_10` | 编码：ISO-8859-10 | [file](api/file.md) |
| `file.ISO_8859_13` | 编码：ISO-8859-13 | [file](api/file.md) |
| `file.ISO_8859_14` | 编码：ISO-8859-14 | [file](api/file.md) |
| `file.ISO_8859_15` | 编码：ISO-8859-15 | [file](api/file.md) |
| `file.ISO_8859_16` | 编码：ISO-8859-16 | [file](api/file.md) |
| `file.KOI8_R` | 编码：KOI8-R | [file](api/file.md) |
| `file.KOI8_U` | 编码：KOI8-U | [file](api/file.md) |
| `file.IBM866` | 编码：IBM866 | [file](api/file.md) |
| `file.MACINTOSH` | 编码：Macintosh | [file](api/file.md) |
| `file.X_MAC_CYRILLIC` | 编码：x-mac-cyrillic | [file](api/file.md) |
| `random.INT` | 整数类型随机数生成器 | [random](api/random.md) |
| `random.FLOAT` | 浮点数类型随机数生成器 | [random](api/random.md) |

### 3.2 方法

> **要求** 列取值：**无** / **开启调试** / **仅游戏** / **关闭安全**（组合写法如「仅游戏 + 关闭安全」）。

| 方法 | 要求 | 返回 | 作用 | 子文档 |
| ---- | ---- | ---- | ---- | ------ |
| `base.ipairs(table)` | 无 | 迭代器、状态表、初始索引 | 按整数键 `1..n` 遍历数组部分 | [base](api/base.md) |
| `base.pairs(table)` | 无 | 迭代器、状态表、nil | 遍历表全部键值对 | [base](api/base.md) |
| `base.next{table, index}` | 无 | 下一组 `key, value` | 顺序返回下一组键值 | [base](api/base.md) |
| `base.select{index, values}` | 无 | 数量或若干值 | 按位置选取值或返回数量 | [base](api/base.md) |
| `base.rawequal{left, right}` | 无 | boolean | 不经元方法比较两值 | [base](api/base.md) |
| `base.rawlen(value)` | 无 | integer | 字符串字节数 / 表数组长度 | [base](api/base.md) |
| `base.tonumber{value, base}` | 无 | number / integer / nil | 字符串或数值转数字 | [base](api/base.md) |
| `base.tostring(value)` | 无 | string | 安全字符串化 | [base](api/base.md) |
| `base.type(value)` | 无 | string | 返回值的类型名 | [base](api/base.md) |
| `math.abs(value)` | 无 | number | 绝对值 | [math](api/math.md) |
| `math.acos(value)` | 无 | number | 反余弦（`-1..=1`） | [math](api/math.md) |
| `math.asin(value)` | 无 | number | 反正弦（`-1..=1`） | [math](api/math.md) |
| `math.atan(value)` | 无 | number | 反正切 | [math](api/math.md) |
| `math.ceil(value)` | 无 | number | 向上取整 | [math](api/math.md) |
| `math.cos(value)` | 无 | number | 余弦 | [math](api/math.md) |
| `math.deg(value)` | 无 | number | 弧度转角度 | [math](api/math.md) |
| `math.exp(value)` | 无 | number | e 的幂 | [math](api/math.md) |
| `math.floor(value)` | 无 | number | 向下取整 | [math](api/math.md) |
| `math.log10(value)` | 无 | number | 以 10 为底对数（`>0`） | [math](api/math.md) |
| `math.rad(value)` | 无 | number | 角度转弧度 | [math](api/math.md) |
| `math.sin(value)` | 无 | number | 正弦 | [math](api/math.md) |
| `math.sqrt(value)` | 无 | number | 平方根（`>=0`） | [math](api/math.md) |
| `math.tan(value)` | 无 | number | 正切 | [math](api/math.md) |
| `math.round(value)` | 无 | number | 四舍五入 | [math](api/math.md) |
| `math.normalize_angle(value)` | 无 | number | 角度归一化到 `[0, 360)` | [math](api/math.md) |
| `math.atan2{left, right}` | 无 | number | 双参反正切 | [math](api/math.md) |
| `math.fmod{left, right}` | 无 | number | 取模（`right≠0`） | [math](api/math.md) |
| `math.ldexp{left, right}` | 无 | number | `left * 2^right` | [math](api/math.md) |
| `math.pow{left, right}` | 无 | number | 幂运算 | [math](api/math.md) |
| `math.round_to{left, right}` | 无 | number | 按指定位数四舍五入 | [math](api/math.md) |
| `math.log{value, base}` | 无 | number | 对数（`value>0`，`base` 可省略） | [math](api/math.md) |
| `math.max{values}` | 无 | number | 最大值 | [math](api/math.md) |
| `math.min{values}` | 无 | number | 最小值 | [math](api/math.md) |
| `math.frexp(value)` | 无 | number, integer | 分解为尾数与指数 | [math](api/math.md) |
| `math.modf(value)` | 无 | number, number | 分离整数部分与小数部分 | [math](api/math.md) |
| `math.tointeger(value)` | 无 | integer / nil | 可精确转换时返回整数 | [math](api/math.md) |
| `math.type(value)` | 无 | string / nil | 数值类型：`integer` / `float` | [math](api/math.md) |
| `math.ult{left, right}` | 无 | boolean | 无符号小于比较 | [math](api/math.md) |
| `math.percent{value, total}` | 无 | number | 计算百分比（`value/total*100`） | [math](api/math.md) |
| `math.factorial(value)` | 无 | number | 阶乘（`0..170`） | [math](api/math.md) |
| `math.combination{n, k}` | 无 | number | 组合数 C(n, k) | [math](api/math.md) |
| `utf8.len(text)` | 无 | integer | 字符数 | [utf8](api/utf8.md) |
| `utf8.byte_len(text)` | 无 | integer | 字节数 | [utf8](api/utf8.md) |
| `utf8.is_ascii(text)` | 无 | boolean | 是否全部为 ASCII | [utf8](api/utf8.md) |
| `utf8.codepoint_to_char{values}` | 无 | string | 码点序列转字符串 | [utf8](api/utf8.md) |
| `utf8.ascii_to_char{values}` | 无 | string | ASCII 码序列转字符串 | [utf8](api/utf8.md) |
| `utf8.char_to_codepoint{text, start, finish}` | 无 | integer... | 字符转码点 | [utf8](api/utf8.md) |
| `utf8.char_to_ascii{text, start, finish}` | 无 | integer / nil... | 字符转 ASCII 码 | [utf8](api/utf8.md) |
| `utf8.char_position{text, index, start}` | 无 | integer / nil | 定位第 N 个字符的字节位置 | [utf8](api/utf8.md) |
| `utf8.codepoints(text)` | 无 | 迭代器 | 遍历码点（位置、码点） | [utf8](api/utf8.md) |
| `utf8.next{text, pos}` | 无 | integer, integer | 返回指定位置后的下一个字符 | [utf8](api/utf8.md) |
| `table.concat{table, separator, start, finish}` | 无 | string | 拼接数组元素 | [table](api/table.md) |
| `table.insert{table, position, value}` | 无 | — | 插入元素 | [table](api/table.md) |
| `table.move{source, start, finish, target_index, target}` | 无 | table | 移动数组元素 | [table](api/table.md) |
| `table.pack{values}` | 无 | table | 打包为数组表（带 `n`） | [table](api/table.md) |
| `table.remove{table, position}` | 无 | any | 删除并返回元素 | [table](api/table.md) |
| `table.sort{table, comparator}` | 无 | — | 原地排序 | [table](api/table.md) |
| `table.unpack{table, start, finish}` | 无 | 若干值 | 展开数组 | [table](api/table.md) |
| `string.lower(text)` | 无 | string | 转小写 | [string](api/string.md) |
| `string.upper(text)` | 无 | string | 转大写 | [string](api/string.md) |
| `string.reverse(text)` | 无 | string | 反转字符串 | [string](api/string.md) |
| `string.regex_escape(text)` | 无 | string | 转义正则特殊字符 | [string](api/string.md) |
| `string.sub{text, start, finish}` | 无 | string | 按字符截取子串 | [string](api/string.md) |
| `string.rep{text, times, sep}` | 无 | string | 重复拼接 | [string](api/string.md) |
| `string.find{text, pattern, init, plain}` | 无 | start, finish, captures... | 查找首个匹配 | [string](api/string.md) |
| `string.match{text, pattern, init}` | 无 | captures... | 提取匹配捕获 | [string](api/string.md) |
| `string.gmatch{text, pattern}` | 无 | 迭代器 | 迭代全部匹配 | [string](api/string.md) |
| `string.gsub{text, pattern, repl, limit}` | 无 | string, integer | 全局替换 | [string](api/string.md) |
| `string.regex_find{text, pattern, init}` | 无 | start, finish, capture_table | 正则查找 | [string](api/string.md) |
| `string.regex_match{text, pattern, init}` | 无 | captures... | 正则提取 | [string](api/string.md) |
| `string.regex_gmatch{text, pattern}` | 无 | 迭代器 | 正则迭代 | [string](api/string.md) |
| `string.regex_gsub{text, pattern, repl, limit}` | 无 | string, integer | 正则替换 | [string](api/string.md) |
| `string.regex_test{text, pattern}` | 无 | boolean | 是否匹配 | [string](api/string.md) |
| `string.regex_split{text, pattern}` | 无 | table | 正则分割为数组 | [string](api/string.md) |
| `string.format{format_string, values}` | 无 | string | 格式化字符串 | [string](api/string.md) |
| `string.rich_text_to_plain_text{text, rich_params, strip_header}` | 无 | string | 富文本转纯文本 | [string](api/string.md) |
| `color.rgb{r, g, b}` | 无 | string | 构造 `rgb(r,g,b)` 颜色 | [color](api/color.md) |
| `color.hex{r, g, b}` | 无 | string | 构造 `#rrggbb` 颜色 | [color](api/color.md) |
| `align.resolve_x{...}` | 无 | integer | 计算水平坐标 | [align](api/align.md) |
| `align.resolve_y{...}` | 无 | integer | 计算垂直坐标 | [align](api/align.md) |
| `align.resolve_rect{...}` | 无 | integer, integer | 计算矩形左上角坐标 | [align](api/align.md) |
| `measurement.get_text_size{...}` | 无 | integer, integer | 测量文本宽高 | [measurement](api/measurement.md) |
| `measurement.get_text_width{...}` | 无 | integer | 测量文本宽度 | [measurement](api/measurement.md) |
| `measurement.get_text_height{...}` | 无 | integer | 测量文本高度 | [measurement](api/measurement.md) |
| `draw.text{...}` | 无（仅 `Render` 阶段） | — | 绘制文本 | [draw](api/draw.md) |
| `draw.fill_rect{...}` | 无（仅 `Render` 阶段） | — | 填充矩形 | [draw](api/draw.md) |
| `draw.stroke_rect{...}` | 无（仅 `Render` 阶段） | — | 描边矩形 | [draw](api/draw.md) |
| `draw.erase_rect{...}` | 无（仅 `Render` 阶段） | — | 擦除矩形区域 | [draw](api/draw.md) |
| `draw.render()` | 无（仅 `Render` 阶段） | — | 请求本帧渲染输出 | [draw](api/draw.md) |
| `debug.print{message, time, level, type_head}` | 开启调试 | — | 输出调试日志 | [debug](api/debug.md) |
| `debug.log(message)` | 开启调试 | — | 输出 info 日志 | [debug](api/debug.md) |
| `debug.warn(message)` | 开启调试 | — | 输出 warn 日志 | [debug](api/debug.md) |
| `debug.error(message)` | 开启调试 | — | 输出 error 日志 | [debug](api/debug.md) |
| `debug.assert{value, message}` | 无 | any | 断言，失败抛错 | [debug](api/debug.md) |
| `debug.pcall{func, values, message}` | 无 | boolean, ... | 受保护调用 | [debug](api/debug.md) |
| `debug.xpcall{func, error_callback, values}` | 无 | boolean, ... | 带错误处理的受保护调用 | [debug](api/debug.md) |
| `game.exit_game()` | 仅游戏 | — | 退出当前游戏 | [game](api/game.md) |
| `game.save_game()` | 仅游戏 | — | 触发保存游戏 | [game](api/game.md) |
| `game.save_best()` | 仅游戏 | — | 触发保存最高分 | [game](api/game.md) |
| `event.skip_action()` | 仅游戏 + 关闭安全 | — | 跳过当前动作 | [event](api/event.md) |
| `event.clear_action()` | 仅游戏 + 关闭安全 | — | 清空动作队列 | [event](api/event.md) |
| `loader.load(path)` | 无 | table | 加载模块并返回实例 | [loader](api/loader.md) |
| `loader.load_execute(path)` | 无 | 若干值 | 加载并直接执行模块 | [loader](api/loader.md) |
| `file.read{path, encoding, end_of_line, event_tip}` | 无 | 异步（事件回调） | 读取文本文件 | [file](api/file.md) |
| `file.write{path, text, encoding, end_of_line, event_tip}` | 仅游戏 + 关闭安全 | 异步（事件回调） | 写入文本文件 | [file](api/file.md) |
| `file.list_dir{path, recursive, file_type, event_tip}` | 仅游戏 + 关闭安全 | 异步（事件回调） | 枚举目录 | [file](api/file.md) |
| `serialization.json_encode(t)` | 无 | string | 将 Lua 表编码为 JSON 字符串 | [serialization](api/serialization.md) |
| `serialization.json_decode(s)` | 无 | table / 基本类型 | 将 JSON 字符串解码为 Lua 表 | [serialization](api/serialization.md) |
| `serialization.csv_encode(t)` | 无 | string | 将二维数组编码为 CSV 字符串 | [serialization](api/serialization.md) |
| `serialization.csv_decode(s)` | 无 | table | 将 CSV 字符串解码为二维数组 | [serialization](api/serialization.md) |
| `serialization.yaml_encode(t)` | 无 | string | 将 Lua 表编码为 YAML 字符串 | [serialization](api/serialization.md) |
| `serialization.yaml_decode(s)` | 无 | table / 基本类型 | 将 YAML 字符串解码为 Lua 表 | [serialization](api/serialization.md) |
| `serialization.toml_encode(t)` | 无 | string | 将 Lua 表编码为 TOML 字符串 | [serialization](api/serialization.md) |
| `serialization.toml_decode(s)` | 无 | table / 基本类型 | 将 TOML 字符串解码为 Lua 表 | [serialization](api/serialization.md) |
| `serialization.ini_encode(t)` | 无 | string | 将 Lua 表编码为 INI 字符串 | [serialization](api/serialization.md) |
| `serialization.ini_decode(s)` | 无 | table | 将 INI 字符串解码为 Lua 表 | [serialization](api/serialization.md) |
| `serialization.xml_encode(t)` | 无 | string | 将 Lua 表编码为 XML 字符串 | [serialization](api/serialization.md) |
| `serialization.xml_decode(s)` | 无 | table / 基本类型 | 将 XML 字符串解码为 Lua 表 | [serialization](api/serialization.md) |
| `serialization.binary_pack(fmt, ...)` | 无 | string | 按格式串打包数据为二进制字符串 | [serialization](api/serialization.md) |
| `serialization.binary_unpack(fmt, s, pos)` | 无 | 若干值, integer | 按格式串从二进制字符串解包数据 | [serialization](api/serialization.md) |
| `serialization.binary_packsize(fmt)` | 无 | integer | 返回按格式打包所需的总字节数 | [serialization](api/serialization.md) |
| `encoding.base64_encode(s)` | 无 | string | 将字符串或二进制数据编码为 Base64 字符串 | [encoding](api/encoding.md) |
| `encoding.base64_decode(s)` | 无 | string | 将 Base64 字符串解码为原始字符串 | [encoding](api/encoding.md) |
| `encoding.url_encode(s)` | 无 | string | 将字符串编码为 URL 安全格式（百分号编码） | [encoding](api/encoding.md) |
| `encoding.url_decode(s)` | 无 | string | 将 URL 编码字符串解码为原始字符串 | [encoding](api/encoding.md) |
| `encoding.hex_encode(s)` | 无 | string | 将字符串编码为十六进制字符串 | [encoding](api/encoding.md) |
| `encoding.hex_decode(s)` | 无 | string | 将十六进制字符串解码为原始字符串 | [encoding](api/encoding.md) |
| `random.randint{min, max}` | 无 | integer | 直接生成 `[min, max]` 区间整数 | [random](api/random.md) |
| `random.randfloat{min, max}` | 无 | number | 直接生成 `[min, max]` 区间浮点数 | [random](api/random.md) |
| `random.create{type, min, max, seed, step}` | 无 | string | 创建生成器并返回生成器 ID | [random](api/random.md) |
| `random.delete(id)` | 无 | boolean | 删除指定生成器 | [random](api/random.md) |
| `random.clear()` | 无 | — | 删除所有生成器 | [random](api/random.md) |
| `random.list()` | 无 | table | 返回所有生成器 ID 的数组 | [random](api/random.md) |
| `random.count()` | 无 | integer | 返回当前生成器的总数 | [random](api/random.md) |
| `random.generate(id)` | 无 | integer / number | 用指定生成器生成一个随机数 | [random](api/random.md) |
| `random.set_params{id, type, min, max, seed, step}` | 无 | boolean | 修改生成器的参数 | [random](api/random.md) |
| `random.set_type{id, type}` | 无 | boolean | 修改生成器的类型 | [random](api/random.md) |
| `random.set_range{id, min, max}` | 无 | boolean | 修改生成器的随机区间 | [random](api/random.md) |
| `random.set_seed{id, seed}` | 无 | boolean | 修改生成器的种子 | [random](api/random.md) |
| `random.set_step{id, step}` | 无 | boolean | 修改生成器的步进数 | [random](api/random.md) |
| `random.get_type(id)` | 无 | string | 返回生成器的类型 | [random](api/random.md) |
| `random.get_range(id)` | 无 | number / integer, number / integer | 返回生成器的随机区间 `(min, max)` | [random](api/random.md) |
| `random.get_seed(id)` | 无 | integer | 返回生成器的种子 | [random](api/random.md) |
| `random.get_step(id)` | 无 | integer | 返回生成器当前的步进数 | [random](api/random.md) |
| `random.get_info(id)` | 无 | table | 返回生成器的完整信息表 | [random](api/random.md) |
| `random.exists(id)` | 无 | boolean | 检查生成器是否存在 | [random](api/random.md) |
