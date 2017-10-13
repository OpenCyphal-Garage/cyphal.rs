use std::str;

use {
    Comment,
};

named!(comment<Comment>, map!(map_res!(preceded!(tag!("#"), take_until_s!("\n")), str::from_utf8), Comment::from));


#[cfg(test)]
mod tests {
    use super::*;

    use nom::{
        IResult,
    };
    
    #[test]
    fn parse_comment() {
        assert_eq!(comment(&b"# This is a comment\n"[..]), IResult::Done(&b"\n"[..], Comment(String::from(" This is a comment"))));
    }
}
