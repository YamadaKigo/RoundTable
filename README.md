<p align="center">
  <img src="assets/logo/RoundTable_64.ico" width="64" height="64" alt="RoundTable logo">
</p>

<h1 align="center">RoundTable 圆桌</h1>

<p align="center">一个坚持 1 逻辑点 = 1 物理像素、界面全部由 Painter 手绘的 Windows 行情图表工作台。</p>

RoundTable 使用 Rust、eframe、egui 与 wgpu 构建。窗口、菜单、按钮、输入框、滚动条、图表和指针都由纯色矩形与 1px 线条逐像素绘制，不使用圆角、阴影、渐变、透明混色或抗锯齿。

项目当前包含随机游走行情、K 线/美国线、成交量、MACD、KDJ、RSI、BOLL、OBV、CCI、BIAS、指标代码编辑、浮动子窗口和调试控制台。核心数据由轻量 ECS 组织，主题、指标脚本、界面符号和位图资源都放在可替换的资源目录中。

## 当前特性

- 无边框全屏主窗口，客户区铺满窗口。
- 强制 1 逻辑点等于 1 物理像素。
- 9px 界面节距：`GRID = 8px`、`LINE_W = 1px`、`PITCH = 9px`。
- 默认打开深色主题，可切换到浅色主题。
- 中文使用宋体 16px 点阵，ASCII 使用 Fixedsys 8x16 点阵，字形保持 1bpp 硬边。
- 轻量 ECS：实体、组件表、世界资源与系统分层。
- 每个图表独立保存缩放、平移、可见范围和十字光标状态。
- 鼠标滚轮围绕指针位置缩放，左键拖动图表横向平移。
- 子窗口支持拖动、自由尺寸缩放、置顶、最小化、最大化和关闭。
- 控制面板、指标代码窗和调试控制台默认置顶。
- 非焦点子窗口禁用鼠标和键盘输入，悬停可见区域后转移焦点。
- 指标使用通达信风格公式代码，编辑后立即解析并刷新图表。
- 用户指标保存为独立副本，不覆盖 `assets/scripts` 中的系统脚本。
- 单实例运行，重复启动不会打开第二个主程序。
- 像素级自动化测试覆盖布局、字距、主题、滚动条、脚本、焦点和交互。

## 运行环境

- Windows 10/11 x64。
- Rust 1.85 或更高版本，使用 2024 edition。
- MSVC 工具链。
- 支持 wgpu 的显卡驱动。

项目使用 eframe 的 wgpu 后端。若在虚拟机或远程桌面中出现渲染问题，请先更新显卡驱动，或确认系统支持 DirectX 12/Vulkan。

## 快速开始

```powershell
git clone <你的仓库地址>
cd RoundTable
cargo run
```

调试构建会从仓库的 `assets/` 目录读取资源。发布版可以这样构建：

```powershell
cargo build --release
Copy-Item -Recurse -Force assets target\release\assets
.\target\release\RoundTable.exe
```

资源目录优先在可执行文件旁查找 `assets/`；找不到时会回退到编译时记录的仓库路径。源码运行无需复制资源，分发时必须随 exe 一起复制完整的 `assets/`。

## 界面与操作

### 主窗口

- 启动后直接进入全屏，初始工作区默认只显示控制面板，不创建图表窗。
- 窗口移动、子窗口拖动和边缘缩放都由应用逐帧处理。
- 菜单栏包含文件、数据、分析、布局、工具、帮助。布局菜单可打开控制面板或新增图表；其余菜单项目前作为界面占位，点击结果写入状态栏。
- 右键工作区空白处可新增图表、打开调试控制台、切换主题，或打开编辑器快捷菜单。

### 图表

- 鼠标滚轮：围绕指针位置缩放 K 线。
- 鼠标左键拖动：横向平移。
- 悬停在图表上：显示十字光标，以及索引、价格和时间标签。
- 图表标题栏：切换指标、打开当前图表的指标代码。
- 图表底部状态栏：显示缩放、平移和可见根数。

### 子窗口

- 拖动标题栏移动窗口。
- 拖动左、右、下边缘或右下、左下角连续缩放，精度为 1px。
- 标题栏按钮依次为置顶、最小化、最大化/还原、关闭。
- 颜色面板支持置顶、最小化和关闭，不提供最大化。
- 下拉菜单绘制在当前窗口最上层，并限制在该窗口矩形内。

### 控制面板

控制面板默认置顶，当前按钮为：

