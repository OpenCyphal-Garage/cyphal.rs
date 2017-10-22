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

    let tail_array_optimizable = is_dynamic_array(&variant_data.fields().last().unwrap().ty);        

    let number_of_flattened_fields = {
        let mut flattened_fields_builder = Tokens::new();

        for (i, field) in variant_data.fields().iter().enumerate() {
            let field_type = &field.ty;
            
            if i != 0 {
                flattened_fields_builder.append(quote!{+});
            }
            if is_primitive_type(field_type) || is_dynamic_array(field_type) {
                flattened_fields_builder.append(quote!{1});
            } else {
                flattened_fields_builder.append(quote!{#field_type::FLATTENED_FIELDS_NUMBER});
            }
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
                    if uavcan::types::PrimitiveType::serialize(&self.#field_ident, bit, buffer) == uavcan::SerializationResult::Finished {
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
                        let mut skewed_bit = *bit + #field_type_path_segment::<#element_type>::length_bit_length();
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
                serialize_builder.append(quote!{if *flattened_field >= #field_index && *flattened_field < #field_index + #field_type::FLATTENED_FIELDS_NUMBER {
                    let mut current_field = *flattened_field - #field_index;
                    if self.#field_ident.serialize(&mut current_field, bit, buffer) == uavcan::SerializationResult::Finished {
                        *flattened_field = #field_index + #field_type::FLATTENED_FIELDS_NUMBER;
                        *bit = 0;
                    } else {
                        *flattened_field = #field_index + current_field;
                        return uavcan::SerializationResult::BufferFull;
                    }
                }});
                field_index.append(quote!{ + #field_type::FLATTENED_FIELDS_NUMBER});
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
                    if uavcan::types::PrimitiveType::deserialize(&mut self.#field_ident, bit, buffer) == uavcan::DeserializationResult::Finished {
                        *flattened_field += 1;
                        *bit = 0;
                    } else {
                        return uavcan::DeserializationResult::BufferInsufficient;
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
                        self.#field_ident.set_length( ( <#element_type as uavcan::types::PrimitiveType>::BIT_LENGTH-1 + *bit + buffer.bit_length()) / <#element_type as uavcan::types::PrimitiveType>::BIT_LENGTH );
                        self.#field_ident.deserialize(&mut skewed_bit, buffer);
                        *bit = skewed_bit - #field_type_path_segment::<#element_type>::length_bit_length();
                        return uavcan::DeserializationResult::Finished;                         
                    }});
                    field_index.append(quote!{ +1});
                } else {
                    deserialize_builder.append(quote!{if *flattened_field == #field_index {
                        if self.#field_ident.deserialize(bit, buffer) == uavcan::DeserializationResult::Finished {
                            *flattened_field += 1;
                            *bit = 0;
                        } else {
                            return uavcan::DeserializationResult::BufferInsufficient;
                        }
                    }});
                    field_index.append(quote!{ +1});
                }
            } else {                
                deserialize_builder.append(quote!{if *flattened_field >= #field_index && *flattened_field < #field_index + #field_type::FLATTENED_FIELDS_NUMBER {
                    let mut current_field = *flattened_field - #field_index;
                    if self.#field_ident.deserialize(&mut current_field, bit, buffer) == uavcan::DeserializationResult::Finished {
                        *flattened_field = #field_index + #field_type::FLATTENED_FIELDS_NUMBER;
                        *bit = 0;
                    } else {
                        *flattened_field = #field_index + current_field;
                        return uavcan::DeserializationResult::BufferInsufficient;
                    }
                }});
                field_index.append(quote!{ + #field_type::FLATTENED_FIELDS_NUMBER});
            }
        }
        deserialize_builder

    };

    let bit_length_body = {
        let mut bit_length_builder = Tokens::new();
        
        for (i, field) in variant_data.fields().iter().enumerate() {
            let field_type = &field.ty;
            let field_type_path_segment: &syn::Ident = {
                if let syn::Ty::Path(_, ref path) = *field_type {
                    &path.segments.as_slice().last().unwrap().ident
                } else {
                    panic!("Type name not found")
                }
            };
            
            if i != 0 {bit_length_builder.append(quote!{ + });}
            
            if i == variant_data.fields().len() - 1 && is_dynamic_array(field_type) {
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
                
                bit_length_builder.append(bit_length(field));
                bit_length_builder.append(quote!{ - #field_type_path_segment::<#element_type>::length_bit_length()});
            } else {
                bit_length_builder.append(bit_length(field));
            }
        }
        
        bit_length_builder
    };
    
    
    quote!{
        impl uavcan::Struct for #name {
            const TAIL_ARRAY_OPTIMIZABLE: bool = #tail_array_optimizable;
            const FLATTENED_FIELDS_NUMBER: usize = #number_of_flattened_fields;

            fn bit_length(&self) -> usize {
                #bit_length_body
            }

            #[allow(unused_comparisons)]
            fn serialize(&self, flattened_field: &mut usize, bit: &mut usize, buffer: &mut uavcan::SerializationBuffer) -> uavcan::SerializationResult {
                assert!(*flattened_field < Self::FLATTENED_FIELDS_NUMBER);
                while *flattened_field != Self::FLATTENED_FIELDS_NUMBER{
                    #serialize_body
                }
                uavcan::SerializationResult::Finished
            }

            #[allow(unused_comparisons)]
            fn deserialize(&mut self, flattened_field: &mut usize, bit: &mut usize, buffer: &mut uavcan::DeserializationBuffer) -> uavcan::DeserializationResult {
                assert!(*flattened_field < Self::FLATTENED_FIELDS_NUMBER);
                while *flattened_field != Self::FLATTENED_FIELDS_NUMBER{
                    #deserialize_body
                }
                uavcan::DeserializationResult::Finished
            }

        }

    }
}


fn is_primitive_type(type_name: &syn::Ty) -> bool {
    if *type_name == syn::parse::ty("u2").expect("") ||
        *type_name == syn::parse::ty("u3").expect("") ||
        *type_name == syn::parse::ty("u4").expect("") ||
        *type_name == syn::parse::ty("u5").expect("") ||
        *type_name == syn::parse::ty("u7").expect("") ||
        *type_name == syn::parse::ty("u8").expect("") ||
        *type_name == syn::parse::ty("u16").expect("") ||
        *type_name == syn::parse::ty("u32").expect("") ||
        *type_name == syn::parse::ty("f16").expect("") {
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

fn bit_length(field: &syn::Field) -> Tokens {
    let field_ident = &field.ident;
    let field_type = &field.ty;
    let field_type_path_segment: &syn::Ident = {
        if let syn::Ty::Path(_, ref path) = *field_type {
            &path.segments.as_slice().last().unwrap().ident
        } else {
            panic!("Type name not found")
        }
    };

    if is_primitive_type(field_type) {
        quote!{<#field_type as uavcan::types::PrimitiveType>::BIT_LENGTH}
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

        quote!{(#field_type_path_segment::<#element_type>::length_bit_length() + self.#field_ident.length().current_length * #element_type::BIT_LENGTH)}
    } else {
        quote!{self.#field_ident.bit_length()}
    }
}
