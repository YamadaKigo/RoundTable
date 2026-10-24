# RoundTable Architecture

## 1. Dependency Direction

Production code is split into four layers:

```text
engine
  ├── platform
  ├── domain
  └── application
          ├── platform
          └── domain
ui -> application + domain + engine + platform
```

- `platform`: Win32 message loop, GDI handles, raw input, pointer capture, filesystem assets.
- `engine`: math, immediate UI kit, layered software canvas, glyph atlas, theme.
- `domain`: market bars, chart model, indicators, indicator script.
- `application`: `StockApp`, frame orchestration, window sessions, editor/source workflow.
- `ui`: views and interactions. UI may depend on all lower layers; lower layers never depend on UI.

Module roots:

```text
src/
  application/
    app.rs
  domain/
    chart.rs
    data/
    indicator/
    layout.rs
    script.rs
  engine/
    core/
    font.rs
    gfx/
    kit.rs
    theme.rs
  platform/
    assets.rs
    mod.rs
    pointer.rs
    win32.rs
  ui/
```

## 2. Ownership

`StockApp` is the thin composition root. It owns only the application state and
the window manager as disjoint fields:

```text
StockApp
  state          AppState
  win_mgr        WinMgr
```

`AppState` owns the renderer and interaction state:

```text
AppState
  chart_world    ChartWorld
  chart_ui       ChartUiState
  hwnd           native window handle
  assets         AppAssets
  pointer        PointerManager
  stats          FrameStats
  shell          ShellState
  workbench      WorkbenchState
```

Window-manager planning and drawing take `&mut AppState` and `&mut WinMgr`
separately. UI modules receive `&mut AppState`, never the whole `StockApp`.
There is no `mem::replace`, no temporary empty manager, and no panic-catching
bridge to manufacture split borrows. Chart access remains explicit as
`app.chart_world`.

## 3. Frame Lifecycle

One frame follows this sequence:

1. `platform::win32` drains Win32 messages into a reusable event queue.
2. Mouse movement is coalesced to the configured frame rate for rendering, but
   every raw relative sample is retained for the pointer trajectory.
3. Button, keyboard, wheel and IME events use the urgent path and render immediately.
4. `StockApp::ui` resets transient UI state and advances the software pointer.
5. Shell and window-manager planning update layout and interaction ownership;
   the manager and application state are borrowed as disjoint fields.
6. Chart and editor views update domain state.
7. UI emits layered shapes tagged as `Base`, `Chart` or `Dynamic`.
8. `Canvas` consumes hashes accumulated while each mesh is built, updates only
   changed layers and composites changed tiles.
9. `platform::win32` presents merged dirty rectangles into the DIB without a subset copy.
10. Frame deadlines advance on a fixed cadence so render time is not added to
    the next wait interval.

## 4. Event and Pointer Model

- `Context` is single-threaded and uses `Rc<RefCell<Frame>>`.
- `InputState` persists pointer buttons, press origin, click timing and the last absolute position.
- Frame input moves the persistent input state instead of cloning it, and the
  platform recycles the event vector after each frame.
- Raw mouse packets use a fixed stack buffer and keep every relative sample, so
  the trajectory renderer receives a continuous input path without allocating
  at the polling rate.
- Normal pointer motion follows the absolute `WM_MOUSEMOVE` coordinate; it is not accelerated twice.
- Locked sessions consume raw mouse deltas 1:1.
- Free-motion rendering uses the native hardware cursor; the application only
  draws its own pointer while a session is locked to a fixed visual position.
- Locked visual position is pinned independently from the virtual drag position.
- Native cursor displacement is consumed once and cannot create an inertial tail.
- A panic propagates normally; the release profile is `panic = "abort"`, so no
  unreachable recovery branch is retained.

## 5. Rendering Model

`engine::kit` performs immediate-mode layout and produces immutable meshes.

`engine::gfx::Canvas` owns:

- the final 32-bit top-down framebuffer;
- three transparent layer surfaces;
- per-layer content hashes, per-tile content hashes and 32px coverage masks;
- final dirty tiles and a bounded dirty rectangle list.