| 按钮 | 作用 |
| --- | --- |
| 重新生成 | 更换种子，重新生成一段随机行情 |
| 设置涨色 | 打开调色板修改上涨色 |
| 设置跌色 | 打开调色板修改下跌色 |
| 切换鼠标控制 | 在软件鼠标与系统鼠标之间切换 |
| 鼠标轨迹 | 显示最近一段轨迹、速度、加速度和坐标 |
| 重置视图 | 把当前图表缩放和平移恢复初始值 |
| 切换主题 | 在深色与浅色主题之间切换 |
| 图表配色跟随系统 | 让涨跌色重新跟随当前主题 |
| 新增图表 | 新增图表窗，最多 8 块 |
| 打开调试控制台 | 打开运行状态、日志和命令输入窗口 |
| 打印当前状态 | 把图表、窗口、主题和视图状态写入状态栏 |

图表缩放只通过鼠标滚轮完成。控制面板不提供放大、缩小、鼠标速度加减或鼠标加速度加减按钮。

### 调色板

- RGB 与 HSV 色块、输入框和滑块都由 Painter 手绘。
- 可拖动滑块把手，也可直接点击轨道定位。
- 应用后立即刷新对应图表的涨色或跌色。

## 主题

默认主题：

```text
assets/themes/dark.json
```

备用主题：

```text
assets/themes/light.json
```

主题文件是运行时真源。启动时加载深色主题；通过控制面板按钮或调试控制台 `theme` 命令切换时，会重新读取目标 JSON。文件缺失、JSON 语法错误、字段缺失或颜色格式非法时，程序会输出错误并回退到内置配色，不会阻止启动。

### 主题文件格式

主题文件必须包含以下顶层字段：

```json
{
  "format": "roundtable-theme",
  "version": 1,
  "id": "dark",
  "name": "深色",
  "colors": {
    "window": {},
    "chart": {},
    "indicators": {},
    "pointer": {}
  }
}
```

字段规则：

- `format` 固定为 `roundtable-theme`。
- `version` 当前固定为 `1`。
- `id` 必须与文件名一致：`light.json` 使用 `light`，`dark.json` 使用 `dark`。
- `name` 是给主题文件阅读者看的说明字段，不能为空。
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
| `syn_comment` | 指标代码注释文字色 |

### `chart`

| 字段 | 用途 |
| --- | --- |
| `chart_bg` | 图表绘图区底色 |
| `chart_line` | 图表框线与坐标线 |
| `axis_bg` | 坐标轴区域底色 |
| `tick` | 刻度色 |
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

### `pointer`

| 字段 | 用途 |
| --- | --- |
| `pointer_trail` | 软件指针轨迹 |
| `pointer_trail_hot` | 快速移动时的指针轨迹 |

修改主题后重启程序，或点击“切换主题”往返一次，让程序重新读取 JSON。

## 指标代码

每个内置指标对应一份运行时可编辑脚本：

```text
assets/scripts/
  candle.rti
  bars.rti
  volume.rti
  macd.rti
  kdj.rti
  rsi.rti
  boll.rti
  obv.rti
  cci.rti
  bias.rti
```

图表标题栏的“代码”按钮打开当前图表的指标代码。编辑器修改会实时解析并应用到当前图表，不执行 `cargo check` 或编译命令。语法错误时保留上一版图形，并在状态栏显示错误原因。

系统指标位于 `assets/scripts`。编辑器保存时不会修改这些文件，而是写入：

```text
userdata/indicators/paneN_xxx.rti
```

`userdata` 用于存放用户自定义内容，已加入 `.gitignore`。打开指标代码时优先加载该面板已有的用户副本；点击“还原原始指标代码”可重新读取系统脚本。

### 基本变量

| 变量 | 含义 |
| --- | --- |
| `O` | 开盘价序列 |
| `H` | 最高价序列 |
| `L` | 最低价序列 |
| `C` | 收盘价序列 |
| `V` | 成交量序列 |

### 函数

| 函数 | 用途 |
| --- | --- |
| `MA(X,N)` | N 周期简单平均 |
| `EMA(X,N)` | N 周期指数平均 |
| `REF(X,N)` | 向前引用 N 根 |
| `SUM(X,N)` | N 周期求和；`N=0` 表示累计 |
| `STD(X,N)` | N 周期总体标准差 |
| `HHV(X,N)` / `LLV(X,N)` | N 周期最高 / 最低 |
| `MAX(A,B)` / `MIN(A,B)` | 逐根取最大 / 最小 |
| `ABS(X)` | 绝对值 |
| `IF(条件,真值,假值)` | 条件选择 |
| `CROSS(A,B)` | A 上穿 B |

