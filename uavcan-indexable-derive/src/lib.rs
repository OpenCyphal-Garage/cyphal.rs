extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::Body;
use quote::Tokens;


#[proc_macro_derive(UavcanIndexable)]
pub fn uavcan_sized(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = impl_uavcan_indexable(&ast);
    gen.parse().unwrap()
}

fn impl_uavcan_indexable(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    let variant_data = match ast.body {
        Body::Enum(_) => panic!("UavcanSized is not derivable for enum"),
        Body::Struct(ref variant_data) => variant_data,
    };

    let primitive_fields_sum = {
        let mut tokens = Tokens::new();
        for field in variant_data.fields() {
            tokens.append("self.");
            tokens.append(field.ident.as_ref().unwrap());
            tokens.append(".number_of_primitive_fields() + ");  
        }
        tokens.append("0");
        tokens
    };

//    let field_start = match 
    
    quote! {
        impl UavcanIndexable for #name {
            fn number_of_primitive_fields(&self) -> usize {
                #primitive_fields_sum
            }

            fn primitive_field_as_mut(&mut self, field_number: usize) -> Option<&mut UavcanPrimitiveField> {
                unimplemented!()
            }
                         
        }

    }
}

