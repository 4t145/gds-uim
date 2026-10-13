# uim

为 GPUI 提供声明式元素宏：`el!` 构建元素树，`div!`、`svg!`、`img!` 等
10 种快捷宏调用内置元素工厂。

```rust
use gpui::{prelude::*, px};
use uim::div;

let element = div! {
    { flex; flex_col; gap: 8.0.px; }
    [
        "Hello",
        $div! { child: "GPUI"; },
    ]
};
```

属性用 `;` 分隔，同一属性的参数用 `,` 分隔。`value.function` 调用
`function(value)`，Rust 表达式通过 `$` 引入，例如 `$theme.surface`、
`$move |event, window, cx| { ... }`。

运行 [clicker 示例](examples/clicker.rs)，点击加一或重置：

```sh
cargo run --locked --example clicker
```

验证：

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

完整语法见 [docs/grammer.md](docs/grammer.md)。解析器按职责拆分在
[src/macros](src/macros)，clicker 使用 Rust 默认递归上限。
库没有运行时依赖；快捷宏的调用方需要 GPUI 依赖和相关扩展 trait。
