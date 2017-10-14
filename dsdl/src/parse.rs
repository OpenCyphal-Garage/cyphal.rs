use std::str;

use {
    Comment,
};

use nom::{
    not_line_ending,
};

named!(comment<Comment>, map!(map_res!(preceded!(tag!("#"), not_line_ending), str::from_utf8), Comment::from));


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
}
