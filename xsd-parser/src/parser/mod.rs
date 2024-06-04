mod all;
mod any;
mod any_attribute;
mod attribute;
mod attribute_group;
mod choice;
mod complex_content;
mod complex_type;
pub mod constants;
mod element;
mod extension;
mod group;
mod import;
mod list;
mod node_parser;
mod restriction;
pub mod schema;
mod sequence;
mod simple_content;
mod simple_type;
mod tests;
pub mod types;
mod union;
mod utils;
pub mod xsd_elements;

use std::collections::HashMap;

use crate::parser::{
    schema::parse_schema,
    types::{RsEntity, RsFile},
};

// FIXME: Actually pass up errors
#[allow(clippy::result_unit_err)]
pub fn parse(text: &str) -> Result<RsFile, ()> {
    let doc = roxmltree::Document::parse(text).expect("Parse document error");
    let root = doc.root();

    let mut map = HashMap::new();

    let schema =
        root.children().filter(|e| e.is_element()).last().expect("Schema element is required");

    let mut schema_rs = parse_schema(&schema);
    for ty in &schema_rs.types {
        if let RsEntity::Struct(st) = ty {
            map.extend(st.get_types_map());
        }
    }
    for ag in &schema_rs.attribute_groups {
        if let RsEntity::Struct(st) = ag {
            map.extend(st.get_types_map());
        }
    }
    for ag in &schema_rs.groups {
        if let RsEntity::Struct(st) = ag {
            map.extend(st.get_types_map());
        }
    }

    let mut extended_types = Vec::new();
    for ty in &schema_rs.types {
        if let RsEntity::Struct(st) = ty {
            extended_types.extend(st.extend_base(&map));
            st.extend_attribute_group(&map);
            st.extend_group(&map);
        }
    }

    let extended_types = extended_types.into_iter().filter(|f| !schema_rs.types.iter().any(|field|if let RsEntity::Struct(st) = field{ st.name == f.name} else {false})).collect::<Vec<_>>();
    schema_rs.types.extend(extended_types.into_iter().map(|v| RsEntity::Struct(v)));

    Ok(schema_rs)
}
