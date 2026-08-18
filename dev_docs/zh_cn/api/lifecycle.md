# Lifecycle 库

## 基本库说明

`Lifecycle` 提供脚本的生命周期回调。

---

## 目录

### 回调

| 回调名        | 说明         |
| ------------- | ------------ |
| `Init`        | 初始化       |
| `HandleEvent` | 事件处理     |
| `Update`      | 物理更新     |
| `UpdateFrame` | 帧更新       |
| `Render`      | 绘制         |
| `SaveGame`    | 保存游戏数据 |
| `SaveBest`    | 保存最佳成绩 |

---

## 回调

## `Init`

初始化

### 调用

```lua
function Init(ctx)
end
```

### 参数结构

| 字段            | 类型    | 说明             |
| --------------- | ------- | ---------------- |
| `package_id`    | string  | 模组包 ID        |
| `package_type`  | string  | 模组包类型       |
| `base`          | table   | 基础画布状态信息 |
| `base.width`    | integer | 基础画布宽度     |
| `base.height`   | integer | 基础画布高度     |
| `api_version`   | integer | API 版本         |
| `continue_data` | any     | 继续游戏数据     |

事件返回，请查看⌊[事件结构](../EVENT.md)⌉文档⌊i18n⌉部分。

### 示例

```lua
i18n.create {}

function HandleEvent(event)
  if event.type == "i18n" then
    debug.print { message = serialization.json_encode(event) }
  end
end
```

输出：

```json
{
  "type": "i18n",
  "frame": X,
  "sequence": X,
  "data": {
    "kind": "created",
    "ok": true,
    "language_code": "zh_cn",
    "callback_language_code": "en_us",
    "message": "i18n instance created",
  },
}
```

### 额外补充

- 该 API 为创建**单例**，每个脚本环境仅存在一个实例对象，若已经存在时重复调用，忽视请求。
- `i18n` 库 API 的使用要求**特殊**的资源目录结构，请查看⌊[国际化语言](../I18N.md)⌉文档。

---

## `get_value`

获取指定命名空间下的键值。

### 调用

```lua
-- 表参数
i18n.get_value{}
```

### 参数

| 参数名      | 类型   | 必填 | 默认值 | 说明         |
| ----------- | ------ | ---- | ------ | ------------ |
| `namespace` | string | 是   | -      | 命名空间名称 |
| `key`       | string | 是   | -      | 键名         |

### 返回

直接返回一个值。

| 类型   | 说明 |
| ------ | ---- |
| string | 键值 |

### 示例

```lua
assets/
- language/
  + zh_cn/
  | - test.json: { "title": "标题" }
  - en_us/
    - test.json: { "title": "Title", "setting": "Setting" }

v1 = i18n.get_value { namespace = "test", key = "title" }
debug.print { message = v1 }

v2 = i18n.get_value { namespace = "test", key = "setting" }
debug.print { message = v2 }

v3 = i18n.get_value { namespace = "test", key = "start" }
debug.print { message = v3 }
```

输出：

```text
标题
Setting
[缺少 i18n 键：test.start]
```

---

## `get_language_code`

获取当前系统所设置的语言代码。

### 调用

```lua
-- 单参数
i18n.get_language_code()
```

### 参数

无。

### 返回

直接返回一个值。

| 类型   | 说明                 |
| ------ | -------------------- |
| string | 系统所设置的语言代码 |

### 示例

```lua
i18n.get_language_code()
```

输出：

```text
zh_cn
```

---

## `reload`

重新加载当前语言。

### 调用

```lua
-- 表参数
i18n.reload{}
```

### 参数

| 参数名                   | 类型   | 必填 | 默认值               | 说明       |
| ------------------------ | ------ | ---- | -------------------- | ---------- |
| `language_code`          | string | 否   | 系统所设置的语言代码 | 首选项语言 |
| `callback_language_code` | string | 否   | `"en_us"`            | 备用语言   |

### 返回

事件返回，请查看⌊[事件结构](../EVENT.md)⌉文档⌊i18n⌉部分。

### 示例

```lua
i18n.reload {}

function HandleEvent(event)
  if event.type == "i18n" then
    debug.print { message = serialization.json_encode(event) }
  end
end
```

输出：

```json
{
  "type": "i18n",
  "frame": X,
  "sequence": X,
  "data": {
    "kind": "created",
    "ok": true,
    "language_code": "zh_cn",
    "callback_language_code": "en_us",
    "message": "i18n instance created",
  },
}
```

### 额外补充

- 该 API 在实例未创建时忽视请求。
