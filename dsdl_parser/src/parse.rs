use std::str;
use std::str::FromStr;

use *;

use nom::{
    not_line_ending,
    is_digit,
    is_hex_digit,
};

named!(whitespace, take_while!(is_whitespace));

named!(version<Version>, do_parse!(
    major: map!(map_res!(take_while!(is_digit), str::from_utf8), String::from) >>
        _dot: tag!(".") >>
        minor: map!(map_res!(take_while!(is_digit), str::from_utf8), String::from) >>
        (Version{major: major, minor: minor})
));


named!(comment<Comment>, map!(map_res!(complete!(preceded!(tag!("#"), not_line_ending)), str::from_utf8), Comment::from));

named!(directive<Directive>, map_res!(map_res!(do_parse!(_tag: tag!("@") >> name: take_while!(is_allowed_in_directive_name) >> (name)), str::from_utf8), Directive::from_str));

named!(service_response_marker<ServiceResponseMarker>, do_parse!(_srm: tag!("---") >> (ServiceResponseMarker{})));

named!(constant<Const>, alt!(
    complete!(do_parse!(_value: tag!("true") >> (Const::Bool(true)) )) |
    complete!(do_parse!(_value: tag!("false") >> (Const::Bool(false)) )) |
    complete!(do_parse!(_format: tag!("0x") >> value: map_res!(take_while!(is_hex_digit), str::from_utf8) >> (Const::Hex(String::from(value))))) |
    complete!(do_parse!(_format: tag!("0b") >> value: map_res!(take_while!(is_bin_digit), str::from_utf8) >> (Const::Bin(String::from(value))))) |
    complete!(do_parse!(achar: delimited!(tag!("'"), map_res!(take_until!("'"), str::from_utf8), tag!("'")) >> (Const::Char(String::from(achar))))) |
    complete!(do_parse!(value: map_res!(take_while!(allowed_in_decimal_number), str::from_utf8) >> (Const::Dec(String::from(value)))))
));

named!(cast_mode<CastMode>, map_res!(map_res!(
    alt!(
        complete!(tag!("saturated")) |
        complete!(tag!("truncated")) 
    ), str::from_utf8), CastMode::from_str)
);

named!(field_name<Ident>, map!(map_res!(
    verify!(take_while!(is_allowed_in_field_name), |x:&[u8]| x.len() >= 1 && is_lowercase_char(x[0])),
    str::from_utf8), Ident::from)
);

named!(const_name<Ident>, map!(map_res!(
    verify!(take_while!(is_allowed_in_const_name), |x:&[u8]| x.len() >= 1 && is_uppercase_char(x[0])),
    str::from_utf8), Ident::from)
);

named!(composite_type_name<CompositeType>, map_res!(map_res!(
    take_while!(is_allowed_in_composite_type_name),
    str::from_utf8), CompositeType::from_str)
);

named!(primitive_type<PrimitiveType>, map_res!(map_res!(take_while!(is_allowed_in_primitive_type_name), str::from_utf8), PrimitiveType::from_str));

named!(type_name<Ty>, alt!(
    map!(complete!(primitive_type), Ty::from) |
    map!(complete!(composite_type_name), Ty::from)
));

named!(array_info<ArrayInfo>, alt!(
    complete!(do_parse!(intro: tag!("[<=") >> num: map_res!(take_while!(is_digit), str::from_utf8) >> exit: tag!("]") >> (ArrayInfo::Dynamic(Index::from_str(num).unwrap())))) |
    complete!(do_parse!(intro: tag!("[<") >> num: map_res!(map_res!(take_while!(is_digit), str::from_utf8), u32::from_str) >> exit: tag!("]") >> (ArrayInfo::Dynamic(Index::from(num-1))))) |
    complete!(do_parse!(intro: tag!("[") >> num: map_res!(take_while!(is_digit), str::from_utf8) >> exit: tag!("]") >> (ArrayInfo::Static(Index::from_str(num).unwrap())))) |
    complete!(do_parse!(empty: tag!("") >> (ArrayInfo::Single)))
));





named!(void_definition<FieldDefinition>, sep!(whitespace, 
                                             do_parse!(
                                                 type_name: verify!(primitive_type, |x:PrimitiveType| x.is_void()) >>
                                                     (FieldDefinition{cast_mode: None, field_type: Ty::Primitive(type_name), array: ArrayInfo::Single, name: None}))
));


