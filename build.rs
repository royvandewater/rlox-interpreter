use convert_case::{Case, Casing};
use std::{env, fs::File, path::Path};

use genco::{
    fmt,
    prelude::{rust::Tokens, *},
};

const RULES: &'static [&'static str] = &[
    "Binary   : Expr left, Token operator, Expr right",
    "Grouping : Expr expression",
    "Literal  : Literal value",
    "Unary    : Token operator, Expr right",
];

fn main() -> anyhow::Result<()> {
    let literal = rust::import("crate::tokens", "Literal");
    let token = rust::import("crate::tokens", "Token");

    let tokens: rust::Tokens = quote! {
        mod expr_generated {
            type Literal = super::$literal;
            type Token = super::$token;

            pub(crate) trait Visitor<T> {
                $(define_visitor_trait())
            }

            pub(crate) enum Expr {
                $(define_expr_enum())
            }

            pub(crate) fn walk_expr<T>(visitor: &dyn Visitor<T>, expr: Expr) -> T {
                match expr {
                    $(define_walk_expr())
                }
            }

            $(define_exprs())
        }
    };

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("expr_generated.rs");
    let file = File::create(dest_path)?;

    let mut w = fmt::IoWriter::new(file);
    let fmt = fmt::Config::from_lang::<Rust>().with_indentation(fmt::Indentation::Space(4));
    let config = rust::Config::default().with_default_import(rust::ImportMode::Direct);
    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

fn define_visitor_trait() -> Tokens {
    let mut tokens = Tokens::new();

    for rule in RULES.iter() {
        let raw_token_name = rule.split_once(" ").unwrap().0;

        let token_snake = &raw_token_name.to_case(Case::Snake);
        let token_title = &raw_token_name.to_case(Case::Title);

        tokens.append(quote! {
            fn visit_$token_snake(&self, expr: $(token_title)Expr) -> T;
        })
    }

    return tokens;
}

fn define_expr_enum() -> Tokens {
    let mut tokens = Tokens::new();

    for rule in RULES.iter() {
        let title = &rule.split_once(" ").unwrap().0.to_case(Case::Title);

        tokens.append(quote! {
            $title($(title)Expr),
        })
    }

    return tokens;
}

fn define_walk_expr() -> Tokens {
    let mut tokens = Tokens::new();

    for rule in RULES.iter() {
        let raw_token_name = rule.split_once(" ").unwrap().0;

        let var = &raw_token_name.to_case(Case::Snake);
        let class = &raw_token_name.to_case(Case::Title);
        let c = &var.chars().next().unwrap().to_string();

        tokens.append(quote! {
            Expr::$class($c) => visitor.visit_$var($c),
        })
    }

    return tokens;
}

fn define_exprs() -> Tokens {
    let mut tokens = Tokens::new();

    for rule in RULES.iter() {
        tokens.append(define_type(rule));
    }

    return tokens;
}

struct Field {
    type_name: String,
    name: String,
}

fn define_type(rule: &str) -> Tokens {
    let (raw_name, raw_rules) = rule.split_once(":").unwrap();

    let class = format!("{}Expr", &raw_name.trim().to_case(Case::Title));
    let fields: Vec<Field> = raw_rules.split(", ").map(parse_field).collect();

    quote! {
        pub(crate) struct $(&class) {
            $(define_struct_fields(&fields))
        }

        impl $(&class) {
            pub(crate) fn new($(define_constructor_parameters(&fields))) -> $(&class) {
                $(&class) {
                    $(define_constructor_assignment(&fields))
                }
            }
        }
    }
}

fn define_struct_fields(fields: &Vec<Field>) -> Tokens {
    let mut tokens = Tokens::new();

    for field in fields {
        let name = &field.name;
        let type_name = match field.type_name.as_str() {
            "Expr" => "Box<Expr>",
            v => v,
        };

        tokens.append(quote! {
            pub $name: $type_name,
        });
    }

    return tokens;
}

fn define_constructor_parameters(fields: &Vec<Field>) -> Tokens {
    let mut tokens = Tokens::new();

    for field in fields {
        let name = &field.name;
        let type_name = &field.type_name;

        tokens.append(quote! {
            $name: $type_name,
        });
    }

    return tokens;
}

fn define_constructor_assignment(fields: &Vec<Field>) -> Tokens {
    let mut tokens = Tokens::new();

    for field in fields {
        let name = &field.name;
        let assigned_name = match field.type_name.as_str() {
            "Expr" => format!("Box::new({})", name),
            _ => name.to_string(),
        };

        tokens.append(match name == &assigned_name {
            true => quote! { $name, },
            false => quote! { $name: $assigned_name, },
        });
    }

    return tokens;
}

fn parse_field(field: &str) -> Field {
    let (type_name, name) = field.trim().split_once(" ").unwrap();

    Field {
        type_name: type_name.to_string(),
        name: name.to_string(),
    }
}
