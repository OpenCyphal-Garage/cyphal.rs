use std::fmt::Error;
use std::fmt::Formatter;
use std::fmt::Display;

use *;


impl Display for CastMode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            CastMode::Saturated => write!(f, "saturated"),
            CastMode::Truncated => write!(f, "truncated"),
        }
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Line::Empty => write!(f, ""),
            Line::Comment(ref comment) => write!(f, "#{}", comment),
            _ => unimplemented!(),
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn display_line() {
        assert_eq!(format!("{}", Line::Comment(Comment(String::from(" test comment")))), "# test comment");   
    }
}
