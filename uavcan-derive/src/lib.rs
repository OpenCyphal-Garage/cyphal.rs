extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::Body;
use quote::Tokens;


#[proc_macro_derive(UavcanStruct)]
pub fn uavcan_sized(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = impl_uavcan_struct(&ast);
    gen.parse().unwrap()
}

fn impl_uavcan_struct(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let variant_data = match ast.body {
        Body::Enum(_) => panic!("UavcanSized is not derivable for enum"),
        Body::Struct(ref variant_data) => variant_data,
    };

    let number_of_fields = variant_data.fields().len();
    

    let field_as_mut_body = {
        let mut primitive_fields_builder = Tokens::new();
        let mut primitive_fields_cases = Tokens::new();
        
        for (i, field) in variant_data.fields().iter().enumerate() {
            let field_name = field.ident.as_ref().unwrap();
            primitive_fields_cases.append(quote!{
                #i => self.#field_name.as_mut_uavcan_field(),
            });
        }
        
        primitive_fields_cases.append(quote!{
            x => panic!("The index: {}, is not a valid Uavcan Struct field", x),
        });
        
        primitive_fields_builder.append(quote!{
            match field_number {
                #primitive_fields_cases
            }
        });

        primitive_fields_builder
    };
        
    let field_body = {
        let mut primitive_fields_builder = Tokens::new();
        let mut primitive_fields_cases = Tokens::new();

        for (i, field) in variant_data.fields().iter().enumerate() {
            let field_name = field.ident.as_ref().unwrap();
            primitive_fields_cases.append(quote!{
                #i => self.#field_name.as_uavcan_field(),
            });
        }
        
        primitive_fields_cases.append(quote!{
            x => panic!("The index: {}, is not a valid Uavcan Struct field", x),
        });
        
        primitive_fields_builder.append(quote!{
            match field_number {
                #primitive_fields_cases
            }
        });
        
        primitive_fields_builder
    };
    
    
    
    
    quote!{
        impl AsUavcanField for #name{
            fn as_uavcan_field(&self) -> UavcanField{
                UavcanField::UavcanStruct(self)
            }
            fn as_mut_uavcan_field(&mut self) -> MutUavcanField{
                MutUavcanField::UavcanStruct(self)
            }
        }

        impl UavcanStruct for #name {
            fn fields_len(&self) -> usize {
                #number_of_fields
            }

            fn field_as_mut(&mut self, field_number: usize) -> MutUavcanField {
                #field_as_mut_body
            }

            fn field(&self, field_number: usize) -> UavcanField {
                #field_body
            }
        }

    }
}


        