named!(field_definition<FieldDefinition>, sep!(whitespace, do_parse!(
    cast_mode: opt!(cast_mode) >>
        field_type: type_name >>
        array: array_info >>
        name: field_name >>
        (FieldDefinition{cast_mode: cast_mode, field_type: field_type, array: array, name: Some(name)})
)));


named!(const_definition<ConstDefinition>, sep!(whitespace, do_parse!(
    cast_mode: opt!(cast_mode) >>
        field_type: type_name >>
        name: const_name >>
        _eq: tag!("=") >>
        constant: constant >>
        (ConstDefinition{cast_mode: cast_mode, field_type: field_type, name: name, constant: constant})
)));





named!(attribute_definition<AttributeDefinition>, complete!(sep!(whitespace, alt!(
    map!(const_definition, AttributeDefinition::from) |
    map!(field_definition, AttributeDefinition::from) |
    map!(void_definition, AttributeDefinition::from)
))));



named!(line<Line>, sep!(whitespace, alt!(
    do_parse!(
        attribute_definition: attribute_definition >>
            comment: opt!(comment) >>
            _eol: verify!(not_line_ending, |x:&[u8]| x.len() == 0) >>
            (Line::Definition(attribute_definition, comment))
    ) |
    do_parse!(
        directive: directive >>
            comment: opt!(comment) >>
            _eol: verify!(not_line_ending, |x:&[u8]| x.len() == 0) >>
            (Line::Directive(directive, comment))
    ) |
    do_parse!(
        comment: comment >>
            _eol: verify!(not_line_ending, |x:&[u8]| x.len() == 0) >>
            (Line::Comment(comment))
    ) |
    do_parse!(
        _eol: verify!(not_line_ending, |x:&[u8]| x.len() == 0) >>
            (Line::Empty)
    )
)));                       

named!(lines<Vec<Line>>, many0!(ws!(line)));

named!(message_definition<MessageDefinition>, do_parse!(lines: lines >> (MessageDefinition(lines))));

named!(service_definition<ServiceDefinition>, ws!(do_parse!(
    request: message_definition >>
        _srm: service_response_marker >>
        response: message_definition >>
        (ServiceDefinition{request: request, response: response})
)));

named!(pub type_definition<TypeDefinition>, alt!(
    map!(complete!(service_definition), TypeDefinition::from) |
    map!(complete!(message_definition), TypeDefinition::from)
));




fn is_whitespace(chr: u8) -> bool {
    chr == b' ' || chr == b'\t'
}


fn is_lowercase_char(chr: u8) -> bool {
    chr >= b'a' && chr <= b'z'
}

fn is_uppercase_char(chr: u8) -> bool {
    chr >= b'A' && chr <= b'Z'
}

fn is_bin_digit(chr: u8) -> bool {
    chr == b'0' || chr == b'1'
}

fn allowed_in_decimal_number(chr: u8) -> bool {
    is_digit(chr) || chr == b'.' || chr == b'E' || chr == b'e' || chr == b'-'
}

fn is_allowed_in_field_name(chr: u8) -> bool {
    is_lowercase_char(chr) || is_digit(chr) || chr == b'_'
}
    
fn is_allowed_in_primitive_type_name(chr: u8) -> bool {
    is_lowercase_char(chr) || is_digit(chr)
}
    
fn is_allowed_in_const_name(chr: u8) -> bool {
    is_uppercase_char(chr) || is_digit(chr) || chr == b'_'
}
    
fn is_allowed_in_composite_type_name(chr: u8) -> bool {
    is_uppercase_char(chr) || is_lowercase_char(chr) || is_digit(chr) || chr == b'.'
}
    
fn is_allowed_in_directive_name(chr: u8) -> bool {
    is_lowercase_char(chr)
}
    
        


#[cfg(test)]
mod tests {
    use super::*;

    use nom::{
        IResult,
    };

    #[test]
    fn parse_version() {
        assert_eq!(version(&b"2.3"[..]), IResult::Done(&b""[..], Version{minor: String::from("3"), major: String::from("2")}));
        assert_eq!(version(&b"2.3s"[..]), IResult::Done(&b"s"[..], Version{minor: String::from("3"), major: String::from("2")}));
        
        assert!(version(&b"s2.2"[..]).is_err());
    }
    
