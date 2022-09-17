use crate::structures::gen_const::{ConstantEntity, ConstantType};
use crate::structures::gen_enum::{EnumEntity, EnumMember, MemberType, NamedComponentSignature};
use crate::structures::gen_impl::ImplEntity;
use crate::structures::gen_struct::{Field, StructEntity};
use crate::structures::generics::{Bounds, Generic, Generics};
use crate::structures::method::{Argument, FunctionEntity, Method};
use crate::structures::visibility::Visibility;
use crate::structures::{
    Annotations, ComponentSignature, Derives, Import, Module, Ownership, RustType, Signature,
    Synchronicity, ToSourceFilePart, TypeDef, TypeDefDeclaration,
};
use std::path::Path;

pub mod structures;
#[macro_use]
mod util;
pub use util::casing::{fix_keyword, InferCase, RustCase};
mod errors;

pub trait HasAnnotationBuilder {
    fn set_annotation(self) -> Self;
}

pub struct ModuleBuilder {
    mod_file: FileBuilder,
    module_files: Vec<ModuleFile>,
    submodules: Vec<Submodule>,
}

pub struct ModuleFile {
    visibility: Visibility,
    builder: FileBuilder,
}

pub struct Submodule {
    visibility: Visibility,
    builder: ModuleBuilder,
}

impl ModuleBuilder {
    pub fn new(mod_file: FileBuilder) -> Self {
        Self {
            mod_file,
            module_files: vec![],
            submodules: vec![],
        }
    }

    pub fn add_module_file(mut self, visibility: Visibility, file_builder: FileBuilder) -> Self {
        self.module_files.push(ModuleFile {
            visibility,
            builder: file_builder,
        });
        self
    }

    pub fn add_submodule(mut self, visibility: Visibility, module_builder: ModuleBuilder) -> Self {
        self.submodules.push(Submodule {
            visibility,
            builder: module_builder,
        });
        self
    }

    pub fn write_to_disk(self, root: impl AsRef<Path>) -> std::io::Result<()> {
        let mut modules = vec![];
        for file in &self.module_files {
            std::fs::write(
                root.as_ref().join(format!("{}.rs", file.builder.name)),
                file.builder.format_file(),
            )?;
            modules.push(Module::new(file.visibility, &file.builder.name));
        }
        for mut submod in self.submodules {
            let mod_name_ref = &submod.builder.mod_file.name;
            modules.push(Module::new(submod.visibility, mod_name_ref));
            let new_path = root.as_ref().join(mod_name_ref);
            std::fs::create_dir_all(&new_path)?;
            submod.builder.mod_file.name = "mod".to_owned();
            submod.builder.write_to_disk(&new_path)?;
        }
        std::fs::write(
            root.as_ref().join(format!("{}.rs", self.mod_file.name)),
            self.mod_file.format_submodule(&modules),
        )?;
        Ok(())
    }
}

struct OrderedFormat<T>
where
    T: ToSourceFilePart,
{
    order: usize,
    value: T,
}

impl<T> OrderedFormat<T>
where
    T: ToSourceFilePart,
{
    pub fn new(order: usize, value: T) -> Self {
        Self { order, value }
    }
}

pub struct FileBuilder {
    name: String,
    annotations: Annotations,
    imports: Vec<Import>,
    constants: Vec<OrderedFormat<ConstantBuilder>>,
    type_defs: Vec<OrderedFormat<TypeDef>>,
    functions: Vec<OrderedFormat<FunctionBuilder>>,
    enums: Vec<OrderedFormat<EnumBuilder>>,
    structs: Vec<OrderedFormat<StructBuilder>>,
    implementations: Vec<OrderedFormat<ImplBuilder>>,
    parts: usize,
}

