use codegen_rs::structures::gen_enum::NamedComponentSignature;
use codegen_rs::structures::visibility::Visibility;
use codegen_rs::structures::{ComponentSignature, Ownership, RustType, Signature};
use codegen_rs::{
    FileBuilder, FunctionBuilder, ImplBuilder, MethodBuilder, ModuleBuilder, StructBuilder,
};

fn main() {
    let my_mod = "my_mod";
    let my_struct = Signature::simple(RustType::from_package(
        format!("crate::{my_mod}"),
        "MyStruct",
    ));
    let my_val = "my_val";
    let my_val_type = RustType::in_scope("i32");
    let my_val_value = 5;
    ModuleBuilder::new(
        FileBuilder::new("main")
            .add_function(FunctionBuilder::new("main")
                .set_body(format!("let {my_val} = {}{{ {my_val}: {my_val_value} }};\nassert_eq!({my_val_value}, {my_val}.get_val());\nprintln!(\"{my_val} is {my_val_value}!\");", my_struct.format()))
            )
    ).add_submodule(Visibility::Private, ModuleBuilder::new(
        FileBuilder::new(my_mod)
            .add_struct(StructBuilder::new_from_signature(&my_struct)
                .set_visibility(Visibility::Public)
                .add_field(Visibility::Public, NamedComponentSignature::new_simple_type("my_val", my_val_type.clone()))
            )
            .add_impl(ImplBuilder::new(my_struct)
                .add_method(MethodBuilder::new("get_val")
                    .set_visibility(Visibility::Public)
                    .set_self_ownership(Ownership::Ref)
                    .set_return_type(ComponentSignature::Signature(Signature::simple(my_val_type)))
                    .set_body(format!("return self.{};", my_val))
                )
            )
    ))
        .write_to_disk("examples").unwrap();
}
