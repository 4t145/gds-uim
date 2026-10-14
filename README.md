# gds-uim

Cargo 包名为 `gds-uim`，Rust 中使用 `gds_uim` 导入。

为 GPUI 提供声明式宏：`el!` 构建元素树，`m!`（map）用相同的属性语法
构建通用构建器闭包。元素通过 `#div`、`#svg`、`#img(...)` 等工厂语法创建。

```rust
use gpui::{div, prelude::*, px};
use gds_uim::el;

let element = el! {
    #div { flex; flex_col; gap: 8.0.px; } [
        "Hello",
        #div ["GPUI"],
    ]
};
```

属性用 `;` 分隔，同一属性的参数用 `,` 分隔。`value.function` 调用
`function(value)`，Rust 表达式通过 `$` 引入，例如 `$theme.surface`、
`$move |event, window, cx| { ... }`。

在元素和子元素位置，顶层 `{...}` / `[...]` 是 DSL 属性块和子元素列表；
Rust 的 `match`、`if`、代码块、数组与索引请用 `$(...)` 明确边界，例如
`#div [$(match selected { true => "yes", false => "no" }), $(labels[0])]`。
括号之后仍可继续配置 DSL 属性、子元素和后缀。

`m!` 可以用于 `hover`、`active`、`focus` 等样式回调，也可以传给 `when`、
`map` 来修改元素。属性块里的 `method => { ... }` 等价于
`method: $m! { ... }`，可以嵌套使用：

```rust
use gpui::{div, prelude::*, rgb};
use gds_uim::{el, m};

let element = el! {
    #div {
        id: "button";
        hover => { bg: 0x818cf8.rgb; text_color: 0xffffff.rgb };
        active => { bg: 0x6366f1.rgb };
    }
};
let padded = div().map(m! { p_4; rounded_lg });
```

`=>` 写法无需单独导入 `m`。需要转移捕获变量所有权时，可用
`method => move { ... }` 或 `m!(move { ... })`。

条件也支持简写：`@selected => { border_color: ACCENT.rgb };` 等价于
`@selected: $m! { border_color: ACCENT.rgb };`，支持 `=> move` 和嵌套条件。
这里的 `ACCENT.rgb` 是 DSL 转换；`$ACCENT.rgb` 会被解析为 Rust 字段访问。

完整元素之后也可以接 `.m! { ... }`，然后继续接 `.deferred` 等后缀：

```rust
use gpui::{deferred, div, prelude::*, rgb};
use gds_uim::{el, m};

let element = el! {
    #div { id: "reset"; } ["Reset"]
        .m! { rounded_lg; hover => { bg: 0x475569.rgb } }
        .deferred
};
```

运行 [clicker 示例](examples/clicker.rs)，点击加一或重置：

```sh
cargo run --locked --example clicker
```

更完整的 [task_board 示例](examples/task_board.rs) 包含任务列表、详情面板、
统计卡片和进度条。可以按状态筛选、只看优先任务、完成或重新打开任务，
并修改优先级；空结果和选中项会同步更新。数据保存在内存中，可通过
“Reset demo” 恢复初始状态。

```sh
cargo run --locked --example task_board
```

验证：

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

完整语法见 [docs/grammer.md](docs/grammer.md)。解析器按职责拆分在
[src/macros](src/macros)，clicker 使用 Rust 默认递归上限。
库没有运行时依赖；元素工厂、构建器 trait 和值转换函数由调用方提供。
属性与子树独立展开，完整节点跳过多余的后缀扫描；32 层带属性元素树有默认递归上限回归测试，
展开结构与求值顺序约束见 [docs/expansion.md](docs/expansion.md)。
