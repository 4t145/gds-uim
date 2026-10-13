```rust
el! {
    List { // List::new()
        flex;
        flex_col;
        gap: 8.0.px;
        @condition: $|element| element.child("dynamic");
    } [
        Button with_label("confirm") {
            label: "确定";
        },
        ..rest,
    ]
}

el! {
    List(a, b) { // List::new(a, b)
        flex;
    }
}

el! {
    List() { // List::new()
        flex;
    }
}

el! {
    List []
}

el! {
    List
}
```

通过 `use uim::el;` 导入宏。元素、函数和扩展 trait 由调用方提供；使用
GPUI 时，通常需要导入 `gpui::prelude::*`。宏本身没有 GPUI 依赖。

属性用分号 `;` 分隔，同一属性的参数用逗号 `,` 分隔。子元素和构造器参数
也用逗号分隔。最后一个属性的分号、最后一个参数或子元素的逗号均可省略。

第一个例子等价于：

```rust
List::new()
    .flex()
    .flex_col()
    .gap(px(8.0))
    .when(condition, |element| element.child("dynamic"))
    .child(Button::with_label("confirm").label("确定"))
    .children(rest)
```

`{属性}` 和 `[子元素]` 均可省略或为空。每个阶段先应用属性，再添加子元素，
然后才应用后缀包装。各阶段、参数和子元素按书写顺序求值，每个值只求值一次；
构建方法和包装函数可以改变返回类型。

元素类型支持 `widgets::List` 这样的路径。`List`、`List()` 调用无参数
`List::new()`；`List(a, b)` 调用 `List::new(a, b)`；
`Button with_label("confirm")` 调用指定构造器。

`#` 表示工厂函数调用：`#div` 和 `#div()` 均调用 `div()`，`#img(source)`
调用 `img(source)`。支持模块路径和泛型，例如 `#gpui::div`、
`#make::<T>(value)`。工厂节点后面也可以接属性和子元素。

```rust
use gpui::{div, prelude::*, px};
use uim::el;

let show_details = true;
let existing = div().child("已有元素");
let rest = vec![div(), div()];

let element = el! {
    #div {
        flex;
        flex_col;
        gap: 8.0.px;
        @show_details: $|element| element.child("详情");
    } [
        "标题",
        $existing,
        #div { flex; } ["嵌套内容"],
        ..rest,
    ]
};
```

值默认使用 DSL 语法。字面量、变量名和模块路径可以直接写；`value.function`
将左侧值传给右侧一元函数，连续后缀从左到右执行。这里的 `.` 表示函数应用。

| 写法 | 含义 |
| --- | --- |
| `6.0`、`"标题"`、`true` | 字面量 |
| `spacing`、`MouseButton::Left` | 变量或模块路径 |
| `6.0.px` | `px(6.0)` |
| `spacing.px` | `px(spacing)` |
| `a.b` | `b(a)` |
| `value.f.g` | `g(f(value))` |
| `value.gpui::px` | `gpui::px(value)` |
| `value.convert::<T>` | `convert::<T>(value)` |

Rust 表达式由 `$` 引入。`$a.b` 是 Rust 字段访问；`$calculate()` 是 Rust
调用；`$move |event, window, cx| { ... }` 中的整个闭包都是 Rust，闭包内部不再
使用 DSL。运算、元组、代码块、索引、宏调用、条件表达式同样需要 `$`。

`$expression` 持续到下一个参数或子元素分隔符。`$(expression)` 将括号中的
内容完整交给 Rust，然后返回 DSL；需要继续接后缀函数时使用这个明确边界：

```rust
el! {
    #div {
        gap: $(theme.spacing).px;
        w: $(width + 8.0).px;
        bg: $theme.surface;
    }
}
```

`$(...)` 的括号限定 Rust 表达式的范围。例如，完整的 Rust 方法链写
`$(items[index].clone())`，而 `$(items[index]).convert` 调用自由函数
`convert(items[index])`。不带 `$` 的括号只用于 DSL 分组。

属性多参数写成 `method: a, b;`。`method(a, b);` 形式也支持，参数使用相同的
值规则。元组作为一个参数时需要 `$(a, b)`：

```rust
el! {
    #div {
        id: "button";
        size: 40.0.px;
        on_mouse_down: MouseButton::Left, $move |event, window, cx| {
            // Rust 回调，参数列表中的逗号不会拆成属性参数。
        };
        on_click: $move |event, window, cx| {
            // Rust 回调。
        };
    }
}
```

