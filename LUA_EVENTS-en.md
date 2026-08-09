# Event Reference

This document lists every event type your game can receive via `HandleEvent(event)` along with their data structures.

## Event Structure

Every event follows the same outer envelope:

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

| Field | Type | Description |
|---|---|---|
| `type` | `string` | Event type. See all available values below. |
| `sequence` | `integer` | A globally incrementing event number. Events are filtered per session, so gaps in the sequence are normal. |
| `frame` | `integer` | The frame number when this event was generated. |
| `data` | `table` | Event-specific payload, whose structure depends on `type`. |

Accessing a field that does not exist yields `nil`.

Events are delivered either to `HandleEvent(event)` or to the callback you registered when creating an object / initiating a request. The same event is never delivered to both.

Each session has its own event queue. At most 128 events are delivered per frame, with a queue capacity of 1024. Events generated during the current frame are deferred to the next frame.

## Session Routing

| Event Source | Game Session | Screensaver Session |
|---|---|---|
| Actions, mouse input | ✓ | ✗ |
| Terminal resize, focus change | ✓ | ✓ |
| Screensaver start / stop | ✓ | ✗ |
| Your own timers, animations | ✓ | ✓ |
| Your own file requests | ✓ | Read-only |
| Your own image conversions | ✓ | ✓ |
| Your own network requests | ✓ | Pending permission API |
| Your own audio objects | ✓ | ✓ |
| Your own interactive UI objects | ✓ | ✗ |
| Language load / reload (i18n) | ✓ | ✓ |

While the screensaver is active, the game will not receive `action`, `mouse`, or interactive UI events. However, `resize`, `focus`, screensaver lifecycle events, and results of background operations are still delivered.

## System Events

### `action`

Key action events. Not sent during screensaver.

| `data` field | Type | Description |
|---|---|---|
| `action` | `string` | The action ID defined in your game package (raw keys are never exposed). |
| `state` | `string` | `pressed`, `held`, or `released`. |

### `mouse`

Mouse events. Only sent when the terminal is focused and the mouse is inside the game's visible area. Coordinates are relative to the game area's top-left corner, starting from 0.

| `data` field | Type | Description |
|---|---|---|
| `kind` | `string` | `pressed`, `released`, `moved`, `dragged`, `held`, or `scrolled`. |
| `button` | `string \| nil` | `left`, `middle`, or `right`; absent for non-button events. |
| `scroll` | `string \| nil` | Scroll direction: `up`, `down`, `left`, or `right`; only present in `scrolled` events. |
| `x` | `integer` | Horizontal cell coordinate (0-based). |
| `y` | `integer` | Vertical cell coordinate (0-based). |

### `resize`

Sent when the terminal size changes. Delivered to both the game and screensaver.

| `data` field | Type | Description |
|---|---|---|
| `width` | `integer` | New terminal width in cells. |
| `height` | `integer` | New terminal height in cells. |

### `focus`

Sent when the terminal gains or loses focus. Delivered to both the game and screensaver.

| `data` field | Type | Description |
|---|---|---|
| `gained` | `boolean` | `true` when focus is gained, `false` when lost. |

> Losing focus does NOT automatically generate `released` events for held keys. It's recommended to reset your key state when `gained == false`.

### `screensaver_started`

Sent to the game when the screensaver launches. `data` is an empty table.

### `screensaver_stopped`

Sent to the game when the screensaver stops. `data` is an empty table.

## Timer & Animation Events

These events are only delivered to the session that created them. All IDs are opaque integers scoped to the session.

### `timer`

Sent when a timer fires or finishes.

| `data` field | Type | Description |
|---|---|---|
| `id` | `integer` | Session-scoped timer ID. |
| `timer_kind` | `string` | Timer type: `timer`, `delay`, `repeat`, or `sleep`. |
| `kind` | `string` | Event kind: `tick` (fired) or `finished` (ended). |
| `executed_count` | `integer \| nil` | Number of times executed so far; only present in `tick` events from repeat timers. |

`timer` and `delay` only produce `finished` at the end. Repeat timers produce `tick` on each interval and a final `finished`. Sleep timers only produce `finished`.

### `animation`

Sent when an animation's lifecycle changes.

| `data` field | Type | Description |
|---|---|---|
| `id` | `integer` | Session-scoped animation ID. |
| `kind` | `string` | Event kind: `started`, `marker`, `loop`, `finished`, or `cancelled`. |
| `name` | `string \| nil` | Marker name; only present when `kind == "marker"`. |
| `completed` | `integer \| nil` | Completed loop count; only present when `kind == "loop"`. |

## Async Request Results

The following events are the final results of your async requests. If you registered a callback when making the request, the event is delivered to that callback; otherwise, it goes to `HandleEvent`.

### Error Object

On failure, `data` includes an `error` table:

```lua
error = {
  code = "timeout",
  message = "request timed out"
}
```

| Field | Type | Description |
|---|---|---|
| `code` | `string` | Stable error code suitable for programmatic matching. |
| `message` | `string` | Human-readable error description. Does not contain internal details like file paths, request IDs, or request bodies. |

Error codes: `invalid_request`, `permission_denied`, `not_found`, `too_large`, `invalid_utf8`, `cancelled`, `timeout`, `io`, `network`, `unsupported`, `decode`, `backend_unavailable`, `internal`.

### `file`

Sent when a file operation completes. Screensavers only receive read results.

| `data` field | Type | Description |
|---|---|---|
| `request_id` | `integer` | Session-scoped request ID. |
| `kind` | `string` | `read_text`, `read_bytes`, `write_text`, or `write_bytes`. |
| `path` | `string` | The path you submitted (engine-internal paths are never exposed). |
| `ok` | `boolean` | Whether the operation succeeded. |
| `text` | `string \| nil` | UTF-8 text result on successful `read_text`. |
| `bytes` | `string \| nil` | Binary string result on successful `read_bytes`. |
| `error` | `table \| nil` | Error object on failure. |

