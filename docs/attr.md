为带 `#[attr]` 的组件字段生成消费 `self`、返回 `Self` 的链式 setter。
构造器手写，未标注的字段保留给内部状态和手写方法。

```rust
use gds_uim::{component, el};

#[derive(Debug, PartialEq)]
pub enum Icon {
    Check,
}

#[component]
struct MyButton {
    id: &'static str,
    #[attr(option, into, option_setter = "icon_opt")]
    icon: Option<Icon>,
    /// 按钮文本。
    #[attr(into)]
    text: String,
    #[attr]
    width: f32,
    #[attr]
    height: f32,
    #[attr]
    disabled: bool,
}

impl MyButton {
    fn new(id: &'static str) -> Self {
        Self {
            id,
            icon: None,
            text: String::new(),
            width: 0.0,
            height: 0.0,
            disabled: false,
        }
    }
}

let button = el! {
    "confirm".MyButton::new {
        icon: Icon::Check;
        text: "确定";
        width: 120.0;
        height: 32.0;
        disabled: false;
    }
};
assert_eq!(button.id, "confirm");
assert_eq!(button.icon, Some(Icon::Check));
assert_eq!(button.text, "确定");

let button = button.icon_opt(None);
assert!(button.icon.is_none());
```

字段可以保持私有；生成的方法默认是 `pub`，方法名默认与字段名相同。
重复设置属性会覆盖原值，值按所有权移动，不要求 `Clone` 或 `Default`。

| 标记 | 字段类型 | 方法入参 | 赋值表达式 |
| --- | --- | --- | --- |
| `#[attr]` | `T` | `T` | `value` |
| `#[attr(into)]` | `T` | `impl Into<T>` | `value.into()` |
| `#[attr(option)]` | `Option<T>` | `T` | `Some(value)` |
| `#[attr(option, into)]` | `Option<T>` | `impl Into<T>` | `Some(value.into())` |
| `#[attr(any_element)]` | `AnyElement` | `impl gpui::IntoElement` | `gpui::IntoElement::into_any_element(value)` |
| `#[attr(any_element, option)]` | `Option<AnyElement>` | `impl gpui::IntoElement` | `Some(gpui::IntoElement::into_any_element(value))` |

`option` 只表示调用时省略 `Some`，不负责字段初始化或必填检查。
它要求字段类型写成 `Option<T>`，也支持 `std::option::Option<T>`、
`core::option::Option<T>` 等限定路径；宏不解析整个 `Option<T>` 的类型别名。
没有 `option` 时，`Option<T>` 与普通字段一样处理。

例如，`icon` 和 `text` 的方法等价于：

```rust,ignore
pub fn icon(mut self, value: impl Into<Icon>) -> Self {
    self.icon = Some(value.into());
    self
}

pub fn text(mut self, value: impl Into<String>) -> Self {
    self.text = value.into();
    self
}
```

`any_element` 用于 GPUI 元素插槽，将传入的组件、元素或文本转换为 `AnyElement`。
它与 `into` 互斥，可与 `option` 组合；不带 `option` 时字段类型为 `AnyElement`，
带 `option` 时为 `Option<AnyElement>`。生成代码使用 `::gpui::IntoElement` 的
完整路径，使用项目需要提供 `gpui` 依赖，无需在调用处额外导入该 trait。

```rust
use gds_uim::{component, el};
use gpui::{AnyElement, div};

#[component]
struct Panel {
    #[attr(any_element)]
    expanded: AnyElement,
    #[attr(any_element, option, option_setter = "collapsed_opt")]
    collapsed: Option<AnyElement>,
}

impl Panel {
    fn new() -> Self {
        Self {
            expanded: gpui::IntoElement::into_any_element(div()),
            collapsed: None,
        }
    }
}

let panel = el! {
    .Panel::new {
        expanded: .div;
        collapsed: "收起时的内容";
    }
};
assert!(panel.collapsed.is_some());
let panel = panel.collapsed_opt(None);
assert!(panel.collapsed.is_none());
```

`expanded` 的 setter 直接保存 `gpui::IntoElement::into_any_element(value)`，
`collapsed` 的 setter 则保存 `Some(gpui::IntoElement::into_any_element(value))`。
`collapsed_opt` 接收完整的 `Option<AnyElement>`，直接覆盖或清空插槽。

额外配置可以与 `into`、`any_element`、`option` 组合：

- `name = "label"`：修改 setter 名，字段名保持不变。Rust 关键字使用
  `name = "r#type"` 这样的原始标识符。
- `vis = "pub(crate)"`：指定方法可见性，也支持 `pub(super)`、`pub(self)`
  等 Rust 可见性，`vis = ""` 表示私有。默认 `pub` 与字段可见性无关。
- `option_setter = "icon_opt"`：与 `option` 一起使用，额外生成接收完整
  `Option<T>` 的 setter；它与主 setter 使用相同的可见性。
  传入 `None` 清空字段，传入已有的 `Option<T>` 则直接覆盖。

```rust
use gds_uim::component;

#[component]
struct Label {
    #[attr(into, name = "text", vis = "pub(crate)")]
    content: String,
}

let label = Label { content: String::new() }.text("Hello");
assert_eq!(label.content, "Hello");
```

宏支持具名字段结构体，保留泛型、生命周期、const 泛型和 `where` 条件。
结构体与字段的条件编译会应用到生成的实现和方法，字段文档会复制到方法上。
字段的其他属性保持在原字段上，`#[attr]` 在展开时移除。

每个字段使用一个 `#[attr(...)]`，支持尾逗号。未知选项、重复选项和无效组合
会报错，例如在普通字段上使用 `option`：

```compile_fail
use gds_uim::component;

#[component]
struct Label {
    #[attr(option)]
    content: String,
}
```

方法名冲突由 Rust 编译器检查，包括与手写方法的冲突。
布尔字段生成接收 `bool` 的方法；回调包装、子元素追加、样式委托和渲染实现
通过普通 Rust 方法与 trait 编写，可以和生成的 setter 一起用于 `el!`、`m!`。
