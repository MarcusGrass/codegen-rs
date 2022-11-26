use crate::structures::gen_enum::NamedComponentSignature;
use crate::structures::generics::{Generic, Generics};
use crate::structures::visibility::Visibility;
use std::fmt::{Display, Formatter};

pub mod gen_const;
pub mod gen_enum;
pub mod gen_impl;
pub mod gen_struct;
pub mod gen_trait;
pub mod generics;
pub mod method;
pub mod visibility;

pub const BASE_INDENT: usize = 4;

pub trait ToSourceFilePart {
    fn format_source_file_part(&self) -> String;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Ownership {
    Owned,
    OwnedMut,
    Ref,
    MutRef,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Synchronicity {
    Sync,
    Async,
}

impl Synchronicity {
    pub fn from_bool(is_sync: bool) -> Self {
        if is_sync {
            Synchronicity::Sync
        } else {
            Synchronicity::Async
        }
    }

    pub fn format(&self) -> &'static str {
        match self {
            Synchronicity::Sync => "",
            Synchronicity::Async => "async ",
        }
    }
}

impl Display for Synchronicity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.format())
    }
}

impl Ownership {
    pub fn format(&self) -> &'static str {
        match self {
            Ownership::Owned => "",
            Ownership::OwnedMut => "mut ",
            Ownership::Ref => "&",
            Ownership::MutRef => "&mut ",
        }
    }
}

impl Display for Ownership {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.format())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Signature {
    BoundedType(RustType, Generics),
    SingleGeneric(Generics),
}

impl Signature {
    pub fn generic_container(rust_type: RustType, generics: Generics) -> Self {
        Self::BoundedType(rust_type, generics)
    }

    pub fn simple(rust_type: RustType) -> Self {
        Self::BoundedType(rust_type, Generics::default())
    }

    pub fn simple_generic(generic: Generic) -> Self {
        Self::SingleGeneric(Generics::Single(generic))
    }

    pub(crate) fn rust_type(&self) -> Option<&RustType> {
        if let Self::BoundedType(rt, _) = self {
            Some(rt)
        } else {
            None
        }
    }

    pub(crate) fn get_any_alias(&self) -> String {
        match self {
            Signature::BoundedType(rt, _) => rt.name.clone(),
            Signature::SingleGeneric(g) => g.format(),
        }
    }

    pub fn get_generics(&self) -> Generics {
        match self {
            Signature::BoundedType(_, g) | Signature::SingleGeneric(g) => g.clone(),
        }
    }

    pub fn get_associated_generics(&self) -> Generics {
        if let Self::BoundedType(_, g) = self {
            g.clone()
        } else {
            Generics::default()
        }
    }