impl FileBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            annotations: Annotations::empty(),
            imports: vec![],
            constants: vec![],
            type_defs: vec![],
            functions: vec![],
            enums: vec![],
            structs: vec![],
            implementations: vec![],
            parts: 0,
        }
    }

    add_annotation!();

    pub fn add_import(mut self, import: Import) -> Self {
        self.imports.push(import);
        self
    }

    pub fn add_const(mut self, constant_builder: ConstantBuilder) -> Self {
        self.constants
            .push(OrderedFormat::new(self.parts, constant_builder));
        self.parts += 1;
        self
    }

    pub fn add_type_def_simple_type(
        self,
        visibility: Visibility,
        name: impl Into<String>,
        rust_type: RustType,
    ) -> Self {
        self.add_type_def(TypeDef::Const(TypeDefDeclaration::new(
            visibility,
            NamedComponentSignature::new_simple_type(name, rust_type),
        )))
    }

    pub fn add_type_def_unbounded_generic(
        self,
        visibility: Visibility,
        name: impl Into<String>,
        generic_alias: impl Into<String>,
    ) -> Self {
        self.add_type_def(TypeDef::Const(TypeDefDeclaration::new(
            visibility,
            NamedComponentSignature::new_unbounded_generic(name, generic_alias),
        )))
    }

    pub fn add_type_def(mut self, type_def: TypeDef) -> Self {
        self.type_defs
            .push(OrderedFormat::new(self.parts, type_def));
        self.parts += 1;
        self
    }

    pub fn add_struct(mut self, struct_builder: StructBuilder) -> Self {
        self.structs
            .push(OrderedFormat::new(self.parts, struct_builder));
        self.parts += 1;
        self
    }

    pub fn add_function(mut self, function_builder: FunctionBuilder) -> Self {
        self.functions
            .push(OrderedFormat::new(self.parts, function_builder));
        self.parts += 1;
        self
    }

    pub fn add_enum(mut self, enum_builder: EnumBuilder) -> Self {
        self.enums
            .push(OrderedFormat::new(self.parts, enum_builder));
        self.parts += 1;
        self
    }

    pub fn add_impl(mut self, impl_builder: ImplBuilder) -> Self {
        self.implementations
            .push(OrderedFormat::new(self.parts, impl_builder));
        self.parts += 1;
        self
    }

    pub fn format_file(&self) -> String {
        self.format_submodule(&[])
    }

    pub fn format_submodule(&self, exposed_modules: &[Module]) -> String {
        let mut formatted = self
            .type_defs
            .iter()
            .map(|i| (i.order, i.value.format_source_file_part()))
            .chain(
                self.constants
                    .iter()
                    .map(|i| (i.order, i.value.format_source_file_part())),
            )
            .chain(
                self.functions
                    .iter()
                    .map(|i| (i.order, i.value.format_source_file_part())),
            )
            .chain(
                self.enums
                    .iter()
                    .map(|i| (i.order, i.value.format_source_file_part())),
            )
            .chain(
                self.structs
                    .iter()
                    .map(|i| (i.order, i.value.format_source_file_part())),
            )
            .chain(
                self.implementations
                    .iter()
                    .map(|i| (i.order, i.value.format_source_file_part())),
            )
            .collect::<Vec<(usize, String)>>();
        formatted.sort_by(|a, b| a.0.cmp(&b.0));
        std::iter::once(self.annotations.format())
            .chain(self.imports.iter().map(|i| i.format_source_file_part()))
            .chain(exposed_modules.iter().map(Module::format))
            .chain(formatted.into_iter().map(|(_, s)| s))
            .collect::<Vec<String>>()
            .join("")
    }
}

#[derive(Clone)]
pub struct ConstantBuilder {
    annotations: Annotations,
    constant_type: ConstantType,
    visibility: Visibility,
    name: String,
    rust_type: RustType,
    value: String,
}

impl ConstantBuilder {
    pub fn static_builder(
        name: impl Into<String>,
        rust_type: RustType,
        value: impl Into<String>,
    ) -> Self {
        Self {
            annotations: Annotations::empty(),
            constant_type: ConstantType::Static,
            visibility: Visibility::Private,
            name: name.into(),
            rust_type,
            value: value.into(),
        }
    }

    pub fn const_builder(
        name: impl Into<String>,
        rust_type: RustType,
        value: impl Into<String>,
    ) -> Self {
        Self {
            annotations: Annotations::empty(),
            constant_type: ConstantType::Const,
            visibility: Visibility::Private,
            name: name.into(),
            rust_type,
            value: value.into(),
        }
    }

    pub fn build(self) -> ConstantEntity {
        ConstantEntity::new(
            self.annotations,
            self.constant_type,
            self.visibility,
            self.name,
            self.rust_type,
            self.value,
        )
    }

    add_annotation!();
    set_visibility!();
}

impl ToSourceFilePart for ConstantBuilder {
    fn format_source_file_part(&self) -> String {
        self.clone().build().format()
    }
}

#[derive(Clone)]
pub struct FunctionBuilder {
    annotations: Annotations,
    synchronicity: Synchronicity,
    visibility: Visibility,
    name: String,
    args: Vec<Argument>,
    body: String,
    return_type: Option<ComponentSignature>,
}

