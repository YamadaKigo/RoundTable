<p align="center">
  <img src="assets/logo/RoundTable_64.ico" width="64" height="64" alt="RoundTable logo">
</p>

<h1 align="center">RoundTable 圆桌</h1>

<p align="center">一个坚持 1 逻辑点 = 1 物理像素、全部界面由 Painter 手绘的 Windows 行情图表工作台。</p>

RoundTable 是使用 Rust、eframe、egui 与 wgpu 构建的桌面图表应用。它不使用传统控件皮肤，不依赖圆角、阴影、渐变、透明混色或抗锯齿；窗口、菜单、按钮、输入框、滚动条、图表和指针都由 1px 线条与纯色矩形逐像素绘制。

项目当前内置随机游走行情、K 线/美国线、成交量、MACD、KDJ、RSI、BOLL、OBV、CCI、BIAS、源码编辑器、浮动子窗口和调试控制台。核心数据使用轻量 ECS 组织，主题、界面符号和位图资源来自可替换的资源目录。

## 特性

- 无边框主窗口，客户区铺满窗口，窗口移动和子窗口拖动逐帧完成。
- 全局强制 1 逻辑点等于 1 物理像素。
- 9px 界面节距：`GRID = 8px`、`LINE_W = 1px`、`PITCH = 9px`。
- 纯色矩形与 1px 硬边，不使用 egui 默认控件外观。
- 中文宋体 16px 点阵字与 ASCII Fixedsys 8x16 点阵字。
- 轻量 ECS 核心：实体、组件表、世界资源与系统分层。
- 可编辑 JSON 主题，默认读取 `assets/themes/light.json`。
- 标题按钮使用可换色 PNG 符号，颜色由主题 JSON 控制。
- 子窗口支持置顶，置顶窗口按置顶先后顺序排列层级。
- 每个图表拥有独立的缩放、平移、可见范围和十字光标状态。
- 启动即全屏，初始工作区不创建图表窗。
- 单实例运行，重复启动不会打开第二个主程序。
- 多图表窗口、控制面板、源码窗、控制台窗、调色板和右键菜单。
- 工作区左上角固定显示 `RoundTable_32.ico`。
- 182 项像素级自动化测试，覆盖布局、线宽、字距、主题、符号、编辑器和交互。

## 运行环境

- Windows 10/11 x64。
- Rust 1.85 或更高版本，使用 2024 edition。
- MSVC 工具链。
- 支持 wgpu 的显卡驱动。

项目使用 `eframe` 的 `wgpu` 后端。若在虚拟机或远程桌面中出现渲染问题，请先更新显卡驱动或确认系统支持 DirectX 12/Vulkan。

## 快速开始

```powershell
git clone <你的仓库地址>
cd RoundTable
cargo run
```

调试构建会从仓库的 `assets/` 目录读取资源。Windows 下也可以生成不显示控制台窗口的发布版：

```powershell
cargo build --release
Copy-Item -Recurse -Force assets target\release\assets
.\target\release\RoundTable.exe
```

发布目录至少需要包含：

```text
RoundTable.exe
assets\
  cursors\
  icons\
  logo\
  themes\
```

资源目录优先在可执行文件旁查找 `assets/`；找不到时会回退到编译时记录的仓库路径。因此源码运行无需复制资源，分发时必须随 exe 一起复制 `assets/`。

## 操作

- 鼠标滚轮：围绕指针位置缩放 K 线。
- 鼠标左键拖动图表：横向平移。
- 十字光标：悬停图表查看索引、价格和时间标签。
- 图表标题栏：可切换指标、叠加均线/EMA/BOLL、打开源码。
- 子窗口标题栏：置顶、最小化、最大化/还原、关闭；置顶按钮位于最小化左侧。
- 控制面板：重新生成行情、修改涨跌色、缩放、切换主题、UI 网格测试、新增图表和打开控制台。
- 窗口标题栏：拖动、最小化、最大化、关闭。
- 窗口边缘：自由尺寸缩放，连续到 1px。
- 菜单栏：文件、数据、分析、布局、工具、帮助。
- 右键空白区域：图表面板快捷菜单。

主题按钮或控制台 `theme` 命令会在浅色与深色主题之间切换。切换时会重新读取对应的 JSON 文件。

## 默认资源

### 程序图标与工作区 Logo

应用暂不设置自定义程序图标，Windows 使用系统默认程序图标。窗口 Viewport 也不再传入 egui/eframe 的图标。