    #[test]
    fn parse_directive() {
        assert_eq!(directive(&b"@union"[..]), IResult::Done(&b""[..], Directive::Union));
    }
    
    #[test]
    fn parse_comment() {
        assert_eq!(comment(&b"#This is a comment\n"[..]), IResult::Done(&b"\n"[..], Comment(String::from("This is a comment"))));
        assert_eq!(comment(&b"#This is a comment"[..]), IResult::Done(&b""[..], Comment(String::from("This is a comment"))));
        assert_eq!(comment(&b"#This is a comment\r\n"[..]), IResult::Done(&b"\r\n"[..], Comment(String::from("This is a comment"))));
        assert_eq!(comment(&b"#This is a longer comment"[..]), IResult::Done(&b""[..], Comment(String::from("This is a longer comment"))));
        assert_eq!(comment(&b"# This is a comment"[..]), IResult::Done(&b""[..], Comment(String::from(" This is a comment"))));
        assert_eq!(comment(&b"#"[..]), IResult::Done(&b""[..], Comment(String::from(""))));   
    }

    #[test]
    fn parse_constant() {
        assert_eq!(constant(&b"0x123"[..]), IResult::Done(&b""[..], Const::Hex(String::from("123"))));
        assert_eq!(constant(&b"0xAABBCC"[..]), IResult::Done(&b""[..], Const::Hex(String::from("AABBCC"))));
        assert_eq!(constant(&b"0b000111"[..]), IResult::Done(&b""[..], Const::Bin(String::from("000111"))));
        assert_eq!(constant(&b"12354"[..]), IResult::Done(&b""[..], Const::Dec(String::from("12354"))));
        assert_eq!(constant(&[39,92,b'n', 39]), IResult::Done(&b""[..], Const::Char(String::from("\\n"))));
        assert_eq!(constant(&b"'b'"[..]), IResult::Done(&b""[..], Const::Char(String::from("b"))));
        assert_eq!(constant(&b"true"[..]), IResult::Done(&b""[..], Const::Bool(true)));
        assert_eq!(constant(&b"false"[..]), IResult::Done(&b""[..], Const::Bool(false)));
    }

    #[test]
    fn parse_field_name() {
        assert_eq!(field_name(&b"variable23"[..]), IResult::Done(&b""[..], Ident(String::from("variable23"))));
        assert_eq!(field_name(&b"var_iable23"[..]), IResult::Done(&b""[..], Ident(String::from("var_iable23"))));
        assert!(field_name(&b"2variable23"[..]).is_err());
    }

    #[test]
    fn parse_const_name() {
        assert_eq!(const_name(&b"CONST"[..]), IResult::Done(&b""[..], Ident(String::from("CONST"))));
        assert_eq!(const_name(&b"CONST23"[..]), IResult::Done(&b""[..], Ident(String::from("CONST23"))));
        assert_eq!(const_name(&b"CON_ST"[..]), IResult::Done(&b""[..], Ident(String::from("CON_ST"))));
        assert_eq!(const_name(&b"CON_ST1_2345"[..]), IResult::Done(&b""[..], Ident(String::from("CON_ST1_2345"))));
        assert!(const_name(&b"2CON"[..]).is_err());
    }

    #[test]
    fn parse_composite_type_name() {
        assert_eq!(composite_type_name(&b"TypeName"[..]), IResult::Done(&b""[..], CompositeType{namespace: None, name: Ident(String::from("TypeName")) } ));
        assert_eq!(composite_type_name(&b"TypeName1234"[..]), IResult::Done(&b""[..], CompositeType{namespace: None, name: Ident(String::from("TypeName1234")) } ));

        assert_eq!(composite_type_name(&b"uavcan.protocol.TypeName"[..]), IResult::Done(&b""[..], CompositeType{namespace: Some(Ident(String::from("uavcan.protocol"))), name: Ident(String::from("TypeName"))}));
        assert_eq!(composite_type_name(&b"uavcan.protocol.TypeName1234"[..]), IResult::Done(&b""[..], CompositeType{namespace: Some(Ident(String::from("uavcan.protocol"))), name: Ident(String::from("TypeName1234"))}));
    }

