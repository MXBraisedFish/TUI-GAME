# Lua 事件参考

本文档说明当前 Lua 运行时能够识别的全部事件结构。

系统与生命周期事件由宿主自动产生。服务和交互对象事件只会发送给拥有对应对象或异步请求的 Lua Session。部分事件对应的 Lua 创建 API 尚未开放；本文档中存在事件定义不代表 Lua 目前已经可以调用其创建 API。

## 事件信封

所有传递给 Lua 的事件都使用相同的信封结构：

```lua
{
  type = "action",
  sequence = 42,
  frame = 1800,
  data = {
    action = "jump",
    state = "pressed"
  }
}
```

| 字段 | Lua 类型 | 含义 |
|---|---|---|
| `type` | `string` | 事件分类，所有可用值见下文。 |
| `sequence` | `integer` | Runtime 全局单调递增的事件序号。事件可能按 Session 过滤，因此序号不连续是正常情况。 |
| `frame` | `integer` | 事件入队时所处的宿主帧。 |
| `data` | `table` | 对应事件类型的数据。 |

事件只会被传递给 `HandleEvent(event)` 或对应操作注册的回调函数，两者不会重复接收。同一字段不存在时，在 Lua 中读取结果为 `nil`。

每个 Session 拥有独立的 FIFO 队列。每个宿主帧最多投递 128 个事件，最多允许 1,024 个事件等待处理。Lua 处理当前批次时新产生的事件会延迟到下一宿主帧。

## Session 路由

| 事件分类 | 游戏 Session | 屏保 Session |
|---|---:|---:|
| 动作与鼠标输入 | 是 | 否 |
| 终端尺寸与焦点 | 是 | 是 |
| 屏保生命周期 | 是 | 否 |
| Session 自己创建的计时器与动画 | 是 | 是 |
| Session 自己创建的文件请求结果 | 是 | 仅限读取 |
| Session 自己创建的图片请求结果 | 是 | 是 |
| Session 自己创建的网络请求结果 | 是 | 是，但需经过未来的权限 API |
| Session 自己创建的交互 UI 对象 | 是 | 否 |

屏保运行期间，游戏不会收到 `action`、`mouse` 或交互 UI 对象事件，但仍可收到 `resize`、`focus`、屏保生命周期事件及游戏自身后台操作的结果。

原始终端按键、宿主 UI、日志、弹窗、包扫描、截屏、录屏、数据导出、视频导出以及通用宿主任务事件不会暴露给 Lua。

## 系统与生命周期事件

### `action`

全局宿主动作优先消费输入，剩余游戏动作才会发送给当前游戏。屏保运行期间不会发送。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `action` | `string` | 游戏包定义的动作 ID，不会暴露原始按键。 |
| `state` | `string` | `pressed`、`held` 或 `released`。 |

### `mouse`

终端处于聚焦状态，且鼠标事件发生在 Base 可视区域内时发送给当前游戏。坐标从零开始，并且相对于 Base 区域。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `kind` | `string` | `pressed`、`released`、`moved`、`dragged`、`held` 或 `scrolled`。 |
| `button` | `string \| nil` | `left`、`middle` 或 `right`；事件不包含鼠标按键时不存在。 |
| `scroll` | `string \| nil` | `up`、`down`、`left` 或 `right`；仅滚轮事件存在。 |
| `x` | `integer` | 从零开始的水平单元格坐标。 |
| `y` | `integer` | 从零开始的垂直单元格坐标。 |

### `resize`

终端尺寸发生变化时发送，同时投递给当前游戏和屏保 Session。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `width` | `integer` | 新的终端宽度，单位为单元格。 |
| `height` | `integer` | 新的终端高度，单位为单元格。 |

### `focus`

终端获得或失去焦点时发送，同时投递给当前游戏和屏保 Session。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `gained` | `boolean` | 获得焦点时为 `true`，失去焦点时为 `false`。 |

宿主不会在焦点丢失后自动生成所有动作的 `released` 事件。游戏应在 `gained == false` 时自行清理持续按下状态。

### `screensaver_started`

屏保 Session 成功启动后发送给游戏。

`data` 为空表。

### `screensaver_stopped`

当前屏保 Session 停止后发送给游戏。

`data` 为空表。

## 时间与动画事件

这些事件只会发送给创建对应计时器或动画的 Session。所有 ID 都是 Session 内部的不透明整数，不是宿主对象 ID。

### `timer`

所属计时器触发或结束时发送。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `id` | `integer` | Session 内部的计时器 ID。 |
| `timer_kind` | `string` | `timer`、`delay`、`repeat` 或 `sleep`。 |
| `kind` | `string` | `tick` 或 `finished`。 |
| `executed_count` | `integer \| nil` | 已执行次数，仅重复计时器存在。 |

`timer` 和 `delay` 当前只产生 `finished`；重复计时器产生 `tick`，并在结束时产生 `finished`；异步休眠产生 `finished`。

### `animation`

所属动画的生命周期发生变化、到达标记或完成循环时发送。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `id` | `integer` | Session 内部的动画 ID。 |
| `kind` | `string` | `started`、`marker`、`loop`、`finished` 或 `cancelled`。 |
| `name` | `string \| nil` | 标记名称，仅当 `kind == "marker"` 时存在。 |
| `completed` | `integer \| nil` | 已完成的循环次数，仅当 `kind == "loop"` 时存在。 |

## 异步服务事件

服务事件是 Session 自有异步请求的最终结果。宿主任务 ID 永远不会暴露给 Lua。如果提交操作时注册了回调，则完整事件信封只发送给该回调；否则发送给 `HandleEvent`。

### 通用错误对象

