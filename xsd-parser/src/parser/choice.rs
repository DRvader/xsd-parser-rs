use roxmltree::Node;

use crate::parser::{
    node_parser::parse_node,
    types::{Enum, EnumSource, RsEntity},
    xsd_elements::{ElementType, XsdNode},
};

use super::types::EnumCase;

pub fn parse_choice(choice: &Node) -> RsEntity {
    let mut sub_type_count = 0;
    let enum_cases = choice
        .children()
        .filter(|n| {
            n.is_element()
                && (n.xsd_type() == ElementType::Element || n.xsd_type() == ElementType::Sequence)
        })
        .map(|n| match parse_node(&n, choice) {
            RsEntity::EnumCase(case) => case,
            RsEntity::Struct(mut st) => {
                let name = if sub_type_count > 0 {
                    format!("{}{}", st.name, sub_type_count)
                } else {
                    st.name.clone()
                };
                sub_type_count += 1;

                st.name = name;

                EnumCase {
                    name: st.name.clone(),
                    type_name: Some(st.name.to_string()),
                    source: EnumSource::Choice,
                    subtypes: vec![RsEntity::Struct(st)],
                    ..Default::default()
                }
            }
            _ => unreachable!("Elements in choice must be a enum variants"),
        })
        .collect();

    RsEntity::Enum(Enum {
        cases: enum_cases,
        type_name: "std::string::String".to_string(),
        source: EnumSource::Choice,
        ..Default::default()
    })
}
