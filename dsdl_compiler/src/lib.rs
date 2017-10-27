#[macro_use]
extern crate quote;
extern crate dsdl_parser;
extern crate syn;

pub trait Compile<T> {
    fn compile(self) -> T;
}


impl Compile<(syn::Body, Vec<syn::Attribute>)> for dsdl_parser::MessageDefinition {
    fn compile(self) -> (syn::Body, Vec<syn::Attribute>) {
        let (directives, not_directives): (Vec<dsdl_parser::Line>, Vec<dsdl_parser::Line>) = self.0.into_iter().partition(|x| x.is_directive());
        
        // first scan through directives
        let mut union = false;

        for directive in directives {
            if let dsdl_parser::Line::Directive(dsdl_parser::Directive::Union, _) = directive {
                union = true;
            }
        }

        

        if union {
            unimplemented!("Unions are not implemented yet")
        } else {
            let mut fields = Vec::new();
            let mut current_comments = Vec::new();
            
            for line in not_directives.clone() {
                if let dsdl_parser::Line::Comment(comment) = line {
                    current_comments.push(comment.compile());
                } else {
                    break
                }
            }
            let attributes = current_comments.clone();

            for line in not_directives {
                match line {
                    dsdl_parser::Line::Empty => current_comments = Vec::new(),
                    dsdl_parser::Line::Comment(comment) => current_comments.push(comment.compile()),
                    dsdl_parser::Line::Definition(dsdl_parser::AttributeDefinition::Field(def), opt_comment) => {
                        if let Some(comment) = opt_comment {
                            current_comments.push(comment.compile());
                        }
                        let mut field = def.compile();
                        field.attrs = current_comments.clone();
                        fields.push(field);
                        
                        current_comments = Vec::new();
                    },
                    dsdl_parser::Line::Definition(dsdl_parser::AttributeDefinition::Const(_), _) => (), // const definitions is only used in the impls
                    dsdl_parser::Line::Directive(_, _) => unreachable!("All directives was removed at the start"),
                }
            }
            (syn::Body::Struct(syn::VariantData::Struct(fields)), attributes)
        }
    }
}


impl Compile<syn::Field> for dsdl_parser::FieldDefinition {
    fn compile(self) -> syn::Field {
        let ty = match self.array {
            dsdl_parser::ArrayInfo::Single => self.field_type.compile(),
            dsdl_parser::ArrayInfo::DynamicLess(size) => syn::Ty::Path(
                None, syn::Path{
                    global: false,
                    segments: vec![syn::PathSegment{
                        ident: syn::Ident::from("Dynamic"),
                        parameters: syn::PathParameters::AngleBracketed(syn::AngleBracketedParameterData{
                            lifetimes: Vec::new(),
                            types: vec![syn::Ty::Array(Box::new(self.field_type.compile()), dsdl_parser::Size::from(u64::from(size) - 1).compile())],
                            bindings: Vec::new(),
                        })
                    }],
                }),
            dsdl_parser::ArrayInfo::DynamicLeq(size) => syn::Ty::Path(
                None, syn::Path{
                    global: false,
                    segments: vec![syn::PathSegment{
                        ident: syn::Ident::from("Dynamic"),
                        parameters: syn::PathParameters::AngleBracketed(syn::AngleBracketedParameterData{
                            lifetimes: Vec::new(),
                            types: vec![syn::Ty::Array(Box::new(self.field_type.compile()), size.compile())],
                            bindings: Vec::new(),
                        })
                    }],
                }),
            dsdl_parser::ArrayInfo::Static(size) => syn::Ty::Array(Box::new(self.field_type.compile()), size.compile()),
        };
        
        syn::Field{
            ident: self.name.map(|x| x.compile()),
            vis: syn::Visibility::Public,
            attrs: Vec::new(),
            ty: ty,
        }
    }
}

impl Compile<syn::ConstExpr> for dsdl_parser::Size {
    fn compile(self) -> syn::ConstExpr {
        syn::ConstExpr::Lit(self.compile())
    }    
}
    
impl Compile<syn::Lit> for dsdl_parser::Size {
    fn compile(self) -> syn::Lit {
        syn::Lit::Int(self.into(), syn::IntTy::Unsuffixed)
    }    
}
    
