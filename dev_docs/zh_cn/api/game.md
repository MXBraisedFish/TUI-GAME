# game 库

## 基本库说明

`game` 提供游戏脚本生命周期控制 API。

---

## 目录

### 方法

| 方法名 | 说明 | 索引 |
| ----- | ---- | ---- |
| `exit_game` | 请求结束当前游戏脚本运行 | [exit_game](#exit_game) |
| `save_game` | 请求执行一次 `SaveGame` 回调，将游戏数据保存在"继续游戏"槽位 | [save_game](#save_game) |
| `save_best` | 请求执行一次 `SaveBest` 回调，在游戏列表展示本游戏最佳成绩记录 | [save_best](#save_best) |

---

## 方法

## `exit_game`

请求结束当前游戏脚本运行。

> 仅游戏脚本可用。

### 调用

```lua
-- 单参数
game.exit_game()
```

### 参数

无。

### 返回

无。

### 示例

```lua
game.exit_game()
```

### 额外补充

- 当 `package.json` 中，`game.save` 字段为 true 时，调用该 API 会在游戏脚本结束前，自动调用一次 `SaveGame` 回调。
- 当 `package.json` 中，`game.score.enabled` 字段为 true 时，调用该 API 会在游戏脚本结束前，自动调用一次 `SaveBest` 回调。
- **不可**在 `Init`, `SaveGame`, `SaveBest` 回调中调用。

---

## `save_game`

请求执行一次 `SaveGame` 回调, 将游戏数据保存在 `继续游戏` 槽位。

> 仅游戏脚本可用。
> 仅当 `package.json` 中，`game.save` 字段为 true 时可用。

### 调用

```lua
-- 单参数
game.save_game()
```

### 参数

无。

### 返回

无。

### 示例

```lua
game.save_game()
```

### 额外补充

- 保存的数据会在玩家使用 `继续游戏` 进入时，传递给 `Init` 回调。
- **不可**在 `SaveGame` 回调中调用。

---

## `save_best`

请求执行一次 `SaveBest` 回调, 在游戏列表展示本游戏最佳成绩记录。

> 仅游戏脚本可用。
> 仅当 `package.json` 中，`game.score.enabled` 字段为 true 时可用。

### 调用

```lua
-- 单参数
game.save_best()
```

### 参数

无。

### 返回

无。

### 示例

```lua
game.save_best()
```

### 额外补充

- **不可**在 `SaveBest` 回调中调用。