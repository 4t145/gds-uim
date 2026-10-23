# UIM 语法

`el!` 将表达式、属性和子元素组合成普通 Rust 构建器调用。元素、已有值、
构造器参数和属性参数使用同一套表达式语法。裸类型名不隐式调用 `Type::new()`。

```rust
use gds_uim::el;
use gpui::{div, prelude::*, px, rgb};

let selected = true;
let loading = false;
let element = el! {
    .div {
        # "root";
        flex;
        gap: 8.0.px;
        hover: { bg: 0x818cf8.rgb };
        when: selected, { border_1 };
    } [
        @if (loading)
            "加载中",
        @else
            .div { # "child"; w: 100.0.px } ["Hello"],
        @end,
    ]
};
```

`@` 专用于控制指令。属性调用统一写 `method: args`，闭包是普通参数值。
`=>` 专用于定义带显式参数的 UIM 闭包；match 分支用 `@case (...)`。

## 表达式与调用

`.` 表示将左侧结果传给右侧可调用值，连续调用从左到右结合。

| DSL | Rust 含义 |
| --- | --- |
| `value` | 已有值，不隐式调用构造器 |
| `.div`、`().div` | `div()` |
| `(a, b).f` | `f(a, b)` |
| `a.f`、`(a).f` | `f(a)` |
| `a.f.g` | `g(f(a))` |
| `1.0.px` | `px(1.0)` |
| `value.convert::<T>` | `convert::<T>(value)` |
| `value.$(make_handler())` | 先求 value，再生成 handler，最后调用 handler |
| `value.{ flex }` | 将 value 传给裸块生成的闭包 |
| `value.m! { flex }` | 将 value 传给宏生成的闭包 |

调用位置接受函数、关联函数、函数指针、闭包、控制表达式和返回可调用值的宏。
参数个数、可调用性与类型匹配交给 Rust 检查。单参数优先写
`"ok".Button::with_label`，多参数写 `(a, b).Widget::with_size`，无参数写 `.f`。
参数列表可以为空，末尾逗号可省略。

`(a, b).f` 是调用，`(a, b) => body` 是闭包定义，由右括号后的符号区分。
单独 `(a, b)` 不表示元组或 DSL 分组；Rust 元组写成 `$(a, b)`：

```text
(a, b).f    → f(a, b)
$(a, b).f   → f((a, b))
().f        → f()
$().f       → f(())
```

值支持 Rust 字面量、路径、宏调用、Rust 转义、`~value`、闭包和控制表达式。
路径包括模块路径、`::<...>` 泛型参数和 `<T as Trait>::item`。
负数字面量可以直接写，普通值位置的其他运算放进 Rust 转义。

数字遵循 Rust 词法：`1.px` 可以直接写；整数后接 `$`、`~`、`{` 等符号
开头的可调用值时，在点前留空格，例如 `1 .$(handler)`。
紧贴的 `1.$(...)` 会先被 Rust 读成浮点字面量 `1.`，失去 DSL 的点。

## Rust 表达式与宏

`$(...)` 将整个括号作为 Rust 括号表达式或元组解析，然后返回 DSL。
括号内支持字段访问、方法调用、运算、闭包、代码块、`if`、索引等。
也接受不带括号的 `$rust-expr`，这时从 `$` 后开始解析一个完整 Rust 表达式：

```text
theme.spacing             → spacing(theme)
$theme.spacing            → Rust 字段访问
$(theme.spacing)          → Rust 字段访问，然后返回 DSL
$(theme.spacing + 4.0).px  → px(theme.spacing + 4.0)
$move |event, window, cx| { /* Rust */ }
```

裸 `$` 后的字段、方法、索引、运算和闭包内部的逗号均按 Rust 规则解析，不作为
DSL 后缀或参数分隔符。要在 Rust 结果后接 DSL 函数、属性或 children，使用
`$(...)` 明确边界，特别是 `$value { ... }` 可能成为 Rust 结构体表达式，
`$value[...]` 会成为 Rust 索引。`$(value) { attrs } [children]` 没有这个歧义。
`$` 后立即跟圆括号时总是选择隔离形式；例如 `$(a).f` 是 DSL 调用 `f(a)`。

Rust 区域内部不解释 UIM；需要时显式嵌套 `el!` 或 `m!`。在 Rust 区域中，
花括号仍是普通 Rust 块。宏调用本身就是值，无需 `$`，支持三种括号及限定路径：

```rust
el! { .div [format!("count: {}", count)].gds_uim::m! { p: 8.0.px } }
```

