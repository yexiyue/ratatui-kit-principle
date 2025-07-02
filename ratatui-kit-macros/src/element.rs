use quote::{ToTokens, quote};
use syn::{
    Expr, FieldValue, Lit, Member, Result, Token, TypePath, braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Comma, Paren},
};
use uuid::Uuid;

/// 表示一个子元素，可以是嵌套的 ParsedElement 或表达式（如 #(...expr)）。
enum ParsedElementChild {
    Element(ParsedElement), // 嵌套子元素
    Expr(Expr),             // 动态表达式
}

/// 解析后的 UI 元素结构，包含类型、属性和子元素。
pub struct ParsedElement {
    ty: TypePath,                         // 元素类型
    props: Punctuated<FieldValue, Comma>, // 属性列表
    children: Vec<ParsedElementChild>,    // 子元素列表
}

/// 支持从宏输入流中解析出 ParsedElement 结构。
impl Parse for ParsedElement {
    fn parse(input: ParseStream) -> Result<Self> {
        // 解析类型名
        let ty: TypePath = input.parse()?;

        // 解析属性 (可选)
        let props = if input.peek(Paren) {
            let props_input;
            parenthesized!(props_input in input);
            Punctuated::parse_terminated(&props_input)?
        } else {
            Punctuated::new()
        };

        // 解析子元素 (可选)
        let mut children = Vec::new();
        if input.peek(Brace) {
            let children_input;
            braced!(children_input in input);
            while !children_input.is_empty() {
                if children_input.peek(Token![#]) {
                    // 支持 #(...) 语法，插入表达式作为子节点
                    children_input.parse::<Token![#]>()?;
                    let child_input;
                    parenthesized!(child_input in children_input);
                    children.push(ParsedElementChild::Expr(child_input.parse()?));
                } else {
                    // 递归解析嵌套子元素
                    children.push(ParsedElementChild::Element(children_input.parse()?));
                }
            }
        }

        Ok(Self {
            props,
            ty,
            children,
        })
    }
}

/// 将 ParsedElement 转换为 TokenStream，实现宏展开时的代码生成。
impl ToTokens for ParsedElement {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = &self.ty;

        // 生成唯一 key（如未指定 key 属性则自动生成）
        let decl_key = Uuid::new_v4().as_u128();

        // 查找 key 属性，如果有则用 (decl_key, key_expr)，否则用 decl_key
        let key = self
            .props
            .iter()
            .find_map(|FieldValue { member, expr, .. }| match member {
                Member::Named(ident) if ident == "key" => Some(quote!((#decl_key, #expr))),
                _ => None,
            })
            .unwrap_or_else(|| quote!(#decl_key));

        // 生成属性赋值代码，支持百分比语法 sugar（如 50pct）
        let prop_assignments = self
            .props
            .iter()
            .filter_map(|FieldValue { member, expr, .. }| match member {
                Member::Named(ident) if ident == "key" => None, // key 属性不赋值到 props
                _ => Some(match expr {
                    Expr::Lit(lit) => match &lit.lit {
                        Lit::Int(lit) if lit.suffix() == "pct" => {
                            let value = lit.base10_parse::<f32>().unwrap();
                            quote!(_props.#member = ::ratatui_kit_principle::Percent(#value).into())
                        }
                        Lit::Float(lit) if lit.suffix() == "pct" => {
                            let value = lit.base10_parse::<f32>().unwrap();
                            quote!(_props.#member = ::ratatui_kit_principle::Percent(#value).into())
                        }
                        _ => quote!(_props.#member = (#expr).into()),
                    },
                    _ => quote!(_props.#member = (#expr).into()),
                }),
            })
            .collect::<Vec<_>>();

        // 生成子元素扩展代码，支持嵌套和 #(...) 表达式
        let set_children = if !self.children.is_empty() {
            let children = self.children.iter().map(|child| match child {
                ParsedElementChild::Element(child) => quote!(#child),
                ParsedElementChild::Expr(expr) => quote!(#expr),
            });
            Some(quote! {
                #(::ratatui_kit_principle::element::extend_with_elements(&mut _element.props.children, #children);)*
            })
        } else {
            None
        };

        // 生成最终 Element 构造代码
        tokens.extend(quote! {
            {
                type Props<'a> = <#ty as ::ratatui_kit_principle::element::ElementType>::Props<'a>;
                let mut _props: Props = Default::default();
                #(#prop_assignments;)*
                let mut _element = ::ratatui_kit_principle::element::Element::<#ty>{
                    key: ::ratatui_kit_principle::element::ElementKey::new(#key),
                    props: _props,
                };
                #set_children
                _element
            }
        });
    }
}
