use crate::{
    generator::{
        Generator,
    },
    parser::types::{StructField, StructFieldSource, TypeModifier},
};

pub trait StructFieldGenerator {
    fn generate(&self, entity: &StructField, gen: &Generator) -> String {
        if entity.type_modifiers.contains(&TypeModifier::Empty) {
            return "".into();
        }
        let mut extra_macro = "";
        // This would be incorrectly treated as std::string::String
        if entity.type_name == "string" {
            extra_macro = ", force_struct";
        }

        format!(
            "{comment}{macros}{indent}pub {name}: {typename},",
            comment = self.format_comment(entity, gen),
            macros = self.macros(entity, gen, extra_macro),
            indent = gen.base().indent(),
            name = self.get_name(entity, gen),
            typename = self.get_type_name(entity, gen),
        )
    }

    fn get_type_name(&self, entity: &StructField, gen: &Generator) -> String {
        gen.base()
            .modify_type(
                gen.base().format_type_name(entity.type_name.as_str(), gen).as_ref(),
                &entity.type_modifiers,
            )
            .into()
    }

    fn get_name(&self, entity: &StructField, gen: &Generator) -> String {
        gen.base().format_name(entity.name.as_str()).into()
    }

    fn format_comment(&self, entity: &StructField, gen: &Generator) -> String {
        gen.base().format_comment(entity.comment.as_deref(), gen.base().indent_size())
    }

    fn macros(&self, _entity: &StructField, _gen: &Generator, _extra: &str) -> String {
        /*
        let indent = gen.base().indent();
        if entity.type_modifiers.contains(&TypeModifier::Flatten) {
            yaserde_for_flatten_element(indent.as_str(), extra)
        } else {
            match entity.source {
                StructFieldSource::Choice | StructFieldSource::Sequence => {
                    yaserde_for_flatten_element(indent.as_str(), extra)
                }
                StructFieldSource::Attribute => {
                    yaserde_for_attribute(entity.name.as_str(), indent.as_str(), extra)
                }
                StructFieldSource::Element => yaserde_for_element(
                    entity.name.as_str(),
                    gen.target_ns.borrow().as_ref(),
                    indent.as_str(),
                    extra,
                ),
                _ => {
                    if extra.len() > 0 {
                        format!("{indent}#[yaserde({extra})]", indent = gen.base().indent())
                    } else {
                        "".into()
                    }
                }
            }
        }
        */
        String::new()
    }

    fn deserialize(&self, field: &StructField, gen: &Generator) -> String {
        let mut flatten =
            matches!(field.source, StructFieldSource::Choice | StructFieldSource::Sequence);

        let attribute = matches!(field.source, StructFieldSource::Attribute);

        let mut field_getter = String::new();
        for modifier in &field.type_modifiers {
            let ty = if field_getter.is_empty() { "popper" } else { "inter" };

            let pop_func = match modifier {
                crate::parser::types::TypeModifier::None => None,
                crate::parser::types::TypeModifier::Array => {
                    Some(if attribute { "pop_attributes" } else { "pop_children" })
                }
                crate::parser::types::TypeModifier::Option => {
                    Some(if attribute { "maybe_pop_attribute" } else { "maybe_pop_child" })
                }
                crate::parser::types::TypeModifier::Recursive => {
                    // if field.type_modifiers.contains(&crate::parser::types::TypeModifier::Option) ||
                    //  field.type_modifiers.contains(&crate::parser::types::TypeModifier::Array) {
                    //     None
                    // } else {
                    //     Some(if attribute { "pop_boxed_attribute" } else { "pop_boxed_child" })
                    // }
                    None
                }
                crate::parser::types::TypeModifier::Empty => None,
                crate::parser::types::TypeModifier::Flatten => {
                    flatten = true;
                    None
                }
            };

            if let Some(pop_func) = pop_func {
                field_getter
                    .push_str(&format!("let inter = {ty}.{pop_func}(\"{}\")?;\n", field.name));
            }
        }

        if field_getter.is_empty() {
            if attribute {
                field_getter = format!("let inter = popper.pop_attribute(\"{}\")?;\n", field.name);
            } else {
                field_getter = format!("let inter = popper.pop_child(\"{}\")?;\n", field.name);
            }
        }

        if flatten {
            // Complex case...
            // we need to clone the popper, and if the nested call is successful replace our main popper
            // if unsuccessful we will just return without changing our primary popper.
            format!(
                r#"
                    let mut inter = popper.clone();
                    let result = |popper: &mut XmlPopper| {{
                            <{} as XmlDeserialize>::xml_deserialize(popper)
                    }};

                    let field = match (result)(&mut inter) {{
                        Ok(result) => {{
                            result
                        }}
                        Err(err) => {{
                            return Err(err);
                        }}
                    }};
                    popper = inter;
                "#,
                self.get_type_name(field, gen)
            )
        } else {
            format!("{field_getter}\nlet field = inter;")
        }
    }
}

pub struct DefaultStructFieldGen;
impl StructFieldGenerator for DefaultStructFieldGen {}
