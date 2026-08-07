# measurement 库

## 基本库说明

`measurement` 提供文本尺寸测量。游戏与屏保会话均可使用。库表为只读。文本参数与 `draw.text` 一致（`x`/`y` 除外），测量结果不要求处于 `Render` 阶段。

## 目录

### 常量

本库无常量。

### 方法

| 方法名 | 说明 |
| ------ | ---- |
| `measurement.get_text_size{...}` | 测量文本宽高 |
| `measurement.get_text_width{...}` | 测量文本宽度 |
| `measurement.get_text_height{...}` | 测量文本高度 |

## 方法

### `get_text_size`

- **方法作用**：测量给定文本的显示宽度与高度（单位：显示格）。
- **方法要求**：无
- **方法参数**：

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 额外补充 |
| ------ | ---- | ---- | ------ | ---- | -------- |
| `text` | string | 是 | — | 要测量的文本 | — |
| `x` | integer | 否 | 忽略 | 位置参数 | 测量时忽略该参数 |
| `y` | integer | 否 | 忽略 | 位置参数 | 测量时忽略该参数 |
| `fg` | string | 否 | `nil` | 前景色 | 仅参与校验，不影响测量 |
| `bg` | string | 否 | `nil` | 背景色 | 仅参与校验，不影响测量 |
| `horizontal_align` | string | 否 | `"left"` | 行内对齐 | 支持 `left` / `horizontal_center` / `center` / `right` / `auto` |
| `auto_wrap` | boolean | 否 | 自动 | 是否启用自动换行 | `nil`/`true` 为自动换行，`false` 为普通换行 |
| `word_wrap` | boolean | 否 | `true` | 是否整词换行 | 关闭后按字符任意断行 |
| `max_width` | integer | 否 | `nil` | 最大宽度约束 | 必须满足 `1..65535` |
| `max_height` | integer | 否 | `nil` | 最大高度约束 | 必须满足 `1..65535` |
| `overflow_marker` | string | 否 | `"..."` | 溢出省略标记 | — |
| `rich_params` | table | 否 | `nil` | 富文本参数表 | 键值均为字符串/数字/布尔 |
| `bold` | boolean | 否 | `false` | 粗体 | — |
| `italic` | boolean | 否 | `false` | 斜体 | — |
| `underline` | boolean | 否 | `false` | 下划线 | — |
| `strike` | boolean | 否 | `false` | 删除线 | — |
| `blink` | boolean | 否 | `false` | 闪烁 | — |
| `reverse` | boolean | 否 | `false` | 反显 | — |
| `hidden` | boolean | 否 | `false` | 隐藏 | — |
| `dim` | boolean | 否 | `false` | 暗淡 | — |
| `text_mode` | string | 否 | `"auto"` | 文本模式 | 使用 `string.AUTO/PLAIN_TEXT/RICH_TEXT` |
| `slice_layer` | string | 否 | `"base"` | 图层 | 当前仅支持 `"base"` |

- **方法返回**：

| 返回值名 | 类型 | 说明 | 额外补充 |
| -------- | ---- | ---- | -------- |
| `width` | integer | 文本显示宽度（列） | — |
| `height` | integer | 文本显示高度（行） | — |

- **方法的使用**：

```lua

```

---

### `get_text_width`

- **方法作用**：测量给定文本的显示宽度。
- **方法要求**：无
- **方法参数**：同 [get_text_size](#get_text_size)。

- **方法返回**：

| 返回值名 | 类型 | 说明 | 额外补充 |
| -------- | ---- | ---- | -------- |
| `width` | integer | 文本显示宽度（列） | — |

- **方法的使用**：

```lua

```

---

### `get_text_height`

- **方法作用**：测量给定文本的显示高度。
- **方法要求**：无
- **方法参数**：同 [get_text_size](#get_text_size)。

- **方法返回**：

| 返回值名 | 类型 | 说明 | 额外补充 |
| -------- | ---- | ---- | -------- |
| `height` | integer | 文本显示高度（行） | — |

- **方法的使用**：

```lua

```
