use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, Result, Token, braced, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct EkaInput {
    dal: Ident,
    root_element: ElementDef,
}

struct ElementDef {
    binding: Option<Ident>,
    element_type: ElementType,
}

enum ElementType {
    Label {
        text: Expr,
        style: Option<Expr>,
        common: CommonAttrs,
    },
    Button {
        text: Expr,
        on_click: Expr,
        style: Option<Expr>,
        common: CommonAttrs,
    },
    Panel {
        style: Option<Expr>,
        children: Vec<ElementDef>,
        common: CommonAttrs,
    },
    Checkbox {
        checked: Expr,
        common: CommonAttrs,
    },
    TextInput {
        text: Expr,
        common: CommonAttrs,
    },
}

#[derive(Default)]
struct CommonAttrs {
    on_click: Option<Expr>,
    on_hover: Option<Expr>,
}

impl Parse for EkaInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let dal: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let root_element = input.parse()?;
        Ok(EkaInput { dal, root_element })
    }
}

impl Parse for ElementDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut binding = None;
        if input.peek2(Token![=]) {
            binding = Some(input.parse::<Ident>()?);
            input.parse::<Token![=]>()?;
        }

        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);

        let element_type = match name.to_string().as_str() {
            "Label" => {
                let mut text = None;
                let mut style = None;
                let mut common = CommonAttrs::default();

                while !content.is_empty() {
                    let field: Ident = content.parse()?;
                    content.parse::<Token![:]>()?;
                    match field.to_string().as_str() {
                        "text" => text = Some(content.parse::<Expr>()?),
                        "style" => style = Some(content.parse::<Expr>()?),
                        "on_click" => common.on_click = Some(content.parse::<Expr>()?),
                        "on_hover" => common.on_hover = Some(content.parse::<Expr>()?),
                        _ => return Err(content.error("Unknown field for Label")),
                    }
                    if !content.is_empty() {
                        content.parse::<Token![,]>()?;
                    }
                }

                ElementType::Label {
                    text: text.ok_or_else(|| content.error("Missing 'text' for Label"))?,
                    style,
                    common,
                }
            }
            "Button" => {
                let mut text = None;
                let mut on_click = None;
                let mut style = None;
                let mut common = CommonAttrs::default();

                while !content.is_empty() {
                    let field: Ident = content.parse()?;
                    content.parse::<Token![:]>()?;
                    match field.to_string().as_str() {
                        "text" => text = Some(content.parse::<Expr>()?),
                        "on_click" => on_click = Some(content.parse::<Expr>()?),
                        "on_hover" => common.on_hover = Some(content.parse::<Expr>()?),
                        "style" => style = Some(content.parse::<Expr>()?),
                        _ => return Err(content.error("Unknown field for Button")),
                    }
                    if !content.is_empty() {
                        content.parse::<Token![,]>()?;
                    }
                }

                ElementType::Button {
                    text: text.ok_or_else(|| content.error("Missing 'text' for Button"))?,
                    on_click: on_click
                        .ok_or_else(|| content.error("Missing 'on_click' for Button"))?,
                    style,
                    common,
                }
            }
            "Panel" => {
                let mut style = None;
                let mut children = Vec::new();
                let mut common = CommonAttrs::default();

                while !content.is_empty() {
                    let field: Ident = content.parse()?;
                    content.parse::<Token![:]>()?;
                    match field.to_string().as_str() {
                        "style" => style = Some(content.parse::<Expr>()?),
                        "on_click" => common.on_click = Some(content.parse::<Expr>()?),
                        "on_hover" => common.on_hover = Some(content.parse::<Expr>()?),
                        "children" => {
                            let children_content;
                            bracketed!(children_content in content);
                            while !children_content.is_empty() {
                                children.push(children_content.parse()?);
                                if !children_content.is_empty() {
                                    children_content.parse::<Token![,]>()?;
                                }
                            }
                        }
                        _ => return Err(content.error("Unknown field for Panel")),
                    }
                    if !content.is_empty() {
                        content.parse::<Token![,]>()?;
                    }
                }

                ElementType::Panel {
                    style,
                    children,
                    common,
                }
            }
            "Checkbox" => {
                let mut checked = None;
                let mut common = CommonAttrs::default();

                while !content.is_empty() {
                    let field: Ident = content.parse()?;
                    content.parse::<Token![:]>()?;
                    match field.to_string().as_str() {
                        "checked" => checked = Some(content.parse::<Expr>()?),
                        "on_click" => common.on_click = Some(content.parse::<Expr>()?),
                        "on_hover" => common.on_hover = Some(content.parse::<Expr>()?),
                        _ => return Err(content.error("Unknown field for Checkbox")),
                    }
                    if !content.is_empty() {
                        content.parse::<Token![,]>()?;
                    }
                }

                ElementType::Checkbox {
                    checked: checked
                        .ok_or_else(|| content.error("Missing 'checked' for Checkbox"))?,
                    common,
                }
            }
            "TextInput" => {
                let mut text = None;
                let mut common = CommonAttrs::default();

                while !content.is_empty() {
                    let field: Ident = content.parse()?;
                    content.parse::<Token![:]>()?;
                    match field.to_string().as_str() {
                        "text" => text = Some(content.parse::<Expr>()?),
                        "on_click" => common.on_click = Some(content.parse::<Expr>()?),
                        "on_hover" => common.on_hover = Some(content.parse::<Expr>()?),
                        _ => return Err(content.error("Unknown field for TextInput")),
                    }
                    if !content.is_empty() {
                        content.parse::<Token![,]>()?;
                    }
                }

                ElementType::TextInput {
                    text: text.ok_or_else(|| content.error("Missing 'text' for TextInput"))?,
                    common,
                }
            }
            _ => return Err(syn::Error::new(name.span(), "Unknown element type")),
        };

        Ok(ElementDef {
            binding,
            element_type,
        })
    }
}

