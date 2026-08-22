# math 库

## 基本库说明

`math` 提供数学运算与数学常量。

---

## 目录

### 常量

| 常量名              | 说明              | 索引                                    |
| ------------------- | ----------------- | --------------------------------------- |
| `PI`                | 圆周率 π          | [PI](#PI)                               |
| `E`                 | 自然常数 e        | [E](#E)                                 |
| `POSITIVE_INFINITE` | 正无穷            | [POSITIVE_INFINITE](#POSITIVE_INFINITE) |
| `INFINITE`          | 正无穷（别名）    | [INFINITE](#INFINITE)                   |
| `NEGATIVE_INFINITE` | 负无穷            | [NEGATIVE_INFINITE](#NEGATIVE_INFINITE) |
| `DEG`               | 弧度转角度系数    | [DEG](#DEG)                             |
| `RAD`               | 角度转弧度系数    | [RAD](#RAD)                             |
| `MAX_INTEGER`       | 最大整数 `2^63-1` | [MAX_INTEGER](#MAX_INTEGER)             |
| `MIN_INTEGER`       | 最小整数 `-2^63`  | [MIN_INTEGER](#MIN_INTEGER)             |

### 方法

| 方法名            | 说明                           | 索引                                |
| ----------------- | ------------------------------ | ----------------------------------- |
| `abs`             | 计算绝对值                     | [abs](#abs)                         |
| `acos`            | 计算反余弦                     | [acos](#acos)                       |
| `asin`            | 计算反正弦                     | [asin](#asin)                       |
| `atan`            | 计算反正切（单参）             | [atan](#atan)                       |
| `ceil`            | 向上取整                       | [ceil](#ceil)                       |
| `cos`             | 计算余弦                       | [cos](#cos)                         |
| `deg`             | 弧度转角度                     | [deg](#deg)                         |
| `exp`             | 计算 `e^x`                     | [exp](#exp)                         |
| `floor`           | 向下取整                       | [floor](#floor)                     |
| `log10`           | 计算以 10 为底的对数           | [log10](#log10)                     |
| `rad`             | 角度转弧度                     | [rad](#rad)                         |
| `sin`             | 计算正弦                       | [sin](#sin)                         |
| `sqrt`            | 计算平方根                     | [sqrt](#sqrt)                       |
| `tan`             | 计算正切                       | [tan](#tan)                         |
| `round`           | 四舍五入到最近整数             | [round](#round)                     |
| `normalize_angle` | 将角度归一化到 `[0, 360)` 区间 | [normalize_angle](#normalize_angle) |
| `atan2`           | 计算 `atan(y/x)`，正确处理象限 | [atan2](#atan2)                     |
| `fmod`            | 计算取模（余数）               | [fmod](#fmod)                       |
| `ldexp`           | 计算 `x * 2^exp`               | [ldexp](#ldexp)                     |
| `pow`             | 计算幂运算 `x^y`               | [pow](#pow)                         |
| `round_to`        | 按指定位数四舍五入             | [round_to](#round_to)               |
| `log`             | 计算对数（可指定底数）         | [log](#log)                         |
| `max`             | 返回一组数中的最大值           | [max](#max)                         |
| `min`             | 返回一组数中的最小值           | [min](#min)                         |
| `frexp`           | 分解尾数与二进制指数           | [frexp](#frexp)                     |
| `modf`            | 分离整数与小数部分             | [modf](#modf)                       |
| `tointeger`       | 精确转换为整数                 | [tointeger](#tointeger)             |
| `type`            | 返回数值类型名                 | [type](#type)                       |
| `ult`             | 无符号整数比较                 | [ult](#ult)                         |
| `percent`         | 计算百分比                     | [percent](#percent)                 |
| `factorial`       | 计算阶乘                       | [factorial](#factorial)             |
| `combination`     | 计算组合数 `C(n, k)`           | [combination](#combination)         |

---

## 常量

## `PI`

圆周率 π。

**可用于**

- 任意

### 调用

```lua
math.PI
```

### 示例

```lua
debug.print { message = tostring(math.PI) }
```

输出：

```text
3.1415926535898
```

---

## `E`

自然常数 e。

**可用于**

- 任意

### 调用

```lua
math.E
```

### 示例

```lua
debug.print { message = tostring(math.E) }
```

输出：

```text
2.718281828459
```

---

## `POSITIVE_INFINITE` / `INFINITE`

正无穷。

**可用于**

- 任意

### 调用

```lua
math.POSITIVE_INFINITE
```

### 示例

```lua
debug.print { message = tostring(math.POSITIVE_INFINITE) }
```

输出：

```text
inf
```

### 额外补充

- 该值永远大于任何数。
- 不可用于计算。

---

## `NEGATIVE_INFINITE`

负无穷。

**可用于**

- 任意

### 调用

```lua
math.NEGATIVE_INFINITE
```

### 示例

```lua
debug.print { message = tostring(math.NEGATIVE_INFINITE) }
```

输出：

```text
-inf
```

### 额外补充

- 仅可读取；数学运算不接受非有限输入。
- **等价于原版**：Lua 5.4 `-math.huge`

---

## `DEG`

弧度转角度系数，`180 / π`。

**可用于**

- 任意

### 调用

```lua
math.DEG
```

### 示例

```lua
debug.print { message = tostring(math.DEG) }
```

输出：

```text
57.295779513082
```

### 额外补充

- 也可使用 `math.deg(value)` 函数进行转换。

---

## `RAD`

角度转弧度系数，`π / 180`。

**可用于**

- 任意

### 调用

```lua
math.RAD
```

### 示例

```lua
debug.print { message = tostring(math.RAD) }
```

输出：

```text
0.017453292519943
```

### 额外补充

- 也可使用 `math.rad(value)` 函数进行转换。

---

## `MAX_INTEGER`

最大可表示的整数 `2^63-1`。

**可用于**

- 任意

### 调用

```lua
math.MAX_INTEGER
```

### 示例

```lua
debug.print { message = tostring(math.MAX_INTEGER) }
```

输出：

```text
9223372036854775807
```

### 额外补充

- 整数溢出边界。
- **等价于原版**：Lua 5.4 `math.maxinteger`

---

## `MIN_INTEGER`

最小可表示的整数 `-2^63`。

**可用于**

- 任意

### 调用

```lua
math.MIN_INTEGER
```

### 示例

```lua
debug.print { message = tostring(math.MIN_INTEGER) }
```

输出：

```text
-9223372036854775808
```

### 额外补充

- 整数下溢边界。
- **等价于原版**：Lua 5.4 `math.mininteger`

---

## 方法

## `abs`

计算绝对值。

### 调用

```lua
-- 单参数
math.abs(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明             |
| ------- | ------- | ---- | ------ | ---------------- |
| `value` | integer | 是   | -      | 要取绝对值的数值 |

### 返回

直接返回一个值。

| 类型    | 说明   |
| ------- | ------ |
| integer | 绝对值 |

### 示例

```lua
n = math.abs(-5)
debug.print { message = tostring(n) }
```

输出：

```text
5
```

### 额外补充

- 参数必须为有限数，否则抛出错误。
- **等价于原版**：Lua 5.4 `math.abs`

---

## `acos`

计算反余弦（弧度制）。

### 调用

```lua
-- 单参数
math.acos(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明                  |
| ------- | ------- | ---- | ------ | --------------------- |
| `value` | integer | 是   | -      | 余弦值，需在 `[-1,1]` |

### 返回

直接返回一个值。

| 类型   | 说明             |
| ------ | ---------------- |
| number | 弧度制的反余弦值 |

### 示例

```lua
r = math.acos(0.5)
debug.print { message = tostring(r) }
```

输出：

```text
1.0471975511966
```

### 额外补充

- 越界输入（超出 `[-1,1]`）会抛出错误。
- **等价于原版**：Lua 5.4 `math.acos`

---

## `asin`

计算反正弦（弧度制）。

### 调用

```lua
-- 单参数
math.asin(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明                  |
| ------- | ------- | ---- | ------ | --------------------- |
| `value` | integer | 是   | -      | 正弦值，需在 `[-1,1]` |

### 返回

直接返回一个值。

| 类型   | 说明             |
| ------ | ---------------- |
| number | 弧度制的反正弦值 |

### 示例

```lua
r = math.asin(0.5)
debug.print { message = tostring(r) }
```

输出：

```text
0.5235987755983
```

### 额外补充

- 越界输入（超出 `[-1,1]`）会抛出错误。
- **等价于原版**：Lua 5.4 `math.asin`

---

## `atan`

计算反正切（单参形式，弧度制）。

### 调用

```lua
-- 单参数
math.atan(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明   |
| ------- | ------- | ---- | ------ | ------ |
| `value` | integer | 是   | -      | 斜率值 |

### 返回

直接返回一个值。

| 类型   | 说明             |
| ------ | ---------------- |
| number | 弧度制的反正切值 |

### 示例

```lua
r = math.atan(1)
debug.print { message = tostring(r) }
```

输出：

```text
0.78539816339745
```

### 额外补充

- **等价于原版**：Lua 5.4 `math.atan`

---

## `ceil`

向上取整。

### 调用

```lua
-- 单参数
math.ceil(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明         |
| ------- | ------- | ---- | ------ | ------------ |
| `value` | integer | 是   | -      | 要取整的数值 |

### 返回

直接返回一个值。

| 类型   | 说明                      |
| ------ | ------------------------- |
| number | 不小于 `value` 的最小整数 |

### 示例

```lua
n = math.ceil(3.14)
debug.print { message = tostring(n) }
```

输出：

```text
4
```

### 额外补充

- 返回值类型为 `number`，但值为整数。
- **等价于原版**：Lua 5.4 `math.ceil`

---

## `cos`

计算余弦（弧度制）。

### 调用

```lua
-- 单参数
math.cos(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明           |
| ------- | ------- | ---- | ------ | -------------- |
| `value` | integer | 是   | -      | 弧度制的角度值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| number | 余弦值 |

### 示例

```lua
c = math.cos(math.PI)
debug.print { message = tostring(c) }
```

输出：

```text
-1
```

### 额外补充

- **等价于原版**：Lua 5.4 `math.cos`

---

## `deg`

将弧度转换为角度。

### 调用

```lua
-- 单参数
math.deg(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明   |
| ------- | ------- | ---- | ------ | ------ |
| `value` | integer | 是   | -      | 弧度值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| number | 角度值 |

### 示例

```lua
d = math.deg(math.PI)
debug.print { message = tostring(d) }
```

输出：

```text
180
```

### 额外补充

- **等价于原版**：Lua 5.4 `math.deg`

---

## `exp`

计算 `e^value`。

### 调用

```lua
-- 单参数
math.exp(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明   |
| ------- | ------- | ---- | ------ | ------ |
| `value` | integer | 是   | -      | 指数值 |

### 返回

直接返回一个值。

| 类型   | 说明               |
| ------ | ------------------ |
| number | `e^value` 的计算值 |

### 示例

```lua
e2 = math.exp(2)
debug.print { message = tostring(e2) }
```

输出：

```text
7.3890560989307
```

### 额外补充

- 溢出（结果超出双精度范围）会抛出错误。
- **等价于原版**：Lua 5.4 `math.exp`

---

## `floor`

向下取整。

### 调用

```lua
-- 单参数
math.floor(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明         |
| ------- | ------- | ---- | ------ | ------------ |
| `value` | integer | 是   | -      | 要取整的数值 |

### 返回

直接返回一个值。

| 类型   | 说明                      |
| ------ | ------------------------- |
| number | 不大于 `value` 的最大整数 |

### 示例

```lua
n = math.floor(3.14)
debug.print { message = tostring(n) }
```

输出：

```text
3
```

### 额外补充

- 返回值类型为 `number`，但值为整数。
- **等价于原版**：Lua 5.4 `math.floor`

---

## `log10`

计算以 10 为底的对数。

### 调用

```lua
-- 单参数
math.log10(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明           |
| ------- | ------- | ---- | ------ | -------------- |
| `value` | integer | 是   | -      | 真数，需 `> 0` |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| number | 常用对数值 |

### 示例

```lua
l = math.log10(100)
debug.print { message = tostring(l) }
```

输出：

```text
2
```

### 额外补充

- 非正输入会抛出错误。
- **等价于原版**：Lua 5.4 `math.log10`

---

## `rad`

将角度转换为弧度。

### 调用

```lua
-- 单参数
math.rad(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明   |
| ------- | ------- | ---- | ------ | ------ |
| `value` | integer | 是   | -      | 角度值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| number | 弧度值 |

### 示例

```lua
r = math.rad(180)
debug.print { message = tostring(r) }
```

输出：

```text
3.1415926535898
```

### 额外补充

- **等价于原版**：Lua 5.4 `math.rad`

---

## `sin`

计算正弦（弧度制）。

### 调用

```lua
-- 单参数
math.sin(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明           |
| ------- | ------- | ---- | ------ | -------------- |
| `value` | integer | 是   | -      | 弧度制的角度值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| number | 正弦值 |

### 示例

```lua
s = math.sin(math.PI / 2)
debug.print { message = tostring(s) }
```

输出：

```text
1
```

### 额外补充

- **等价于原版**：Lua 5.4 `math.sin`

---

## `sqrt`

计算平方根。

### 调用

```lua
-- 单参数
math.sqrt(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明                |
| ------- | ------- | ---- | ------ | ------------------- |
| `value` | integer | 是   | -      | 被开方数，需 `>= 0` |

### 返回

直接返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| number | 平方根值 |

### 示例

```lua
r = math.sqrt(16)
debug.print { message = tostring(r) }
```

输出：

```text
4
```

### 额外补充

- 负数输入会抛出错误。
- **等价于原版**：Lua 5.4 `math.sqrt`

---

## `tan`

计算正切（弧度制）。

### 调用

```lua
-- 单参数
math.tan(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明           |
| ------- | ------- | ---- | ------ | -------------- |
| `value` | integer | 是   | -      | 弧度制的角度值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| number | 正切值 |

### 示例

```lua
t = math.tan(math.PI / 4)
debug.print { message = tostring(t) }
```

输出：

```text
1
```

### 额外补充

- 非有限结果（如 `π/2` 附近）会抛出错误。
- **等价于原版**：Lua 5.4 `math.tan`

---

## `round`

四舍五入到最近的整数（`.5` 远离零取整）。

### 调用

```lua
-- 单参数
math.round(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明         |
| ------- | ------- | ---- | ------ | ------------ |
| `value` | integer | 是   | -      | 要取整的数值 |

### 返回

直接返回一个值。

| 类型   | 说明             |
| ------ | ---------------- |
| number | 四舍五入后的整数 |

### 示例

```lua
n1 = math.round(3.5)
n2 = math.round(-3.5)
debug.print { message = tostring(n1) .. ", " .. tostring(n2) }
```

输出：

```text
4, -4
```

---

## `normalize_angle`

将角度归一化到 `[0, 360)` 区间。

### 调用

```lua
-- 单参数
math.normalize_angle(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明               |
| ------- | ------- | ---- | ------ | ------------------ |
| `value` | integer | 是   | -      | 角度值（单位：度） |

### 返回

直接返回一个值。

| 类型   | 说明                            |
| ------ | ------------------------------- |
| number | 归一化后的角度，范围 `[0, 360)` |

### 示例

```lua
a1 = math.normalize_angle(450)
a2 = math.normalize_angle(-90)
debug.print { message = tostring(a1) .. ", " .. tostring(a2) }
```

输出：

```text
90, 270
```

---

## `atan2`

计算 `atan(y/x)`，并正确处理象限（弧度制）。

### 调用

```lua
-- 双参数
math.atan2(y, x)
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明   |
| ------ | ------- | ---- | ------ | ------ |
| `y`    | integer | 是   | -      | 纵坐标 |
| `x`    | integer | 是   | -      | 横坐标 |

### 返回

直接返回一个值。

| 类型   | 说明             |
| ------ | ---------------- |
| number | 弧度制的反正切值 |

### 示例

```lua
a = math.atan2(1, 1)
debug.print { message = tostring(a) }
```

输出：

```text
0.78539816339745
```

### 额外补充

- 非有限结果会抛出错误。
- **等价于原版**：Lua 5.4 `math.atan`（双参形式，原名 `math.atan2`）

---

## `fmod`

计算取模（余数），符号与被除数一致。

### 调用

```lua
-- 双参数
math.fmod(x, y)
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明           |
| ------ | ------- | ---- | ------ | -------------- |
| `x`    | integer | 是   | -      | 被除数         |
| `y`    | integer | 是   | -      | 除数，不能为 0 |

### 返回

直接返回一个值。

| 类型   | 说明 |
| ------ | ---- |
| number | 余数 |

### 示例

```lua
r = math.fmod(7, 3)
debug.print { message = tostring(r) }
```

输出：

```text
1
```

### 额外补充

- 除数为零会抛出错误。
- **等价于原版**：Lua 5.4 `math.fmod`

---

## `ldexp`

计算 `x * 2^exp`。

### 调用

```lua
-- 双参数
math.ldexp(x, exp)
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明 |
| ------ | ------- | ---- | ------ | ---- |
| `x`    | integer | 是   | -      | 尾数 |
| `exp`  | integer | 是   | -      | 指数 |

### 返回

直接返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| number | 计算结果 |

### 示例

```lua
v = math.ldexp(3, 2)
debug.print { message = tostring(v) }
```

输出：

```text
12
```

### 额外补充

- 非有限结果会抛出错误。
- **等价于原版**：Lua 5.4 `math.ldexp`

---

## `pow`

计算幂运算 `x^y`。

### 调用

```lua
-- 双参数
math.pow(x, y)
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明 |
| ------ | ------- | ---- | ------ | ---- |
| `x`    | integer | 是   | -      | 底数 |
| `y`    | integer | 是   | -      | 指数 |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| number | 幂运算结果 |

### 示例

```lua
p = math.pow(2, 10)
debug.print { message = tostring(p) }
```

输出：

```text
1024
```

### 额外补充

- 非有限结果会抛出错误。
- **等价于原版**：Lua 5.4 `^` 运算符（旧版 `math.pow`）

---

## `round_to`

按指定位数四舍五入。

### 调用

```lua
-- 双参数
math.round_to(value, digits)
```

### 参数

| 参数名   | 类型    | 必填 | 默认值 | 说明                 |
| -------- | ------- | ---- | ------ | -------------------- |
| `value`  | integer | 是   | -      | 要取整的数值         |
| `digits` | integer | 是   | -      | 小数位数（可为负数） |

### 返回

直接返回一个值。

| 类型   | 说明               |
| ------ | ------------------ |
| number | 保留指定位数的结果 |

### 示例

```lua
r1 = math.round_to(3.14159, 2)
r2 = math.round_to(12345, -2)
debug.print { message = tostring(r1) .. ", " .. tostring(r2) }
```

输出：

```text
3.14, 12300
```

### 额外补充

- `digits` 必须为 `[-308, 308]` 内的整数。
- `digits` 为负数时按 10 的幂取整。

---

## `log`

计算对数，可指定底数；省略 `base` 时为自然对数。

### 调用

```lua
-- 表参数
math.log{}
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明                     |
| ------- | ------- | ---- | ------ | ------------------------ |
| `value` | integer | 是   | -      | 真数，需 `> 0`           |
| `base`  | integer | 否   | `nil`  | 底数，需 `> 0` 且 `!= 1` |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| number | 对数值 |

### 示例

```lua
ln = math.log { value = math.E }
lg = math.log { value = 100, base = 10 }
debug.print { message = tostring(ln) .. ", " .. tostring(lg) }
```

输出：

```text
1, 2
```

### 额外补充

- 非有限结果会抛出错误。
- **等价于原版**：Lua 5.4 `math.log`

---

## `max`

返回一组数中的最大值。

### 调用

```lua
-- 表参数
math.max{}
```

### 参数

| 参数名   | 类型  | 必填 | 默认值 | 说明                    |
| -------- | ----- | ---- | ------ | ----------------------- |
| `values` | table | 是   | -      | 数值数组，至少包含 1 项 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| number | 最大值 |

### 示例

```lua
m = math.max { values = { 1, 5, 3, 9, 2 } }
debug.print { message = tostring(m) }
```

输出：

```text
9
```

### 额外补充

- 空数组或包含非有限数会抛出错误。
- **等价于原版**：Lua 5.4 `math.max`

---

## `min`

返回一组数中的最小值。

### 调用

```lua
-- 表参数
math.min{}
```

### 参数

| 参数名   | 类型  | 必填 | 默认值 | 说明                    |
| -------- | ----- | ---- | ------ | ----------------------- |
| `values` | table | 是   | -      | 数值数组，至少包含 1 项 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| number | 最小值 |

### 示例

```lua
m = math.min { values = { 1, 5, 3, 9, 2 } }
debug.print { message = tostring(m) }
```

输出：

```text
1
```

### 额外补充

- 空数组或包含非有限数会抛出错误。
- **等价于原版**：Lua 5.4 `math.min`

---

## `frexp`

将数值分解为尾数与二进制指数，使得 `value = mantissa * 2^exponent`。

### 调用

```lua
-- 单参数
math.frexp(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明         |
| ------- | ------- | ---- | ------ | ------------ |
| `value` | integer | 是   | -      | 要分解的数值 |

### 返回

直接返回两个值。

| 返回值名   | 类型    | 说明                             |
| ---------- | ------- | -------------------------------- |
| `mantissa` | number  | 尾数，绝对值在 `[0.5, 1)` 或为 0 |
| `exponent` | integer | 指数                             |

### 示例

```lua
m, e = math.frexp(12.8)
debug.print { message = tostring(m) .. ", " .. tostring(e) }
```

输出：

```text
0.8, 4
```

### 额外补充

- 当 `value = 0` 时返回 `0, 0`。
- **等价于原版**：Lua 5.4 `math.frexp`

---

## `modf`

分离数值的整数部分与小数部分，两值之和等于原值。

### 调用

```lua
-- 单参数
math.modf(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明         |
| ------- | ------- | ---- | ------ | ------------ |
| `value` | integer | 是   | -      | 要分解的数值 |

### 返回

直接返回两个值。

| 返回值名          | 类型   | 说明                 |
| ----------------- | ------ | -------------------- |
| `integer_part`    | number | 整数部分（向零截断） |
| `fractional_part` | number | 小数部分             |

### 示例

```lua
i, f = math.modf(3.14)
debug.print { message = tostring(i) .. ", " .. tostring(f) }
```

输出：

```text
3, 0.14
```

### 额外补充

- **等价于原版**：Lua 5.4 `math.modf`

---

## `tointeger`

将数值精确转换为整数，若无法精确转换则返回 `nil`。

### 调用

```lua
-- 单参数
math.tointeger(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明         |
| ------- | ------- | ---- | ------ | ------------ |
| `value` | integer | 是   | -      | 要转换的数值 |

### 返回

直接返回一个值。

| 类型    | 说明               |
| ------- | ------------------ |
| integer | 精确转换后的整数   |
| nil     | 无法精确转换时返回 |

### 示例

```lua
i1 = math.tointeger(3.0)
i2 = math.tointeger(3.14)
debug.print { message = tostring(i1) .. ", " .. tostring(i2) }
```

输出：

```text
3, nil
```

### 额外补充

- **等价于原版**：Lua 5.4 `math.tointeger`

---

## `type`

返回数值的类型名。

### 调用

```lua
-- 单参数
math.type(value)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明         |
| ------- | ------- | ---- | ------ | ------------ |
| `value` | integer | 是   | -      | 要判断的数值 |

### 返回

直接返回一个值。

| 类型   | 说明                     |
| ------ | ------------------------ |
| string | `"integer"` 或 `"float"` |
| nil    | 参数不是数值时返回       |

### 示例

```lua
t1 = math.type(3)
t2 = math.type(3.14)
t3 = math.type("3")
debug.print { message = tostring(t1) .. ", " .. tostring(t2) .. ", " .. tostring(t3) }
```

输出：

```text
integer, float, nil
```

### 额外补充

- **等价于原版**：Lua 5.4 `math.type`

---

## `ult`

以无符号整数比较两个整数（负数的二进制补码解释）。

### 调用

```lua
-- 双参数
math.ult(left, right)
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明       |
| ------- | ------- | ---- | ------ | ---------- |
| `left`  | integer | 是   | -      | 左侧操作数 |
| `right` | integer | 是   | -      | 右侧操作数 |

### 返回

直接返回一个值。

| 类型    | 说明                    |
| ------- | ----------------------- |
| boolean | 无符号小于时返回 `true` |

### 示例

```lua
b1 = math.ult(-1, 1)  -- -1 解释为 2^64-1
b2 = math.ult(1, -1)
debug.print { message = tostring(b1) .. ", " .. tostring(b2) }
```

输出：

```text
false, true
```

### 额外补充

- **等价于原版**：Lua 5.4 `math.ult`

---

## `percent`

计算百分比值 `value / total * 100`。

### 调用

```lua
-- 表参数
math.percent{}
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明           |
| ------- | ------- | ---- | ------ | -------------- |
| `value` | integer | 是   | -      | 分子           |
| `total` | integer | 是   | -      | 分母，不能为 0 |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| number | 百分比数值 |

### 示例

```lua
p = math.percent { value = 25, total = 80 }
debug.print { message = tostring(p) }
```

输出：

```text
31.25
```

### 额外补充

- 分母为零会抛出错误。

---

## `factorial`

计算阶乘 `n!`。

### 调用

```lua
-- 单参数
math.factorial(n)
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明                    |
| ------ | ------- | ---- | ------ | ----------------------- |
| `n`    | integer | 是   | -      | 阶乘数，需在 `[0, 170]` |

### 返回

直接返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| number | 阶乘结果 |

### 示例

```lua
f = math.factorial(5)
debug.print { message = tostring(f) }
```

输出：

```text
120
```

### 额外补充

- 超出 `[0, 170]` 范围会抛出错误（`170!` 约等于 `1.2e308`，接近双精度上限）。

---

## `combination`

计算组合数 `C(n, k)`。

### 调用

```lua
-- 表参数
math.combination{}
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明                         |
| ------ | ------- | ---- | ------ | ---------------------------- |
| `n`    | integer | 是   | -      | 总数，需满足 `0 <= k <= n`   |
| `k`    | integer | 是   | -      | 选取数，内部取 `min(k, n-k)` |

### 返回

直接返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| number | 组合数值 |

### 示例

```lua
c = math.combination { n = 5, k = 2 }
debug.print { message = tostring(c) }
```

输出：

```text
10
```

### 额外补充

- `n` 最大支持 `1000000`，但结果可能溢出双精度范围，此时会抛出错误。
- 内部自动优化为 `min(k, n-k)` 以提升性能。