工作区左上角固定绘制：

```text
assets/logo/RoundTable_32.ico
```

该 Logo 以 32×32 像素硬边绘制，不随子窗口移动。其余 `RoundTable_*.ico` 可通过 `tools/make_app_icon.ps1` 系列工具维护。

### 界面符号

标题栏按钮符号位于：

```text
assets/symbols/
  minimize.png
  maximize.png
  restore.png
  close.png
  pin.png
  unpin.png
```

符号固定为 16×16 PNG。`#FF00FF` 表示透明，其他像素只作为蒙版；运行时统一使用主题字段 `window.symbol` 的颜色重绘，因此与文字一样跟随主题变化。关闭按钮与其他标题按钮共用按钮底色和悬停色，不再单独变红。

### 主题

默认主题：

```text
assets/themes/light.json
```

备用主题：

```text
assets/themes/dark.json
```

主题文件是运行时真源，不需要修改 Rust 代码。启动时加载浅色主题；切换主题时重新加载目标 JSON。若文件缺失、JSON 语法错误、字段缺失或颜色格式非法，程序会输出错误并回退到内置配色，不会阻止启动。

## 主题格式

主题文件必须包含以下顶层字段：

```json
{
  "format": "roundtable-theme",
  "version": 1,
  "id": "light",
  "name": "浅色",
  "colors": {
    "window": {},
    "chart": {},
    "indicators": {},
    "debug": {},
    "pointer": {}
  }
}
```

字段规则：

- `format` 固定为 `roundtable-theme`。
- `version` 当前固定为 `1`。
- `id` 必须与文件名一致：`light.json` 使用 `light`，`dark.json` 使用 `dark`。
- `name` 是给主题文件阅读者看的说明字段，不能为空；界面语言中的主题名仍由 `id` 映射。
- 所有颜色必须是 `#RRGGBB`。
- 所有字段都是必填项，未知字段会报错，避免拼写错误被静默忽略。

### `window`

| 字段 | 用途 |
| --- | --- |
| `title_bg` | 主窗口与子窗口标题栏底色 |
| `menu_bg` | 菜单栏底色 |
| `work_bg` | 子窗口内容区底色 |
| `desktop_bg` | 主窗口工作区桌面底色 |
| `status_bg` | 状态栏与读数区底色 |
| `panel_bg` | 控制面板、编辑器控件底色 |
| `splitter` | 分割条底色 |
| `scroll_track` | 滚动条轨道 |
| `scroll_thumb` | 滚动条滑块 |
| `btn_face` | 按钮常态底色 |
| `btn_hover` | 按钮悬停底色 |
| `btn_press` | 按钮按下与选中底色 |
| `sel_bg` | 文本选区底色 |
| `symbol` | 标题栏自定义符号颜色 |
| `ctrl_border` | 控件与窗口边框 |
| `line` | 区域分隔线 |
| `text` | 主文字色 |
| `text_white` | 反白文字色 |
| `syn_comment` | 源码注释文字色 |

### `chart`

| 字段 | 用途 |
| --- | --- |
| `chart_bg` | 图表绘图区底色 |
| `chart_line` | 图表框线与坐标线 |
| `axis_bg` | 坐标轴区域底色 |
| `tick` | 刻度文字色 |
| `cross` | 十字光标线 |
| `cross_label` | 十字光标标签底色 |
| `ind_zero` | 指标零轴 |
| `candle_up` | 上涨 K 线 |
| `candle_down` | 下跌 K 线 |
| `text_up` | 上涨数字 |
| `text_down` | 下跌数字 |

### `indicators`

| 字段 | 用途 |
| --- | --- |
| `dif` | MACD DIF |
| `dea` | MACD DEA |
| `kdj_k` | KDJ K |
| `kdj_d` | KDJ D |
| `kdj_j` | KDJ J |
| `rsi_1` | RSI 1 |
| `rsi_2` | RSI 2 |
| `rsi_3` | RSI 3 |
| `boll_mid` | BOLL 中轨 |
| `boll_up` | BOLL 上轨 |
| `boll_down` | BOLL 下轨 |
| `obv` | OBV 线 |
| `cci` | CCI 线 |
| `bias` | BIAS 三线，固定 3 个颜色 |
| `ma` | MA 六线，固定 6 个颜色 |
| `ema` | EMA 六线，固定 6 个颜色 |

数组示例：