#[proc_macro]
pub fn eka(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EkaInput);
    let dal = &input.dal;

    let code = generate_element(&input.root_element, dal, quote!(None::<deka::Element>));

    quote! {
        {
            #code
        }
    }
    .into()
}

fn generate_element(
    def: &ElementDef,
    dal: &Ident,
    parent: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let binding = &def.binding;

    let (creation_code, common) = match &def.element_type {
        ElementType::Label {
            text,
            style,
            common,
        } => {
            let style = match style {
                Some(s) => quote!(Some(#s)),
                None => quote!(None),
            };
            (
                quote! {
                    #dal.new_label(#text, #parent, #style)
                },
                common,
            )
        }
        ElementType::Button {
            text,
            on_click,
            style,
            common,
        } => {
            let style = match style {
                Some(s) => quote!(Some(#s)),
                None => quote!(None),
            };
            (
                quote! {
                    #dal.new_button(#text, #parent, #on_click, #style)
                },
                common,
            )
        }
        ElementType::Panel {
            style,
            children,
            common,
        } => {
            let style = match style {
                Some(s) => quote!(#s),
                None => quote!(deka::heka::Style::default()),
            };

            let panel_ref = quote!(panel_ref);

            let children_code: Vec<_> = children
                .iter()
                .map(|child| generate_element(child, dal, quote!(Some(#panel_ref))))
                .collect();

            (
                quote! {
                    {
                        let #panel_ref = #dal.new_panel(#parent, #style);
                        #( #children_code; )*
                        #panel_ref
                    }
                },
                common,
            )
        }
        ElementType::Checkbox { checked, common } => (
            quote! {
                #dal.new_checkbox(#parent, #checked)
            },
            common,
        ),
        ElementType::TextInput { text, common } => (
            quote! {
                #dal.new_text_input(#parent, #text.to_string())
            },
            common,
        ),
    };

    let element_ident = if let Some(ident) = binding {
        ident.clone()
    } else {
        quote::format_ident!("_el")
    };

    let mut common_code = Vec::new();
    if let Some(on_click) = &common.on_click {
        common_code.push(quote! { #dal.on_click(#element_ident, #on_click); });
    }
    if let Some(on_hover) = &common.on_hover {
        common_code.push(quote! { #dal.on_hover(#element_ident, #on_hover); });
    }

    if let Some(ident) = binding {
        quote! {
            let #ident = #creation_code;
            #( #common_code )*
            #ident
        }
    } else {
        quote! {
            {
                let #element_ident = #creation_code;
                #( #common_code )*
                #element_ident
            }
        }
    }
}