宏后的第一对括号属于宏，后续 `{...}` 才是属性后缀。宏参数交给该宏处理，
不会扫描其中的 `@` 标记或 `=>`。

## 闭包

### 裸属性块

**值位置**的 `{ attrs }` 生成一个接收 builder 并返回配置结果的闭包；
`move { attrs }` 使用移动捕获，空块 `{}` 是返回输入的恒等闭包：

```text
{ flex; gap: 8.0.px } → |this| this.flex().gap(px(8.0))
move { child: label } → move |this| this.child(label)
```

因此 `hover`、`when`、`map` 都是普通方法调用，无需各自的宏语法：

```rust
el! {
    .div {
        hover: { bg: hover_color };
        when: enabled, move { child: label };
        map: { flex; gap: 8.0.px };
    }
}
```

**后缀位置**的 `value { attrs }` 仍直接展开为方法链。
`value.{ attrs }` 和 `value.m! { attrs }` 则创建并调用一个闭包。
简单配置得到相同结果，但后两者具有闭包作用域，会影响 Rust 转义内的
`return`、`?` 和捕获。解析不依赖空格或换行。

`m! { attrs }` 与裸属性闭包生成同样的代码，适合在普通 Rust 中使用；
`m!(move { attrs })` 使用移动捕获。UIM 内的裸闭包不需要导入 `m`。

### 显式参数

`=>` 左侧是参数，右侧是一个完整 UIM 表达式：

```text
item => expr         → |item| expr
(a, b) => expr       → |a, b| expr
() => expr           → || expr
move (a, b) => expr  → move |a, b| expr
```

单参数可以写标识符或 `_`，也可以使用圆括号。括号参数列表支持 Rust 参数
模式、可选类型注解和末尾逗号；不可反驳性等约束由 Rust 检查。
多参数与单个元组参数明确区分：

```text
(a, b) => expr       → |a, b| expr
((a, b),) => expr    → |(a, b)| expr
```

例如：

```rust
el! {
    .div {
        on_click: (event, window, cx) => (event, window, cx).handle_click;
    } [
        ..items { into_iter; map: item => .div [item] },
    ]
}

let configure = el! { (node: Div, width: Pixels) => node { w: width } };
```

类型通常由接收闭包的 API 推断；独立保存且在主体中调用方法的闭包可能需要
参数类型注解，这与普通 Rust 闭包一致。

`=>` 的主体包含后续属性、children 和函数后缀，直到当前层的分隔符、分支
标记或输入结束。嵌套箭头向右结合：`a => b => expr` 返回另一个闭包。
`(a, b) => { flex }` 同样表示返回一个裸属性闭包；不会隐式选择 a 或 b。
配置某个参数时明确写 `(node, width) => node { w: width }`。

## 属性与克隆

属性之间使用 `;`，最后一个分号可以省略，换行不作为分隔符。

| 属性 | 展开 |
| --- | --- |
| `flex`、`flex:` | `.flex()` |
| `set_pair: a, b` | `.set_pair(a, b)` |
| `#value` | `.id(value)` |
| `#? expr` | `.debug_selector(|| expr)` |
| `#? move expr` | `.debug_selector(move || expr)` |
| `hover: { attrs }` | `.hover(闭包)` |
| `hover: move { attrs }` | `.hover(移动捕获的闭包)` |
| `when: condition, handler` | `.when(condition, handler)` |
| `when: condition, move { attrs }` | `.when(condition, 移动捕获的闭包)` |

`attr` 是 Rust 方法标识符，支持 `r#type` 等原始标识符。
`#value` 与 `id: value` 等价，只能放在属性块中。
Rust 2024 将紧贴的 `#"..."` 保留为特殊字符串语法，所以字符串 ID 必须写成
`# "id"`，在 `#` 后留空格。这是 Rust 词法限制，宏无法改变。

`#?` 为 `debug_selector` 自动补上零参数闭包，适用于 `el!` 和 `m!` 的属性块。
后面是完整的 UIM 表达式，可以带属性块、后缀和控制指令：

```rust
.div {
    # "root";
    #? "root" { into };
}
.div {
    # $("task", id);
    #? move format!("task-{id}");
}
```

`#? move expr` 等价于 `debug_selector: move () => expr`；表达式在闭包被调用时
求值，具体调用时机由 `debug_selector` 方法决定。它不隐式转换返回值；GPUI
要求返回 `String`，所以字符串字面量需要 `{ into }` 或 `.String::from`。
`#` 和 `#?` 独立设置 ID 与调试选择器，互不代替。

下面两种写法等价，分别展示函数后缀和方法属性块：

```text
#? "empty-state".String::from;
#? "empty-state" { into };
```

