use codegen_rs::structures::gen_enum::NamedComponentSignature;
use codegen_rs::structures::generics::{Generic, Generics};
use codegen_rs::structures::method::Argument;
use codegen_rs::structures::visibility::Visibility;
use codegen_rs::structures::{ComponentSignature, Import, Ownership, RustType, Signature};
use codegen_rs::{
    FileBuilder, FunctionBuilder, ImplBuilder, MethodBuilder, ModuleBuilder, StructBuilder,
};

fn main() {
    let my_custom_iterator_type = RustType::in_scope("MyIterator");
    let generic_alias = "T";
    let my_custom_iterator = Signature::generic(
        my_custom_iterator_type.clone(),
        Generic::unbounded(generic_alias).into_generics(),
    );
    let rust_vec = RustType::in_scope("Vec");
    let vec_signature = NamedComponentSignature::new(
        "inner",
        ComponentSignature::Signature(Signature::generic(
            rust_vec,
            Generics::single_unbounded(generic_alias),
        )),
    );
    let iterator = "Iterator";
    let mb = ModuleBuilder::new(FileBuilder::new("main")
        .add_import(Import::FullType(RustType::from_package("std::iter", iterator)))
        .add_struct(StructBuilder::new_from_signature(&my_custom_iterator)
            .add_derive_in_scope("Debug")
            .add_derive_in_scope("Clone")
            .add_field(Visibility::Private, vec_signature.clone())
        )
        .add_impl(ImplBuilder::new(my_custom_iterator.clone())
            .add_method(MethodBuilder::new("new")
                .add_argument(Argument::new(Ownership::Owned, vec_signature))
                .set_body("Self {\n inner \n}")
                .set_return_type(ComponentSignature::Signature(my_custom_iterator.clone()))
            )
        )
        .add_impl(ImplBuilder::new(my_custom_iterator.clone())
            .add_type_def_unbounded_generic("Item", generic_alias)
            .implement_for(Signature::simple(RustType::in_scope(iterator)))
            .add_method(MethodBuilder::new("next")
                .set_return_type(ComponentSignature::Signature(Signature::generic(RustType::in_scope("Option"), Generic::unbounded(generic_alias).into_generics())))
                .set_self_ownership(Ownership::MutRef)
                .set_body("return self.inner.pop();")
            )
        )
        .add_function(FunctionBuilder::new("main")
            .set_body(format!("let mut my_iterator = {}::new(vec![0, 1, 2]);\nassert_eq!(Some(2), my_iterator.next());\nprintln!(\"It works!\");", my_custom_iterator_type.format()))
        )
    );
    mb.write_to_disk("examples").unwrap();
}
