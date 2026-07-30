# 事件参考文档

本文档列出了你的游戏可以通过 `HandleEvent(event)` 收到的所有事件类型及其数据结构。

## 事件结构

所有事件都遵循同一个外层结构：

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

| 字段 | 类型 | 说明 |
|---|---|---|
| `type` | `string` | 事件类型，所有可选值见下文。 |
| `sequence` | `integer` | 全局递增的事件序号。事件会按 Session 过滤，序号不连续是正常的。 |
| `frame` | `integer` | 事件产生时所在的帧号。 |
| `data` | `table` | 具体的事件数据，结构由 `type` 决定。 |

读取不存在的字段会得到 `nil`。

事件会传递给 `HandleEvent(event)` 或你在创建对象／发起请求时注册的回调函数，两者不会重复收到同一条事件。

每个 Session 有独立的事件队列，每帧最多接收 128 条事件，队列上限为 1024 条。当前帧新产生的事件会推迟到下一帧处理。

## 哪些事件归哪个 Session

| 事件来源 | 游戏 Session | 屏保 Session |
|---|---|---|
| 动作按键、鼠标输入 | ✓ | ✗ |
| 终端尺寸变化、焦点变化 | ✓ | ✓ |
| 屏保启动／停止 | ✓ | ✗ |
| 自己创建的计时器、动画 | ✓ | ✓ |
| 自己发起的文件请求 | ✓ | 只读 |
| 自己发起的图片转换 | ✓ | ✓ |
| 自己发起的网络请求 | ✓ | 待权限 API |
| 自己的音频对象 | ✓ | ✓ |
| 自己创建的交互 UI 对象 | ✓ | ✗ |

屏保运行期间，游戏收不到 `action`、`mouse` 和交互 UI 事件，但 `resize`、`focus`、屏保生命周期以及后台操作的结果依然可以收到。

## 系统事件

### `action`

按键动作事件。屏保期间不会发送。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `action` | `string` | 在游戏包中定义的动作 ID（不会暴露原始按键）。 |
| `state` | `string` | `pressed`（按下）、`held`（按住）、`released`（释放）。 |

### `mouse`

鼠标事件。仅当终端处于聚焦状态且鼠标落在游戏可视区域内时发送，坐标相对于游戏区域左上角，从 0 开始。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `kind` | `string` | `pressed`、`released`、`moved`、`dragged`、`held`、`scrolled`。 |
| `button` | `string \| nil` | `left`、`middle`、`right`；非按键事件不出现。 |
| `scroll` | `string \| nil` | 滚轮方向：`up`、`down`、`left`、`right`；仅 `scrolled` 事件出现。 |
| `x` | `integer` | 水平单元格坐标（从 0 开始）。 |
| `y` | `integer` | 垂直单元格坐标（从 0 开始）。 |

### `resize`

终端尺寸变化时发送，游戏和屏保都会收到。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `width` | `integer` | 终端宽度（单元格）。 |
| `height` | `integer` | 终端高度（单元格）。 |

### `focus`

终端焦点变化时发送，游戏和屏保都会收到。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `gained` | `boolean` | `true` 获得焦点，`false` 失去焦点。 |

> 失去焦点时不会自动生成按键的 `released` 事件。建议收到 `gained == false` 时自行重置按键状态。

### `screensaver_started`

屏保启动时发送给游戏。`data` 为空表。

### `screensaver_stopped`

屏保停止时发送给游戏。`data` 为空表。

## 计时器与动画事件

这些事件只会发给创建它们的 Session。ID 都是 Session 内部的不透明整数。

### `timer`

计时器触发或结束时发送。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `integer` | Session 内部计时器 ID。 |
| `timer_kind` | `string` | 计时器类型：`timer`、`delay`、`repeat`、`sleep`。 |
| `kind` | `string` | 事件类型：`tick`（触发）或 `finished`（结束）。 |
| `executed_count` | `integer \| nil` | 已执行的次数，仅重复计时器的 `tick` 事件出现。 |

`timer` 和 `delay` 只在结束时产生 `finished`；重复计时器每次触发产生 `tick`，最后产生 `finished`；异步休眠只在结束时产生 `finished`。

### `animation`

动画生命周期变化时发送。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `integer` | Session 内部动画 ID。 |
| `kind` | `string` | 事件类型：`started`、`marker`、`loop`、`finished`、`cancelled`。 |
| `name` | `string \| nil` | 标记名称，仅 `kind == "marker"` 时出现。 |
| `completed` | `integer \| nil` | 已完成的循环次数，仅 `kind == "loop"` 时出现。 |

## 异步请求结果

以下事件是你发起的异步请求的最终结果。如果你在发起请求时注册了回调，事件会直接发给该回调，否则发给 `HandleEvent`。

### 错误对象

失败时 `data` 中会包含一个 `error` 表：

```lua
error = {
  code = "timeout",
  message = "request timed out"
}
```

| 字段 | 类型 | 说明 |
|---|---|---|
| `code` | `string` | 稳定的错误码，可用于程序逻辑判断。 |
| `message` | `string` | 供阅读的错误描述，不含路径、ID、请求内容等内部信息。 |

错误码一览：`invalid_request`、`permission_denied`、`not_found`、`too_large`、`invalid_utf8`、`cancelled`、`timeout`、`io`、`network`、`unsupported`、`decode`、`backend_unavailable`、`internal`。

