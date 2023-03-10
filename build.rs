use convert_case::{Case, Casing};
use std::{env, fs::File, path::Path};

use genco::{
    fmt,
    prelude::{rust::Tokens, *},
};

type RulesList = [&'static str];

const EXPRESSIONS: &'static RulesList = &[
    "Assign   : Token name, Expr value",
    "Binary   : Expr left, Token operator, Expr right",
    "Call     : Expr callee, Vec<Expr> arguments",
    "Get      : Expr object, Token name",
    "Grouping : Expr expression",
    "Literal  : Literal value",
    "Logical  : Expr left, Token operator, Expr right",
    "Set      : Expr object, Token name, Expr value",
    "Super    : Token keyword, Token method",
    "This     : Token keyword",
    "Unary    : Token operator, Expr right",
    "Variable : Token name",
];

const STATEMENTS: &'static RulesList = &[
    "Block      : Vec<Stmt> statements",
    "Class      : Token name, Option<VariableExpr> superclass, Vec<FunctionStmt> methods",
    "Expression : Expr expression",
    "Function   : Token name, Vec<Token> params, Vec<Stmt> body",
    "If         : Expr condition, Stmt then_branch, Stmt else_branch",
    "Print      : Expr expression",
    "Return     : Expr value",
    "Var        : Token name, Expr initializer",
    "While      : Expr condition, Stmt body",
];

fn main() -> anyhow::Result<()> {
    define_ast("expr", EXPRESSIONS)?;
    define_ast("stmt", STATEMENTS)?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

fn define_ast(base: &str, rules: &RulesList) -> anyhow::Result<()> {
    let token = rust::import("crate::tokens", "Token");

    let base_snake = &base.to_case(Case::Snake);
    let base_title = &base.to_case(Case::Title);

    let tokens: rust::Tokens = quote! {
        mod $(base_snake)_generated {
            type Token = super::$token;

            $(optional_imports(base_snake))

            pub(crate) trait Visitor<T> {
                $(define_visitor_trait(base_title, base_snake, rules))
            }

            #[derive(Clone, Debug, Hash, Eq, PartialEq)]
            pub(crate) enum $(base_title) {
                $(define_enum(base_title, rules))
            }

            pub(crate) fn walk_$(base_snake)<T>(visitor: &dyn Visitor<T>, $(base_snake): &$(base_title)) -> T {
                match $(base_snake) {
                    $(define_walk(base_title, rules))
                }
            }

            $(define_structs(base_title, rules))
        }
    };

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(format!("{}_generated.rs", base_snake));
    let file = File::create(dest_path)?;

    let mut w = fmt::IoWriter::new(file);
    let fmt = fmt::Config::from_lang::<Rust>().with_indentation(fmt::Indentation::Space(4));
    let config = rust::Config::default().with_default_import(rust::ImportMode::Direct);
    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;

    Ok(())
}

fn optional_imports(base_snake: &str) -> Tokens {
    match base_snake {
        "expr" => {
            let literal = rust::import("crate::tokens", "Literal");

            quote! {
                type Literal = super::$literal;
            }
        }
        "stmt" => {
            let expr = rust::import("crate::expr", "Expr");
            let variable_expr = rust::import("crate::expr", "VariableExpr");

            quote! {
                type Expr = super::$expr;
                type VariableExpr = super::$variable_expr;
            }
        }
        _ => quote! {},
    }
}

fn define_visitor_trait(base_title: &str, base_snake: &str, rules: &RulesList) -> Tokens {
    let mut tokens = Tokens::new();

    for rule in rules.iter() {
        let raw_token_name = rule.split_once(" ").unwrap().0;

        let token_snake = &raw_token_name.to_case(Case::Snake);
        let token_title = &raw_token_name.to_case(Case::Title);

        tokens.append(quote! {
            fn visit_$token_snake(&self, $base_snake: &$token_title$base_title) -> T;
        });
    }

    return tokens;
}

fn define_enum(base_title: &str, rules: &RulesList) -> Tokens {
    let mut tokens = Tokens::new();

    for rule in rules.iter() {
        let title = &rule.split_once(" ").unwrap().0.to_case(Case::Title);

        tokens.append(quote! {
            $title($title$base_title),
        })
    }

    return tokens;
}

fn define_walk(base_title: &str, rules: &RulesList) -> Tokens {
    let mut tokens = Tokens::new();

    for rule in rules.iter() {
        let raw_token_name = rule.split_once(" ").unwrap().0;

        let var = &raw_token_name.to_case(Case::Snake);
        let class = &raw_token_name.to_case(Case::Title);

        tokens.append(quote! {
            $(base_title)::$class(v) => visitor.visit_$var(&v),
        })
    }

    return tokens;
}

fn define_structs(base_title: &str, rules: &RulesList) -> Tokens {
    let mut tokens = Tokens::new();

    for rule in rules.iter() {
        tokens.append(define_type(base_title, rule));
    }

    return tokens;
}

struct Field {
    type_name: String,
    name: String,
}

fn define_type(base_title: &str, rule: &str) -> Tokens {
    let (raw_name, raw_rules) = rule.split_once(":").unwrap();

    let name_title = &raw_name.trim().to_case(Case::Title);

    let class = &format!("{}{}", name_title, base_title);
    let fields: Vec<Field> = raw_rules.split(", ").map(parse_field).collect();

    quote! {
        #[derive(Clone, Debug, Hash, Eq, PartialEq)]
        pub(crate) struct $class {
            $("// each instance needs an id to make it unique when we hash it")
            $("// otherwise two variables with the same name will hash the same")
            $("// causing the resolver to mess up for loops")
            pub id: usize,
            $(define_struct_fields(&fields))
        }

        impl $class {
            pub(crate) fn new(id: usize, $(define_constructor_parameters(&fields))) -> $class {
                $class {
                    id: id,
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
            "Stmt" => "Box<Stmt>",
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
            "Stmt" => format!("Box::new({})", name),
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
