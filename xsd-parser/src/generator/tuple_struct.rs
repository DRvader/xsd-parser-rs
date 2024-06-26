use std::borrow::Cow;

use crate::{
    generator::{
        validator::{gen_facet_validation, gen_validate_impl},
        Generator,
    },
    parser::{types::TupleStruct, xsd_elements::FacetType},
};

pub trait TupleStructGenerator {
    fn generate(&self, entity: &TupleStruct, gen: &Generator) -> String {
        let display_gen = format!(
            r#"impl std::fmt::Display for {name} {{
{indent}fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{
{indent}{indent}write!(f, "{{}}", self.0)
{indent}}}
}}"#,
            name = self.get_name(entity, gen),
            indent = gen.base().indent()
        );

        let parse_gen = format!(
            r#"impl std::str::FromStr for {name} {{
type Err = std::convert::Infallible;
{indent}fn from_str(s: &str) -> Result<Self, Self::Err> {{
{indent}{indent}let output = {name}(s.parse().unwrap());
{indent}{indent}Ok(output)
{indent}}}
}}"#,
            indent = gen.base().indent(),
            name = self.get_name(entity, gen)
        );

        format!(
            "{comment}{macros}pub struct {name} (pub {typename});\n{display_gen}\n{parse_gen}\n{validation}\n{deserialize}\n{subtypes}\n",
            comment = self.format_comment(entity, gen),
            name = self.get_name(entity, gen),
            macros = self.macros(entity, gen),
            typename = self.get_type_name(entity, gen),
            subtypes = self.subtypes(entity, gen),
            validation = self.validation(entity, gen),
            deserialize = self.deserialize(entity, gen)
        )
    }

    fn deserialize(&self, entity: &TupleStruct, gen: &Generator) -> String {
        format!(
            r#"impl XmlDeserialize for {typename} {{
              fn xml_deserialize(popper: &mut XmlPopper) -> Result<Self, DeError> {{
            Ok({typename}(popper.pop_child("{name}")?))
        }}
    }}"#,
            typename = self.get_name(entity, gen),
            name = entity.name
        )
    }

    fn subtypes(&self, entity: &TupleStruct, gen: &Generator) -> String {
        gen.base().join_subtypes(entity.subtypes.as_ref(), gen)
    }

    fn get_type_name(&self, entity: &TupleStruct, gen: &Generator) -> String {
        gen.base()
            .modify_type(
                gen.base().format_type_name(entity.type_name.as_str(), gen).as_ref(),
                &entity.type_modifiers,
            )
            .into()
    }

    fn get_name(&self, entity: &TupleStruct, gen: &Generator) -> String {
        gen.base().format_type_name(entity.name.as_str(), gen).into()
    }

    fn macros(&self, entity: &TupleStruct, _gen: &Generator) -> Cow<'static, str> {
        let extra = if entity.facets.iter().any(|f| {
            matches!(
                f.facet_type,
                FacetType::MinExclusive(_)
                    | FacetType::MaxExclusive(_)
                    | FacetType::MinInclusive(_)
                    | FacetType::MaxInclusive(_)
            )
        }) {
            ", PartialOrd"
        } else {
            ""
        };

        // HACK(drosen): Just to get validation working
        let extra = if entity.type_name == "xs:decimal" { ", PartialOrd" } else { extra };

        format!("#[derive(Default, PartialEq, Debug{extra})]\n").into()
    }

    fn format_comment(&self, entity: &TupleStruct, gen: &Generator) -> String {
        gen.base().format_comment(entity.comment.as_deref(), 0)
    }

    fn validation(&self, entity: &TupleStruct, gen: &Generator) -> Cow<'static, str> {
        let body = entity
            .facets
            .iter()
            .map(|f| gen_facet_validation(&f.facet_type, "0", &self.get_type_name(entity, gen)))
            .fold(String::new(), |x, y| (x + &y));
        Cow::Owned(gen_validate_impl(self.get_name(entity, gen).as_str(), body.as_str()))
    }
}

pub struct DefaultTupleStructGen;
impl TupleStructGenerator for DefaultTupleStructGen {}