impl ToSourceFilePart for FunctionBuilder {
    fn format_source_file_part(&self) -> String {
        self.clone().build().format()
    }
}

impl FunctionBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            annotations: Annotations::empty(),
            synchronicity: Synchronicity::Sync,
            visibility: Visibility::Private,
            name: name.into(),
            args: vec![],
            body: "".into(),
            return_type: None,
        }
    }

    add_annotation!();
    set_async!();
    set_visibility!();
    add_argument!();
    set_body!();
    set_return_type!();

    fn build(self) -> FunctionEntity {
        FunctionEntity::new(
            self.annotations,
            self.synchronicity,
            self.visibility,
            self.name,
            self.args,
            self.body,
            self.return_type,
        )
    }
}

#[derive(Debug, Clone)]
pub struct EnumBuilder {
    annotations: Annotations,
    derives: Derives,
    visibility: Visibility,
    pub name: String,
    members: Vec<EnumMember>,
}

impl ToSourceFilePart for EnumBuilder {
    fn format_source_file_part(&self) -> String {
        self.clone().build().format()
    }
}

impl EnumBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            annotations: Annotations::empty(),
            derives: Derives::empty(),
            visibility: Visibility::Private,
            name: name.into(),
            members: vec![],
        }
    }
    add_annotation!();
    add_derive!();
    set_visibility!();

    pub fn add_tag_member(mut self, name: impl Into<String>) -> Self {
        self.members
            .push(EnumMember::new(name, MemberType::Empty(None)));
        self
    }

    pub fn add_tag_member_with_value(
        mut self,
        name: impl Into<String>,
        value_literal: impl Into<String>,
    ) -> Self {
        self.members.push(EnumMember::new(
            name,
            MemberType::Empty(Some(value_literal.into())),
        ));
        self
    }

    pub fn add_type_member(mut self, name: impl Into<String>, type_signature: Signature) -> Self {
        self.members
            .push(EnumMember::new(name, MemberType::Type(type_signature)));
        self
    }

    pub fn add_pattern_match_member(
        mut self,
        name: impl Into<String>,
        patterns: Vec<NamedComponentSignature>,
    ) -> Self {
        self.members
            .push(EnumMember::new(name, MemberType::Pattern(patterns)));
        self
    }

    fn build(self) -> EnumEntity {
        EnumEntity::new(
            self.annotations,
            self.derives,
            self.visibility,
            self.name,
            self.members,
        )
    }
}

#[derive(Debug, Clone)]
pub struct StructBuilder {
    annotations: Annotations,
    derives: Derives,
    visibility: Visibility,
    pub name: String,
    pub fields: Vec<Field>,
}

impl ToSourceFilePart for StructBuilder {
    fn format_source_file_part(&self) -> String {
        self.clone().build().format()
    }
}