Write operations return no result body on success. `text` and `bytes` are mutually exclusive.

### `image`

Sent when an image conversion completes.

| `data` field | Type | Description |
|---|---|---|
| `request_id` | `integer` | Session-scoped request ID. |
| `kind` | `string` | Always `convert`. |
| `ok` | `boolean` | Whether the conversion succeeded. |
| `output` | `string \| nil` | Output path or ID on success. |
| `error` | `table \| nil` | Error object on failure. |

### `network`

Sent when an HTTP request completes. Note: HTTP-level 404, 500, etc. are considered successful completion, so `ok` will be `true`.

| `data` field | Type | Description |
|---|---|---|
| `request_id` | `integer` | Session-scoped request ID. |
| `kind` | `string` | `get` or `post`. |
| `url` | `string` | The normalized request URL. |
| `ok` | `boolean` | Whether the network interaction completed normally. |
| `final_url` | `string \| nil` | Final URL after redirects; only present on success. |
| `status` | `integer \| nil` | HTTP status code; only present on success. |
| `headers` | `table<string, string> \| nil` | Filtered response headers (lowercase keys); only present on success. |
| `text` | `string \| nil` | Response body in text mode (UTF-8 validated). |
| `bytes` | `string \| nil` | Response body in binary mode. |
| `error` | `table \| nil` | Error object on failure. |

`text` and `bytes` are mutually exclusive.

## Audio Events

### `audio`

Sent when the state of an audio object you own changes.

| `data` field | Type | Description |
|---|---|---|
| `id` | `integer` | Session-scoped audio object ID. |
| `kind` | `string` | `ready`, `started`, `paused`, `resumed`, `stopped`, `finished`, or `failed`. |
| `duration_ms` | `integer \| nil` | Audio duration in milliseconds; only present for `ready` and `finished`. |
| `position_ms` | `integer \| nil` | Playback position in milliseconds; only present for `started`, `paused`, and `resumed`. |
| `error` | `table \| nil` | Error object on `failed`. Audio-specific error codes: `decode`, `backend_unavailable`. |

Every state change for the same audio object is sent to the same callback. The object is not automatically reclaimed after `finished` (you can replay it). It is only reclaimed when you manually delete it or the session stops.

## i18n Events

Returned by the host after you call `i18n.create` or `i18n.reload`, and delivered to the session that made the call.

| `data` field | Type | Description |
|---|---|---|
| `kind` | `string` | `created` (from `create`) or `reloaded` (from `reload`). |
| `ok` | `boolean` | Whether the language loaded successfully (still `true` with an empty table when the language directory or files do not exist). |
| `message` | `string` | Load message: `"loaded"` for `create` and `"reloaded"` for `reload` on success; an error description when scanning fails. |
| `language_code` | `string` | The language code that was actually loaded. |
| `callback_language_code` | `string` | The fallback language code used by `get_key` when lookups fail. |

## Interactive UI Events

The following events are only available to the game session and are produced by your own UI objects. All IDs are opaque integers scoped to the session.

### `hit_area`

Sent when a hit area receives mouse interaction.

| `data` field | Type | Description |
|---|---|---|
| `id` | `integer` | Session-scoped hit area ID. |
| `kind` | `string` | `hover_enter`, `hover_move`, `hover_leave`, `press`, `release`, `click`, or `drag`. |
| `x` | `integer` | Horizontal event coordinate. |
| `y` | `integer` | Vertical event coordinate. |
| `button` | `string \| nil` | `left`, `middle`, or `right` (button events). |
| `dx` | `integer \| nil` | Horizontal drag distance; only present for `drag`. |
| `dy` | `integer \| nil` | Vertical drag distance; only present for `drag`. |

### `hyperlink`

Sent when a hyperlink is clicked.

| `data` field | Type | Description |
|---|---|---|
| `id` | `integer` | Session-scoped hyperlink ID. |
| `kind` | `string` | Always `clicked`. |
| `link` | `string` | The link target URL. |

### `markdown`

Sent when a link inside a Markdown view is clicked.

| `data` field | Type | Description |
|---|---|---|
| `id` | `integer` | Session-scoped Markdown view ID. |
| `kind` | `string` | Always `link_clicked`. |
| `href` | `string` | The link target. |
| `text` | `string` | The link's display text. |

### `text_input`

Sent when a text input's state or content changes.

| `data` field | Type | Description |
|---|---|---|
| `id` | `integer` | Session-scoped text input ID. |
| `kind` | `string` | `focused`, `blurred`, `changed`, `submit`, `cancel`, `pressed`, or `pressed_outside`. |
| `value` | `string \| nil` | Current text content (for `changed`, `submit`, and `cancel` events). |

### `scroll_box`

Sent when a scroll box is scrolled.

| `data` field | Type | Description |
|---|---|---|
| `id` | `integer` | Session-scoped scroll box ID. |
| `kind` | `string` | Always `scrolled`. |
| `x` | `integer` | Current horizontal scroll position. |
| `y` | `integer` | Current vertical scroll position. |

## Event Coalescing

To prevent high-frequency input from flooding the queue, the engine automatically merges unprocessed events of the same kind using the following rules:

- `resize`: Keeps only the latest.
- `mouse`: Keeps the latest `moved` or `held` event of the same kind.
- `hit_area`: Keeps the latest `hover_move` for the same object.
- `scroll_box`: Keeps the latest scroll event for the same object.

The following events are never coalesced: press, release, drag, scroll wheel, focus, lifecycle, timers, animations, and all async completion events.
