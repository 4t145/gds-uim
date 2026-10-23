use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use syn::{
    Expr, ExprPath, Ident, Lit, Pat, Result, Token, braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token,
};

mod kw {
    syn::custom_keyword!(case);
    syn::custom_keyword!(end);
}

pub struct Expression {
    pub head: Head,
    pub body: Body,
    pub suffixes: Vec<(Value, Body)>,
}

pub enum Head {
    Value(Value),
    Call(Vec<Expression>, Value),
}

pub enum Value {
    Rust(Box<Expr>),
    Clone(Token![~], Box<Value>),
    Mapper(Box<Mapper>),
    Closure(Box<Closure>),
    Control(Box<Control<Expression>>),
}

impl Value {
    pub fn span(&self) -> Span {
        match self {
            Self::Rust(value) => value.span(),
            Self::Clone(tilde, _) => tilde.span(),
            Self::Mapper(mapper) => mapper.span,
            Self::Closure(closure) => closure.arrow.span(),
            Self::Control(control) => control.span,
        }
    }
}

pub struct Closure {
    pub capture: Option<Token![move]>,
    pub inputs: Punctuated<Pat, Token![,]>,
    pub arrow: Token![=>],
    pub body: Expression,
}

pub struct Control<T> {
    pub span: Span,
    pub kind: ControlKind<T>,
}

pub enum ControlKind<T> {
    If {
        branches: Vec<(Expr, T)>,
        otherwise: Option<Box<T>>,
    },
    Match {
        value: Box<Expr>,
        arms: Vec<Arm<T>>,
    },
}

pub struct Arm<T> {
    pub pattern: Pat,
    pub guard: Option<Expr>,
    pub body: T,
}

pub struct Body(pub Vec<Section>);

pub enum Section {
    Attributes(Attributes),
    Children(Children),
}

#[derive(Default)]
pub struct Attributes(pub Vec<Attribute>);

