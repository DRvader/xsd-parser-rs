use std::{cell::RefCell, collections::HashMap};

use roxmltree::Namespace;

use crate::parser::{constants::tag, xsd_elements::FacetType};

#[derive(Debug, Clone, Default)]
pub struct RsFile<'input> {
    pub name: String,
    pub namespace: Option<String>,
    pub types: Vec<RsEntity>,
    pub attribute_groups: Vec<RsEntity>,
    pub groups: Vec<RsEntity>,
    pub target_ns: Option<Namespace<'input>>,
    pub xsd_ns: Option<Namespace<'input>>,
}

#[derive(Debug, Default, Clone)]
pub struct Struct {
    pub name: String,
    pub comment: Option<String>,
    pub fields: RefCell<Vec<StructField>>,
    pub attribute_groups: RefCell<Vec<Alias>>,
    pub groups: RefCell<Vec<Alias>>,
    pub subtypes: Vec<RsEntity>,
}

impl Struct {
    pub fn get_types_map(&self) -> HashMap<&String, &Self> {
        let mut map = HashMap::new();
        map.insert(&self.name, self);
        for ty in &self.subtypes {
            if let RsEntity::Struct(st) = ty {
                map.extend(st.get_types_map());
            }
        }
        map
    }

    pub fn extend_base(&self, types: &HashMap<&String, &Self>) -> Vec<Self> {
        let mut extended_types = Vec::new();

        self.fields.borrow_mut().iter_mut().for_each(|f| f.extend_base(types));

        let mut fields = self
            .fields
            .borrow()
            .iter()
            .filter(|f| f.name.as_str() == tag::BASE)
            .flat_map(|f| {
                let key = f.type_name.split(':').last().unwrap().to_string();
                types.get(&key).map(|s| s.fields.borrow().clone()).unwrap_or_default()
            })
            .filter(|f| {
                //TODO: remove this workaround for fields names clash
                !self.fields.borrow().iter().any(|field| field.name == f.name)
            })
            .collect::<Vec<StructField>>();

        self.fields.borrow_mut().append(&mut fields);

        self.fields.borrow_mut().retain(|field| field.name.as_str() != tag::BASE);

        let mut fields = self
            .groups
            .borrow()
            .iter()
            .flat_map(|f| {
                let key = f.original.split(':').last().unwrap().to_string();

                if let Some(v) = types.get(&key) {
                    v.extend_attribute_group(types)
                }

                if !f.type_modifiers.is_empty() {
                    // extended_types.push((*types.get(&key).unwrap()).clone());

                    let ty = (*types.get(&key).unwrap()).clone();
                    for field in ty.fields.borrow_mut().iter_mut() {
                        if field.type_name == "part-group" {
                            field.type_name = "super::PartGroup".to_string();
                        }
                    }

                    vec![StructField {
                        name: f.name.clone(),
                        type_name: types.get(&key).unwrap().name.clone(),
                        comment: f.comment.clone(),
                        type_modifiers: f.type_modifiers.clone(),
                        subtypes: vec![RsEntity::Struct(ty)],
                        ..Default::default()
                    }]
                } else {
                    types.get(&key).map(|s| s.fields.borrow().clone()).unwrap_or_default()
                }
            })
            .filter(|f| {
                //TODO: remove this workaround for fields names clash
                !self.fields.borrow().iter().any(|field| field.name == f.name)
            })
            .collect::<Vec<StructField>>();

        self.fields.borrow_mut().append(&mut fields);

        let mut fields = self
            .attribute_groups
            .borrow()
            .iter()
            .flat_map(|f| {
                let key = f.original.split(':').last().unwrap().to_string();

                if let Some(v) = types.get(&key) {
                    v.extend_attribute_group(types)
                }

                if !f.type_modifiers.is_empty() {
                    extended_types.push((*types.get(&key).unwrap()).clone());
                    vec![StructField {
                        name: f.name.clone(),
                        type_name: types.get(&key).unwrap().name.clone(),
                        comment: f.comment.clone(),
                        type_modifiers: f.type_modifiers.clone(),
                        ..Default::default()
                    }]
                } else {
                    types.get(&key).map(|s| s.fields.borrow().clone()).unwrap_or_default()
                }
            })
            .filter(|f| {
                //TODO: remove this workaround for fields names clash
                !self.fields.borrow().iter().any(|field| {
                    field.name == f.name && matches!(field.source, StructFieldSource::Attribute)
                })
            })
            .map(|mut f| {
                //TODO: remove this workaround for fields names clash
                if self.fields.borrow().iter().any(|field| field.name == f.name) {
                    f.name = format!("{}_attr", f.name);
                }

                f
            })
            .collect::<Vec<StructField>>();

        self.fields.borrow_mut().append(&mut fields);

        let output_fields = self.fields.clone().into_inner();
        *self.fields.borrow_mut() = output_fields
            .into_iter()
            .map(|mut f| {
                //TODO: remove this workaround for fields names clash
                if matches!(f.source, StructFieldSource::Attribute) {
                    if self.fields.borrow().iter().any(|field| {
                        field.name == f.name
                            && !matches!(field.source, StructFieldSource::Attribute)
                    }) {
                        f.name = format!("{}_attr", f.name);
                    }
                }

                f
            })
            .collect::<Vec<StructField>>();

        for subtype in &self.subtypes {
            if let RsEntity::Struct(s) = subtype {
                extended_types.extend(s.extend_base(types));
            }
        }

        extended_types
    }

