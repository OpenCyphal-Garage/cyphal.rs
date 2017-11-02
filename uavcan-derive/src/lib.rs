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

enum UavcanType {
    PrimitiveType,
    DynamicArray,
    StaticArray,
    Struct,
}


#[proc_macro_derive(UavcanStruct, attributes(DSDLSignature, DataTypeSignature, UavcanCrateName))]
pub fn uavcan_sized(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = impl_uavcan_struct(&ast);
    gen.parse().unwrap()
}

fn impl_uavcan_struct(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;

    // first handle the attributes
    let mut dsdl_signature = quote!{0x00};
    let mut data_type_signature = quote!{0x00};
    let mut crate_name = quote!{uavcan};
    
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
            } else if ident == "UavcanCrateName" {
                if let syn::Lit::Str(ref lit_str, _) = *lit {
                    let value = Ident::from(lit_str.clone()); // hack needed since only string literals is supported for attributes
                    crate_name = quote!{#value};
                } else {
                    panic!("Crate name must be on the form \"uavcan_alternative\"");
                }
            }

        }
    }

    
    let number_of_flattened_fields = match ast.body {
        Body::Enum(_) => panic!("UavcanSized is not derivable for enum"),
        Body::Struct(syn::VariantData::Struct(ref fields)) => {
            let mut flattened_fields_builder = Tokens::new();
            
            flattened_fields_builder.append(quote!{0});

            for field in fields {
                let field_type = &field.ty;
                
                flattened_fields_builder.append(quote!{+});
                
                match classify_type(field_type) {
                    UavcanType::PrimitiveType | UavcanType::DynamicArray | UavcanType::StaticArray => flattened_fields_builder.append(quote!{1}),
                    UavcanType::Struct => flattened_fields_builder.append(quote!{#field_type::FLATTENED_FIELDS_NUMBER}),
                }
            }
            
            flattened_fields_builder
        },
        Body::Struct(syn::VariantData::Unit) => quote!{0},
        _ => panic!("UavcanStruct is only derivable for enums and named structs"),
    };
    
    let serialize_body = match ast.body {
        Body::Enum(_) => panic!("UavcanSized is not derivable for enum"),
        Body::Struct(syn::VariantData::Struct(ref fields)) => {
            
            let mut serialize_builder = Tokens::new();
            let mut field_index = Tokens::new();
            
            field_index.append(quote!{0});
            
            for (i, field) in fields.iter().enumerate() {
                let field_ident = &field.ident;
                let field_type = &field.ty;
                
                if i != 0 { serialize_builder.append(quote!{ else });}
                
                match classify_type(field_type) {
                    UavcanType::PrimitiveType => {
                        serialize_builder.append(quote!{if *flattened_field == #field_index {
                            if ::#crate_name::types::PrimitiveType::serialize(&self.#field_ident, bit, buffer) == ::#crate_name::SerializationResult::Finished {
                                *flattened_field += 1;
                                *bit = 0;
                            } else {
                                return ::#crate_name::SerializationResult::BufferFull;
                            }
                        }});
                        field_index.append(quote!{ +1});
                    },
                    UavcanType::StaticArray => {
                        serialize_builder.append(quote!{if *flattened_field == #field_index {
                            if ::#crate_name::types::Array::serialize(&self.#field_ident, bit, buffer) == ::#crate_name::SerializationResult::Finished {
                                *flattened_field += 1;
                                *bit = 0;
                            } else {
                                return ::#crate_name::SerializationResult::BufferFull;
                            }
                        }});
                        field_index.append(quote!{ +1});
                    },
                    UavcanType::DynamicArray => {
                        serialize_builder.append(quote!{if *flattened_field == #field_index {
                            if self.#field_ident.serialize(bit, last_field && *flattened_field == (Self::FLATTENED_FIELDS_NUMBER-1), buffer) == ::#crate_name::SerializationResult::Finished {
                                *flattened_field += 1;
                                *bit = 0;
                            } else {
                                return ::#crate_name::SerializationResult::BufferFull;
                            }
                        }});
                        field_index.append(quote!{ +1});
                    },
                    UavcanType::Struct => {
                        serialize_builder.append(quote!{if *flattened_field >= (#field_index) && *flattened_field < (#field_index) + #field_type::FLATTENED_FIELDS_NUMBER {
                            let mut current_field = *flattened_field - (#field_index);
                            if self.#field_ident.serialize(&mut current_field, bit, last_field && *flattened_field == (Self::FLATTENED_FIELDS_NUMBER-1), buffer) == ::#crate_name::SerializationResult::Finished {
                                *flattened_field = (#field_index) + current_field;
                                *bit = 0;
                            } else {
                                *flattened_field = (#field_index) + current_field;
                                return ::#crate_name::SerializationResult::BufferFull;
                            }
                        }});
                        field_index.append(quote!{ + #field_type::FLATTENED_FIELDS_NUMBER});
                    },
                }
            }
            serialize_builder
        },
        Body::Struct(syn::VariantData::Unit) => quote!{0},
        _ => panic!("UavcanStruct is only derivable for enums and named structs"),
    };
    
    let deserialize_body = match ast.body {
        Body::Enum(_) => panic!("UavcanSized is not derivable for enum"),
        Body::Struct(syn::VariantData::Struct(ref fields)) => {
            let mut deserialize_builder = Tokens::new();
            let mut field_index = Tokens::new();
            
            field_index.append(quote!{0});
            
            for (i, field) in fields.iter().enumerate() {
                let field_ident = &field.ident;
                let field_type = &field.ty;
                
                if i != 0 { deserialize_builder.append(quote!{ else });}

                match classify_type(field_type) {
                    UavcanType::PrimitiveType => {
                        deserialize_builder.append(quote!{if *flattened_field == #field_index {
                            if ::#crate_name::types::PrimitiveType::deserialize(&mut self.#field_ident, bit, buffer) == ::#crate_name::DeserializationResult::Finished {
                                *flattened_field += 1;
                                *bit = 0;
                            } else {
                                return ::#crate_name::DeserializationResult::BufferInsufficient;
                            }
                        }});                
                        field_index.append(quote!{ +1});
                    },
                    UavcanType::StaticArray => {
                        deserialize_builder.append(quote!{if *flattened_field == #field_index {
                            if ::#crate_name::types::Array::deserialize(&mut self.#field_ident, bit, buffer) == ::#crate_name::DeserializationResult::Finished {
                                *flattened_field += 1;
                                *bit = 0;
                            } else {
                                return ::#crate_name::DeserializationResult::BufferInsufficient;
                            }
                        }});                
                        field_index.append(quote!{ +1});
                    },
                    UavcanType::DynamicArray => {
                        deserialize_builder.append(quote!{if *flattened_field == #field_index {
                            if self.#field_ident.deserialize(bit, last_field && *flattened_field == (Self::FLATTENED_FIELDS_NUMBER-1), buffer) == ::#crate_name::DeserializationResult::Finished {
                                *flattened_field += 1;
                                *bit = 0;
                            } else {
                                return ::#crate_name::DeserializationResult::BufferInsufficient;
                            }
                        }});
                        field_index.append(quote!{ +1});
                    },
                    UavcanType::Struct => {
                        deserialize_builder.append(quote!{if *flattened_field >= (#field_index) && *flattened_field < (#field_index) + #field_type::FLATTENED_FIELDS_NUMBER {
                            let mut current_field = *flattened_field - (#field_index);
                            if self.#field_ident.deserialize(&mut current_field, bit, last_field && *flattened_field == (Self::FLATTENED_FIELDS_NUMBER-1), buffer) == ::#crate_name::DeserializationResult::Finished {
                                *flattened_field = (#field_index) + current_field;
                                *bit = 0;
                            } else {
                                *flattened_field = (#field_index) + current_field;
                                return ::#crate_name::DeserializationResult::BufferInsufficient;
                            }
                        }});
                        field_index.append(quote!{ + #field_type::FLATTENED_FIELDS_NUMBER});
                    },
                }            
            }
            deserialize_builder   
        },
        Body::Struct(syn::VariantData::Unit) => quote!{0},
        _ => panic!("UavcanStruct is only derivable for enums and named structs"),
    };
    
    
    quote!{
        impl ::#crate_name::Struct for #name {
            const FLATTENED_FIELDS_NUMBER: usize = #number_of_flattened_fields;

            const DSDL_SIGNATURE: u64 = #dsdl_signature;
            const DATA_TYPE_SIGNATURE: u64 = #data_type_signature;

            #[allow(unused_comparisons)]
            #[allow(unused_variables)]
            fn serialize(&self, flattened_field: &mut usize, bit: &mut usize, last_field: bool, buffer: &mut ::#crate_name::SerializationBuffer) -> ::#crate_name::SerializationResult {
                assert!(*flattened_field < Self::FLATTENED_FIELDS_NUMBER);
                while *flattened_field != Self::FLATTENED_FIELDS_NUMBER{
                    assert!(*flattened_field < Self::FLATTENED_FIELDS_NUMBER);
                    #serialize_body
                }
                ::#crate_name::SerializationResult::Finished
            }

            #[allow(unused_comparisons)]
            #[allow(unused_variables)]
            fn deserialize(&mut self, flattened_field: &mut usize, bit: &mut usize, last_field: bool, buffer: &mut ::#crate_name::DeserializationBuffer) -> ::#crate_name::DeserializationResult {
                assert!(*flattened_field < Self::FLATTENED_FIELDS_NUMBER);
                while *flattened_field != Self::FLATTENED_FIELDS_NUMBER{
                    assert!(*flattened_field < Self::FLATTENED_FIELDS_NUMBER);
                    #deserialize_body
                }
                ::#crate_name::DeserializationResult::Finished
            }


       }

    }
}

fn classify_type(ty: &syn::Ty) -> UavcanType {
    if is_primitive_type(ty) {
        UavcanType::PrimitiveType
    } else if is_dynamic_array(ty) {
        UavcanType::DynamicArray
    } else if is_static_array(ty) {
        UavcanType::StaticArray
    } else {
        UavcanType::Struct
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
        let re = Regex::new(r"void([1-9]|[1-5][0-9]|6[0-4])").unwrap();
        re.is_match(path.segments.as_slice().last().unwrap().ident.as_ref())
    } else {
        false
    }
}

fn is_static_array(ty: &syn::Ty) -> bool {
    if let syn::Ty::Array(_, _) = *ty {
        true
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