impl Compile<syn::Ident> for dsdl_parser::Ident {
    fn compile(self) -> syn::Ident {
        syn::Ident::from(self.as_ref())
    }
}

impl Compile<syn::Attribute> for dsdl_parser::Comment {
    fn compile(self) -> syn::Attribute {
        syn::Attribute{
            style: syn::AttrStyle::Outer,
            value: syn::MetaItem::NameValue(syn::Ident::from("doc"), syn::Lit::Str(String::from(self.as_ref()), syn::StrStyle::Raw(0))),
            is_sugared_doc: true,
        }
    }
}

impl Compile<syn::Ty> for dsdl_parser::Ty {
    fn compile(self) -> syn::Ty {
        match self {
            dsdl_parser::Ty::Primitive(x) => x.compile(),
            dsdl_parser::Ty::Composite(x) => x.compile(),
        }
    }
}

impl Compile<syn::Ty> for dsdl_parser::CompositeType {
    fn compile(self) -> syn::Ty {
        let mut path = syn::Path {
            global: false,
            segments: Vec::new(),
        };
        
        if let Some(namespace) = self.namespace {
            for segment in namespace.as_ref().split(".") {
                path.segments.push(syn::PathSegment{ident: syn::Ident::from(segment), parameters: syn::PathParameters::none()});
            }
        }
        
        path.segments.push(syn::PathSegment{ident: self.name.compile(), parameters: syn::PathParameters::none()});
        
        syn::Ty::Path(None, path)
    }
}   

impl Compile<syn::Ty> for dsdl_parser::PrimitiveType {
    fn compile(self) -> syn::Ty {
        match self {
            dsdl_parser::PrimitiveType::Bool => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("bool"), parameters: syn::PathParameters::none()})}),
            
            dsdl_parser::PrimitiveType::Float16 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("f16"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Float32 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("f32"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Float64 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("f64"), parameters: syn::PathParameters::none()})}),
            
            dsdl_parser::PrimitiveType::Uint2 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u2"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint3 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u3"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint4 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u4"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint5 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u5"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint6 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u6"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint7 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u7"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint8 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u8"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint9 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u9"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint10 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u10"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint11 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u11"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint12 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u12"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint13 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u13"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint14 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u14"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint15 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u15"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint16 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u16"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint17 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u17"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint18 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u18"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint19 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u19"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint20 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u20"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint21 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u21"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint22 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u22"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint23 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u23"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint24 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u24"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint25 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u25"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint26 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u26"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint27 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u27"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint28 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u28"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint29 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u29"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint30 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u30"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint31 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u31"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint32 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u32"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint33 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u33"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint34 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u34"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint35 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u35"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint36 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u36"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint37 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u37"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint38 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u38"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint39 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u39"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint40 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u40"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint41 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u41"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint42 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u42"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint43 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u43"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint44 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u44"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint45 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u45"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint46 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u46"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint47 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u47"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint48 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u48"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint49 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u49"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint50 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u50"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint51 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u51"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint52 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u52"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint53 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u53"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint54 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u54"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint55 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u55"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint56 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u56"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint57 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u57"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint58 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u58"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint59 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u59"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint60 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u60"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint61 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u61"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint62 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u62"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint63 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u63"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Uint64 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("u64"), parameters: syn::PathParameters::none()})}),
            
            dsdl_parser::PrimitiveType::Int2 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i2"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int3 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i3"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int4 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i4"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int5 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i5"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int6 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i6"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int7 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i7"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int8 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i8"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int9 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i9"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int10 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i10"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int11 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i11"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int12 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i12"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int13 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i13"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int14 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i14"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int15 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i15"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int16 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i16"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int17 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i17"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int18 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i18"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int19 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i19"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int20 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i20"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int21 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i21"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int22 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i22"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int23 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i23"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int24 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i24"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int25 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i25"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int26 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i26"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int27 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i27"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int28 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i28"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int29 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i29"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int30 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i30"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int31 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i31"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int32 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i32"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int33 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i33"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int34 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i34"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int35 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i35"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int36 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i36"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int37 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i37"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int38 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i38"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int39 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i39"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int40 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i40"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int41 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i41"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int42 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i42"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int43 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i43"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int44 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i44"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int45 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i45"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int46 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i46"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int47 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i47"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int48 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i48"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int49 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i49"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int50 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i50"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int51 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i51"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int52 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i52"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int53 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i53"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int54 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i54"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int55 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i55"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int56 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i56"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int57 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i57"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int58 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i58"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int59 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i59"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int60 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i60"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int61 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i61"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int62 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i62"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int63 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i63"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Int64 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("i64"), parameters: syn::PathParameters::none()})}),
            
            dsdl_parser::PrimitiveType::Void1 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void1"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void2 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void2"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void3 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void3"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void4 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void4"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void5 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void5"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void6 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void6"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void7 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void7"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void8 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void8"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void9 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void9"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void10 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void10"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void11 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void11"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void12 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void12"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void13 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void13"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void14 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void14"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void15 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void15"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void16 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void16"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void17 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void17"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void18 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void18"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void19 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void19"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void20 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void20"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void21 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void21"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void22 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void22"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void23 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void23"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void24 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void24"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void25 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void25"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void26 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void26"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void27 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void27"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void28 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void28"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void29 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void29"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void30 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void30"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void31 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void31"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void32 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void32"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void33 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void33"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void34 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void34"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void35 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void35"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void36 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void36"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void37 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void37"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void38 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void38"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void39 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void39"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void40 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void40"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void41 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void41"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void42 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void42"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void43 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void43"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void44 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void44"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void45 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void45"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void46 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void46"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void47 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void47"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void48 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void48"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void49 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void49"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void50 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void50"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void51 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void51"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void52 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void52"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void53 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void53"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void54 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void54"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void55 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void55"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void56 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void56"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void57 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void57"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void58 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void58"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void59 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void59"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void60 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void60"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void61 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void61"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void62 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void62"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void63 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void63"), parameters: syn::PathParameters::none()})}),
            dsdl_parser::PrimitiveType::Void64 => syn::Ty::Path(None, syn::Path{global: false, segments: vec!(syn::PathSegment{ident: syn::Ident::from("void64"), parameters: syn::PathParameters::none()})}),
        }
    }
}