    pub fn extend_attribute_group(&self, types: &HashMap<&String, &Self>) {
        let mut fields = self
            .attribute_groups
            .borrow()
            .iter()
            .flat_map(|f| {
                let key = f.original.split(':').last().unwrap().to_string();

                if let Some(v) = types.get(&key) {
                    v.extend_attribute_group(types)
                }

                if !f.type_modifiers.is_empty() {
                    vec![StructField {
                        name: f.name.clone(),
                        type_name: types.get(&key).unwrap().name.clone(),
                        comment: f.comment.clone(),
                        type_modifiers: f.type_modifiers.clone(),
                        ..Default::default()
                    }]
                } else {
                    types.get(&key).map(|s| s.fields.borrow().clone()).unwrap_or_default()
                }
            })
            .filter(|f| {
                //TODO: remove this workaround for fields names clash
                !self.fields.borrow().iter().any(|field| field.name == f.name)
            })
            .collect::<Vec<StructField>>();

        self.fields.borrow_mut().append(&mut fields);
    }

    pub fn extend_group(&self, types: &HashMap<&String, &Self>) {
        let mut fields = self
            .groups
            .borrow()
            .iter()
            .flat_map(|f| {
                let key = f.original.split(':').last().unwrap().to_string();

                if let Some(v) = types.get(&key) {
                    v.extend_attribute_group(types)
                }

                if !f.type_modifiers.is_empty() {
                    vec![StructField {
                        name: f.name.clone(),
                        type_name: types.get(&key).unwrap().name.clone(),
                        comment: f.comment.clone(),
                        type_modifiers: f.type_modifiers.clone(),
                        ..Default::default()
                    }]
                } else {
                    types.get(&key).map(|s| s.fields.borrow().clone()).unwrap_or_default()
                }
            })
            .filter(|f| {
                //TODO: remove this workaround for fields names clash
                !self.fields.borrow().iter().any(|field| field.name == f.name)
            })
            .collect::<Vec<StructField>>();

        self.fields.borrow_mut().append(&mut fields);

        for subtype in &self.subtypes {
            if let RsEntity::Struct(s) = subtype {
                s.extend_group(types);
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StructField {
    pub name: String,
    pub type_name: String,
    pub comment: Option<String>,
    pub subtypes: Vec<RsEntity>,
    pub source: StructFieldSource,
    pub type_modifiers: Vec<TypeModifier>,
}

impl StructField {
    pub fn extend_base(&mut self, types: &HashMap<&String, &Struct>) {
        for subtype in &mut self.subtypes {
            if let RsEntity::Struct(st) = subtype {
                st.extend_base(types);
            }
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Default)]
pub enum StructFieldSource {
    Attribute,
    Element,
    Base,
    Choice,
    Sequence,
    #[default]
    NA,
}

#[derive(Debug, Clone)]
pub struct Facet {
    pub facet_type: FacetType,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TupleStruct {
    pub name: String,
    pub comment: Option<String>,
    pub type_name: String,
    pub subtypes: Vec<RsEntity>,
    pub type_modifiers: Vec<TypeModifier>,
    pub facets: Vec<Facet>,
}

#[derive(Debug, Clone, Default)]
pub struct Enum {
    pub name: String,
    pub cases: Vec<EnumCase>,
    pub comment: Option<String>,
    pub type_name: String,
    pub subtypes: Vec<RsEntity>,
    pub type_modifiers: Vec<TypeModifier>,
    pub source: EnumSource,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum EnumSource {
    Restriction,
    Choice,
    Union,
    #[default]
    NA,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeModifier {
    None,
    Array,
    Option,
    Recursive,
    Empty,
    Flatten,
}

#[derive(Debug, Clone, Default)]
pub struct EnumCase {
    pub name: String,
    pub comment: Option<String>,
    pub value: String,
    pub type_name: Option<String>,
    pub type_modifiers: Vec<TypeModifier>,
    pub source: EnumSource,
    pub subtypes: Vec<RsEntity>,
}

#[derive(Debug, Clone, Default)]
pub struct Alias {
    pub name: String,
    pub original: String,
    pub comment: Option<String>,
    pub subtypes: Vec<RsEntity>,
    pub type_modifiers: Vec<TypeModifier>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub name: String,
    pub location: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RsEntity {
    Struct(Struct),
    StructField(StructField),
    TupleStruct(TupleStruct),
    Enum(Enum),
    EnumCase(EnumCase),
    Alias(Alias),
    Import(Import),
}

impl RsEntity {
    pub fn name(&self) -> &str {
        use RsEntity::*;
        match self {
            Struct(s) => s.name.as_str(),
            TupleStruct(tp) => tp.name.as_str(),
            Enum(e) => e.name.as_str(),
            EnumCase(ec) => ec.name.as_str(),
            Alias(al) => al.name.as_str(),
            StructField(sf) => sf.name.as_str(),
            Import(im) => im.name.as_str(),
        }
    }

    pub fn set_name(&mut self, name: &str) {
        use RsEntity::*;
        match self {
            Struct(s) => s.name = name.to_string(),
            TupleStruct(tp) => tp.name = name.to_string(),
            Enum(e) => e.name = name.to_string(),
            EnumCase(ec) => ec.name = name.to_string(),
            Alias(al) => al.name = name.to_string(),
            StructField(sf) => sf.name = name.to_string(),
            Import(im) => im.name = name.to_string(),
        }
    }

    pub fn set_comment(&mut self, comment: Option<String>) {
        use RsEntity::*;
        match self {
            Struct(s) => s.comment = comment,
            TupleStruct(tp) => tp.comment = comment,
            Enum(e) => e.comment = comment,
            EnumCase(ec) => ec.comment = comment,
            Alias(al) => al.comment = comment,
            StructField(sf) => sf.comment = comment,
            Import(im) => im.comment = comment,
        }
    }
}
