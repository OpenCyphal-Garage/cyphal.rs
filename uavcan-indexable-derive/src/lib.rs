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

    

    let primitive_fields_as_mut_body = {
        let mut primitive_fields_builder = Tokens::new();
    
        let mut primitive_fields_start_index = Tokens::new();
        let mut primitive_fields_end_index = Tokens::new();
        primitive_fields_start_index.append("0");
        primitive_fields_end_index.append("0");
        for field in variant_data.fields() {
            let field_name = field.ident.as_ref().unwrap();
            primitive_fields_end_index.append(quote!{+ self.#field_name.number_of_primitive_fields()});
            primitive_fields_builder.append(
                quote!{
                    if field_number >= #primitive_fields_start_index &&
                        field_number < #primitive_fields_end_index {
                            return self.#field_name.primitive_field_as_mut(field_number - (#primitive_fields_start_index));
                        }
                });
            primitive_fields_start_index.append(quote!{+ self.#field_name.number_of_primitive_fields()});
        }
        primitive_fields_builder.append(
            quote!{
                else {
                    unreachable!()
                }
            });
        primitive_fields_builder
    };
        
    let primitive_fields_body = {
        let mut primitive_fields_builder = Tokens::new();
    
        let mut primitive_fields_start_index = Tokens::new();
        let mut primitive_fields_end_index = Tokens::new();
        primitive_fields_start_index.append("0");
        primitive_fields_end_index.append("0");
        for field in variant_data.fields() {
            let field_name = field.ident.as_ref().unwrap();
            primitive_fields_end_index.append(quote!{+ self.#field_name.number_of_primitive_fields()});
            primitive_fields_builder.append(
                quote!{
                    if field_number >= #primitive_fields_start_index &&
                        field_number < #primitive_fields_end_index {
                            return self.#field_name.primitive_field(field_number - (#primitive_fields_start_index));
                        }
                });
            primitive_fields_start_index.append(quote!{+ self.#field_name.number_of_primitive_fields()});
        }
        primitive_fields_builder.append(
            quote!{
                else {
                    unreachable!()
                }
            });
        primitive_fields_builder
    };
        

    
    
    quote! {
        impl UavcanIndexable for #name {
            fn number_of_primitive_fields(&self) -> usize {
                #primitive_fields_sum
            }

            fn primitive_field_as_mut(&mut self, field_number: usize) -> &mut UavcanPrimitiveField {
                assert!(field_number < self.number_of_primitive_fields());
                #primitive_fields_as_mut_body
            }

            fn primitive_field(&self, field_number: usize) -> &UavcanPrimitiveField {
                assert!(field_number < self.number_of_primitive_fields());
                #primitive_fields_body
            }
        }

    }
}



#[proc_macro_derive(UavcanFrame)]
pub fn uavcan_frame(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = impl_uavcan_frame(&ast);
    gen.parse().unwrap()
}


fn impl_uavcan_frame(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    let variant_data = match ast.body {
        Body::Enum(_) => panic!("UavcanFrame is not deriable for enum"),
        Body::Struct(ref variant_data) => variant_data,
    };

    let header = &variant_data.fields()[0];
    let body = &variant_data.fields()[1];

    if *header.ident.as_ref().unwrap() != syn::Ident::from("header") {
        panic!("First field of struct needs to be called 'header' to derive UavcanFrame trait");
    }

    if *body.ident.as_ref().unwrap() != syn::Ident::from("body") {
        panic!("Second field of struct needs to be called 'body' to derive UavcanFrame trait");
    }

    let ref header_type = header.ty;
    let ref body_type = body.ty;
    
    quote!{
        impl UavcanFrame<#header_type, #body_type> for #name {
            fn from_parts(header: #header_type, body: #body_type) -> Self {
                Self{header: header, body: body}
            }

            fn header(&self) -> & #header_type {
                &self.header
            }

            fn body(&self) -> & #body_type {
                &self.body
            }
                              
        }
    }
}

        
