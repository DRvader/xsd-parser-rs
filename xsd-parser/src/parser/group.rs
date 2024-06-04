use roxmltree::Node;

use super::{
    node_parser::parse_node,
    types::{Alias, RsEntity, Struct, TypeModifier},
    utils::get_documentation,
    xsd_elements::{ElementType, XsdNode, MaxOccurs, min_occurs, max_occurs},
};

pub fn group_modifier(node: &Node) -> TypeModifier {
    let min = min_occurs(node);
    let max = max_occurs(node);
    match min {
        0 => match max {
            MaxOccurs::None => TypeModifier::Option,
            MaxOccurs::Unbounded => TypeModifier::Array,
            MaxOccurs::Bounded(val) => {
                if val > 1 {
                    TypeModifier::Array
                } else {
                    TypeModifier::None
                }
            }
        },
        1 => match max {
            MaxOccurs::None => TypeModifier::None,
            MaxOccurs::Unbounded => TypeModifier::Array,
            MaxOccurs::Bounded(val) => {
                if val > 1 {
                    TypeModifier::Array
                } else {
                    TypeModifier::None
                }
            }
        },
        _ => TypeModifier::Array,
    }
}

pub fn parse_group(node: &Node, parent: &Node) -> RsEntity {
    if parent.xsd_type() == ElementType::Schema {
        return parse_global_group(node);
    }

    let reference = node.attr_ref().expect("Non-global groups must be references.").to_string();
    let modifier = group_modifier(node);

    if modifier != TypeModifier::None {
        RsEntity::Alias(Alias {
            name: reference.to_string(),
            original: reference,
            comment: get_documentation(node),
            type_modifiers: vec![modifier],
            ..Default::default()
        })
    } else {
        RsEntity::Alias(Alias {
            name: reference.to_string(),
            original: reference,
            comment: get_documentation(node),
            ..Default::default()
        })
    }
}

fn parse_global_group(node: &Node) -> RsEntity {
    let name = node.attr_name().unwrap_or_else(|| panic!("Name attribute required. {:?}", node));

    let subtypes = node
        .children()
        .filter(|child| child.is_element() && child.xsd_type() != ElementType::Annotation)
        .map(|child| parse_node(&child, node))
        .collect();

    RsEntity::Struct(Struct {
        name: name.to_string(),
        subtypes,
        comment: get_documentation(node),
        ..Default::default()
    })
}
