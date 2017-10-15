use std::str;
use std::str::FromStr;

use {
    Comment,
    Ident,
    Num,
    Value,
    PrimitiveType,
    CastMode,
    Ty,
    ArrayInfo,
    FieldDefinition,
};

use nom::{
    not_line_ending,
    is_digit,
    is_hex_digit,
};

named!(comment<Comment>, map!(map_res!(preceded!(tag!("#"), not_line_ending), str::from_utf8), Comment::from));

named!(constant<Value>, alt!(
    complete!(do_parse!(_value: tag!("true") >> (Value::Bool(true)) )) |
    complete!(do_parse!(_value: tag!("false") >> (Value::Bool(false)) )) |
    complete!(do_parse!(_format: tag!("0x") >> value: map_res!(take_while!(is_hex_digit), str::from_utf8) >> (Value::Hex(String::from(value))))) |
    complete!(do_parse!(_format: tag!("0b") >> value: map_res!(take_while!(is_bin_digit), str::from_utf8) >> (Value::Bin(String::from(value))))) |
    complete!(do_parse!(value: map_res!(take_while!(is_digit), str::from_utf8) >> (Value::Dec(String::from(value)))))
));

named!(cast_mode<CastMode>, map_res!(map_res!(
    alt!(
        complete!(tag!("saturated")) |
        complete!(tag!("truncated")) 
    ), str::from_utf8), CastMode::from_str)
);

named!(field_name<Ident>, map!(map_res!(
    verify!(take_while!(is_allowed_in_field_name), |x:&[u8]| is_lowercase_char(x[0])),
    str::from_utf8), Ident::from)
);

named!(const_name<Ident>, map!(map_res!(
    verify!(take_while!(is_allowed_in_const_name), |x:&[u8]| is_uppercase_char(x[0])),
    str::from_utf8), Ident::from)
);

named!(composite_type_name<Ident>, map!(map_res!(
    verify!(take_while!(is_allowed_in_composite_type_name), |x:&[u8]| is_uppercase_char(x[0])),
    str::from_utf8), Ident::from)
);

named!(primitive_type<PrimitiveType>, map!(map_res!(
    alt!(
        complete!(tag!("uint32")) |
        complete!(tag!("uint16")) |
        complete!(tag!("uint3")) |
        complete!(tag!("uint2"))
    ), str::from_utf8), PrimitiveType::new)
);

named!(type_name<Ty>, alt!(
    map!(primitive_type, Ty::from) |
    map!(composite_type_name, Ty::from)
));

named!(array_info<ArrayInfo>, alt!(
    complete!(do_parse!(intro: tag!("[<=") >> num: map_res!(take_while!(is_digit), str::from_utf8) >> exit: tag!("]") >> (ArrayInfo::Dynamic(Num::from_str(num).unwrap())))) |
    complete!(do_parse!(intro: tag!("[<") >> num: map_res!(map_res!(take_while!(is_digit), str::from_utf8), u32::from_str) >> exit: tag!("]") >> (ArrayInfo::Dynamic(Num(format!("{}", num-1)))))) |
    complete!(do_parse!(intro: tag!("[") >> num: map_res!(take_while!(is_digit), str::from_utf8) >> exit: tag!("]") >> (ArrayInfo::Static(Num::from_str(num).unwrap())))) |
    complete!(do_parse!(empty: tag!("") >> (ArrayInfo::Single)))
));





named!(field_definition<FieldDefinition>, ws!(do_parse!(
    cast_mode: opt!(cast_mode) >>
        field_type: type_name >>
        array: array_info >>
        name: field_name >>
        (FieldDefinition{cast_mode: cast_mode, field_type: field_type, array: array, name: name})
)));
      




fn is_lowercase_char(chr: u8) -> bool {
    chr >= b'a' && chr <= b'z'
}

fn is_uppercase_char(chr: u8) -> bool {
    chr >= b'A' && chr <= b'Z'
}

fn is_bin_digit(chr: u8) -> bool {
    chr == b'0' || chr == b'1'
}


fn is_allowed_in_field_name(chr: u8) -> bool {
    is_lowercase_char(chr) || is_digit(chr) || chr == b'_'
}
    
