use std::cell::RefCell;

use roxmltree::Node;

use crate::parser::{
    node_parser::parse_node,
    types::{RsEntity, Struct, StructField, TypeModifier},
    utils::{enum_to_field, get_documentation, get_parent_name},
    xsd_elements::{ElementType, XsdNode},
};

use super::{
    element::element_modifier,
    utils::{attribute_groups_to_aliases, groups_to_aliases},
};

pub fn parse_sequence(sequence: &Node, parent: &Node) -> RsEntity {
    let name = get_parent_name(sequence);

    let parent_is_ref = parent.attr_ref().is_some();

    RsEntity::Struct(Struct {
        name: name.into(),
        comment: get_documentation(parent),
        subtypes: vec![],
        fields: RefCell::new(elements_to_fields(sequence, name, parent_is_ref, parent)),
        attribute_groups: RefCell::new(attribute_groups_to_aliases(sequence)),
        groups: RefCell::new(groups_to_aliases(sequence)),
        ..Default::default()
    })
}

fn elements_to_fields(sequence: &Node, parent_name: &str, _parent_is_ref: bool, _parent: &Node) -> Vec<StructField> {
    let mut choice_count = 0;
    sequence
        .children()
        .filter(|n| {
            n.is_element()
                && n.xsd_type() != ElementType::Annotation
                && n.xsd_type() != ElementType::Group
                && n.xsd_type() != ElementType::AttributeGroup
        })
        .map(|n| match parse_node(&n, sequence) {
            RsEntity::StructField(sf) => {
                // if sf.type_name.ends_with(parent_name) && !parent_is_ref {
                //     sf.type_modifiers.push(TypeModifier::Recursive);
                //     if let Some(comment) = &mut sf.comment {
                //         comment.push_str(&format!("{:?}", parent));
                //     } else {
                //         sf.comment = Some(parent_name.to_string());
                //     }
                // }
                sf
            }
            RsEntity::Enum(mut en) => {
                en.name = format!("{}Choice{}", parent_name, choice_count);
                choice_count += 1;
                let mut en = enum_to_field(en);
                en.type_modifiers.push(TypeModifier::Flatten);

                en
            }
            RsEntity::Alias(alias) => StructField {
                name: alias.name,
                type_name: alias.original,
                comment: alias.comment,
                subtypes: alias.subtypes,
                ..Default::default()
            },
            RsEntity::Struct(st) => StructField {
                name: st.name.clone(),
                type_name: st.name.clone(),
                source: super::types::StructFieldSource::Sequence,
                subtypes: vec![RsEntity::Struct(st)],
                type_modifiers: vec![element_modifier(&n)],
                ..Default::default()
            },
            _ => unreachable!("\nError: {:?}\n{:?}", n, parse_node(&n, sequence)),
        })
        .collect()
}
