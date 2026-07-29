# Lua Event Reference

This document describes every event shape currently recognized by the Lua runtime.

System and lifecycle events are produced automatically. Service and UI-object events are produced
only for an object or asynchronous request owned by the receiving Lua session. Some producer APIs
are not exposed to Lua yet; defining an event here does not imply that its Lua creation API is
already available.

## Event envelope

Every event passed to Lua has the same envelope:

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

| Field | Lua type | Meaning |
|---|---|---|
| `type` | `string` | Event category. The possible values are documented below. |
| `sequence` | `integer` | Runtime-wide, monotonically increasing event sequence. Gaps are normal because events may be filtered per session. |
| `frame` | `integer` | Host frame in which the event was queued. |
| `data` | `table` | Type-specific payload. |

An event is delivered either to `HandleEvent(event)` or to the callback registered for the
originating operation, never both. A missing optional field reads as `nil`.

Each session has an independent FIFO queue. At most 128 events are delivered per host frame and at
most 1,024 events may remain pending. Events generated while Lua is handling the current batch are
deferred until the next host frame.

## Session routing

| Category | Game session | Screensaver session |
|---|---:|---:|
| Actions and mouse input | Yes | No |
| Resize and focus | Yes | Yes |
| Screensaver lifecycle | Yes | No |
| Session-owned timers and animations | Yes | Yes |
| Session-owned file results | Yes | Read operations only |
| Session-owned image results | Yes | Yes |
| Session-owned network results | Yes | Yes, subject to the future permission API |
| Session-owned interactive UI objects | Yes | No |

While a screensaver is active, the game receives no `action`, `mouse`, or interactive UI-object
events. It may still receive `resize`, `focus`, screensaver lifecycle events, and results from its
own background operations.

Raw terminal key events, host UI events, logs, popups, package scans, screenshots, recordings,
exports, video exports, and generic host task events are never exposed to Lua.

## System and lifecycle events

### `action`

Sent to the active game after host-global actions have consumed their matching input. It is not sent
while a screensaver is active.

| `data` field | Lua type | Meaning |
|---|---|---|
| `action` | `string` | Game action identifier defined by the package. Raw keys are never exposed. |
| `state` | `string` | `pressed`, `held`, or `released`. |

### `mouse`

Sent to the active game when the terminal is focused and the pointer event occurs inside the Base
viewport. Coordinates are zero-based and relative to that viewport.

| `data` field | Lua type | Meaning |
|---|---|---|
| `kind` | `string` | `pressed`, `released`, `moved`, `dragged`, `held`, or `scrolled`. |
| `button` | `string \| nil` | `left`, `middle`, or `right`; absent when the event has no button. |
| `scroll` | `string \| nil` | `up`, `down`, `left`, or `right`; present only for scrolling. |
| `x` | `integer` | Zero-based horizontal cell coordinate. |
| `y` | `integer` | Zero-based vertical cell coordinate. |

### `resize`

Sent when the terminal size changes. It is delivered to both active sessions.

| `data` field | Lua type | Meaning |
|---|---|---|
| `width` | `integer` | New terminal width in cells. |
| `height` | `integer` | New terminal height in cells. |

### `focus`

Sent when the terminal gains or loses focus. It is delivered to both active sessions.

| `data` field | Lua type | Meaning |
|---|---|---|
| `gained` | `boolean` | `true` when focus was gained, `false` when it was lost. |

The host does not synthesize `released` action events after focus loss. Games should clear their
own held-input state when `gained` is `false`.

### `screensaver_started`

Sent to the game after a screensaver session starts successfully.

`data` is an empty table.

### `screensaver_stopped`

Sent to the game after the active screensaver session stops.

`data` is an empty table.

## Time and animation events

These events belong only to the session that created the timer or animation. IDs are opaque,
session-local integers and are not host object IDs.

### `timer`

Sent when an owned timer ticks or finishes.

| `data` field | Lua type | Meaning |
|---|---|---|
| `id` | `integer` | Session-local timer ID. |
| `timer_kind` | `string` | `timer`, `delay`, `repeat`, or `sleep`. |
| `kind` | `string` | `tick` or `finished`. |
| `executed_count` | `integer \| nil` | Number of completed executions; present only for repeat timers. |

`timer` and `delay` currently produce `finished`. A repeat timer produces `tick` and then
`finished`. An asynchronous sleep produces `finished`.

### `animation`

Sent when an owned animation changes lifecycle state, reaches a marker, or completes a loop.

| `data` field | Lua type | Meaning |
|---|---|---|
| `id` | `integer` | Session-local animation ID. |
| `kind` | `string` | `started`, `marker`, `loop`, `finished`, or `cancelled`. |
| `name` | `string \| nil` | Marker name; present only when `kind == "marker"`. |
| `completed` | `integer \| nil` | Completed loop count; present only when `kind == "loop"`. |

## Asynchronous service events

Service events are terminal results for session-owned asynchronous requests. The host task ID is
never exposed. If an operation was submitted with a callback, the callback receives the complete
event envelope; otherwise `HandleEvent` receives it.

### Common error object

Failed service events contain:

```lua
error = {
  code = "timeout",
  message = "request timed out"
}
```

