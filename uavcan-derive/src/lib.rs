#![recursion_limit="128"]

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

    let serialize_body = {
        let mut serialize_builder = Tokens::new();
        let mut field_index = Tokens::new();

        field_index.append(quote!{0});
        
        for (i, field) in variant_data.fields().iter().enumerate() {
            let field_ident = &field.ident;
            let field_type = &field.ty;
            let field_type_path_segment: &syn::Ident = {
                if let syn::Ty::Path(_, ref path) = *field_type {
                    &path.segments.as_slice().last().unwrap().ident
                } else {
                    panic!("Type name not found")
                }
            };

            if i != 0 { serialize_builder.append(quote!{ else });}
            
            if is_primitive_type(field_type) {
                serialize_builder.append(quote!{if *flattened_field == #field_index {
                    if self.#field_ident.serialize(bit, buffer) == uavcan::SerializationResult::Finished {
                        *flattened_field += 1;
                        *bit = 0;
                    } else {
                        return uavcan::SerializationResult::BufferFull;
                    }
                }});                
                field_index.append(quote!{ +1});
            } else if is_dynamic_array(field_type) {
                let element_type = {
                    if let syn::Ty::Path(_, ref path) = *field_type {
                        if let syn::PathParameters::AngleBracketed(ref param_data) = path.segments.as_slice().last().unwrap().parameters {
                            &param_data.types[0]
                        } else {
                            panic!("Element type name not found")
                        }
                    } else {
                        panic!("Type name not found")
                    }
                };
                
                // check for tail optimization
                if i == variant_data.fields().len() - 1 {
                    serialize_builder.append(quote!{if *flattened_field == #field_index {
                        let mut skewed_bit = *bit + #field_type_path_segment::<Uint8>::length_bit_length();
                        if self.#field_ident.serialize(&mut skewed_bit, buffer) == uavcan::SerializationResult::Finished {
                            *flattened_field += 1;
                            *bit = 0;
                        } else {
                            *bit = skewed_bit - #field_type_path_segment::<#element_type>::length_bit_length();
                            return uavcan::SerializationResult::BufferFull;
                        }                        
                    }});
                    field_index.append(quote!{ +1});
                } else {
                    serialize_builder.append(quote!{if *flattened_field == #field_index {
                        if self.#field_ident.serialize(bit, buffer) == uavcan::SerializationResult::Finished {
                            *flattened_field += 1;
                            *bit = 0;
                        } else {
                            return uavcan::SerializationResult::BufferFull;
                        }
                    }});
                    field_index.append(quote!{ +1});
                }
            } else {                
                serialize_builder.append(quote!{if *flattened_field >= #field_index && *flattened_field < #field_index + self.#field_ident.flattened_fields_len() {
                    let mut current_field = *flattened_field - #field_index;
                    if self.#field_ident.serialize(&mut current_field, bit, buffer) == uavcan::SerializationResult::Finished {
                        *flattened_field = #field_index + self.#field_ident.flattened_fields_len();
                        *bit = 0;
                    } else {
                        *flattened_field = #field_index + current_field;
                        return uavcan::SerializationResult::BufferFull;
                    }
                }});
                field_index.append(quote!{ +self.#field_ident.flattened_fields_len()});
            }
        }
        serialize_builder
    };

    let deserialize_body = {
        let mut deserialize_builder = Tokens::new();
        let mut field_index = Tokens::new();

        field_index.append(quote!{0});
        
        for (i, field) in variant_data.fields().iter().enumerate() {
            let field_ident = &field.ident;
            let field_type = &field.ty;
            let field_type_path_segment: &syn::Ident = {
                if let syn::Ty::Path(_, ref path) = *field_type {
                    &path.segments.as_slice().last().unwrap().ident
                } else {
                    panic!("Type name not found")
                }
            };
            
            if i != 0 { deserialize_builder.append(quote!{ else });}
            
            if is_primitive_type(field_type) {
                deserialize_builder.append(quote!{if *flattened_field == #field_index {
                    if self.#field_ident.deserialize(bit, buffer) == uavcan::deserializer::DeserializationResult::Finished {
                        *flattened_field += 1;
                        *bit = 0;
                    } else {
                        return uavcan::deserializer::DeserializationResult::BufferInsufficient;
                    }
                }});                
                field_index.append(quote!{ +1});
            } else if is_dynamic_array(field_type) {
                let element_type = {
                    if let syn::Ty::Path(_, ref path) = *field_type {
                        if let syn::PathParameters::AngleBracketed(ref param_data) = path.segments.as_slice().last().unwrap().parameters {
                            &param_data.types[0]
                        } else {
                            panic!("Element type name not found")
                        }
                    } else {
                        panic!("Type name not found")
                    }
                };
                
                // check for tail optimization
                if i == variant_data.fields().len() - 1 {
                    deserialize_builder.append(quote!{if *flattened_field == #field_index {
                        let mut skewed_bit = *bit + #field_type_path_segment::<#element_type>::length_bit_length();
                        self.#field_ident.set_length( ( #element_type::bit_length()-1 + *bit + buffer.bit_length()) / #element_type::bit_length() );
                        self.#field_ident.deserialize(&mut skewed_bit, buffer);
                        *bit = skewed_bit - #field_type_path_segment::<#element_type>::length_bit_length();
                        return uavcan::deserializer::DeserializationResult::Finished;                         
                    }});
                    field_index.append(quote!{ +1});
                } else {
                    deserialize_builder.append(quote!{if *flattened_field == #field_index {
                        if self.#field_ident.deserialize(bit, buffer) == uavcan::deserializer::DeserializationResult::Finished {
                            *flattened_field += 1;
                            *bit = 0;
                        } else {
                            return uavcan::deserializer::DeserializationResult::BufferInsufficient;
                        }
                    }});
                    field_index.append(quote!{ +1});
                }
            } else {                
                deserialize_builder.append(quote!{if *flattened_field >= #field_index && *flattened_field < #field_index + self.#field_ident.flattened_fields_len() {
                    let mut current_field = *flattened_field - #field_index;
                    if self.#field_ident.deserialize(&mut current_field, bit, buffer) == uavcan::deserializer::DeserializationResult::Finished {
                        *flattened_field = #field_index + self.#field_ident.flattened_fields_len();
                        *bit = 0;
                    } else {
                        *flattened_field = #field_index + current_field;
                        return uavcan::deserializer::DeserializationResult::BufferInsufficient;
                    }
                }});
                field_index.append(quote!{ +self.#field_ident.flattened_fields_len()});
            }
        }
        deserialize_builder

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
        impl uavcan::AsUavcanField for #name{
            fn as_uavcan_field(&self) -> uavcan::UavcanField{
                uavcan::UavcanField::UavcanStruct(self)
            }
            fn as_mut_uavcan_field(&mut self) -> uavcan::MutUavcanField{
                uavcan::MutUavcanField::UavcanStruct(self)
            }
        }

        impl UavcanStruct for #name {
            fn fields_len(&self) -> usize {
                #number_of_fields
            }

            fn flattened_fields_len(&self) -> usize {
                #number_of_flattened_fields
            }

            fn serialize(&self, flattened_field: &mut usize, bit: &mut usize, buffer: &mut uavcan::serializer::SerializationBuffer) -> uavcan::serializer::SerializationResult {
                while *flattened_field != self.flattened_fields_len(){
                    #serialize_body
                }
                uavcan::SerializationResult::Finished
            }

            fn deserialize(&mut self, flattened_field: &mut usize, bit: &mut usize, buffer: &mut uavcan::deserializer::DeserializationBuffer) -> uavcan::deserializer::DeserializationResult {
                while *flattened_field != self.flattened_fields_len(){
                    #deserialize_body
                }
                uavcan::DeserializationResult::Finished
            }

            fn field_as_mut(&mut self, field_number: usize) -> uavcan::MutUavcanField {
                #field_as_mut_body
            }

            fn field(&self, field_number: usize) -> uavcan::UavcanField {
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