`~value` 展开成 `(value).clone()`，只作用于紧随其后的值，优先于 `.`：

```text
~label           → label.clone()
~state.render    → render(state.clone())
~$(state.field)  → (state.field).clone()
value.~handler   → 将 value 传给 handler.clone()
```

它不自动克隆闭包捕获的变量。`move { child: ~label }` 仍会先把 label 移入
闭包，在执行闭包时克隆。需要在创建闭包前克隆时，先用普通 Rust 绑定克隆结果。

`when: condition, handler` 是普通方法调用，handler 表达式本身会正常求值，
即使条件为假也会构造 handler。GPUI 的 `when` 只在条件为真时执行闭包主体。
闭包捕获发生在构造时，不会因为条件为假而自动保留被移入的变量。

## 控制指令

`@if (...)`、`@else if (...)`、`@else`、`@match (...)`、`@case (...)` 和
`@end` 是控制标记。`@end` 结束最近一层控制块。它们不依赖换行或缩进。
连续条件分支写 `@else if (...)`，当前不支持连写的 `@elseif`。

条件的括号内使用 Rust 条件语法，匹配值的括号内解析 Rust 表达式；case 括号内
解析 Rust 模式和可选 `if` 守卫。它们都不需要 `$`，可以直接使用字段、函数、
运算、解构和 `|` 模式。分支内容则继续使用 UIM。

`@if` 和 `@else if` 支持 `let pattern = expr`，也支持用 `&&` 连接的
let chains（调用方使用 Rust 2024 edition）。指令最外层的括号只标定条件边界，
不会给展开后的 Rust 条件额外加括号。模式绑定对后续 `&&` 条件和命中分支可见，
保留 Rust 的借用、移动和短路求值语义：

```rust
el! {
    .div [
        @if (let Some(label) = label && !label.is_empty())
            label,
        @else
            "暂无内容",
        @end,
    ]
}
```

### children 与属性片段

在 children 列表中，每个分支是零个或多个子元素：

```rust
el! {
    .div [
        "标题",
        @if (!a || !b)
            .div ["内容 A"],
            .div ["内容 B"],
        @else if (count > 0)
            "其他内容",
        @else
            "暂无内容",
        @end,
        @match (&result)
        @case (Ok(items) if items.is_empty())
            "空列表",
        @case (Ok(items))
            ..items { iter; map: render_item },
        @case (Err(error))
            .div [format!("加载失败：{error}")],
        @end,
        "页脚",
    ]
}
```

在属性块中，每个分支是零个或多个属性项，也允许嵌套控制块：

```rust
el! {
    .div {
        # "root";
        @if (active)
            bg: active_color;
            hover: { bg: hover_color };
        @else
            bg: normal_color;
        @end;
        flex;
    }
}
```

分支内部沿用所在位置的分隔符。整个控制块也占一个列表项：后面还有属性时，
`@end` 后写 `;`；后面还有 children 时写 `,`。最后一项和分支末项的分隔符可省略。
片段 `@if` 可以省略 `@else`，不满足条件时保持当前 builder 不变。

片段展开成对父 builder 的条件更新。分支里的子元素可以是不同类型，分支
最后返回的 builder 类型必须兼容；无 else 的 if 必须兼容原 builder 类型。
同一次控制块的各分支也可以一致地改变 builder 类型。

### 值位置

根表达式、调用参数和属性参数中的控制块产生一个值，每个分支恰好一个 UIM
表达式，分支表达式后直接接下一个控制标记。值位置的 `@if` 必须有 `@else`：

```rust
el! {
    @if (ready)
        .div ["已就绪"]
    @else
        .div ["等待中"]
    @end { flex } [extra_child].post_process
}
```

此时 `@end` 后的后缀作用于选中分支的结果，各分支返回类型必须兼容。
children/属性片段本身没有单个元素值，不能直接在片段的 `@end` 后接元素后缀。
在 children 中需要条件值时，可将它放到调用参数位置，例如
`(@if (ready) .div ["yes"] @else .div ["no"] @end).post_process`。

指令直接生成 Rust `if`/`match`，不额外引入闭包。条件按分支顺序求值，
只执行被选中的主体。匹配值求值一次，模式绑定只在对应守卫和分支中有效。
`match` 保留 Rust 的借用、移动、守卫和穷尽性检查，不自动补默认分支。
需要兜底时显式写 `@case (_)`；片段中的兜底主体可以为空。

## 子元素与阶段

`[child1, child2]` 按顺序调用 `.child(child1).child(child2)`。
`..expression` 调用 `.children(expression)`。属性块也能配置普通值：

