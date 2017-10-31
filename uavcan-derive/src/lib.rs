#![recursion_limit="128"]

extern crate regex;
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use regex::Regex;
use proc_macro::TokenStream;
use syn::Body;
use syn::Ident;
use quote::Tokens;


#[proc_macro_derive(UavcanStruct, attributes(DSDLSignature, DataTypeSignature))]
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

    // first handle the attributes

    let mut dsdl_signature = quote!{0x00};
    let mut data_type_signature = quote!{0x00};
    
    for attr in &ast.attrs {
        if let syn::MetaItem::NameValue(ref ident, ref lit) = attr.value {
            if ident == "DSDLSignature" {
                if let syn::Lit::Str(ref lit_str, _) = *lit {
                    let value = Ident::from(lit_str.clone()); // hack needed since only string literals is supported for attributes
                    dsdl_signature = quote!{#value};
                } else {
                    panic!("DSDLSignature must be on the form \"0x123456789abc\"");
                }
            } else if ident == "DataTypeSignature" {
                if let syn::Lit::Str(ref lit_str, _) = *lit {
                    let value = Ident::from(lit_str.clone()); // hack needed since only string literals is supported for attributes
                    data_type_signature = quote!{#value};
                } else {
                    panic!("Data type signature must be on the form \"0x123456789abc\"");
                }
            }
        }
    }

    
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
                let array_type = array_from_dynamic(field_type);
                
                // check for tail optimization
                if i == variant_data.fields().len() - 1 {
                    serialize_builder.append(quote!{if *flattened_field == #field_index {
                        let mut skewed_bit = *bit + Dynamic::<#array_type>::LENGTH_BITS;
                        if self.#field_ident.serialize(&mut skewed_bit, buffer) == uavcan::SerializationResult::Finished {
                            *flattened_field += 1;
                            *bit = 0;
                        } else {
                            *bit = skewed_bit - Dynamic::<#array_type>::LENGTH_BITS;
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
                let array_type = array_from_dynamic(field_type).unwrap();
                let element_type = if let syn::Ty::Array(ref element_type, _) = array_type {
                    element_type
                } else {
                    panic!("element type name not found")
                };
                
                // check for tail optimization
                if i == variant_data.fields().len() - 1 {
                    deserialize_builder.append(quote!{if *flattened_field == #field_index {
                        let mut skewed_bit = *bit + Dynamic::<#array_type>::LENGTH_BITS;
                        self.#field_ident.set_length( ( <#element_type as uavcan::types::PrimitiveType>::BIT_LENGTH-1 + *bit + buffer.bit_length()) / <#element_type as uavcan::types::PrimitiveType>::BIT_LENGTH );
                        self.#field_ident.deserialize(&mut skewed_bit, buffer);
                        *bit = skewed_bit - Dynamic::<#array_type>::LENGTH_BITS;
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
            
            if i != 0 {bit_length_builder.append(quote!{ + });}
            
            if i == variant_data.fields().len() - 1 && is_dynamic_array(field_type) {
                let array_type = array_from_dynamic(field_type);
                    
                bit_length_builder.append(bit_length(field));
                bit_length_builder.append(quote!{ - Dynamic::<#array_type>::LENGTH_BITS});
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

            const DSDL_SIGNATURE: u64 = #dsdl_signature;
            const DATA_TYPE_SIGNATURE: u64 = #data_type_signature;

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

fn is_primitive_type(ty: &syn::Ty) -> bool {
    is_unsigned_primitive_type(ty) || is_signed_primitive_type(ty) || is_void_primitive_type(ty)
}

fn is_unsigned_primitive_type(ty: &syn::Ty) -> bool {
    if let syn::Ty::Path(_, ref path) = *ty {
        let re = Regex::new(r"u([2-9]|[1-5][0-9]|6[0-4])").unwrap();
        re.is_match(path.segments.as_slice().last().unwrap().ident.as_ref())
    } else {
        false
    }
}

fn is_signed_primitive_type(ty: &syn::Ty) -> bool {
    if let syn::Ty::Path(_, ref path) = *ty {
        let re = Regex::new(r"i([2-9]|[1-5][0-9]|6[0-4])").unwrap();
        re.is_match(path.segments.as_slice().last().unwrap().ident.as_ref())
    } else {
        false
    }
}

fn is_void_primitive_type(ty: &syn::Ty) -> bool {
    if let syn::Ty::Path(_, ref path) = *ty {
        let re = Regex::new(r"i([2-9]|[1-5][0-9]|6[0-4])").unwrap();
        re.is_match(path.segments.as_slice().last().unwrap().ident.as_ref())
    } else {
        false
    }
}

fn is_dynamic_array(type_name: &syn::Ty) -> bool {
    if let syn::Ty::Path(_, ref path) = *type_name {
        if path.segments.as_slice().last().unwrap().ident == syn::parse::ident("Dynamic").expect("") {
            return true;
        }
    }
    false
}

fn array_from_dynamic(type_name: &syn::Ty) -> Option<syn::Ty> {
    if let syn::Ty::Path(_, ref path) = *type_name {
        if path.segments.as_slice().last().unwrap().ident == syn::Ident::from("Dynamic") {
            if let syn::PathSegment{
                parameters: syn::PathParameters::AngleBracketed(syn::AngleBracketedParameterData{
                    ref types, ..
                }), ..
            } = *path.segments.as_slice().last().unwrap() {
                return Some(types[0].clone());
            }
        }
    }
    None
}

fn bit_length(field: &syn::Field) -> Tokens {
    let field_ident = &field.ident;
    let field_type = &field.ty;

    if is_primitive_type(field_type) {
        quote!{<#field_type as uavcan::types::PrimitiveType>::BIT_LENGTH}
    } else if is_dynamic_array(field_type) {
        let array_type = array_from_dynamic(field_type).unwrap();
        let element_type = if let syn::Ty::Array(ref element_type, _) = array_type {
            element_type
        } else {
            panic!("element type name not found")
        };
        
        quote!{(Dynamic::<#array_type>::LENGTH_BITS + self.#field_ident.length() * #element_type::BIT_LENGTH)}
    } else {
        quote!{self.#field_ident.bit_length()}
    }
}


#[cfg(test)]
mod tests {
    use *;
    
    #[test]
    fn array_from_dynamic_test() {
        assert_eq!(array_from_dynamic(&syn::parse::ty("Dynamic<[u7; 9]>").expect("")), Some(syn::parse::ty("[u7; 9]").expect(""))); 
    }
}
