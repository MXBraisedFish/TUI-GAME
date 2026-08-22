# 屏保包 `package.json`

本文描述当前宿主实际支持的屏保包清单结构。`package.json` 必须是合法 JSON；下方示例使用 JSONC 注释辅助说明，实际文件中不要保留注释。

## 目录结构

```text
screensaver_package/
├── package.json
├── scripts/
│   └── main.lua
└── assets/
    ├── icon.txt
    ├── banner.png
    └── language/
        ├── en_us/
        │   └── display.json
        └── zh_cn/
            └── display.json
```

官方屏保包位于 `scripts/screensaver/<包目录>/`，模组屏保包位于 `data/mod/screensaver/<包目录>/`。清单中的所有相对路径均以包目录为边界，不能使用绝对路径、`..`、反斜杠或符号链接越界。

## 完整结构

```jsonc
{
  "mod_id": "example.screensaver",
  "schema_version": 1,
  "type": "screensaver",

  "version": "1.0.0",
  "version_code": 1,

  "api": {
    "min": 1,
    "max": 1
  },

  "entry": "main",

  "display": {
    "title": "Example Screensaver",
    "description": {
      "type": "i18n",
      "path": "display.json",
      "key": "description",
      "callback": "An example terminal screensaver."
    },
    "author": {
      "type": "text",
      "text": "Author"
    },
    "icon": {
      "type": "text",
      "path": "icon.txt"
    },
    "banner": {
      "type": "image",
      "path": "banner.png"
    }
  },

  "runtime": {
    "min_width": 60,
    "min_height": 20
  },

  "screensaver": {
    "name": {
      "type": "i18n",
      "path": "display.json",
      "key": "name",
      "callback": "Example Screensaver"
    },
    "truecolor": false,
    "command": "example"
  }
}
```

## 顶层字段

| 字段 | 类型 | 必填 | 约束与含义 |
| --- | --- | --- | --- |
| `mod_id` | string | 是 | 包标识，1–128 字节；仅允许 ASCII 字母、数字、`.`、`_`、`-`。同一来源、同一包类型内必须唯一。 |
| `schema_version` | integer | 是 | 必须等于宿主当前清单版本，当前为 `1`。 |
| `type` | string | 是 | 屏保包固定为 `"screensaver"`。 |
| `version` | package-text | 是 | 展示版本文本，解析后不可为空。 |
| `version_code` | integer | 是 | 必须大于 `0`。 |
| `api` | object | 是 | 宿主 API 兼容区间，包含 `min` 和 `max`。必须满足 `min <= max`，且当前宿主 API 位于该闭区间内。 |
| `entry` | string | 是 | 相对于 `scripts/` 的 Lua 入口，可省略 `.lua`。 |
| `display` | object | 是 | 包的展示信息。 |
| `runtime` | object | 是 | 运行所需的最小 Base 画布尺寸。 |
| `screensaver` | object | 是 | 屏保专属配置。 |
| `game` | object | 否 | 屏保包不得提供该对象。 |

## 文本字段 `package-text`

`version`、`display.title`、`display.description`、`display.author` 和 `screensaver.name` 均可使用以下形式。

### 简写纯文本

```json
"name": "Example Screensaver"
```

等价于：

```json
"name": {
  "type": "text",
  "text": "Example Screensaver"
}
```

### i18n 文本

```json
"name": {
  "type": "i18n",
  "path": "display.json",
  "key": "name",
  "callback": "Example Screensaver"
}
```

| 字段 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `type` | string | 是 | `"text"` 或 `"i18n"`。 |
| `text` | string | `type=text` 时是 | 直接使用的文本。 |
| `path` | string | `type=i18n` 时是 | 相对于 `assets/language/<language_code>/` 的安全 `.json` 路径。 |
| `key` | string | `type=i18n` 时是 | JSON 文件中的键，不可为空。 |
| `callback` | string | `type=i18n` 时是 | 当前语言和 `en_us` 均无有效值时使用的最终回退文本。 |

i18n 的查询顺序固定为：用户当前语言 → `en_us` → `callback`。这些展示文本可以包含宿主支持的富文本语法。

## `display`

| 字段 | 类型 | 必填 | 默认值/约束 |
| --- | --- | --- | --- |
| `title` | package-text | 是 | 包标题，解析后不可为空。 |
| `description` | package-text | 是 | 包简介。 |
| `author` | package-text | 是 | 作者信息。 |
| `icon` | display-asset | 否 | 未提供时使用宿主默认图标。 |
| `banner` | display-asset | 否 | 未提供时使用宿主默认横幅。 |

展示资源结构：

```json
{
  "type": "text",
  "path": "ui/icon.txt"
}
```

- `type="image"` 支持 `.png`、`.jpg`、`.jpeg`。
- `type="text"` 仅支持 UTF-8 `.txt`。
- `path` 相对于包内 `assets/`，必须为安全的正向斜杠相对路径。
- 文本图标按 `8×4` 单元格规范化，文本横幅按 `60×14` 单元格规范化。
- 显式资源的类型、扩展名或路径不合法时，整个包会被拒绝扫描；只有省略字段才使用默认资源。

## `runtime`

| 字段 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `min_width` | integer | 是 | 最小 Base 画布宽度；`0` 表示不限制。 |
| `min_height` | integer | 是 | 最小 Base 画布高度；`0` 表示不限制。 |

屏保启动后，宿主依据屏保可用的 Base 画布尺寸检查该要求。尺寸不足时优先显示尺寸提醒，并允许用户关闭屏保。

## `screensaver`

| 字段 | 类型 | 必填 | 默认值/约束 |
| --- | --- | --- | --- |
| `name` | package-text | 是 | 屏保名称，解析后不可为空。 |
| `truecolor` | boolean | 否 | 默认 `false`；表示屏保需要真彩色，仅作为提示。 |
| `command` | string | 是 | 屏保快捷启动命令，去除首尾空白后不可为空。 |

屏保包不提供 `game.actions`；屏保运行时的 Lua 事件范围由事件协议单独定义。

## 必填字段缺失时

缺少必填字段、字段类型错误、值超出约束、入口越界或类型专属配置不一致时，该包会被单独拒绝扫描并记录原因，不影响其他包和宿主继续运行。
