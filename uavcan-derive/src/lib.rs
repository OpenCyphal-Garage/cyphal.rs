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

    let number_of_flattened_fields = {
        let mut flattened_fields_builder = Tokens::new();

        for (i, field) in variant_data.fields().iter().enumerate() {
            let number_of_flattened_fields = number_of_flattened_fields(field);
            
            if i != 0 {
                flattened_fields_builder.append(quote!{+});
            }
            
            flattened_fields_builder.append(quote!{#number_of_flattened_fields});
        }

        flattened_fields_builder
    };

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

            fn flattened_fields_len(&self) -> usize {
                #number_of_flattened_fields
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


fn is_primitive_type(type_name: &syn::Ty) -> bool {
    if *type_name == syn::parse::ty("Uint2").expect("") ||
        *type_name == syn::parse::ty("Uint3").expect("") ||
        *type_name == syn::parse::ty("Uint4").expect("") ||
        *type_name == syn::parse::ty("Uint5").expect("") ||
        *type_name == syn::parse::ty("Uint7").expect("") ||
        *type_name == syn::parse::ty("Uint8").expect("") ||
        *type_name == syn::parse::ty("Uint16").expect("") ||
        *type_name == syn::parse::ty("Uint32").expect("") ||
        *type_name == syn::parse::ty("Float16").expect("") {
            return true;
        }
    false
}

fn is_dynamic_array(type_name: &syn::Ty) -> bool {
    if let syn::Ty::Path(_, ref path) = *type_name {
        if path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray3").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray4").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray5").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray6").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray7").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray8").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray9").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray10").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray11").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray12").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray13").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray14").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray15").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray16").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray31").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray32").expect("") ||
            path.segments.as_slice().last().unwrap().ident == syn::parse::ident("DynamicArray90").expect("") {
                return true;
            }
    }
    false
}

fn number_of_flattened_fields(field: &syn::Field) -> Tokens {
    if is_primitive_type(&field.ty) {
        quote!{1}
    } else if is_dynamic_array(&field.ty){
        quote!{1}
    } else {
        let ident = &field.ident;
        quote!{self.#ident.flattened_fields_len()}
    }
}