    pub fn format(&self) -> String {
        match self {
            Signature::BoundedType(rt, g) => {
                format!("{}{}", rt.format(), g.format())
            }
            Signature::SingleGeneric(g) => g.format(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ComponentSignature {
    Signature(Signature),
    Generic(Generic),
}

impl From<Signature> for ComponentSignature {
    fn from(s: Signature) -> Self {
        ComponentSignature::Signature(s)
    }
}

impl ComponentSignature {
    fn format(&self) -> String {
        match self {
            ComponentSignature::Signature(s) => s.format(),
            ComponentSignature::Generic(g) => g.alias.clone(),
        }
    }
}

impl Display for ComponentSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TypeDef {
    TraitImpl(NamedComponentSignature),
    Const(TypeDefDeclaration),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeDefDeclaration {
    pub visibility: Visibility,
    pub named_component_signature: NamedComponentSignature,
}

impl TypeDefDeclaration {
    pub fn new(visibility: Visibility, named_component_signature: NamedComponentSignature) -> Self {
        Self {
            visibility,
            named_component_signature,
        }
    }
}

impl ToSourceFilePart for TypeDef {
    fn format_source_file_part(&self) -> String {
        self.format()
    }
}

impl TypeDef {
    pub fn format(&self) -> String {
        match self {
            TypeDef::TraitImpl(ncs) => {
                format!("type {} = {};\n", ncs.name, ncs.component_signature)
            }
            TypeDef::Const(tdd) => {
                format!(
                    "{}type {} = {};\n",
                    tdd.visibility,
                    tdd.named_component_signature.name,
                    tdd.named_component_signature.component_signature
                )
            }
        }
    }
}

pub enum Import {
    /// Import a type like std::collections::HashMap;
    FullType(RustType),
    /// Any freetext import like `"use crate::structures::*;\n"`
    Spec(String),
}

impl ToSourceFilePart for Import {
    fn format_source_file_part(&self) -> String {
        self.format()
    }
}

impl Import {
    pub fn spec(spec: impl Into<String>) -> Self {
        Import::Spec(spec.into())
    }
    pub fn format(&self) -> String {
        match self {
            Import::FullType(rt) => format!("use {};\n", rt.format()),
            Import::Spec(s) => format!("use {s};\n"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RustType {
    // crate::blablabla::bla
    package_spec: Option<String>,
    // Box for stack shennanigans
    wrapped: Option<Box<RustType>>,
    pub name: String,
}

impl RustType {
    pub fn in_scope(name: impl Into<String>) -> Self {
        Self {
            package_spec: None,
            wrapped: None,
            name: name.into(),
        }
    }

    pub fn from_package(package_spec: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            package_spec: Some(package_spec.into()),
            wrapped: None,
            name: name.into(),
        }
    }

    pub fn wrap(self, other: RustType) -> Self {
        Self {
            package_spec: self.package_spec,
            wrapped: Some(Box::new(other)),
            name: self.name,
        }
    }

    pub fn format(&self) -> String {
        let begin = if let Some(package) = &self.package_spec {
            format!("{}::", package)
        } else {
            String::new()
        };
        if let Some(wrapped) = &self.wrapped {
            format!("{begin}{}<{}>", self.name, wrapped.format())
        } else {
            format!("{begin}{}", self.name)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Derives {
    pub(crate) rust_types: Vec<RustType>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Annotations {
    pub(crate) annotations: Vec<Annotation>,
}

impl ToSourceFilePart for Annotations {
    fn format_source_file_part(&self) -> String {
        self.format()
    }
}

impl Annotations {
    pub fn new(annotations: Vec<Annotation>) -> Self {
        Self { annotations }
    }

    pub fn empty() -> Self {
        Self {
            annotations: vec![],
        }
    }

    pub fn format(&self) -> String {
        if self.annotations.is_empty() {
            String::new()
        } else {
            self.annotations
                .iter()
                .map(Annotation::format)
                .collect::<Vec<String>>()
                .join("")
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Annotation {
    content: String,
}

impl Annotation {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }

    pub fn format(&self) -> String {
        format!("#[{}]\n", self.content)
    }
}

impl Derives {
    pub fn new(rust_types: Vec<RustType>) -> Self {
        Self { rust_types }
    }

    pub fn empty() -> Self {
        Self { rust_types: vec![] }
    }

    pub fn format(&self) -> String {
        if self.rust_types.is_empty() {
            "".to_owned()
        } else {
            let concatenated = self
                .rust_types
                .iter()
                .map(RustType::format)
                .collect::<Vec<String>>()
                .join(", ");
            format!("#[derive({concatenated})]\n")
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Module {
    visibility: Visibility,
    annotations: Annotations,
    name: String,
}

impl Module {
    pub fn new(visibility: Visibility, name: impl Into<String>, annotations: Annotations) -> Self {
        Self {
            visibility,
            name: name.into(),
            annotations,
        }
    }

    pub fn format(&self) -> String {
        format!(
            "{}{}mod {};\n",
            self.annotations.format(),
            self.visibility,
            self.name
        )
    }
}
#[cfg(test)]
mod tests {
    use crate::structures::generics::Bound;
    use crate::structures::{
        Annotation, Annotations, Generic, Generics, Module, Ownership, RustType, Signature,
        Synchronicity,
    };
    use crate::{Bounds, Visibility};

    #[test]
    fn rust_type_format() {
        let rt = RustType::in_scope("String");
        assert_eq!("String", rt.format());
        let rt = RustType::from_package("crate", "Annotation");
        assert_eq!("crate::Annotation", rt.format());
    }

    #[test]
    fn synchronicity_format() {
        assert_eq!("", Synchronicity::Sync.format());
        assert_eq!("async ", Synchronicity::Async.format());
    }

    #[test]
    fn ownership_format() {
        assert_eq!("", Ownership::Owned.format());
        assert_eq!("&", Ownership::Ref.format());
        assert_eq!("&mut ", Ownership::MutRef.format());
    }

    #[test]
    fn annotation_format() {
        let mut annotations =
            Annotations::new(vec![Annotation::new("cfg(feature = \"debug\")".to_owned())]);
        assert_eq!("#[cfg(feature = \"debug\")]\n", &annotations.format());
        annotations
            .annotations
            .push(Annotation::new("serde(alias = \"MyAlias\")".to_owned()));
        assert_eq!(
            "#[cfg(feature = \"debug\")]\n#[serde(alias = \"MyAlias\")]\n",
            &annotations.format()
        )
    }

    #[test]
    fn signature_format_typed() {
        let signature = Signature::simple(RustType::in_scope("Debug"));
        assert_eq!("Debug", &signature.format());
        let signature = Signature::generic_container(
            RustType::from_package("std::collections", "HashMap"),
            Generics::multiple(vec![
                Generic::bounded(
                    "K",
                    Bounds::multiple(vec![
                        Bound::required(RustType::in_scope("Eq")),
                        Bound::required(RustType::in_scope("Hash")),
                    ]),
                ),
                Generic::unbounded("V"),
            ]),
        );
        assert_eq!("std::collections::HashMap<K, V>", &signature.format());
    }

    #[test]
    fn module_format() {
        let module = Module::new(Visibility::Public, "my_mod", Annotations::empty());
        assert_eq!("pub mod my_mod;\n", module.format());
    }
}