### 绘图命令

| 命令 | 用途 |
| --- | --- |
| `CANDLE()` | 绘制当前 K 线蜡烛 |
| `OHLC()` | 绘制美国线 |
| `VOL()` | 绘制成交量柱 |
| `LINE(X,颜色)` | 绘制折线 |
| `HIST(X,颜色)` | 绘制以零轴为基线的柱 |
| `HLINE(X,颜色)` | 绘制水平线 |
| `ZERO()` | 绘制指标零轴 |

赋值使用通达信风格冒号：

```text
DIF: EMA(C,12)-EMA(C,26);
DEA: EMA(DIF,9);
HIST(DIF-DEA,AUTO);
LINE(DIF,DIF);
LINE(DEA,DEA);
ZERO();
```

颜色可使用 `AUTO`、`UP`、`DOWN`、`TEXT`、`ZERO`、`DIF`、`DEA`、`K`、`D`、`J`、`RSI1` 到 `RSI3`、`BOLL_MID`、`BOLL_UP`、`BOLL_DOWN`、`OBV`、`CCI`、`BIAS1` 到 `BIAS3`、`MA1` 到 `MA6`、`EMA1` 到 `EMA6`，也可使用 `0xRRGGBB`。编辑器支持 `#` 行注释。

## 调试控制台

通过控制面板“打开调试控制台”进入命令输入。常用命令：

| 命令 | 说明 |
| --- | --- |
| `help` / `?` | 显示命令列表 |
| `status` | 输出运行状态快照 |
| `fps` | 输出帧率与帧耗时 |
| `data` | 输出行情数量、种子、缩放和偏移 |
| `windows` | 列出当前窗口 |
| `regen` | 重新生成随机行情 |
| `theme` | 切换主题并重新读取 JSON |
| `pointer` | 输出软件指针位置和速度 |
| `trail on` / `trail off` | 开关指针轨迹 |
| `echo 文本` | 原样输出文本 |
| `clear` / `cls` | 清空日志 |
| `close` / `quit` | 关闭控制台窗口 |

日志区支持滚轮和右侧垂直滚动条。滚动条滑块到达底部时与页脚共用同一条 1px 边框线，不叠加成 2px。

## 代码结构

```text
build.rs              可选宋体 / Fixedsys 字节嵌入
src/
  main.rs             启动、窗口参数、崩溃日志
  app.rs              应用状态组装、帧调度、全局绘制入口
  assets.rs           图标、字体和主题路径解析
  theme.rs            内置回退主题、JSON 主题加载与校验
  grid.rs             9px 网格、1px 硬边与布局工具
  font.rs             宋体 16px 与 Fixedsys 8x16 硬边字形
  pointer.rs          软件指针运动模型与轨迹
  script.rs           指标公式词法、语法解析与求值
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
    mod.rs            指标接口、调色板与绘图指令
    candle.rs         蜡烛图
    bars.rs           美国线
    volume.rs         成交量
    macd.rs           MACD
    kdj.rs            KDJ
    rsi.rs            RSI
    boll.rs           BOLL
    obv.rs            OBV
    cci.rs            CCI
    bias.rs           BIAS
  ui/
    chart.rs          图表绘制、缩放、平移与十字光标
    winmgr.rs         子窗口管理
    floatwin.rs       浮动窗口外壳与拖动/缩放
    panel.rs          控制面板
    menu.rs           菜单栏与多级下拉菜单
    editor.rs         指标代码编辑器和垂直滚动条
    source.rs         指标代码窗口、实时应用与保存
    console.rs        调试控制台与日志滚动条
    colorpanel.rs     RGB/HSV 调色板
    button.rs         文本按钮与图标按钮
    symbol.rs         16px 自定义符号加载与主题着色
    textbox.rs        单行输入框
    textbuf.rs        可编辑文本缓冲
    icon.rs           16px 工具栏图标、光标和 PNG 解码
    pointerdebug.rs   软件指针轨迹与读数叠层
assets/
  cursors/            软件指针位图
  fonts/              可选的内嵌宋体与 Fixedsys 字体
  icons/              工具栏 16x16 图标
  scripts/            内置指标公式代码
  symbols/            标题按钮自定义 PNG 符号
  themes/             深色 / 浅色主题 JSON
userdata/
  indicators/         用户指标副本与自定义内容
```

## ECS 架构

### Entity

`Entity` 是轻量、可复制、可比较的稳定标识。每个图表面板实体都拥有独立 ID，组件表按实体顺序保存对应组件。