```rust
el! {
    .div [
        "标题",
        existing_element,
        .div ["嵌套内容"],
        ..rows,
        ..items { into_iter; map: render_item },
    ]
}
```

`..items { into_iter; map: render_item }` 与
`..$(items.into_iter().map(render_item))` 等价：items 只求值一次，迭代器的惰性保持不变。

值或调用后可以接任意数量的属性块、children 列表和函数后缀，严格按书写顺序
操作当前结果。块和列表可以为空，也可以连续出现同一种块。
每个 `.callable` 接收它左边已经完成的整个结果：

```text
value { flex } [x, y] { f1; f2 } [u, v].some_post_process
```

等价于 `some_post_process(value.flex().child(x).child(y).f1().f2().child(u).child(v))`。
后写的属性不会被提前到 children 前。每一步都可以改变返回类型。

## 求值与所有权

函数路径直接生成 `f(args)`，保留 Rust 的闭包借用、类型推断和参数求值规则。
调用参数、属性和子元素从左到右求值一次，不重复执行任何输入表达式。
动态可调用值则先求所有参数，再求这个可调用值，最后调用。

动态调用通过 `FnOnce` 按值传递。需要复用闭包时可显式借用：
`value.$(&handler)` 或 `value.$(&mut handler)`；也可用 `~handler` 克隆。
直接调用闭包路径 `value.handler` 与 Rust 的 `handler(value)` 使用相同的
借用或移动规则。闭包实现 Fn/FnMut/FnOnce 中的哪些 trait 仍由 Rust 根据主体决定。

DSL 不添加隐式克隆、借用或类型转换。捕获、借用、移动和返回类型由 Rust 检查。

## 形式化语法

EBNF 中 `?` 表示可选，`*` 表示重复，引号中是字面 token。
`control<T>` 中的 T 表示由所在位置决定的分支内容，并非实际输入 token。

```text
uim-expr := uim-head uim-body ("." uim-val uim-body)*
uim-head := uim-val | "." uim-val | "(" uim-args ")" "." uim-val
uim-body := (uim-attr-block | uim-children)*
uim-args := (uim-expr ("," uim-expr)* ","?)?

uim-val := rust-path | rust-literal | rust-macro
         | "$" rust-parenthesized-expression | "$" rust-expr
         | "~" uim-val
         | "move"? uim-attr-block
         | "move"? closure-params "=>" uim-expr
         | control<uim-expr>
closure-params := rust-ident | "_" | "(" closure-param-list ")"
closure-param-list := (closure-param ("," closure-param)* ","?)?
closure-param := rust-param-pattern (":" rust-type)?

uim-attr-block := "{" attr-items "}"
attr-items := (uim-attr-item (";" uim-attr-item)* ";"?)?
uim-attr-item := attr | attr ":" uim-args | "#" uim-expr
               | "#" "?" "move"? uim-expr
               | control<attr-items>

uim-children := "[" child-items "]"
child-items := (uim-child ("," uim-child)* ","?)?
uim-child := ".."? uim-expr | control<child-items>

control<T> := "@if" "(" rust-condition ")" T
              ("@else" "if" "(" rust-condition ")" T)*
              ("@else" T)? "@end"
            | "@match" "(" rust-expr ")"
              ("@case" "(" rust-pattern ("if" rust-expr)? ")" T)*
              "@end"
el-input := uim-expr
m-input := attr-items | "move" uim-attr-block
```

值位置的 if 额外要求 else。children 项直接以 `@` 开始时选择片段解析，
表达式内部则选择值解析。普通括号后若为 `=>` 则解析闭包参数，否则要求
`.callable`。单参数箭头闭包也优先于路径值解析；箭头主体向右延伸。

`rust-path` 包含单独标识符。`rust-parenthesized-expression` 包含括号本身，
可以是分组表达式、元组或 unit；它在 `$` 后优先匹配。
分隔符不能充当空属性或空参数；根表达式后不接逗号。

## 实现

独立的 `gds-uim-macros` 过程宏 crate 由主库重新导出 `el!` 和 `m!`。
Rust 表达式、模式和类型交给 `syn`。生成器用局部绑定保留顺序和类型变化。
函数路径直接调用，以保留高阶回调的类型和生命周期推断；动态可调用值使用
带 `FnOnce` 约束的局部辅助函数，以推断闭包参数类型并保证输入先求值。

`syn`、`quote` 和 `proc-macro2` 仅用于编译期解析与生成。
GPUI 只作为测试和示例的开发依赖。

展开结构、临时借用和递归深度的回归约束见 [宏展开阶段](expansion.md)。
