use std::cell::RefCell;

use roxmltree::Node;

use crate::parser::{
    constants::attribute,
    node_parser::parse_node,
    types::{Enum, EnumCase, EnumSource, RsEntity, Struct},
    utils::{attributes_to_fields, enum_to_field, get_documentation, get_parent_name},
    xsd_elements::{ElementType, XsdNode},
};

use super::types::TypeModifier;

pub fn parse_union(union: &Node) -> RsEntity {
    let mut cases =
        union.attribute(attribute::MEMBER_TYPES).map(create_enum_cases).unwrap_or_default();

    cases.extend(
        union
            .children()
            .filter(|e| e.is_element() && e.xsd_type() == ElementType::SimpleType)
            .enumerate()
            .map(|st| {
                let subtype = enum_subtype_from_node(&st.1, union, st.0);
                EnumCase {
                    name: format!("EnumCase_{}", st.0),
                    type_name: Some(subtype.name().to_string()),
                    source: EnumSource::Union,
                    subtypes: vec![subtype],
                    type_modifiers: vec![TypeModifier::Flatten],
                    ..Default::default()
                }
            }),
    );

    let mut union_enum = Enum {
        cases,
        subtypes: vec![],
        comment: get_documentation(union),
        type_name: "std::string::String".into(),
        source: EnumSource::Union,
        ..Default::default()
    };

    let mut fields = attributes_to_fields(union);

    if fields.is_empty() {
        RsEntity::Enum(union_enum)
    } else {
        union_enum.name = format!("{}Choice", get_parent_name(union));
        fields.push(enum_to_field(union_enum));
        RsEntity::Struct(Struct { fields: RefCell::new(fields), ..Default::default() })
    }
}

fn create_enum_cases(member_types: &str) -> Vec<EnumCase> {
    member_types
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(|mt| EnumCase {
            name: mt.to_string(),
            type_name: Some(mt.to_string()),
            source: EnumSource::Union,
            ..Default::default()
        })
        .collect()
}

fn enum_subtype_from_node(node: &Node, parent: &Node, index: usize) -> RsEntity {
    let mut entity = parse_node(node, parent);
    entity.set_name(format!("EnumCaseType_{}", index).as_str());
    entity
}

#[cfg(test)]
mod test {
    use crate::parser::{
        types::RsEntity,
        union::{create_enum_cases, parse_union},
        utils::find_child,
    };

    #[test]
    fn test_create_enum() {
        let cases = create_enum_cases("Type1 Type2  Type3");
        assert_eq!(cases.len(), 3);
        assert_eq!(cases[0].name, "Type1");
    }

    #[test]
    fn test_parse_union() {
        let doc = roxmltree::Document::parse(
            r#"
    <xs:schema xmlns:tt="http://www.onvif.org/ver10/schema" xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="http://www.onvif.org/ver10/schema">
        <xs:simpleType name="SomeType">
            <xs:annotation><xs:documentation>Some text</xs:documentation></xs:annotation>
            <xs:union memberTypes="Type1 Type2" />
        </xs:simpleType>
    </xs:schema>
    "#
        ).unwrap();

        let simple_type = find_child(&doc.root_element(), "simpleType").unwrap();
        let union = find_child(&simple_type, "union").unwrap();

        let result = parse_union(&union);

        match result {
            RsEntity::Enum(en) => {
                assert_eq!(en.cases.len(), 2);
                assert_eq!(en.cases[0].name, "Type1");
                assert_eq!(en.cases[1].name, "Type2");
                assert_eq!(en.name, String::default());
            }
            _ => unreachable!("Test Failed"),
        }
    }

