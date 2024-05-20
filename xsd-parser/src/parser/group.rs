use roxmltree::Node;

use super::{
    node_parser::parse_node,
    types::{Alias, RsEntity, Struct},
    utils::get_documentation,
    xsd_elements::{ElementType, XsdNode},
};

pub fn parse_group(node: &Node, parent: &Node) -> RsEntity {
    if parent.xsd_type() == ElementType::Schema {
        return parse_global_group(node);
    }

    let reference = node.attr_ref().expect("Non-global groups must be references.").to_string();

    RsEntity::Alias(Alias {
        name: reference.to_string(),
        original: reference,
        comment: get_documentation(node),
        ..Default::default()
    })
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