impl StructBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            annotations: Annotations::empty(),
            derives: Derives::empty(),
            visibility: Visibility::Private,
            name: name.into(),
            fields: vec![],
        }
    }

    pub fn new_from_signature(signature: &Signature) -> Self {
        Self::new(&signature.rust_type.name)
    }

    add_annotation!();
    add_derive!();
    set_visibility!();
    pub fn add_field(
        mut self,
        visibility: Visibility,
        named_component_signature: NamedComponentSignature,
    ) -> Self {
        self.fields
            .push(Field::new(visibility, named_component_signature));
        self
    }

    pub fn add_field_in_scope_simple_type(
        mut self,
        visibility: Visibility,
        field_name: impl Into<String>,
        type_name: impl Into<String>,
    ) -> Self {
        self.fields.push(Field::new(
            visibility,
            NamedComponentSignature::new(
                field_name,
                ComponentSignature::Signature(Signature::simple(RustType::in_scope(type_name))),
            ),
        ));
        self
    }

    pub fn add_field_unbounded_generic(
        mut self,
        visibility: Visibility,
        arg_alias: impl Into<String>,
        generic_alias: impl Into<String>,
    ) -> Self {
        self.fields.push(Field::new(
            visibility,
            NamedComponentSignature::new(
                arg_alias.into(),
                ComponentSignature::Generic(Generic::unbounded(generic_alias)),
            ),
        ));
        self
    }
    pub fn add_field_bounded_generic(
        mut self,
        visibility: Visibility,
        arg_alias: impl Into<String>,
        generic_alias: impl Into<String>,
        bounds: Bounds,
    ) -> Self {
        self.fields.push(Field::new(
            visibility,
            NamedComponentSignature::new(
                arg_alias.into(),
                ComponentSignature::Generic(Generic::bounded(generic_alias, bounds)),
            ),
        ));
        self
    }

    fn build(self) -> StructEntity {
        StructEntity::new(
            self.annotations,
            self.derives,
            self.visibility,
            self.name,
            self.fields,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ImplBuilder {
    annotations: Annotations,
    implementor: Signature,
    implementing: Option<Signature>,
    type_defs: Vec<TypeDef>,
    methods: Vec<MethodBuilder>,
}

impl ToSourceFilePart for ImplBuilder {
    fn format_source_file_part(&self) -> String {
        self.clone().build().format()
    }
}

impl ImplBuilder {
    pub fn new(implementor: Signature) -> Self {
        Self {
            annotations: Annotations::empty(),
            implementor,
            implementing: None,
            type_defs: vec![],
            methods: vec![],
        }
    }

    pub fn implement_for(mut self, implementing: Signature) -> Self {
        self.implementing = Some(implementing);
        self
    }

    pub fn add_type_def(mut self, type_def: TypeDef) -> Self {
        self.type_defs.push(type_def);
        self
    }

    pub fn add_type_def_simple_type(
        mut self,
        name: impl Into<String>,
        rust_type: RustType,
    ) -> Self {
        self.type_defs.push(TypeDef::TraitImpl(
            NamedComponentSignature::new_simple_type(name, rust_type),
        ));
        self
    }

    pub fn add_type_def_unbounded_generic(
        mut self,
        name: impl Into<String>,
        generic_alias: impl Into<String>,
    ) -> Self {
        self.type_defs.push(TypeDef::TraitImpl(
            NamedComponentSignature::new_unbounded_generic(name, generic_alias),
        ));
        self
    }

    pub fn add_type_def_bounded_generic(
        mut self,
        name: impl Into<String>,
        generic_alias: impl Into<String>,
        bounds: Bounds,
    ) -> Self {
        self.type_defs
            .push(TypeDef::TraitImpl(NamedComponentSignature::new(
                name,
                ComponentSignature::Generic(Generic::bounded(generic_alias, bounds)),
            )));
        self
    }

    pub fn add_type_def_signature(mut self, name: impl Into<String>, signature: Signature) -> Self {
        self.type_defs
            .push(TypeDef::TraitImpl(NamedComponentSignature::new(
                name,
                ComponentSignature::Signature(signature),
            )));
        self
    }

    pub fn add_method(mut self, method_builder: MethodBuilder) -> Self {
        self.methods.push(method_builder);
        self
    }

    fn build(self) -> ImplEntity {
        ImplEntity::new(
            self.annotations,
            self.implementor,
            self.implementing,
            self.type_defs,
            self.methods
                .into_iter()
                .map(|mb| mb.build())
                .collect::<Vec<Method>>(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct MethodBuilder {
    annotations: Annotations,
    visibility: Visibility,
    synchronicity: Synchronicity,
    self_ownership: Option<Ownership>,
    name: String,
    args: Vec<Argument>,
    body: String,
    return_type: Option<ComponentSignature>,
}

impl MethodBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            annotations: Annotations::empty(),
            visibility: Visibility::Private,
            synchronicity: Synchronicity::Sync,
            self_ownership: None,
            name: name.into(),
            args: vec![],
            body: "".to_string(),
            return_type: None,
        }
    }

    add_annotation!();
    set_visibility!();
    set_async!();
    pub fn set_self_ownership(mut self, ownership: Ownership) -> Self {
        self.self_ownership = Some(ownership);
        self
    }
    add_argument!();
    set_body!();
    set_return_type!();

    fn build(self) -> Method {
        Method::new(
            self.annotations,
            self.visibility,
            self.synchronicity,
            self.self_ownership,
            self.name,
            self.args,
            Generics::default(),
            self.body,
            self.return_type,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::structures::gen_enum::NamedComponentSignature;
    use crate::structures::{Ownership, Signature};
    use crate::{Argument, ComponentSignature, FunctionBuilder, RustType};

    #[test]
    fn fb() {
        let fb = FunctionBuilder::new("my_func")
            .add_argument(Argument::new(
                Ownership::Ref,
                NamedComponentSignature::new(
                    "arg_1",
                    ComponentSignature::Signature(Signature::simple(RustType::in_scope("String"))),
                ),
            ))
            .add_argument_unbounded_generic(Ownership::Ref, "arg_2", "T")
            .build();
        println!("{}", fb.format())
    }
}
