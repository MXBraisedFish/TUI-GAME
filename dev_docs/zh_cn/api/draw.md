# draw 库

## 基本库说明

`draw` 提供绘制指令：文本、填充矩形、描边矩形、擦除与请求渲染。游戏与屏保会话均可使用。**所有方法只能在 `Render` 回调阶段调用**，其余阶段调用会报错。绘制结果在回调结束后统一提交渲染。库表为只读。

## 目录

### 常量

本库无常量。

### 方法

| 方法名 | 说明 |
| ------ | ---- |
| `draw.text{...}` | 绘制文本 |
| `draw.fill_rect{...}` | 填充矩形 |
| `draw.stroke_rect{...}` | 描边矩形 |
| `draw.erase_rect{...}` | 擦除矩形区域 |
| `draw.render()` | 请求本帧渲染输出 |

## 方法

### `text`

- **方法作用**：在指定位置绘制文本。
- **方法要求**：无（仅限 `Render` 阶段）
- **方法参数**：

| 参数名                | 类型      | 必填  | 默认值      | 说明       | 额外补充                                                          |
| ------------------ | ------- | --- | -------- | -------- | ------------------------------------------------------------- |
| `x`                | integer | 是   | —        | 左上角 x 坐标 | 坐标原点为左上角                                                      |
| `y`                | integer | 是   | —        | 左上角 y 坐标 | —                                                             |
| `text`             | string  | 是   | —        | 要绘制的文本   | —                                                             |
| `fg`               | const  | 否   | `nil`    | 前景色      | 使用 `color.*` 常量或 `color.rgb/hex`                              |
| `bg`               | const  | 否   | `nil`    | 背景色      | 可为 `color.TRANSPARENT`                                        |
| `horizontal_align` | const  | 否   | `"left"` | 行内对齐     | 支持 `left` / `horizontal_center` / `center` / `right` / `auto` |
| `auto_wrap`        | boolean | 否   | 自动       | 是否启用自动换行 | `nil`/`true` 为自动换行，`false` 为普通换行                              |
| `word_wrap`        | boolean | 否   | `true`   | 是否整词换行   | 关闭后按字符任意断行                                                    |
| `max_width`        | integer | 否   | `nil`    | 最大宽度约束   | 必须满足 `1..65535`                                               |
| `max_height`       | integer | 否   | `nil`    | 最大高度约束   | 必须满足 `1..65535`                                               |
| `overflow_marker`  | string  | 否   | `"..."`  | 溢出省略标记   | —                                                             |
| `rich_params`      | table   | 否   | `nil`    | 富文本参数表   | 键值均为字符串/数字/布尔                                                 |
| `bold`             | boolean | 否   | `false`  | 粗体       | —                                                             |
| `italic`           | boolean | 否   | `false`  | 斜体       | —                                                             |
| `underline`        | boolean | 否   | `false`  | 下划线      | —                                                             |
| `strike`           | boolean | 否   | `false`  | 删除线      | —                                                             |
| `blink`            | boolean | 否   | `false`  | 闪烁       | —                                                             |
| `reverse`          | boolean | 否   | `false`  | 反显       | —                                                             |
| `hidden`           | boolean | 否   | `false`  | 隐藏       | —                                                             |
| `dim`              | boolean | 否   | `false`  | 暗淡       | —                                                             |
| `text_mode`        | const  | 否   | `"auto"` | 文本模式     | 使用 `string.AUTO/PLAIN_TEXT/RICH_TEXT`                         |
| `slice_layer`      | string  | 否   | `"base"` | 图层       | 当前仅支持 `"base"`                                                |

- **方法返回**：无返回值。

- **方法的使用**：

```lua

```

---

### `fill_rect`

- **方法作用**：填充一个矩形区域。
- **方法要求**：无（仅限 `Render` 阶段）
- **方法参数**：

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 额外补充 |
| ------ | ---- | ---- | ------ | ---- | -------- |
| `x` | integer | 是 | — | 左上角 x 坐标 | — |
| `y` | integer | 是 | — | 左上角 y 坐标 | — |
| `width` | integer | 是 | — | 宽度 | 必须满足 `1..65535` |
| `height` | integer | 是 | — | 高度 | 必须满足 `1..65535` |
| `char` | string | 否 | `nil` | 填充字符 | 必须为单个显示格字符 |
| `fg` | const | 否 | `nil` | 前景色 | — |
| `bg` | const | 否 | `nil` | 背景色 | 可为 `color.TRANSPARENT` |
| `slice_layer` | string | 否 | `"base"` | 图层 | 当前仅支持 `"base"` |

- **方法返回**：无返回值。

- **方法的使用**：

```lua

```

---

### `stroke_rect`

- **方法作用**：绘制一个矩形边框。
- **方法要求**：无（仅限 `Render` 阶段）
- **方法参数**：

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 额外补充 |
| ------ | ---- | ---- | ------ | ---- | -------- |
| `x` | integer | 是 | — | 左上角 x 坐标 | — |
| `y` | integer | 是 | — | 左上角 y 坐标 | — |
| `width` | integer | 是 | — | 宽度 | 必须满足 `1..65535` |
| `height` | integer | 是 | — | 高度 | 必须满足 `1..65535` |
| `fg` | const | 否 | `nil` | 前景色 | — |
| `bg` | const | 否 | `nil` | 背景色 | 可为 `color.TRANSPARENT` |
| `border_char` | const | 否 | `nil` | 自定义边框字符表 | 填写 `char.LINE/BOLD_LINE/DOUBLE_LINE/ROUNDED_LINE` 常量或自定义边框表，省略时使用单线，单个字符须占一个显示格 |
| `slice_layer` | string | 否 | `"base"` | 图层 | 当前仅支持 `"base"` |

- **方法返回**：无返回值。

- **方法的使用**：

```lua

```

---

### `erase_rect`

- **方法作用**：擦除矩形区域，恢复为下层内容。
- **方法要求**：无（仅限 `Render` 阶段）
- **方法参数**：

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 额外补充 |
| ------ | ---- | ---- | ------ | ---- | -------- |
| `x` | integer | 是 | — | 左上角 x 坐标 | — |
| `y` | integer | 是 | — | 左上角 y 坐标 | — |
| `width` | integer | 是 | — | 宽度 | 必须满足 `1..65535` |
| `height` | integer | 是 | — | 高度 | 必须满足 `1..65535` |
| `slice_layer` | string | 否 | `"base"` | 图层 | 当前仅支持 `"base"` |

- **方法返回**：无返回值。

- **方法的使用**：

```lua

```

---

### `render`

- **方法作用**：请求将本帧已入队的绘制指令输出到终端。
- **方法要求**：无（仅限 `Render` 阶段）
- **方法参数**：无参数。

- **方法返回**：无返回值。

- **方法的使用**：

```lua

```