失败的服务事件包含：

```lua
error = {
  code = "timeout",
  message = "request timed out"
}
```

| 字段 | Lua 类型 | 含义 |
|---|---|---|
| `code` | `string` | 稳定、可供程序判断的错误码。 |
| `message` | `string` | 供开发者阅读的净化错误信息，不含宿主路径、任务 ID、请求头、正文或调用栈。 |

错误码可能为：`invalid_request`、`permission_denied`、`not_found`、`too_large`、`invalid_utf8`、`cancelled`、`timeout`、`io`、`network`、`unsupported` 或 `internal`。

### `file`

所属文件请求结束时发送。屏保只能收到读取操作的结果。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `request_id` | `integer` | Session 内部的请求 ID。 |
| `kind` | `string` | `read_text`、`read_bytes`、`write_text` 或 `write_bytes`。 |
| `path` | `string` | Lua 提交的虚拟路径，不会是宿主绝对路径。 |
| `ok` | `boolean` | 操作是否成功。 |
| `text` | `string \| nil` | `read_text` 成功时返回的 UTF-8 文本。 |
| `bytes` | `string \| nil` | `read_bytes` 成功时返回的 Lua 二进制字符串。 |
| `error` | `table \| nil` | `ok == false` 时存在的通用错误对象。 |

写入成功时不包含结果正文。`text` 和 `bytes` 互斥。

### `image`

所属图片转换请求结束时发送。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `request_id` | `integer` | Session 内部的请求 ID。 |
| `kind` | `string` | 固定为 `convert`。 |
| `ok` | `boolean` | 转换是否成功。 |
| `output` | `string \| nil` | 成功时的转换结果 ID 或虚拟输出路径。 |
| `error` | `table \| nil` | `ok == false` 时存在的通用错误对象。 |

### `network`

所属 HTTP 请求完成、失败或取消时发送一次。404、500 等 HTTP 错误状态属于正常响应，因此使用 `ok = true`。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `request_id` | `integer` | Session 内部的请求 ID。 |
| `kind` | `string` | `get` 或 `post`。 |
| `url` | `string` | 原始规范化请求 URL。 |
| `ok` | `boolean` | HTTP 交互是否正常完成。 |
| `final_url` | `string \| nil` | 重定向后的最终 URL，仅成功时存在。 |
| `status` | `integer \| nil` | HTTP 状态码，仅成功时存在。 |
| `headers` | `table<string, string> \| nil` | 经过过滤且键名为小写的响应头，仅成功时存在。 |
| `text` | `string \| nil` | 文本响应模式下经过严格 UTF-8 验证的正文。 |
| `bytes` | `string \| nil` | 二进制响应模式下的 Lua 二进制字符串。 |
| `error` | `table \| nil` | `ok == false` 时存在的通用错误对象。 |

`text` 和 `bytes` 互斥。Lua 网络提交 API 与包级网络权限声明尚未开放，但事件结构和宿主路由已经完成。

## 交互 UI 对象事件

这些事件仅提供给游戏，并且只由当前游戏 Session 自己拥有的 UI 对象产生。所有 ID 都是 Session 内部的不透明整数。宿主 UI 对象不会产生 Lua 事件。

### `hit_area`

所属点击区域收到鼠标交互时发送。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `id` | `integer` | Session 内部的点击区域 ID。 |
| `kind` | `string` | `hover_enter`、`hover_move`、`hover_leave`、`press`、`release`、`click` 或 `drag`。 |
| `x` | `integer` | 事件水平坐标。 |
| `y` | `integer` | 事件垂直坐标。 |
| `button` | `string \| nil` | 按键事件中的 `left`、`middle` 或 `right`。 |
| `dx` | `integer \| nil` | 水平拖动距离，仅 `drag` 存在。 |
| `dy` | `integer \| nil` | 垂直拖动距离，仅 `drag` 存在。 |

### `hyperlink`

所属超链接被点击时发送。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `id` | `integer` | Session 内部的超链接 ID。 |
| `kind` | `string` | 固定为 `clicked`。 |
| `link` | `string` | 超链接目标。 |

### `markdown`

所属 Markdown 视图中的链接被点击时发送。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `id` | `integer` | Session 内部的 Markdown 视图 ID。 |
| `kind` | `string` | 固定为 `link_clicked`。 |
| `href` | `string` | 链接目标。 |
| `text` | `string` | 链接显示文本。 |

### `text_input`

所属文本输入框的交互状态或内容发生变化时发送。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `id` | `integer` | Session 内部的文本输入框 ID。 |
| `kind` | `string` | `focused`、`blurred`、`changed`、`submit`、`cancel`、`pressed` 或 `pressed_outside`。 |
| `value` | `string \| nil` | `changed`、`submit` 和 `cancel` 事件中的当前文本。 |

### `scroll_box`

所属滚动框的滚动位置发生变化时发送。

| `data` 字段 | Lua 类型 | 含义 |
|---|---|---|
| `id` | `integer` | Session 内部的滚动框 ID。 |
| `kind` | `string` | 固定为 `scrolled`。 |
| `x` | `integer` | 新的水平滚动偏移。 |
| `y` | `integer` | 新的垂直滚动偏移。 |

## 队列事件合并

为避免高频输入耗尽 Session 队列，宿主可以使用最新事件替换以下尚未处理的旧事件：

- `resize`
- 类型与鼠标按键均相同的 `mouse.moved` 或 `mouse.held`
- 同一对象的 `hit_area.hover_move`
- 同一对象的 `scroll_box`

按下、释放、拖动、滚轮、焦点、生命周期、计时器、动画及异步完成事件不会被合并。
