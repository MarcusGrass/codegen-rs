use crate::structures::generics::{Generic, Generics};
use crate::structures::visibility::Visibility;
use crate::structures::{Annotations, ComponentSignature, Signature};
use crate::{Derives, RustType};
use std::fmt::Write;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EnumEntity {
    annotations: Annotations,
    derives: Derives,
    visibility: Visibility,
    name: String,
    members: Vec<EnumMember>,
}

impl EnumEntity {
    pub fn format(&self) -> String {
        let mut union = Generics::default();
        for generics in self
            .members
            .iter()
            .filter_map(|member| match &member.member_type {
                MemberType::Empty(_) => None,
                MemberType::Type(s) => Some(s.get_generics()),
                MemberType::Pattern(ncs) => ncs
                    .iter()
                    .map(|ncs| match &ncs.component_signature {
                        ComponentSignature::Signature(s) => s.get_generics(),
                        ComponentSignature::Generic(g) => Generics::multiple(vec![g.clone()]),
                    })
                    .reduce(|a, b| a.union(&b)),
            })
        {
            union = union.union(&generics);
        }
        let mut base = format!(
            "{}{}{}enum {}{} {}{{\n",
            self.annotations.format(),
            self.derives.format(),
            self.visibility.format(),
            self.name,
            union.format(),
            union.format_where_clause()
        );
        for member in &self.members {
            let _ = base.write_fmt(format_args!(
                "{}\n{}\n",
                member.annotations.format(),
                member.format()
            ));
        }
        base.push_str("}\n");
        base
    }
    pub fn new(
        annotations: Annotations,
        derives: Derives,
        visibility: Visibility,
        name: impl Into<String>,
        members: Vec<EnumMember>,
    ) -> Self {
        Self {
            annotations,
            derives,
            visibility,
            name: name.into(),
            members,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EnumMember {
    name: String,
    member_type: MemberType,
    annotations: Annotations,
}

impl EnumMember {
    pub fn new(name: impl Into<String>, member_type: MemberType, annotations: Annotations) -> Self {
        Self {
            name: name.into(),
            member_type,
            annotations,
        }
    }

    pub fn format(&self) -> String {
        format!("{}{}", self.name, self.member_type.format())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MemberType {
    Empty(Option<String>),
    Type(Signature),
    Pattern(Vec<NamedComponentSignature>),
}

impl MemberType {
    pub fn format(&self) -> String {
        match self {
            MemberType::Empty(v) => v
                .as_ref()
                .map(|v| format!(" = {},", v))
                .unwrap_or_else(|| String::from(",")),
            MemberType::Type(s) => {
                format!("({}),", s.format())
            }
            MemberType::Pattern(c) => {
                let chain = c
                    .iter()
                    .map(|ncs| ncs.format())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!(" {{ {} }}, ", chain)
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NamedComponentSignature {
    pub name: String,
    pub component_signature: ComponentSignature,
}

impl NamedComponentSignature {
    pub fn format(&self) -> String {
        format!("{}: {}", self.name, self.component_signature.format())
    }

    pub fn new(name: impl Into<String>, component_signature: ComponentSignature) -> Self {
        Self {
            name: name.into(),
            component_signature,
        }
    }

    pub fn new_simple_type(name: impl Into<String>, rust_type: RustType) -> Self {
        Self {
            name: name.into(),
            component_signature: ComponentSignature::Signature(Signature::simple(rust_type)),
        }
    }

    pub fn new_unbounded_generic(
        name: impl Into<String>,
        generic_alias: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            component_signature: ComponentSignature::Generic(Generic::unbounded(generic_alias)),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn full_generic_type(&self) -> String {
        self.component_signature.format()
    }
}

#[cfg(test)]
mod tests {
    use crate::structures::gen_enum::{
        EnumEntity, EnumMember, MemberType, NamedComponentSignature,
    };
    use crate::structures::generics::{Bound, Generic, Generics};
    use crate::structures::visibility::Visibility;
    use crate::structures::{Annotation, Annotations, ComponentSignature, RustType, Signature};
    use crate::{Bounds, Derives};

    #[test]
    fn create_enum() {
        let enum_e = EnumEntity::new(
            Annotations::new(vec![Annotation::new("cfg(feature = \"debug\")")]),
            Derives::new(vec![RustType::in_scope("Debug")]),
            Visibility::Public,
            "MyEnum",
            vec![
                EnumMember::new(
                    "MyFirstTag",
                    MemberType::Type(Signature::generic_container(
                        RustType::in_scope("MyStruct"),
                        Generics::multiple(vec![Generic::bounded(
                            "T",
                            Bounds::single(Bound::required(RustType::in_scope("Debug"))),
                        )]),
                    )),
                    Annotations::empty(),
                ),
                EnumMember::new(
                    "MySecondTag",
                    MemberType::Pattern(vec![
                        NamedComponentSignature::new(
                            "first",
                            ComponentSignature::Signature(Signature::simple(RustType::in_scope(
                                "i32",
                            ))),
                        ),
                        NamedComponentSignature::new(
                            "second",
                            ComponentSignature::Signature(Signature::simple(RustType::in_scope(
                                "u32",
                            ))),
                        ),
                    ]),
                    Annotations::empty(),
                ),
                EnumMember::new(
                    "MyThirdTag",
                    MemberType::Empty(Some("Value".to_owned())),
                    Annotations::empty(),
                ),
                EnumMember::new("MyFourthTag", MemberType::Empty(None), Annotations::empty()),
            ],
        );
        assert_eq!("#[cfg(feature = \"debug\")]\n#[derive(Debug)]\npub enum MyEnum<T> where T: Debug {\n\nMyFirstTag(MyStruct<T>),\n\nMySecondTag { first: i32, second: u32 }, \n\nMyThirdTag = Value,\n\nMyFourthTag,\n}\n", enum_e.format());
    }
}