### Component

当前核心组件包括：

- `Pane`：指标对象、指标脚本、绘图指令缓存、绘图区高度和脏标记。
- `Crosshair`：十字光标数值、序号、浮点位置和所属面板。
- `ChartView`：单个图表自己的缩放、偏移、可见根数、平移、滚轮余量和十字光标。
- `MarketBars`：行情序列资源，提供只读切片访问与整体替换。

`ComponentTable<T>` 将实体 ID 与组件值分开保存，并通过 `Deref`、索引和迭代器提供低开销访问。

### Resource

`AppWorld` 保存跨实体共享的核心资源：行情种子、当前指标调色板、图表列高度、用户自定义涨跌色，以及“是否跟随主题”状态。

### System

- `mark_panes_dirty`：数据、指标或配色变化后统一置脏。
- `ensure_pane_computed`：只对脏面板执行指标计算并更新绘图指令。
- `regenerate_market`：替换行情资源并让全部面板重算。

`StockApp` 通过 `Deref`/`DerefMut` 借用 `AppWorld`。界面调度留在应用层，行情、面板和分析结果由 ECS 核心统一持有。

## 绘制约束

这些约束由代码和测试共同维护：

- 静态控件宽高使用 `cells(n) = 9n + 1`。
- 静态骨架、控件和文字落点对齐 9px 网格。
- 子窗口位置、子窗口尺寸、滑块、滚动条和图表绘图区允许按 1px 连续移动。
- 每块子窗口拥有自己的坐标原点，窗口内元素按该原点排版。
- 靠右和靠下的元素按固定像素跟随窗口边，不重新吸附网格。
- 相邻控件共用 1px 线，不允许出现 2px 叠线。
- 自由尺寸产生的余量进入弹性段，使用 `free_band` 与 `free_tail` 一类入口处理。
- 只使用 1px 横线和竖线，不使用圆角、阴影、渐变、透明混色和抗锯齿。
- 区域分割线在内容之后补画，避免被内容覆盖。
- 下拉菜单属于当前窗口最上层，整块限制在该窗口矩形内。
- 悬停提示统一记录到 `StockApp::tooltip`，在整帧最后绘制。
- 全局文字只能通过 `StockApp::text` 绘制，横向落点只使用 `text_left_x` 与 `text_right_x`。
- 用户界面文本只使用中文与 ASCII。

## 开发与验证

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

测试覆盖：

- 9px 网格、1px 线宽与文字落点。
- 子窗口拖动、缩放、最小化与最大化。
- K 线、成交量、指标计算和缓存脏标记。
- 主题 JSON 解析、颜色覆盖与非法文件回退。
- 控制台日志滚动条及底部 1px 边框。
- 颜色面板子窗口按钮与滑块点击定位。
- 编辑器文本缓冲、输入法、选区和滚动条。
- 菜单、调色板、控制台、指针和十字光标交互。
- 窗口悬停焦点、非焦点输入隔离与重叠窗口十字光标归属。
- 指标公式解析、函数求值、实时脚本替换与错误回退。
- 用户指标副本路径隔离与源码字符集限制。

部分渲染测试会在内存中做三角形栅格化，因此全量测试耗时高于普通逻辑测试。

## 扩展指南

### 新增指标

1. 在 `src/indicator/` 新建指标文件并实现 `Indicator`。
2. 在 `src/indicator/mod.rs` 注册到 `builtin()` 和需要的 `default_layout()`。
3. 若需要专属颜色，扩展 `Palette`、`Theme` 与主题 JSON，并同步更新文档。
4. 增加指标数值与绘图指令测试。

### 新增核心数据

先判断它属于实体组件还是世界资源：

- 每个实体各有一份：新增组件类型并放入 `ComponentTable<T>`。
- 全局共享且需要跨系统读取：放入 `AppWorld`。
- 每次数据换代都需要重算图表：通过 `mark_panes_dirty` 或新系统统一处理。

## 发布检查

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

发布包应包含 `RoundTable.exe`、完整 `assets/`、`README.md` 和 `LICENSE`。不要只上传 exe，否则主题、图标、光标和工具栏图标无法按预期加载。

`assets/fonts` 中的字体是可选的。若放入 `simsun.ttc` 或 `vgafix.fon`，构建脚本会在获得相应授权的前提下把它们打包进可执行文件；缺失时回退到系统已安装字体。

## 许可证

本项目使用 MIT License，详见 [LICENSE](LICENSE)。
