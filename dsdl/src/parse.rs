use std::str;

use {
    Comment,
    Ident,
};

use nom::{
    not_line_ending,
};

named!(comment<Comment>, map!(map_res!(preceded!(tag!("#"), not_line_ending), str::from_utf8), Comment::from));

named!(field_name<Ident>, map!(map_res!(
    verify!(take_while!(is_allowed_in_field_name), |x:&[u8]| is_lowercase(x[0])),
    str::from_utf8), Ident::from)
);

fn is_lowercase(chr: u8) -> bool {
    chr >= b'a' && chr <= b'z'
}

fn is_numeric(chr: u8) -> bool {
    chr >= b'0' && chr <= b'9'
}

fn is_allowed_in_field_name(chr: u8) -> bool {
    is_lowercase(chr) || is_numeric(chr) || chr == b'_'
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
    fn parse_field_name() {
        assert_eq!(field_name(&b"variable23"[..]), IResult::Done(&b""[..], Ident(String::from("variable23"))));
        assert_eq!(field_name(&b"var_iable23"[..]), IResult::Done(&b""[..], Ident(String::from("var_iable23"))));
        assert!(field_name(&b"2variable23"[..]).is_err());
    }
}