`@condition: builder;` 调用 `.when(condition, builder)`。条件和 builder 使用
相同的值规则；条件为假时不会执行 builder。复杂条件以 `$` 开头：

```rust
el! {
    List {
        @$state.show_details && settings::ENABLED:
            $|list| list.child("详情");
        when_else: enabled,
            $|list| list.flex(),
            $|list| list.flex_col();
    }
}
```

元素后缀也使用一元函数应用。`#div.deferred` 调用 `deferred(div())`；
有属性和子元素时，将后缀写在完整元素之后。后缀后的属性配置包装结果：

```rust
use gpui::{deferred, div, prelude::*, px};
use uim::el;

let element = el! {
    #div { gap: 6.0.px; } [
        "内容",
    ].deferred {
        priority: 1;
    }
};
```

这等价于 `deferred(div().gap(px(6.0)).child("内容")).priority(1)`。
`element.first.second` 依次调用 `second(first(element))`。
后缀仅接受一元函数路径，不支持占位参数或额外实参。

子元素列表中的裸类型名 `Button` 表示 `Button::new()`；`#render_item` 调用
工厂，`$existing` 插入已有元素。字符串可以直接写；Rust 代码块、`if`、
宏调用等以 `$` 引入。`..items` 展开可迭代值，也可以使用 Rust 表达式或 DSL
转换函数：

```rust
el! {
    #div [
        $existing,
        $(if enabled { div() } else { div().hidden() }),
        $concat!("标", "题"),
        ..rest,
        ..$items.into_iter().map(render_item),
    ]
}
```

构造器参数也使用相同的值规则，Rust 回调同样需要 `$`。
例如 `#img($self.image_path.clone())`、`List(1.increment, $(a + b))`，
或 `#deferred(#div ["内容"])`。函数路径中的泛型参数仍使用 Rust 类型语法。

内置元素也有快捷宏。`div! { flex; gap: 6.0.px; }` 等价于
`el! { #gpui::div { flex; gap: 6.0.px; } }`。快捷宏固定使用 GPUI 工厂路径，
不需要导入同名函数；样式 trait 和 `px` 等值转换函数仍由调用方导入。

| 快捷宏 | 参数形式 |
| --- | --- |
| `div!`、`svg!`、`anchored!` | 直接写属性，或 `{属性} [子元素]` |
| `img!`、`canvas!`、`deferred!`、`image_cache!`、`list!`、`uniform_list!`、`surface!` | 直接传参数；带属性时写 `(参数) {属性} [子元素]` |

```rust
use gpui::{prelude::*, px};
use uim::{div, img};

let image = img! { ("images/avatar.png") { size: 32.0.px; } };
let element = div! {
    { flex; flex_col; gap: 8.0.px; }
    [
        $div! { child: "标题"; },
        $image,
    ]
};
```

快捷宏也使用相同的 DSL 值与 `$` 边界。将快捷宏调用嵌入 `el!` 或其他快捷
宏的子元素列表时，写成 `$div! { ... }`。有泛型参数的工厂可以使用
`canvas! { ::<Pixels>(参数) {属性} }`。

用于渲染的元素树应在 GPUI 的 `Render::render` 或 `RenderOnce::render` 中
构建，让子元素的帧内分配和事件处理器归属正确的视图。

`examples/clicker.rs` 使用 `div!` 和 `el!` 构建加一、重置按钮和计数显示：

```sh
cargo run --locked --example clicker
```

验证使用 GPUI 0.2.2 的 `test-support` 开发依赖：

```sh
cargo test --locked
```

`tests/gpui.rs` 通过真实 GPUI 测试窗口执行视图布局、绘制和事件派发，覆盖
构造器、DSL 单位值、嵌套与展开子元素、条件变化后的重绘、原生文本、多参数
鼠标回调，以及后缀 `Deferred` 的布局和绘制优先级。
库中的 GPUI 使用示例也作为文档测试编译并运行。
clicker 的测试直接点击示例中的按钮，验证计数变化和窗口标题重绘。

内部解析器在 `src/macros` 中按元素、属性、值、子元素和完成回调拆成独立宏。
属性与子宏通过局部构建器绑定独立展开，避免将整个嵌套树串成一次长递归；
clicker 在 Rust 默认递归上限下编译，不需要额外设置 `recursion_limit`。

`Cargo.lock` 使用兼容的 `libc` 0.2.186；GPUI 0.2.2 间接依赖的旧版 `xattr`
引用了新版本 `libc` 已移除的常量。Linux 构建需要 GPUI 的系统开发库，包含
`libxkbcommon-x11-dev`。