```json
{
  "bias": ["#3333CC", "#CC9900", "#FF00FF"],
  "ma": ["#3333CC", "#CC0066", "#009933", "#6600CC", "#009999", "#666666"],
  "ema": ["#FF00FF", "#FF8800", "#00CCCC", "#88AA00", "#8888FF", "#AA00AA"]
}
```

### `debug` 与 `pointer`

| 字段 | 用途 |
| --- | --- |
| `debug.grid_ok` | UI 网格测试中对齐元素框 |
| `debug.grid_bad` | UI 网格测试中未对齐元素框 |
| `debug.grid_cursor` | UI 网格测试中鼠标格与坐标标记 |
| `pointer.pointer_trail` | 软件指针轨迹 |
| `pointer.pointer_trail_hot` | 加速状态下的指针轨迹 |

### 修改主题

1. 编辑 `assets/themes/light.json` 或 `dark.json`。
2. 保持 `id`、`format`、`version` 与文件名一致。
3. 保存文件。
4. 重启程序，或点击“切换主题”往返一次以重新加载。

例如把浅色标题栏改为 `#AABBCC`：

```json
"title_bg": "#AABBCC"
```

## ECS 架构

核心目录：

```text
src/ecs/
  mod.rs
  entity.rs
  component.rs
  components.rs
  world.rs
  systems.rs
```

### Entity

`Entity` 是轻量、可复制、可比较的稳定标识。每个图表面板实体都拥有独立 ID，组件表按实体顺序保存对应组件。

### Component

当前核心组件包括：

- `Pane`：指标对象、绘图指令缓存、绘图区高度、脏标记。
- `Crosshair`：十字光标数值、序号、浮点位置与所属面板。
- `ChartView`：单个图表自己的缩放、偏移、可见根数、平移、滚轮余量和十字光标。
- `MarketBars`：行情序列资源，提供只读切片访问与整体替换。

`ComponentTable<T>` 将实体 ID 与组件值分开保存，并通过 `Deref`、索引和迭代器提供低开销访问。UI 仍可按下标读取面板，但面板的实体身份由 ECS 管理。

### Resource

`AppWorld` 保存跨实体共享的核心资源：

- 行情种子。
- 当前指标调色板。
- 图表列高度与叠加配置。
- 用户自定义涨跌色及“是否跟随主题”状态。

### System

系统函数集中处理数据流转：

- `mark_panes_dirty`：数据、指标或配色变化后统一置脏。
- `ensure_pane_computed`：只对脏面板执行指标计算并更新绘图指令。
- `regenerate_market`：替换行情资源并让全部面板重算。

`StockApp` 通过 `Deref`/`DerefMut` 借用 `AppWorld`。界面调度仍留在应用层，行情、面板和分析结果由 ECS 核心统一持有。`ChartView` 挂在各自的面板实体上，因此图表之间不会共享缩放、平移或十字光标。

## 代码结构

```text
src/
  main.rs             启动、窗口参数、崩溃日志
  app.rs              应用状态组装、帧调度、全局绘制入口
  assets.rs           图标/字体外资源路径解析
  theme.rs            内置回退主题、JSON 主题加载与校验
  grid.rs             9px 网格、1px 硬边与布局工具
  font.rs             宋体/Fixedsys 点阵字栅格化
  pointer.rs          软件指针运动模型与轨迹
  platform.rs         Windows/非 Windows 平台门面
  win32.rs            Win32 窗口、裁剪与系统能力
  tests.rs            像素级单元与集成测试
  ecs/
    entity.rs         实体标识与分配器
    component.rs      通用组件表
    components.rs     核心组件
    world.rs          ECS 世界与共享资源
    systems.rs        面板计算与行情更新系统
  data/
    random_walk.rs    行情模型与随机游走生成器
  indicator/
    mod.rs            指标接口、调色板、绘图指令
    candle.rs         蜡烛图
    bars.rs           美国线
    volume.rs         成交量
    macd.rs            MACD
    kdj.rs            KDJ
    rsi.rs            RSI
    boll.rs           BOLL
    obv.rs            OBV
    cci.rs            CCI
    bias.rs           BIAS
  ui/
    chart.rs          图表绘制、缩放、平移与十字光标
    winmgr.rs         子窗口管理器
    floatwin.rs       浮动窗口外壳与拖动/缩放
    panel.rs          控制面板
    menu.rs           菜单栏与多级下拉菜单
    editor.rs         源码编辑器
    source.rs         源码窗口与编译动作
    console.rs        调试控制台
    colorpanel.rs     RGB/HSV 调色板
    button.rs         文本按钮 / 图标按钮
    symbol.rs         16px 自定义符号加载与主题着色
    textbox.rs        单行输入框
    textbuf.rs        可编辑文本缓冲
    icon.rs           16px 工具栏图标、光标资源和 ICO 解码
assets/
  cursors/            软件指针位图
  icons/              工具栏 16x16 图标
  logo/               工作区 32px Logo 与多尺寸 ICO
  symbols/            标题按钮自定义 PNG 符号
  themes/             浅色/深色主题 JSON
```