fn is_allowed_in_const_name(chr: u8) -> bool {
    is_uppercase_char(chr) || is_digit(chr) || chr == b'_'
}
    
fn is_allowed_in_composite_type_name(chr: u8) -> bool {
    is_uppercase_char(chr) || is_lowercase_char(chr) || is_digit(chr)
}
    
        


#[cfg(test)]
mod tests {
    use super::*;

    use nom::{
        IResult,
    };
    
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
        assert_eq!(constant(&b"0x123"[..]), IResult::Done(&b""[..], Value::Hex(String::from("123"))));
        assert_eq!(constant(&b"0xAABBCC"[..]), IResult::Done(&b""[..], Value::Hex(String::from("AABBCC"))));
        assert_eq!(constant(&b"0b000111"[..]), IResult::Done(&b""[..], Value::Bin(String::from("000111"))));
        assert_eq!(constant(&b"12354"[..]), IResult::Done(&b""[..], Value::Dec(String::from("12354"))));
        assert_eq!(constant(&b"true"[..]), IResult::Done(&b""[..], Value::Bool(true)));
        assert_eq!(constant(&b"false"[..]), IResult::Done(&b""[..], Value::Bool(false)));
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
        assert_eq!(composite_type_name(&b"TypeName"[..]), IResult::Done(&b""[..], Ident(String::from("TypeName"))));
        assert_eq!(composite_type_name(&b"TypeName1234"[..]), IResult::Done(&b""[..], Ident(String::from("TypeName1234"))));
        assert!(composite_type_name(&b"typeName"[..]).is_err());
        assert!(composite_type_name(&b"2typeName"[..]).is_err());
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
        assert_eq!(type_name(&b"uint2"[..]), IResult::Done(&b""[..], Ty::PrimitiveType(PrimitiveType::Uint2)));
        assert_eq!(type_name(&b"uint3"[..]), IResult::Done(&b""[..], Ty::PrimitiveType(PrimitiveType::Uint3)));
        assert_eq!(type_name(&b"uint16"[..]), IResult::Done(&b""[..], Ty::PrimitiveType(PrimitiveType::Uint16)));
        assert_eq!(type_name(&b"uint32"[..]), IResult::Done(&b""[..], Ty::PrimitiveType(PrimitiveType::Uint32)));

        assert_eq!(type_name(&b"TypeName"[..]), IResult::Done(&b""[..], Ty::Path(Ident(String::from("TypeName")))));
        assert_eq!(type_name(&b"TypeName1234"[..]), IResult::Done(&b""[..], Ty::Path(Ident(String::from("TypeName1234")))));
        assert!(type_name(&b"typeName"[..]).is_err());
        assert!(type_name(&b"2typeName"[..]).is_err());
        
        assert!(type_name(&b"2variable23"[..]).is_err());
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
        assert_eq!(array_info(&b"[<=4]"[..]), IResult::Done(&b""[..], ArrayInfo::Dynamic(Num::from_str("4").unwrap())));
        assert_eq!(array_info(&b"[<5]"[..]), IResult::Done(&b""[..], ArrayInfo::Dynamic(Num::from_str("4").unwrap())));
        
        assert_eq!(array_info(&b"[<=128]"[..]), IResult::Done(&b""[..], ArrayInfo::Dynamic(Num::from_str("128").unwrap())));
        assert_eq!(array_info(&b"[<129]"[..]), IResult::Done(&b""[..], ArrayInfo::Dynamic(Num::from_str("128").unwrap())));

        assert_eq!(array_info(&b"[4]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(Num::from_str("4").unwrap())));
        assert_eq!(array_info(&b"[5]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(Num::from_str("5").unwrap())));
        assert_eq!(array_info(&b"[128]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(Num::from_str("128").unwrap())));
        assert_eq!(array_info(&b"[129]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(Num::from_str("129").unwrap())));
        
    }








    #[test]
    fn parse_field_definition() {
        assert_eq!(
            field_definition(&b"uint32 uptime_sec"[..]),
            IResult::Done(&b""[..], FieldDefinition{
                cast_mode: None,
                field_type: Ty::PrimitiveType(PrimitiveType::Uint32),
                array: ArrayInfo::Single,
                name: Ident(String::from("uptime_sec")),
            })
        );

        
    }
}