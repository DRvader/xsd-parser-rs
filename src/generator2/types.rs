use core::fmt;

use crate::generator2::utils::{get_field_comment, get_structure_comment, get_type_name};

pub struct File {
    pub name: String,
    pub namespace: Option<String>,
    pub types: Vec<RsEntity>
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "//generated file\n{types}",
            types = self
                .types
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

pub struct Struct {
    pub name: String,
    pub comment: Option<String>,
    pub fields: Vec<StructField>,
    pub macros: String,
    pub subtypes: Vec<RsEntity>,
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "{comment}{macros}pub struct {name} {{\n{fields}\n}}\n{subtypes}",
            comment = get_structure_comment(self.comment.as_deref()),
            macros = self.macros,
            name = self.name,
            fields = self
                .fields
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join("\n\n"),
            subtypes = self
                .subtypes
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join("\n\n"),
        )
    }
}

pub struct StructField {
    pub name: String,
    pub type_name: String,
    pub comment: Option<String>,
    pub macros: String,
    pub subtypes: Vec<RsEntity>
}

impl fmt::Display for StructField {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "{comment}{macros}  pub {name}: {typename},",
            macros = self.macros,
            name = self.name,
            typename = self.type_name,
            comment = get_field_comment(self.comment.as_deref())
        )
    }
}

pub struct TupleStruct {
    pub name: String,
    pub comment: Option<String>,
    pub type_name: String,
    pub macros: String,
    pub subtypes: Vec<RsEntity>,
}

impl fmt::Display for TupleStruct {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "{comment}{macros}pub struct {name} (pub {typename});\n{subtypes}",
            comment = get_structure_comment(self.comment.as_deref()),
            macros = self.macros,
            name = self.name,
            typename = self.type_name,
            subtypes = self
                .subtypes
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

pub struct Enum {
    pub name: String,
    pub cases: Vec<EnumCase>,
    pub comment: Option<String>,
    pub type_name: String,
}

impl fmt::Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "{comment}pub enum {name} {{\n{cases}  \n__Unknown__({typename})\n}}\n",
            comment = get_structure_comment(self.comment.as_deref()),
            name = self.name,
            cases = self
                .cases
                .iter()
                .map(|case| case.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
            typename = self.type_name
        )
    }
}

pub struct EnumCase {
    pub name: String,
    pub comment: Option<String>,
    pub value: String,
    pub type_name: Option<String>,
}

impl fmt::Display for EnumCase {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let name = get_type_name(self.name.as_str());
        match &self.type_name {
            Some(tn) => write!(
                f,
                "  {name}({typename}),  {comment}",
                name = name,
                typename = tn,
                comment = get_field_comment(self.comment.as_deref()),
            ),
            None => write!(
                f,
                "  {name},  {comment}",
                name = name,
                comment = get_field_comment(self.comment.as_deref())
            ),
        }
    }
}

pub struct Alias {
    pub name: String,
    pub original: String,
    pub comment: Option<String>,
    pub subtypes: Vec<RsEntity>,
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let is_same_name = self.name == self.original;
        write!(
            f,
            "{comment}{visibility}type {name} = {original};\n",
            visibility = if is_same_name { "//" } else { "pub " },
            comment = get_field_comment(self.comment.as_deref()),
            name = self.name,
            original = self.original
        )
    }
}

pub struct Import {
    pub name: String,
    pub location: String,
}

impl fmt::Display for Import {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "//use {}  {};\n",
            self.location,
            self.name,
        )
    }
}

pub enum RsEntity {
    Struct(Struct),
    StructField(StructField),
    TupleStruct(TupleStruct),
    Enum(Enum),
    EnumCase(EnumCase),
    Alias(Alias),
    File(File),
    Import(Import),
}

impl fmt::Display for RsEntity {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use RsEntity::*;
        match self {
            Struct(s) => write!(f, "{}", s),
            TupleStruct(tp) => write!(f, "{}", tp),
            Enum(e) => write!(f, "{}", e),
            EnumCase(ec) => write!(f, "{}", ec),
            Alias(al) => write!(f, "{}", al),
            StructField(sf) => write!(f, "{}", sf),
            File(file) => write!(f, "{}", file),
            Import(im) => write!(f, "{}", im),
        }
    }
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
            File(file) => file.name.as_str(),
            Import(im) => im.name.as_str(),
        }
    }
}