### `file`

文件操作完成时发送。屏保只能收到读结果。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `request_id` | `integer` | Session 内部请求 ID。 |
| `kind` | `string` | `read_text`、`read_bytes`、`write_text`、`write_bytes`。 |
| `path` | `string` | 你提交的路径（不会暴露引擎内部路径）。 |
| `ok` | `boolean` | 是否成功。 |
| `text` | `string \| nil` | `read_text` 成功时返回的 UTF-8 文本。 |
| `bytes` | `string \| nil` | `read_bytes` 成功时返回的二进制串。 |
| `error` | `table \| nil` | 失败时的错误对象。 |

写操作成功时不返回正文。`text` 和 `bytes` 互斥。

### `image`

图片转换完成时发送。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `request_id` | `integer` | Session 内部请求 ID。 |
| `kind` | `string` | 固定为 `convert`。 |
| `ok` | `boolean` | 是否成功。 |
| `output` | `string \| nil` | 成功时的输出路径或 ID。 |
| `error` | `table \| nil` | 失败时的错误对象。 |

### `network`

HTTP 请求完成时发送。注意：HTTP 层面的 404、500 等状态码属于正常完成，此时 `ok` 为 `true`。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `request_id` | `integer` | Session 内部请求 ID。 |
| `kind` | `string` | `get` 或 `post`。 |
| `url` | `string` | 请求的规范化 URL。 |
| `ok` | `boolean` | 网络交互是否正常完成。 |
| `final_url` | `string \| nil` | 重定向后的最终 URL，仅成功时出现。 |
| `status` | `integer \| nil` | HTTP 状态码，仅成功时出现。 |
| `headers` | `table<string, string> \| nil` | 过滤后的响应头（键名为小写），仅成功时出现。 |
| `text` | `string \| nil` | 文本模式下的响应正文（UTF-8 验证）。 |
| `bytes` | `string \| nil` | 二进制模式下的响应正文。 |
| `error` | `table \| nil` | 失败时的错误对象。 |

`text` 和 `bytes` 互斥。

## 音频事件

### `audio`

你的音频对象加载或播放状态发生变化时发送。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `integer` | Session 内部音频对象 ID。 |
| `kind` | `string` | `ready`、`started`、`paused`、`resumed`、`stopped`、`finished`、`failed`。 |
| `duration_ms` | `integer \| nil` | 音频时长（毫秒），仅 `ready` 和 `finished` 出现。 |
| `position_ms` | `integer \| nil` | 播放位置（毫秒），仅 `started`、`paused`、`resumed` 出现。 |
| `error` | `table \| nil` | `failed` 时的错误对象。音频特有错误码：`decode`、`backend_unavailable`。 |

同一个音频对象的每次状态变化都会发给同一个回调。`finished` 后不会自动回收对象（允许重播），只有你手动删除或 Session 停止时才回收。

## 交互 UI 事件

以下事件仅游戏 Session 可收到，由你自己的 UI 对象产生。ID 均为 Session 内部不透明整数。

### `hit_area`

点击区域收到鼠标交互时发送。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `integer` | Session 内部点击区域 ID。 |
| `kind` | `string` | `hover_enter`、`hover_move`、`hover_leave`、`press`、`release`、`click`、`drag`。 |
| `x` | `integer` | 事件水平坐标。 |
| `y` | `integer` | 事件垂直坐标。 |
| `button` | `string \| nil` | `left`、`middle`、`right`（按键事件）。 |
| `dx` | `integer \| nil` | 水平拖动距离，仅 `drag` 出现。 |
| `dy` | `integer \| nil` | 垂直拖动距离，仅 `drag` 出现。 |

### `hyperlink`

超链接被点击时发送。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `integer` | Session 内部超链接 ID。 |
| `kind` | `string` | 固定为 `clicked`。 |
| `link` | `string` | 链接目标 URL。 |

### `markdown`

Markdown 视图中的链接被点击时发送。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `integer` | Session 内部 Markdown 视图 ID。 |
| `kind` | `string` | 固定为 `link_clicked`。 |
| `href` | `string` | 链接目标。 |
| `text` | `string` | 链接显示文本。 |

### `text_input`

文本输入框状态发生变化时发送。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `integer` | Session 内部文本输入框 ID。 |
| `kind` | `string` | `focused`、`blurred`、`changed`、`submit`、`cancel`、`pressed`、`pressed_outside`。 |
| `value` | `string \| nil` | 当前文本内容（`changed`、`submit`、`cancel` 事件）。 |

### `scroll_box`

滚动框滚动时发送。

| `data` 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `integer` | Session 内部滚动框 ID。 |
| `kind` | `string` | 固定为 `scrolled`。 |
| `x` | `integer` | 当前水平滚动位置。 |
| `y` | `integer` | 当前垂直滚动位置。 |

## 事件合并

为防止高频输入塞满队列，引擎会自动合并队列中尚未处理的同类事件，合并规则如下：

- `resize`：保留最新的一条。
- `mouse`：同一类型的 `moved` 或 `held` 事件保留最新的。
- `hit_area`：同一对象的 `hover_move` 保留最新的。
- `scroll_box`：同一对象的滚动事件保留最新的。

以下事件不会被合并：按下、释放、拖动、滚轮、焦点、生命周期、计时器、动画以及所有异步完成事件。