pub enum Attribute {
    Method(Ident, Vec<Expression>),
    Id(Token![#], Expression),
    DebugSelector(Token![#], Option<Token![move]>, Expression),
    Control(Box<Control<Attributes>>),
}

pub struct Mapper {
    pub span: Span,
    pub capture: Option<Token![move]>,
    pub attributes: Attributes,
}

pub struct Children(pub Vec<Child>);

pub enum Child {
    Element {
        spread: Option<Token![..]>,
        expression: Expression,
    },
    Control(Box<Control<Children>>),
}

// Control markers are only recognized at the current token-tree level. Rust
// escapes, macro arguments and nested DSL groups retain their own parsers.
fn at_boundary(input: ParseStream) -> bool {
    if !input.peek(Token![@]) {
        return false;
    }
    let fork = input.fork();
    let _ = fork.parse::<Token![@]>();
    fork.peek(Token![else]) || fork.peek(kw::case) || fork.peek(kw::end)
}

fn closure_ahead(input: ParseStream) -> bool {
    let fork = input.fork();
    if fork.peek(Token![move]) {
        let _ = fork.parse::<Token![move]>();
    }
    if fork.peek(token::Paren) || fork.peek(Ident) || fork.peek(Token![_]) {
        let _ = fork.parse::<TokenTree>();
        fork.peek(Token![=>])
    } else {
        false
    }
}

impl Parse for Expression {
    fn parse(input: ParseStream) -> Result<Self> {
        let head = if input.peek(token::Paren) && !closure_ahead(input) {
            let content;
            parenthesized!(content in input);
            let arguments = arguments(&content)?;
            if !input.peek(Token![.]) || input.peek(Token![..]) {
                return Err(input.error("expected .callable after an argument list or => after closure parameters; use $(...) for a Rust value or tuple"));
            }
            input.parse::<Token![.]>()?;
            Head::Call(arguments, input.parse()?)
        } else if input.peek(Token![.]) && !input.peek(Token![..]) {
            input.parse::<Token![.]>()?;
            Head::Call(Vec::new(), input.parse()?)
        } else {
            Head::Value(input.parse()?)
        };

        let body = input.parse()?;
        let mut suffixes = Vec::new();
        while input.peek(Token![.]) && !input.peek(Token![..]) {
            input.parse::<Token![.]>()?;
            suffixes.push((input.parse()?, input.parse()?));
        }
        if !input.is_empty()
            && !input.peek(Token![,])
            && !input.peek(Token![;])
            && !input.peek(Token![:])
            && !at_boundary(input)
        {
            return Err(input.error(
                "expected .callable, a block, or a separator; use $ or $(...) for Rust expressions",
            ));
        }
        Ok(Self {
            head,
            body,
            suffixes,
        })
    }
}

impl Parse for Value {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![~]) {
            return Ok(Self::Clone(input.parse()?, Box::new(input.parse()?)));
        }
        if closure_ahead(input) {
            return Ok(Self::Closure(Box::new(input.parse()?)));
        }
        if input.peek(token::Brace) || input.peek(Token![move]) {
            return Ok(Self::Mapper(Box::new(Mapper::block(input)?)));
        }
        if input.peek(Token![@]) {
            let control: Control<Expression> = input.parse()?;
            if matches!(
                &control.kind,
                ControlKind::If {
                    otherwise: None,
                    ..
                }
            ) {
                return Err(syn::Error::new(
                    control.span,
                    "@if used as a value requires @else; only attribute and children fragments may omit it",
                ));
            }
            return Ok(Self::Control(Box::new(control)));
        }
        if input.peek(Token![$]) {
            input.parse::<Token![$]>()?;
            if !input.peek(token::Paren) {
                return Ok(Self::Rust(Box::new(input.parse()?)));
            }
            let content;
            parenthesized!(content in input);
            let tokens: TokenStream = content.parse()?;
            // Keep parentheses: $(a, b) is a tuple and $() is unit.
            return Ok(Self::Rust(Box::new(syn::parse2(quote! { (#tokens) })?)));
        }
        if input.peek(Lit) {
            let literal: Lit = input.parse()?;
            return Ok(Self::Rust(Box::new(syn::parse2(quote! { #literal })?)));
        }
        if input.peek(Token![-]) {
            let minus: Token![-] = input.parse()?;
            let literal: Lit = input.parse()?;
            if !matches!(literal, Lit::Int(_) | Lit::Float(_)) {
                return Err(syn::Error::new(
                    literal.span(),
                    "expected a numeric literal after -; use $(...) for negation",
                ));
            }
            return Ok(Self::Rust(Box::new(syn::parse2(
                quote! { #minus #literal },
            )?)));
        }
        if input.peek(Ident)
            || input.peek(Token![::])
            || input.peek(Token![<])
            || input.peek(Token![self])
            || input.peek(Token![Self])
            || input.peek(Token![super])
            || input.peek(Token![crate])
        {
            let path: ExprPath = input.parse()?;
            if input.peek(Token![!]) {
                let bang: Token![!] = input.parse()?;
                let group: TokenTree = input.parse()?;
                if !matches!(group, TokenTree::Group(_)) {
                    return Err(syn::Error::new(
                        group.span(),
                        "expected a macro argument group after !",
                    ));
                }
                return Ok(Self::Rust(Box::new(syn::parse2(
                    quote! { #path #bang #group },
                )?)));
            }
            return Ok(Self::Rust(Box::new(Expr::Path(path))));
        }
        Err(input.error("expected a value, closure, @if/@match, .callable, (arguments).callable, ~value, or $(Rust expression)"))
    }
}

fn closure_parameter(input: ParseStream) -> Result<Pat> {
    let pattern = Pat::parse_single(input)?;
    if input.peek(Token![:]) {
        Ok(Pat::Type(syn::PatType {
            attrs: Vec::new(),
            pat: Box::new(pattern),
            colon_token: input.parse()?,
            ty: input.parse()?,
        }))
    } else {
        Ok(pattern)
    }
}

impl Parse for Closure {
    fn parse(input: ParseStream) -> Result<Self> {
        let capture = input.parse()?;
        let inputs = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            content.parse_terminated(closure_parameter, Token![,])?
        } else {
            let mut parameters = Punctuated::new();
            parameters.push(Pat::parse_single(input)?);
            parameters
        };
        Ok(Self {
            capture,
            inputs,
            arrow: input.parse()?,
            body: input.parse()?,
        })
    }
}

fn rust_header(input: ParseStream) -> Result<Expr> {
    let content;
    parenthesized!(content in input);
    let expression = content.parse()?;
    if !content.is_empty() {
        return Err(content.error("expected one Rust expression in the directive header"));
    }
    Ok(expression)
}

impl<T: Parse> Parse for Control<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let at: Token![@] = input.parse()?;
        let kind = if input.peek(Token![if]) {
            input.parse::<Token![if]>()?;
            let mut branches = vec![(rust_header(input)?, input.parse()?)];
            let mut otherwise = None;
            loop {
                input
                    .parse::<Token![@]>()
                    .map_err(|_| input.error("expected @else or @end to close @if"))?;
                if input.peek(kw::end) {
                    input.parse::<kw::end>()?;
                    break;
                }
                input
                    .parse::<Token![else]>()
                    .map_err(|_| input.error("expected @else or @end in @if"))?;
                if input.peek(Token![if]) {
                    input.parse::<Token![if]>()?;
                    branches.push((rust_header(input)?, input.parse()?));
                } else {
                    otherwise = Some(Box::new(input.parse()?));
                    input
                        .parse::<Token![@]>()
                        .map_err(|_| input.error("expected @end after @else"))?;
                    input
                        .parse::<kw::end>()
                        .map_err(|_| input.error("expected @end after @else"))?;
                    break;
                }
            }
            ControlKind::If {
                branches,
                otherwise,
            }
        } else if input.peek(Token![match]) {
            input.parse::<Token![match]>()?;
            let value = Box::new(rust_header(input)?);
            let mut arms = Vec::new();
            loop {
                input
                    .parse::<Token![@]>()
                    .map_err(|_| input.error("expected @case (...) or @end in @match"))?;
                if input.peek(kw::end) {
                    input.parse::<kw::end>()?;
                    break;
                }
                input
                    .parse::<kw::case>()
                    .map_err(|_| input.error("expected @case (...) or @end in @match"))?;
                let header;
                parenthesized!(header in input);
                let pattern = Pat::parse_multi_with_leading_vert(&header)?;
                let guard = if header.peek(Token![if]) {
                    header.parse::<Token![if]>()?;
                    Some(header.parse()?)
                } else {
                    None
                };
                if !header.is_empty() {
                    return Err(header.error("expected a Rust pattern and optional if guard"));
                }
                arms.push(Arm {
                    pattern,
                    guard,
                    body: input.parse()?,
                });
            }
            ControlKind::Match { value, arms }
        } else {
            return Err(input.error("expected @if (...) or @match (...); use when: condition, handler for conditional methods"));
        };
        Ok(Self {
            span: at.span,
            kind,
        })
    }
}

impl Parse for Body {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut sections = Vec::new();
        loop {
            if input.peek(token::Brace) {
                sections.push(Section::Attributes(attribute_block(input)?));
            } else if input.peek(token::Bracket) {
                let content;
                bracketed!(content in input);
                sections.push(Section::Children(content.parse()?));
                if !content.is_empty() {
                    return Err(content.error("unexpected control marker outside a control block"));
                }
            } else {
                break;
            }
        }
        Ok(Self(sections))
    }
}

impl Parse for Children {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut children = Vec::new();
        while !input.is_empty() && !at_boundary(input) {
            children.push(if input.peek(Token![@]) {
                Child::Control(Box::new(input.parse()?))
            } else {
                Child::Element {
                    spread: input.parse()?,
                    expression: input.parse()?,
                }
            });
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else if !input.is_empty() && !at_boundary(input) {
                return Err(input.error("expected , between children"));
            }
        }
        Ok(Self(children))
    }
}

// Arguments share the expression parser, including closures and value controls.
fn arguments(input: ParseStream) -> Result<Vec<Expression>> {
    let mut args = Vec::new();
    while !input.is_empty() && !input.peek(Token![;]) && !at_boundary(input) {
        args.push(input.parse()?);
        if !input.is_empty() && !input.peek(Token![;]) && !at_boundary(input) {
            input.parse::<Token![,]>()?;
        }
    }
    Ok(args)
}

fn attribute_block(input: ParseStream) -> Result<Attributes> {
    let content;
    braced!(content in input);
    let attributes = content.parse()?;
    if !content.is_empty() {
        return Err(content.error("unexpected control marker outside a control block"));
    }
    Ok(attributes)
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attributes = Vec::new();
        while !input.is_empty() && !at_boundary(input) {
            attributes.push(input.parse()?);
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
            } else if !input.is_empty() && !at_boundary(input) {
                return Err(input.error("expected ; between attributes"));
            }
        }
        Ok(Self(attributes))
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![#]) {
            let hash = input.parse()?;
            if input.peek(Token![?]) {
                input.parse::<Token![?]>()?;
                return Ok(Self::DebugSelector(hash, input.parse()?, input.parse()?));
            }
            return Ok(Self::Id(hash, input.parse()?));
        }
        if input.peek(Token![@]) {
            return Ok(Self::Control(Box::new(input.parse()?)));
        }
        let method = input.parse()?;
        if input.peek(Token![=>]) {
            return Err(
                input.error("=> defines a closure; use method: { ... } or method: move { ... }")
            );
        }
        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Ok(Self::Method(method, arguments(input)?))
        } else {
            Ok(Self::Method(method, Vec::new()))
        }
    }
}

impl Mapper {
    fn block(input: ParseStream) -> Result<Self> {
        let span = input.span();
        Ok(Self {
            span,
            capture: input.parse()?,
            attributes: attribute_block(input)?,
        })
    }
}

impl Parse for Mapper {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![move]) {
            Self::block(input)
        } else {
            Ok(Self {
                span: input.span(),
                capture: None,
                attributes: input.parse()?,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_incomplete_calls_invalid_attributes_and_bad_separators() {
        for source in [
            "",
            ".",
            "~",
            "#div",
            "List()",
            "(1, 2)",
            "(1;).f",
            "(,1).f",
            "(1,,2).f",
            ".div { flex gap: 1 }",
            ".div { # \"id\" gap: 1 }",
            ".div { #? }",
            ".div { #? move }",
            ".div { #?? \"id\" }",
            ".div { flex;; }",
            ".div [a b]",
            ".div [a,,b]",
            ".div [..]",
            ".div,",
            ".div { @true }",
            ".div { @true: a, b }",
            ".div { map => {} }",
            "-$value",
            "-\"text\"",
            "f! value",
            "$(1 +)",
            "() =>",
            "move (a,) =>",
            "(,a) => a",
            "(a,,b) => a",
            "move value",
        ] {
            assert!(
                syn::parse_str::<Expression>(source).is_err(),
                "accepted invalid input: {source}"
            );
        }
    }

    #[test]
    fn errors_explain_call_and_rust_boundaries() {
        let error = syn::parse_str::<Expression>("(a, b)").err().unwrap();
        assert!(
            error
                .to_string()
                .contains(".callable after an argument list")
        );
        let error = syn::parse_str::<Expression>("value { flex } trailing")
            .err()
            .unwrap();
        assert!(
            error
                .to_string()
                .contains("expected .callable, a block, or a separator")
        );
    }

    #[test]
    fn mappers_require_complete_blocks_and_attribute_separators() {
        for source in [
            "move",
            "move {} extra",
            "flex gap: 1",
            "map =>",
            "#",
            "#?",
            "#? move",
            "@true =>",
        ] {
            assert!(
                syn::parse_str::<Mapper>(source).is_err(),
                "accepted invalid mapper: {source}"
            );
        }
        for source in [
            "",
            "flex;",
            "move {}",
            "# \"id\"; when: true, move { flex }",
            "# \"id\"; #? \"selector\" { into }; flex",
            "#? move format!(\"task-{id}\"); gap: 1",
        ] {
            assert!(
                syn::parse_str::<Mapper>(source).is_ok(),
                "rejected mapper: {source}"
            );
        }
    }

    #[test]
    fn rejects_unbalanced_controls_and_wrong_branch_shapes() {
        for source in [
            "@if (true) a @end",
            "@if (true) @else b @end",
            "@if (true) a @else b",
            "@if (true) a @else b @else c @end",
            "@if true a @else b @end",
            "@if (true) a, b @else c @end",
            "@if (true) a @case (_) b @end",
            "@match (x) @else a @end",
            "@match (x) @case (_) => a @end",
            "@match (x) @case (_) a, b @end",
            "@end",
            "@case (_) a",
            "@unknown (x)",
            ".div { @else flex }",
            ".div [@end]",
            ".div { @if (true) flex }",
            ".div [@if (true) a]",
            ".div [@if (true) a @end b]",
            ".div { @if (true) flex @end gap: 1 }",
        ] {
            assert!(
                syn::parse_str::<Expression>(source).is_err(),
                "accepted invalid control: {source}"
            );
        }
        let error = syn::parse_str::<Expression>("@if (true) a @end")
            .err()
            .unwrap();
        assert!(error.to_string().contains("requires @else"));
    }
}