#[cfg(test)]
mod tests {
    use *;
    use dsdl_parser::PrimitiveType;
    use dsdl_parser::AttributeDefinition;
    use dsdl_parser::Ty;
    use dsdl_parser::ArrayInfo;
    use dsdl_parser::Size;
    use dsdl_parser::Comment;
    use dsdl_parser::Line;


    #[test]
    fn compile_struct_body() {
        let body = dsdl_parser::MessageDefinition(
            vec![Line::Comment(Comment::from("about struct0")),
                 Line::Comment(Comment::from("about struct1")),
                 Line::Empty,
                 Line::Comment(Comment::from("test comment0")),
                 Line::Definition(AttributeDefinition::Field(dsdl_parser::FieldDefinition{
                     cast_mode: None,
                     field_type: Ty::Primitive(PrimitiveType::Uint8),
                     array: ArrayInfo::Single,
                     name: Some(dsdl_parser::Ident::from("node_status")),
                 }) , Some(Comment::from("test comment1"))),
                 Line::Comment(Comment::from("ignored comment")),
                 Line::Empty,
                 Line::Comment(Comment::from("test comment2")),
                 Line::Definition(AttributeDefinition::Field(dsdl_parser::FieldDefinition{
                     cast_mode: None,
                     field_type: Ty::Primitive(PrimitiveType::Uint7),
                     array: ArrayInfo::Single,
                     name: Some(dsdl_parser::Ident::from("node_something")),
                 }) , Some(Comment::from("test comment3"))),

            ]
        ).compile();

        let struct_body = if let syn::Body::Struct(x) = body.0 {
            x
        } else {
            unreachable!("This is a struct")
        };

        let struct_attributes = body.1;

        assert_eq!(quote!(
            ///about struct0
            ///about struct1
        ), quote!{#(#struct_attributes)*});
        
        assert_eq!(quote!({
            ///test comment0
            ///test comment1
            pub node_status: u8,
            ///test comment2
            ///test comment3
            pub node_something: u7
        }), quote!{#struct_body});
    }
    
    #[test]
    fn compile_field_def() {
        let simple_field = dsdl_parser::FieldDefinition{
            cast_mode: None,
            field_type: dsdl_parser::Ty::Primitive(PrimitiveType::Uint3),
            array: dsdl_parser::ArrayInfo::Single,
            name: Some(dsdl_parser::Ident::from("name")),
        }.compile();

        assert_eq!(quote!(pub name: u3), quote!{#simple_field});

        let composite_field = dsdl_parser::FieldDefinition{
            cast_mode: None,
            field_type: Ty::Composite(dsdl_parser::CompositeType{namespace: Some(dsdl_parser::Ident::from("uavcan.protocol")), name: dsdl_parser::Ident::from("NodeStatus")}),
            array: dsdl_parser::ArrayInfo::Single,
            name: Some(dsdl_parser::Ident::from("name")),
        }.compile();

        assert_eq!(quote!(pub name: uavcan::protocol::NodeStatus), quote!{#composite_field});

        let array_field = dsdl_parser::FieldDefinition{
            cast_mode: None,
            field_type: dsdl_parser::Ty::Primitive(PrimitiveType::Uint3),
            array: dsdl_parser::ArrayInfo::Static(Size::from(19u64)),
            name: Some(dsdl_parser::Ident::from("name")),
        }.compile();

        assert_eq!(quote!(pub name: [u3; 19]), quote!{#array_field});

        let dynleq_array_field = dsdl_parser::FieldDefinition{
            cast_mode: None,
            field_type: dsdl_parser::Ty::Primitive(PrimitiveType::Int29),
            array: dsdl_parser::ArrayInfo::DynamicLeq(Size::from(191u64)),
            name: Some(dsdl_parser::Ident::from("name")),
        }.compile();

        assert_eq!(quote!(pub name: Dynamic<[i29; 191]>), quote!{#dynleq_array_field});
        
        let dynless_array_field = dsdl_parser::FieldDefinition{
            cast_mode: None,
            field_type: dsdl_parser::Ty::Primitive(PrimitiveType::Bool),
            array: dsdl_parser::ArrayInfo::DynamicLeq(Size::from(370u64)),
            name: Some(dsdl_parser::Ident::from("name")),
        }.compile();
        
        assert_eq!(quote!(pub name: Dynamic<[bool; 370]>), quote!{#dynless_array_field});

    }
        

    #[test]
    fn compile_type() {
        let composite = Ty::Composite(dsdl_parser::CompositeType{namespace: Some(dsdl_parser::Ident::from("uavcan.protocol")), name: dsdl_parser::Ident::from("NodeStatus")}).compile();
        assert_eq!(quote!(uavcan::protocol::NodeStatus), quote!{#composite});

        let primitive = Ty::Primitive(PrimitiveType::Uint2).compile();
        assert_eq!(quote!(u2), quote!{#primitive});

    }
    
    #[test]
    fn compile_composite_type() {
        let t = dsdl_parser::CompositeType{namespace: Some(dsdl_parser::Ident::from("uavcan.protocol")), name: dsdl_parser::Ident::from("NodeStatus")}.compile();
        assert_eq!(quote!(uavcan::protocol::NodeStatus), quote!{#t});
    }
    
    #[test]
    fn compile_primitive_type() {
        let uint2 = PrimitiveType::Uint2.compile();
        assert_eq!(quote!(u2), quote!{#uint2});
        
        let int9 = PrimitiveType::Int9.compile();
        assert_eq!(quote!(i9), quote!{#int9});
        
        let void23 = PrimitiveType::Void23.compile();
        assert_eq!(quote!(void23), quote!{#void23});
        
        let b = PrimitiveType::Bool.compile();
        assert_eq!(quote!(bool), quote!{#b});
        
        let float64 = PrimitiveType::Float64.compile();
        assert_eq!(quote!(f64), quote!{#float64});
    }
    
    #[test]
    fn compile_comment() {
        let comment = Comment::from(" test comment").compile();
        assert_eq!(quote!{/// test comment
        }, quote!{#comment});
    }
}