    #[test]
    fn parse_primitive_type() {
        assert_eq!(primitive_type(&b"uint2"[..]), IResult::Done(&b""[..], PrimitiveType::Uint2));
        assert_eq!(primitive_type(&b"uint3"[..]), IResult::Done(&b""[..], PrimitiveType::Uint3));
        assert_eq!(primitive_type(&b"uint16"[..]), IResult::Done(&b""[..], PrimitiveType::Uint16));
        assert_eq!(primitive_type(&b"uint32"[..]), IResult::Done(&b""[..], PrimitiveType::Uint32));
        
        assert!(primitive_type(&b"2variable23"[..]).is_err());
    }

    #[test]
    fn parse_type_name() {
        assert_eq!(type_name(&b"uint2"[..]), IResult::Done(&b""[..], Ty::Primitive(PrimitiveType::Uint2)));
        assert_eq!(type_name(&b"uint3"[..]), IResult::Done(&b""[..], Ty::Primitive(PrimitiveType::Uint3)));
        assert_eq!(type_name(&b"uint16"[..]), IResult::Done(&b""[..], Ty::Primitive(PrimitiveType::Uint16)));
        assert_eq!(type_name(&b"uint32"[..]), IResult::Done(&b""[..], Ty::Primitive(PrimitiveType::Uint32)));

        assert_eq!(type_name(&b"TypeName"[..]), IResult::Done(&b""[..], Ty::Composite(CompositeType{namespace: None, name: Ident(String::from("TypeName"))})));
        assert_eq!(type_name(&b"TypeName1234"[..]), IResult::Done(&b""[..], Ty::Composite(CompositeType{namespace: None, name: Ident(String::from("TypeName1234"))})));

        assert_eq!(type_name(&b"uavcan.protocol.TypeName"[..]), IResult::Done(&b""[..], Ty::Composite(CompositeType{namespace: Some(Ident(String::from("uavcan.protocol"))), name: Ident(String::from("TypeName"))})));
        assert_eq!(type_name(&b"uavcan.protocol.TypeName1234"[..]), IResult::Done(&b""[..], Ty::Composite(CompositeType{namespace: Some(Ident(String::from("uavcan.protocol"))), name: Ident(String::from("TypeName1234"))})));
        
    }

    #[test]
    fn parse_cast_mode() {
        assert_eq!(cast_mode(&b"saturated"[..]), IResult::Done(&b""[..], CastMode::Saturated));
        assert_eq!(cast_mode(&b"truncated"[..]), IResult::Done(&b""[..], CastMode::Truncated));
        
        assert!(cast_mode(&b"2variable23"[..]).is_err());
        assert!(cast_mode(&b""[..]).is_err());
    }