    #[test]
    fn test_parse_union_with_nested_types() {
        let doc = roxmltree::Document::parse(
            r#"
    <xs:schema xmlns:tt="http://www.onvif.org/ver10/schema" xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="http://www.onvif.org/ver10/schema">
        <xs:simpleType name="SomeType">
            <xs:annotation><xs:documentation>Some text</xs:documentation></xs:annotation>
            <xs:union memberTypes="Type1 Type2">
                <xs:simpleType>
                    <xs:list itemType="ListOfType" />
                </xs:simpleType>
                <xs:simpleType>
                    <xs:list itemType="ListOfType1" />
                </xs:simpleType>
                <xs:simpleType>
                    <xs:list itemType="ListOfType2" />
                </xs:simpleType>
            </xs:union>
        </xs:simpleType>
    </xs:schema>
    "#
        ).unwrap();

        let simple_type = find_child(&doc.root_element(), "simpleType").unwrap();
        let union = find_child(&simple_type, "union").unwrap();

        let result = parse_union(&union);
        match result {
            RsEntity::Enum(en) => {
                assert_eq!(en.cases.len(), 5);
                assert_eq!(en.cases[0].name, "Type1");
                assert_eq!(en.cases[1].name, "Type2");
                assert_eq!(en.cases[2].name, "EnumCase_0");
                assert_eq!(en.cases[3].name, "EnumCase_1");
                assert_eq!(en.cases[4].name, "EnumCase_2");

                assert_eq!(en.cases[2].type_name.as_ref().unwrap(), "EnumCaseType_0");
                assert_eq!(en.cases[3].type_name.as_ref().unwrap(), "EnumCaseType_1");
                assert_eq!(en.cases[4].type_name.as_ref().unwrap(), "EnumCaseType_2");

                assert_eq!(en.cases[2].subtypes.len(), 1);
                assert_eq!(en.cases[2].subtypes[0].name(), "EnumCaseType_0");
                assert_eq!(en.cases[3].subtypes.len(), 1);
                assert_eq!(en.cases[3].subtypes[0].name(), "EnumCaseType_1");
                assert_eq!(en.cases[4].subtypes.len(), 1);
                assert_eq!(en.cases[4].subtypes[0].name(), "EnumCaseType_2");
                assert_eq!(en.name, String::default());
            }
            _ => unreachable!("Test Failed"),
        }
    }

    #[test]
    fn test_parse_union_with_nested_types_and_attributes() {
        let doc = roxmltree::Document::parse(
            r#"
    <xs:schema xmlns:tt="http://www.onvif.org/ver10/schema" xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="http://www.onvif.org/ver10/schema">
        <xs:simpleType name="SomeType">
            <xs:annotation><xs:documentation>Some text</xs:documentation></xs:annotation>
            <xs:union memberTypes="Type1 Type2">
                <xs:simpleType>
                    <xs:list itemType="ListOfType" />
                </xs:simpleType>
                <xs:simpleType>
                    <xs:list itemType="ListOfType1" />
                </xs:simpleType>
                <xs:simpleType>
                    <xs:list itemType="ListOfType2" />
                </xs:simpleType>
                <xs:attribute name="Attr1" type="AttrType1"/>
                <xs:attribute name="Attr2" type="AttrType2"/>
            </xs:union>
        </xs:simpleType>
    </xs:schema>
    "#
        ).unwrap();

        let simple_type = find_child(&doc.root_element(), "simpleType").unwrap();
        let union = find_child(&simple_type, "union").unwrap();

        let result = parse_union(&union);
        let subtype = match &result {
            RsEntity::Struct(st) => {
                assert!(st.name.is_empty());
                assert_eq!(st.subtypes.len(), 0);
                assert_eq!(st.fields.borrow().len(), 3);
                let ty = &st.fields.borrow()[2];
                ty.subtypes[0].clone()
            }
            _ => unreachable!("Test Failed!"),
        };

        match subtype {
            RsEntity::Enum(en) => {
                assert_eq!(en.cases.len(), 5);
                assert_eq!(en.cases[0].name, "Type1");
                assert_eq!(en.cases[1].name, "Type2");
                assert_eq!(en.cases[2].name, "EnumCase_0");
                assert_eq!(en.cases[3].name, "EnumCase_1");
                assert_eq!(en.cases[4].name, "EnumCase_2");

                assert_eq!(en.cases[2].type_name.as_ref().unwrap(), "EnumCaseType_0");
                assert_eq!(en.cases[3].type_name.as_ref().unwrap(), "EnumCaseType_1");
                assert_eq!(en.cases[4].type_name.as_ref().unwrap(), "EnumCaseType_2");

                assert_eq!(en.cases[2].subtypes.len(), 1);
                assert_eq!(en.cases[2].subtypes[0].name(), "EnumCaseType_0");
                assert_eq!(en.cases[3].subtypes.len(), 1);
                assert_eq!(en.cases[3].subtypes[0].name(), "EnumCaseType_1");
                assert_eq!(en.cases[4].subtypes.len(), 1);
                assert_eq!(en.cases[4].subtypes[0].name(), "EnumCaseType_2");

                assert_eq!(en.name, "SomeTypeChoice");
            }
            _ => unreachable!("Test Failed"),
        }
    }
}
