use std::borrow::Cow;

use crate::{
    generator::{validator::gen_validate_impl, Generator},
    parser::types::{Enum, EnumSource},
};

pub trait EnumGenerator {
    fn generate(&self, entity: &Enum, gen: &Generator) -> String {
        let name = self.get_name(entity, gen);
        let default_case = format!(
            "impl Default for {name} {{\n\
            {indent}fn default() -> {name} {{\n\
            {indent}{indent}Self::__Unknown__(Default::default())\n\
            {indent}}}\n\
            }}",
            name = name,
            indent = gen.base().indent()
        );

        let mut display_contents = String::new();
        let mut parse_contents = String::new();
        let mut easy_display = true;

        let mod_name = self.mod_name(entity, gen);
        for case in &entity.cases {
            let mut case = case.clone();

            if case.type_name.is_some() {
                easy_display = false;
                break;
            }

            if let Some(tn) = &mut case.type_name {
                if !case.subtypes.is_empty() {
                    *tn = format!("{}::{}", mod_name, gen.base().format_type_name(tn.as_str(), gen))
                }
            }

            // let type_name = gen.enum_case_gen().get_type_name(&case, gen);
            let name = gen.enum_case_gen().get_name(&case, gen);

            display_contents.push_str(&format!(
                "{indent}{indent}{indent}Self::{} => \"{}\".to_string(),\n",
                name,
                case.name,
                indent = gen.base().indent()
            ));
            parse_contents.push_str(&format!(
                "{indent}{indent}{indent}\"{}\" => Self::{},\n",
                case.name,
                name,
                indent = gen.base().indent()
            ));
        }

        // For now we will only generate for unit enums
        let display_enum = if easy_display {
            format!(
                r#"impl std::fmt::Display for {name} {{
{indent}fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{
{indent}{indent}write!(f, "{{}}", match &self {{
{display_contents}
{indent}{indent}{indent}Self::__Unknown__(val) => ::std::format!("__Unknown__({{val}})"),
{indent}{indent}}})
{indent}}}
}}"#,
                indent = gen.base().indent()
            )
        } else {
            "".into()
        };
        let parse_enum = if easy_display && false {
            format!(
                r#"impl std::str::FromStr for {name} {{
type Err = std::convert::Infallible;
{indent}fn from_str(s: &str) -> Result<Self, Self::Err> {{
{indent}{indent}let output = match s {{
{parse_contents}
{indent}{indent}{indent}val => Self::__Unknown__(<{typename} as std::str::FromStr>::from_str(val)?),
{indent}{indent}}};
{indent}{indent}Ok(output)
{indent}}}
}}"#,
                indent = gen.base().indent(),
                typename = self.get_type_name(entity, gen)
            )
        } else {
            "".into()
        };

        format!(
            "{comment}{macros}\n\
            pub enum {name} {{\n\
                {cases}\n\
                {indent}__Unknown__({typename}),\n\
            }}\n\n\
            {default}\n\n\
            {display_enum}\n\n\
            {parse_enum}\n\n\
            {validation}\n\n\
            {deserialize}\n\n\
            {subtypes}\n\n",
            indent = gen.base().indent(),
            comment = self.format_comment(entity, gen),
            macros = self.macros(entity, gen),
            name = name,
            cases = self.cases(entity, gen),
            typename = self.get_type_name(entity, gen),
            default = default_case,
            subtypes = self.subtypes(entity, gen),
            validation = self.validation(entity, gen),
            deserialize = self.deserialize(entity, gen)
        )
    }

    fn deserialize(&self, entity: &Enum, gen: &Generator) -> String {
        let mod_name = self.mod_name(entity, gen);

        let mut cases = String::new();
        let mut case_gens = String::new();
        for (index, mut case) in entity.cases.iter().cloned().enumerate() {
            if let Some(tn) = &mut case.type_name {
                if !case.subtypes.is_empty() {
                    *tn = format!("{}::{}", mod_name, gen.base().format_type_name(tn.as_str(), gen))
                }
            }

            let (case_de, case_assign) = gen.enum_case_gen().deserialize(&case, gen);

            cases.push_str(&case_de);

            let mut case_gen = String::new();
            case_gen.push('(');
            for i in 0..entity.cases.len() {
                if i == index {
                    case_gen.push_str("core::option::Option::Some(value), ");
                } else {
                    case_gen.push_str("core::option::Option::None, ");
                }
            }
            case_gen.push_str(") => {\n");

            case_gen.push_str("\n");

            case_gen.push_str(&case_assign);

            case_gen.push_str("\n");

            case_gen.push_str("}\n");

            case_gens.push_str(&case_gen);
        }

        case_gens.push_str(&format!(
            r#"_ => {{
            return Err(DeError::User("Found multiple possible matches {}".to_string()));
        }}"#,
            entity.name
        ));

        format!(
            r#"
            impl XmlDeserialize for {} {{
            fn xml_deserialize(outer_popper: &mut XmlPopper) -> Result<Self, DeError> {{
                let mut popper = outer_popper.recursive_clone();

                let results = ({cases});

                let output = match results {{
                    {case_gens}
                }};

                *outer_popper = popper;

                Ok(output)
        }}}}"#,
            self.get_name(entity, gen)
        )
    }

    fn cases(&self, entity: &Enum, gen: &Generator) -> String {
        let mod_name = self.mod_name(entity, gen);

        let cases = entity
            .cases
            .iter()
            .cloned()
            .map(|mut f| {
                if let Some(tn) = &mut f.type_name {
                    if !f.subtypes.is_empty() {
                        *tn = format!(
                            "{}::{}",
                            mod_name,
                            gen.base().format_type_name(tn.as_str(), gen)
                        )
                    }
                }

                f
            })
            .map(|f| gen.enum_case_gen().generate(&f, gen))
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
            .join("\n");

        cases
    }

    fn subtypes(&self, entity: &Enum, gen: &Generator) -> String {
        let case_subtypes = entity
            .cases
            .iter()
            .map(|f| gen.base().join_subtypes(f.subtypes.as_ref(), gen))
            .collect::<Vec<String>>()
            .join("");

        let subtypes = gen.base().join_subtypes(entity.subtypes.as_ref(), gen);

        if !case_subtypes.is_empty() || !subtypes.is_empty() {
            format!(
                "\npub mod {name} {{\n{indent}use super::*;\n{st}\n{cst}}}\n",
                name = self.mod_name(entity, gen),
                st = subtypes,
                indent = gen.base().indent(),
                cst = self.shift(&case_subtypes, gen.base().indent().as_str())
            )
        } else {
            format!("{}\n{}", subtypes, case_subtypes)
        }
    }

    fn mod_name(&self, entity: &Enum, gen: &Generator) -> String {
        gen.base().mod_name(entity.name.as_str())
    }

    fn shift(&self, text: &str, indent: &str) -> String {
        text.replace("\n\n\n", "\n") // TODO: fix this workaround replace
            .split('\n')
            .map(|s| if !s.is_empty() { format!("\n{}{}", indent, s) } else { "\n".to_string() })
            .fold(indent.to_string(), |acc, x| acc + &x)
    }

    fn get_type_name(&self, entity: &Enum, gen: &Generator) -> String {
        gen.base().format_type_name(entity.type_name.as_str(), gen).into()
    }

    fn get_name(&self, entity: &Enum, gen: &Generator) -> String {
        gen.base().format_type_name(entity.name.as_str(), gen).into()
    }

    fn macros(&self, entity: &Enum, _gen: &Generator) -> Cow<'static, str> {
        if entity.source == EnumSource::Union {
            return "#[derive(PartialEq, Debug)]".into();
        }

        let derives = "#[derive(PartialEq, Debug)]";
        // let tns = gen.target_ns.borrow();
        // match tns.as_ref() {
        //     Some(tn) => match tn.name() {
        //         Some(name) => format!(
        //             "{derives}\n#[yaserde(prefix = \"{prefix}\", namespace = \"{prefix}: {uri}\")]",
        //             derives = derives,
        //             prefix = name,
        //             uri = tn.uri()
        //         ),
        //         None => format!(
        //             "{derives}\n#[yaserde(namespace = \"{uri}\")]",
        //             derives = derives,
        //             uri = tn.uri()
        //         ),
        //     },
        //     None => format!("{derives}", derives = derives),
        // }
        // .into()
        derives.into()
    }

    fn format_comment(&self, entity: &Enum, gen: &Generator) -> String {
        gen.base().format_comment(entity.comment.as_deref(), 0)
    }

    fn validation(&self, entity: &Enum, gen: &Generator) -> Cow<'static, str> {
        // Empty validation
        Cow::Owned(gen_validate_impl(self.get_name(entity, gen).as_str(), ""))
    }
}

pub struct DefaultEnumGen;
impl EnumGenerator for DefaultEnumGen {}