`main.rs` 只负责程序启动与崩溃输出；大规模 UI 像素测试已迁入 `src/tests.rs`，避免启动入口被测试辅助代码淹没。

## 绘制约束

这些约束是项目代码和测试共同维护的硬规则：

- 静态控件宽高使用 `cells(n) = 9n + 1`。
- 静态骨架、控件和文字落点对齐 9px 网格。
- 子窗口位置、子窗口尺寸、滑块、滚动条和图表绘图区允许 1px 连续移动。
- 每块子窗口拥有自己的坐标原点。
- 靠右和靠下的元素按固定像素跟随窗口边缘，不重新吸附网格。
- 相邻控件共用 1px 线，不允许 2px 叠线。
- 自由尺寸余量进入弹性段，使用 `free_band`、`free_tail` 等入口处理。
- 只使用 1px 横线和竖线，不使用圆角、阴影、渐变、透明混色和抗锯齿。
- 区域分割线在内容之后补画，避免被内容覆盖。
- 下拉菜单限制在当前窗口矩形内，并绘制在该窗口最上层。
- 悬停提示统一记录到 `StockApp::tooltip`，在整帧最后绘制。
- 所有文字通过 `StockApp::text` 绘制，横向落点只使用 `text_left_x` 与 `text_right_x`。

## 调试控制台

通过控制面板“打开调试控制台”进入命令输入。常用命令：

| 命令 | 说明 |
| --- | --- |
| `help` / `?` | 显示命令列表 |
| `status` | 输出运行状态 |
| `fps` | 输出帧率与帧耗时 |
| `data` | 输出行情数量、种子、缩放和偏移 |
| `windows` | 列出当前窗口 |
| `regen` | 重新生成随机行情 |
| `theme` | 切换主题并重新读取 JSON |
| `grid on` / `grid off` | 开关 UI 网格测试 |
| `pointer` | 输出软件指针状态 |
| `speed 1.0` | 设置软件指针速度 |
| `accel 0.1` | 设置软件指针加速度 |
| `trail on` / `trail off` | 开关指针轨迹 |
| `echo 文本` | 原样输出文本 |
| `clear` / `cls` | 清空日志 |
| `close` / `quit` | 关闭控制台窗口 |

## 开发与验证

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

测试包含：

- 9px 网格、1px 线宽与文字落点。
- 子窗口拖动、缩放、最小化与最大化的像素行为。
- K 线、成交量、指标计算和缓存脏标记。
- 主题 JSON 解析、颜色覆盖与非法文件回退。
- ICO 尺寸、Logo 像素一致性。
- 编辑器文本缓冲、输入法、选区与滚动条。
- 菜单、调色板、控制台、指针和十字光标交互。
- 源码字符集限制。

部分渲染测试会在内存中做三角形栅格化，因此全量测试耗时高于普通逻辑测试。

## 扩展指南

### 新增指标

1. 在 `src/indicator/` 新建指标文件并实现 `Indicator`。
2. 在 `src/indicator/mod.rs` 注册到 `builtin()` 和需要的 `default_layout()`。
3. 若需要专属颜色，扩展 `Palette`、`Theme` 与主题 JSON，并同步更新 `version` 规则和文档。
4. 增加指标数值与绘图指令测试。

### 新增核心数据

优先判断它属于实体组件还是世界资源：

- 每个实体各有一份：新增组件类型并放入 `ComponentTable<T>`。
- 全局共享且需要跨系统读取：放入 `AppWorld`。
- 每次数据换代都需要重算图表：通过 `mark_panes_dirty` 或新系统统一处理。

## 发布检查

上传 GitHub 前建议执行：

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

发布压缩包应包含 `RoundTable.exe`、完整 `assets/`、`README.md` 与 `LICENSE`。不要只上传 exe，否则主题、图标、光标和工具栏图标无法按预期加载。

## 许可证

本项目使用 MIT License，详见 [LICENSE](LICENSE)。