    #[test]
    fn parse_array_info() {
        assert_eq!(array_info(&b""[..]), IResult::Done(&b""[..], ArrayInfo::Single));
        assert_eq!(array_info(&b"[<=4]"[..]), IResult::Done(&b""[..], ArrayInfo::Dynamic(Index::from_str("4").unwrap())));
        assert_eq!(array_info(&b"[<5]"[..]), IResult::Done(&b""[..], ArrayInfo::Dynamic(Index::from_str("4").unwrap())));
        
        assert_eq!(array_info(&b"[<=128]"[..]), IResult::Done(&b""[..], ArrayInfo::Dynamic(Index::from_str("128").unwrap())));
        assert_eq!(array_info(&b"[<129]"[..]), IResult::Done(&b""[..], ArrayInfo::Dynamic(Index::from_str("128").unwrap())));

        assert_eq!(array_info(&b"[4]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(Index::from_str("4").unwrap())));
        assert_eq!(array_info(&b"[5]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(Index::from_str("5").unwrap())));
        assert_eq!(array_info(&b"[128]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(Index::from_str("128").unwrap())));
        assert_eq!(array_info(&b"[129]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(Index::from_str("129").unwrap())));
        
    }







    #[test]
    fn parse_void_definition() {
        assert_eq!(
            void_definition(&b"void2"[..]),
            IResult::Done(&b""[..], FieldDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Void2),
                array: ArrayInfo::Single,
                name: None,
            })
        );        
    }

    #[test]
    fn parse_field_definition() {
        assert_eq!(
            field_definition(&b"uint32 uptime_sec"[..]),
            IResult::Done(&b""[..], FieldDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint32),
                array: ArrayInfo::Single,
                name: Some(Ident(String::from("uptime_sec"))),
            })
        );

        
    }

    #[test]
    fn parse_const_definition() {
        assert_eq!(
            const_definition(&b"uint2 HEALTH_OK              = 0"[..]),
            IResult::Done(&b""[..], ConstDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint2),
                name: Ident(String::from("HEALTH_OK")),
                constant: Const::Dec(String::from("0")),
            })
        );

        
    }





    #[test]
    fn parse_attribute_definition() {
        assert_eq!(
            attribute_definition(&b"void2"[..]),
            IResult::Done(&b""[..], AttributeDefinition::Field(FieldDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Void2),
                array: ArrayInfo::Single,
                name: None,
            }))
        );

        assert_eq!(
            attribute_definition(&b"uint32 uptime_sec"[..]),
            IResult::Done(&b""[..], AttributeDefinition::Field(FieldDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint32),
                array: ArrayInfo::Single,
                name: Some(Ident(String::from("uptime_sec"))),
            }))
        );
        
        assert_eq!(
            attribute_definition(&b"uint2 HEALTH_OK              = 0"[..]),
            IResult::Done(&b""[..], AttributeDefinition::Const(ConstDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint2),
                name: Ident(String::from("HEALTH_OK")),
                constant: Const::Dec(String::from("0")),
            }))
        );

        
    }





    #[test]
    fn parse_line() {
        assert_eq!(
            line(&b"# Test comment"[..]),
            IResult::Done(&b""[..], Line::Comment(
                Comment(String::from(" Test comment"))
            ))
        );
        
        assert_eq!(
            line(&b"void2\n"[..]),
            IResult::Done(&b"\n"[..], Line::Definition(
                AttributeDefinition::Field(FieldDefinition{
                    cast_mode: None,
                    field_type: Ty::Primitive(PrimitiveType::Void2),
                    array: ArrayInfo::Single,
                    name: None,
                }),
                None
            ))
        );
        
        assert_eq!(
            line(&b"void3"[..]),
            IResult::Done(&b""[..], Line::Definition(
                AttributeDefinition::Field(FieldDefinition{
                    cast_mode: None,
                    field_type: Ty::Primitive(PrimitiveType::Void3),
                    array: ArrayInfo::Single,
                    name: None,
                }),
                None
            ))
        );

        assert_eq!(
            line(&b"void2 # test comment\n"[..]),
            IResult::Done(&b"\n"[..], Line::Definition(
                AttributeDefinition::Field(FieldDefinition{
                    cast_mode: None,
                    field_type: Ty::Primitive(PrimitiveType::Void2),
                    array: ArrayInfo::Single,
                    name: None
                }),
                Some(Comment(String::from(" test comment")))
            ))
        );

        assert_eq!(
            line(&b"uint32 uptime_sec"[..]),
            IResult::Done(&b""[..], Line::Definition(
                AttributeDefinition::Field(FieldDefinition{
                    cast_mode: None,
                    field_type: Ty::Primitive(PrimitiveType::Uint32),
                    array: ArrayInfo::Single,
                    name: Some(Ident(String::from("uptime_sec"))),
                }),
                None,
            ))
        );
        
        assert_eq!(
            line(&b"uint2 HEALTH_OK              = 0"[..]),
            IResult::Done(&b""[..], Line::Definition(
                AttributeDefinition::Const(ConstDefinition{
                    cast_mode: None,
                    field_type: Ty::Primitive(PrimitiveType::Uint2),
                    name: Ident(String::from("HEALTH_OK")),
                    constant: Const::Dec(String::from("0")),
                }),
                None,
            ))
        );
        
    }

    
    #[test]
    fn parse_lines() {
        assert_eq!(
            lines(&b"void2
# test comment
void3
void2 # test comment"[..]),
            IResult::Done(&b""[..], vec!(
                Line::Definition(AttributeDefinition::Field(FieldDefinition{cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: ArrayInfo::Single, name: None}), None),
                Line::Comment(Comment(String::from(" test comment"))),
                Line::Definition(AttributeDefinition::Field(FieldDefinition{cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void3), array: ArrayInfo::Single, name: None}), None),
                Line::Definition(AttributeDefinition::Field(FieldDefinition{cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: ArrayInfo::Single, name: None}), Some(Comment(String::from(" test comment")))),
            ))  
        );
        
        
    }
    
    
    
}
