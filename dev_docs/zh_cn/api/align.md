# align 库

## 基本库说明

`align` 根据当前终端尺寸（Base 可视区域）计算对齐坐标。游戏与屏保会话均可使用。坐标基于终端左上角原点，**结果允许为负数**。库表为只读。

## 目录

### 常量

| 常量名 | 说明 |
| ------ | ---- |
| `align.AUTO` | 自动对齐（水平/垂直居中） |
| `align.LEFT` | 左对齐 |
| `align.HORIZONTAL_CENTER` | 水平居中 |
| `align.RIGHT` | 右对齐 |
| `align.TOP` | 顶部对齐 |
| `align.VERTICAL_CENTER` | 垂直居中 |
| `align.BOTTOM` | 底部对齐 |
| `align.CENTER` | 双向居中 |

### 方法

| 方法名 | 说明 |
| ------ | ---- |
| `align.resolve_x{...}` | 计算水平坐标 |
| `align.resolve_y{...}` | 计算垂直坐标 |
| `align.resolve_rect{...}` | 计算矩形左上角坐标 |

## 常量

### `AUTO`

| 项目 | 内容 |
| ---- | ---- |
| **可应用参数** | `horizontal_align`、`vertical_align` |
| **作用** | 自动对齐：水平时等价 `HORIZONTAL_CENTER`，垂直时等价 `VERTICAL_CENTER` |
| **额外补充** | 值为字符串 `"auto"` |

---

### `LEFT`

| 项目 | 内容 |
| ---- | ---- |
| **可应用参数** | `horizontal_align` |
| **作用** | 左对齐：以左边缘为锚点 |
| **额外补充** | 值为字符串 `"left"` |

---

### `HORIZONTAL_CENTER`

| 项目 | 内容 |
| ---- | ---- |
| **可应用参数** | `horizontal_align` |
| **作用** | 水平居中：以终端水平中心为锚点 |
| **额外补充** | 值为字符串 `"horizontal_center"` |

---

### `RIGHT`

| 项目 | 内容 |
| ---- | ---- |
| **可应用参数** | `horizontal_align` |
| **作用** | 右对齐：以右边缘为锚点 |
| **额外补充** | 值为字符串 `"right"` |

---

### `TOP`

| 项目 | 内容 |
| ---- | ---- |
| **可应用参数** | `vertical_align` |
| **作用** | 顶部对齐：以上边缘为锚点 |
| **额外补充** | 值为字符串 `"top"` |

---

### `VERTICAL_CENTER`

| 项目 | 内容 |
| ---- | ---- |
| **可应用参数** | `vertical_align` |
| **作用** | 垂直居中：以终端垂直中心为锚点 |
| **额外补充** | 值为字符串 `"vertical_center"` |

---

### `BOTTOM`

| 项目 | 内容 |
| ---- | ---- |
| **可应用参数** | `vertical_align` |
| **作用** | 底部对齐：以下边缘为锚点 |
| **额外补充** | 值为字符串 `"bottom"` |

---

### `CENTER`

| 项目 | 内容 |
| ---- | ---- |
| **可应用参数** | `horizontal_align`、`vertical_align` |
| **作用** | 双向居中：水平/垂直均可使用，等价对应方向的居中常量 |
| **额外补充** | 值为字符串 `"center"` |

---

## 方法

### `resolve_x`

- **方法作用**：根据元素宽度与水平对齐方式，计算元素左边缘的 x 坐标。
- **方法要求**：无
- **方法参数**：

| 参数名                | 类型      | 必填  | 默认值      | 说明         | 额外补充                                                |
| ------------------ | ------- | --- | -------- | ---------- | --------------------------------------------------- |
| `width`            | integer | 是   | —        | 元素宽度（显示列数） | 必须为正整数                                              |
| `horizontal_align` | const   | 是   | —        | 水平对齐方式     | 使用 `align.AUTO/LEFT/HORIZONTAL_CENTER/RIGHT/CENTER` |
| `offset_x`         | integer | 否   | `0`      | 锚点上的水平偏移   | 正数向右、负数向左                                           |
| `relative_x`       | integer | 否   | `nil`    | 自定义水平锚点    | 省略时按对齐方式取边缘/中心作为锚点                                  |
| `slice_layer`      | string  | 否   | `"base"` | 图层         | 当前仅支持 `"base"`                                      |

- **方法返回**：

| 返回值名 | 类型 | 说明 | 额外补充 |
| -------- | ---- | ---- | -------- |
| `x` | integer | 解析后的水平坐标 | 允许为负数 |

- **方法的使用**：

```lua

```

---

### `resolve_y`

- **方法作用**：根据元素高度与垂直对齐方式，计算元素上边缘的 y 坐标。
- **方法要求**：无
- **方法参数**：

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 额外补充 |
| ------ | ---- | ---- | ------ | ---- | -------- |
| `height` | integer | 是 | — | 元素高度（显示行数） | 必须为正整数 |
| `vertical_align` | const | 是 | — | 垂直对齐方式 | 使用 `align.AUTO/TOP/VERTICAL_CENTER/BOTTOM/CENTER` |
| `offset_y` | integer | 否 | `0` | 锚点上的垂直偏移 | 正数向下、负数向上 |
| `relative_y` | integer | 否 | `nil` | 自定义垂直锚点 | 省略时按对齐方式取边缘/中心作为锚点 |
| `slice_layer` | string | 否 | `"base"` | 图层 | 当前仅支持 `"base"` |

- **方法返回**：

| 返回值名 | 类型 | 说明 | 额外补充 |
| -------- | ---- | ---- | -------- |
| `y` | integer | 解析后的垂直坐标 | 允许为负数 |

- **方法的使用**：

```lua

```

---

### `resolve_rect`

- **方法作用**：同时计算矩形左上角坐标，一次得到 `x, y`。
- **方法要求**：无
- **方法参数**：

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 额外补充 |
| ------ | ---- | ---- | ------ | ---- | -------- |
| `width` | integer | 是 | — | 矩形宽度 | 必须为正整数 |
| `height` | integer | 是 | — | 矩形高度 | 必须为正整数 |
| `horizontal_align` | const | 是 | — | 水平对齐方式 | 使用 `align.AUTO/LEFT/HORIZONTAL_CENTER/RIGHT/CENTER` |
| `vertical_align` | const | 是 | — | 垂直对齐方式 | 使用 `align.AUTO/TOP/VERTICAL_CENTER/BOTTOM/CENTER` |
| `offset_x` | integer | 否 | `0` | 水平偏移 | — |
| `offset_y` | integer | 否 | `0` | 垂直偏移 | — |
| `relative_x` | integer | 否 | `nil` | 自定义水平锚点 | 省略时按对齐方式计算 |
| `relative_y` | integer | 否 | `nil` | 自定义垂直锚点 | 省略时按对齐方式计算 |
| `slice_layer` | string | 否 | `"base"` | 图层 | 当前仅支持 `"base"` |

- **方法返回**：

| 返回值名 | 类型 | 说明 | 额外补充 |
| -------- | ---- | ---- | -------- |
| `x` | integer | 矩形左边缘坐标 | 允许为负数 |
| `y` | integer | 矩形上边缘坐标 | 允许为负数 |

- **方法的使用**：

```lua

```
