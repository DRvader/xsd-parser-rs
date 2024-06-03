use crate::{
    generator::{default::default_format_type, utils::split_name, Generator},
    parser::types::{EnumCase, EnumSource},
};

pub trait EnumCaseGenerator {
    fn generate(&self, entity: &EnumCase, gen: &Generator) -> String {
        let typename = if entity.type_name.is_some() {
            let mut output = format!("({})", self.get_type_name(entity, gen));
            if let Some(name) = &entity.type_name {
                // This would be incorrectly treated as std::string::String
                // if name == "string" {
                //     output =
                //         format!("(#[yaserde(force_struct)] {})", self.get_type_name(entity, gen));
                // }
            }

            output
        } else {
            "".into()
        };
        format!(
            "{comment}{macros}{indent}{name}{typename},",
            indent = gen.base().indent(),
            name = self.get_name(entity, gen),
            comment = self.format_comment(entity, gen),
            macros = self.macros(entity, gen, ""),
            typename = typename
        )
    }

    fn get_name(&self, entity: &EnumCase, gen: &Generator) -> String {
        default_format_type(entity.name.as_str(), &gen.target_ns.borrow())
            .split("::")
            .last()
            .expect(&format!("Couldn't format {}", entity.name))
            .to_string()
    }

    fn get_type_name(&self, entity: &EnumCase, gen: &Generator) -> String {
        let formatted_type = gen.base().format_type_name(entity.type_name.as_ref().unwrap(), gen);
        gen.base().modify_type(formatted_type.as_ref(), &entity.type_modifiers).into()
    }

    fn format_comment(&self, entity: &EnumCase, gen: &Generator) -> String {
        gen.base().format_comment(entity.comment.as_deref(), gen.base().indent_size())
    }

    fn macros(&self, entity: &EnumCase, gen: &Generator, extra: &str) -> String {
        if entity.source == EnumSource::Union {
            return "".into();
        }

        /*
        let (prefix, field_name) = split_name(entity.name.as_str());
        match prefix {
            Some(p) => format!(
                "{indent}#[yaserde(prefix = \"{prefix}\", rename = \"{rename}\"{extra})]\n",
                indent = gen.base().indent(),
                prefix = p,
                rename = field_name
            ),
            None => {
                if field_name == self.get_name(entity, gen) {
                    if extra.len() > 0 {
                        format!("{indent}#[yaserde({extra})]", indent = gen.base().indent())
                    } else {
                        "".into()
                    }
                } else {
                    format!(
                        "{indent}#[yaserde(rename = \"{rename}\"{extra})]\n",
                        indent = gen.base().indent(),
                        rename = field_name
                    )
                }
            }
        }
        */

        String::new()
    }

    fn deserialize(&self, case: &EnumCase, gen: &Generator) -> (String, String) {
        let case_getter = if case.source == EnumSource::Union || case.type_name.is_none() {
            // special case: we are just parsing the value

            format!(
                r#"
                let value = popper.pop_value()?;
                if value == "{}" {{
                    core::option::Option::Some(value)
                }} else {{
                    core::option::Option::None
                }}
                "#,
                case.name
            )
        } else {
            let mut case_getter = String::new();

            let mut flatten = false;
            for modifier in &case.type_modifiers {
                let ty = if case_getter.is_empty() { "popper" } else { "inter" };

                let pop_func = match modifier {
                    crate::parser::types::TypeModifier::None => None,
                    crate::parser::types::TypeModifier::Array => Some("pop_children"),
                    crate::parser::types::TypeModifier::Option => Some("maybe_pop_child"),
                    crate::parser::types::TypeModifier::Recursive => Some("pop_child"),
                    crate::parser::types::TypeModifier::Empty => None,
                    crate::parser::types::TypeModifier::Flatten => {
                        flatten = true;
                        None
                    }
                };

                if let Some(pop_func) = pop_func {
                    case_getter
                        .push_str(&format!("let inter = {ty}.{pop_func}(\"{}\")?;\n", case.name));
                }
            }

            if flatten {
                case_getter = format!("{}::xml_deserialize(popper)", self.get_type_name(case, gen));
            } else if case_getter.is_empty() {
                case_getter = format!("let inter = popper.pop_child(\"{}\")?;\nOk::<_, DeError>(inter)\n", case.name);
            }

            case_getter
        };

        let assign = if let Some(_tn) = &case.type_name {
            format!("Self::{}(value)", self.get_name(case, gen))
        } else {
            // No typename means we can ignore the result
            format!("Self::{}", self.get_name(case, gen))
        };

        (
            format!(
                r#"
                    {{
                        let mut inter = popper.clone();
                        let result = |popper: &mut XmlPopper| {{
                            {case_getter}
                        }};

                        let field = match (result)(&mut inter) {{
                            Ok(result) => {{
                                core::option::Option::Some(result)
                            }}
                            Err(err) => {{
                                core::option::Option::None
                            }}
                        }};

                        popper = inter;

                        field
                    }},
                "#
            ),
            assign,
        )
    }
}

pub struct DefaultEnumCaseGen;
impl EnumCaseGenerator for DefaultEnumCaseGen {}