Rules:

- `Base` contains static shell and non-topmost window content.
- `Chart` contains the topmost chart's data, axes and status bands.
- `Dynamic` contains crosshair lines and labels, pointer, tooltip, status text and caret.
- Mesh construction maintains its content hash incrementally. Frame comparison
  reads one hash per mesh instead of walking every vertex and index.
- A layer whose hash did not change is not rasterized.
- Within a changed layer, only 32px tiles whose primitive hashes changed are
  cleared and rasterized. Static full-screen backgrounds therefore do not
  defeat local damage tracking.
- A removed primitive changes every tile hash it touched, so stale pixels are
  cleared even when another primitive still covers the same tile.
- `WM_PAINT`, resize and texture replacement force a full cache rebuild.
- Dirty rectangles reuse one platform-owned `Vec`. The full top-down DIB is
  passed through a row pointer, with `ySrc = 0` and `uScan = 0`, so window DCs
  read the intended band without copying a subset bitmap.
- Window-local refresh text stays in that window's paint layer. It never moves
  to the global `Dynamic` layer, so an overlapping front window still masks it.

## 6. Domain Model

- `ChartWorld` owns bars, chart panes and palette state.
- `Pane` owns indicator implementation, script program, draw-command index and viewport.
- `PaneCmds` stores raw commands plus bar-to-command indexing.
- Visible command indices are written into a reusable scratch buffer.
- Indicator recomputation occurs only when a pane is dirty.
- Script parsing happens on source load or indicator replacement; per-frame work uses parsed commands.
- The legacy in-process indicator calculations are retained only under
  `cfg(test)` as compatibility fixtures. Production builds contain only the
  script renderer and the minimal indicator metadata needed by a pane.

## 7. Editor Model

- `TextBuffer` owns lines, cursor, selection, undo and redo.
- Character columns are Unicode scalar indices, not byte offsets.
- `TextEditor` owns scroll, IME preedit, focus, hit testing and layout cache.
- Layout cache stores cumulative line widths, syntax runs, line-number strings and user identifiers.
- Input events are consumed directly from `InputState` without cloning.
- Consecutive edits within 500ms share one undo snapshot; navigation breaks the group.
- Text selection follows absolute pointer coordinates for the entire drag and
  never replaces the mouse. The text hit position is clamped separately, so
  dragging beyond the editor continues selection and auto-scroll.
- Logical editor focus survives temporary app deactivation. Clicking back inside
  an existing selection restores focus without collapsing that selection.

## 8. Asset and Font Backends

- PNG and JSON decoding remain self-implemented.
- Theme colors have one declarative source that generates the built-in theme,
  JSON field decoding, and the theme struct.
- Startup loads icons, symbols and cursors in parallel using `std::thread`.
- GDI fonts preserve the required hard-edged 1bpp appearance.
- Glyph storage uses bounded 1024x1024 atlas pages, up to eight pages.
- A full atlas recycles the least-recently-used page and lazily rerasterizes evicted glyphs.

## 9. Invariants

- `Cargo.toml` has no third-party dependencies.
- Input, window, PNG, JSON and rendering remain self-implemented.
- UI text is English and ASCII.
- Input and editor content still accept Chinese through the embedded 16px
  ChillBitmap font; UI labels use the ASCII Sweet16 regular and Spleen 8x16
  bold bitmaps.
- Every output pixel is hard-edged; no antialiasing or alpha blending is introduced.
- All visible width/height logic snaps to the 9px grid unless an explicit pixel-pan API is used.
- No production dependency may point upward into `ui`.
- There are no blanket or `dead_code` suppressions. Narrow `expect` attributes
  remain for renderer primitives whose geometry arguments are intentionally
  positional, and each carries an explicit reason.
- Release builds retain debug symbols so crash backtraces include function and
  file locations. Crash-log write failures are reported to the user.

## 10. Verification

The test suite is the executable behavior specification. Required gates:

```text
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo test --all-targets --all-features
cargo build --release
```

Performance regression gates are documented in `PERFORMANCE_REPORT.md`.

