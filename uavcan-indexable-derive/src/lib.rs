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
    let size_sum = match ast.body {
        Body::Enum(_) => panic!("UavcanSized is not derivable for enum"),
        Body::Struct(ref variant_data) => {
            let mut tokens = Tokens::new();
            for field in variant_data.fields() {
                tokens.append("self.");
                tokens.append(field.ident.as_ref().unwrap());
                tokens.append(".uavcan_bit_size() + ");  
            }
            tokens.append("0");
            tokens
        },
    };
    
    quote! {
        impl UavcanIndexable for #name {
            fn uavcan_bit_size(&self) -> usize {
                #size_sum
            }
            
            fn field_start_from_field_num(&self, field_num: usize) -> Option<usize> {
                unimplemented!()                
            }
            
            fn field_length_from_field_num(&self, field_num: usize) -> Option<usize> {
                unimplemented!()                
            }
        }

    }
}