| Field | Lua type | Meaning |
|---|---|---|
| `code` | `string` | Stable machine-readable error code. |
| `message` | `string` | Sanitized, developer-readable message without host paths, task IDs, headers, bodies, or stack traces. |

Possible codes are `invalid_request`, `permission_denied`, `not_found`, `too_large`,
`invalid_utf8`, `cancelled`, `timeout`, `io`, `network`, `unsupported`, and `internal`.

### `file`

Sent when an owned file request finishes. Screensavers may receive results only for read operations.

| `data` field | Lua type | Meaning |
|---|---|---|
| `request_id` | `integer` | Session-local request ID. |
| `kind` | `string` | `read_text`, `read_bytes`, `write_text`, or `write_bytes`. |
| `path` | `string` | Virtual path supplied by Lua; never an absolute host path. |
| `ok` | `boolean` | Whether the operation succeeded. |
| `text` | `string \| nil` | UTF-8 text returned by a successful `read_text`. |
| `bytes` | `string \| nil` | Binary Lua string returned by a successful `read_bytes`. |
| `error` | `table \| nil` | Common error object when `ok == false`. |

Successful writes contain no result body. `text` and `bytes` are mutually exclusive.

### `image`

Sent when an owned image conversion finishes.

| `data` field | Lua type | Meaning |
|---|---|---|
| `request_id` | `integer` | Session-local request ID. |
| `kind` | `string` | Always `convert`. |
| `ok` | `boolean` | Whether conversion succeeded. |
| `output` | `string \| nil` | Converted output identifier or virtual output path on success. |
| `error` | `table \| nil` | Common error object when `ok == false`. |

### `network`

Sent once when an owned HTTP request finishes, fails, or is cancelled. HTTP error statuses such as
404 and 500 are valid responses and therefore use `ok = true`.

| `data` field | Lua type | Meaning |
|---|---|---|
| `request_id` | `integer` | Session-local request ID. |
| `kind` | `string` | `get` or `post`. |
| `url` | `string` | Original normalized request URL. |
| `ok` | `boolean` | Whether the HTTP exchange completed successfully. |
| `final_url` | `string \| nil` | Final URL after redirects; present on success. |
| `status` | `integer \| nil` | HTTP status code; present on success. |
| `headers` | `table<string, string> \| nil` | Filtered response headers with lowercase names; present on success. |
| `text` | `string \| nil` | Strict UTF-8 response body in text mode. |
| `bytes` | `string \| nil` | Binary Lua string response body in bytes mode. |
| `error` | `table \| nil` | Common error object when `ok == false`. |

`text` and `bytes` are mutually exclusive. The Lua network submission API and package permission
declaration are not exposed yet; this event schema and its host-side routing are already defined.

## Interactive UI-object events

These events are available only to games and only for UI objects owned by that game session. IDs are
opaque, session-local integers. Host UI objects never generate Lua events.

### `hit_area`

Sent when an owned hit area receives pointer interaction.

| `data` field | Lua type | Meaning |
|---|---|---|
| `id` | `integer` | Session-local hit-area ID. |
| `kind` | `string` | `hover_enter`, `hover_move`, `hover_leave`, `press`, `release`, `click`, or `drag`. |
| `x` | `integer` | Horizontal event coordinate. |
| `y` | `integer` | Vertical event coordinate. |
| `button` | `string \| nil` | `left`, `middle`, or `right` for button events. |
| `dx` | `integer \| nil` | Horizontal drag delta; present only for `drag`. |
| `dy` | `integer \| nil` | Vertical drag delta; present only for `drag`. |

### `hyperlink`

Sent when an owned hyperlink is clicked.

| `data` field | Lua type | Meaning |
|---|---|---|
| `id` | `integer` | Session-local hyperlink ID. |
| `kind` | `string` | Always `clicked`. |
| `link` | `string` | Hyperlink target. |

### `markdown`

Sent when a link in an owned Markdown view is clicked.

| `data` field | Lua type | Meaning |
|---|---|---|
| `id` | `integer` | Session-local Markdown view ID. |
| `kind` | `string` | Always `link_clicked`. |
| `href` | `string` | Link destination. |
| `text` | `string` | Visible link text. |

### `text_input`

Sent when an owned text input changes interaction state or value.

| `data` field | Lua type | Meaning |
|---|---|---|
| `id` | `integer` | Session-local text-input ID. |
| `kind` | `string` | `focused`, `blurred`, `changed`, `submit`, `cancel`, `pressed`, or `pressed_outside`. |
| `value` | `string \| nil` | Current value for `changed`, `submit`, and `cancel`. |

### `scroll_box`

Sent when an owned scroll box scroll position changes.

| `data` field | Lua type | Meaning |
|---|---|---|
| `id` | `integer` | Session-local scroll-box ID. |
| `kind` | `string` | Always `scrolled`. |
| `x` | `integer` | New horizontal scroll offset. |
| `y` | `integer` | New vertical scroll offset. |

## Queue coalescing

To prevent high-frequency input from exhausting a session queue, the host may replace an older
pending event with the newest event for:

- `resize`
- `mouse` events of the same `moved` or `held` kind and button
- `hit_area` `hover_move` events for the same object
- `scroll_box` events for the same object

Press, release, drag, scroll-wheel, focus, lifecycle, timer, animation, and asynchronous completion
events are never coalesced.
