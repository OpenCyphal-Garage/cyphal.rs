// The file compiled from lalrpop.rs
include!(concat!(env!("OUT_DIR"), "/src/parser/parser.rs"));

#[cfg(test)]
mod tests {
    use *;
    use super::*;

    #[test]
    fn parse_array_info() {
        let mut errors = Vec::new();

        assert_eq!(
            ArrayInfo::Static(2),
            ArrayInfoParser::new()
                .parse(&mut errors, lexer::Lexer::new("[2]"))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);

        assert_eq!(
            ArrayInfo::DynamicLess(2),
            ArrayInfoParser::new()
                .parse(&mut errors, lexer::Lexer::new("[<2]"))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);

        assert_eq!(
            ArrayInfo::DynamicLeq(2),
            ArrayInfoParser::new()
                .parse(&mut errors, lexer::Lexer::new("[<=2]"))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);
    }


    #[test]
    fn parse_directive() {
        let mut errors = Vec::new();

        assert_eq!(
            Ok(Directive::Union),
            DirectiveParser::new()
                .parse(&mut errors, lexer::Lexer::new("@union"))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn parse_field_definition() {
        let mut errors = Vec::new();

        assert_eq!(
            FieldDefinition {
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Void2),
                array: None,
                name: None,
            },
            FieldDefinitionParser::new()
                .parse(&mut errors, lexer::Lexer::new("void2"))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);

        assert_eq!(
            FieldDefinition {
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint32),
                array: None,
                name: Some(Ident::from_str("uptime_sec").unwrap()),
            },
            FieldDefinitionParser::new()
                .parse(&mut errors, lexer::Lexer::new("uint32 uptime_sec"))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);

        assert_eq!(
            FieldDefinition {
                cast_mode: Some(CastMode::Truncated),
                field_type: Ty::Primitive(PrimitiveType::Uint32),
                array: None,
                name: Some(Ident::from_str("test").unwrap()),
            },
            FieldDefinitionParser::new()
                .parse(&mut errors, lexer::Lexer::new("truncated uint32 test"))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);

        assert_eq!(
            FieldDefinition {
                cast_mode: None,
                field_type: Ty::Composite(CompositeType { namespace: None, name: Ident::from(String::from("Test")) }),
                array: None,
                name: Some(Ident::from_str("test").unwrap()),
            },
            FieldDefinitionParser::new()
                .parse(&mut errors, lexer::Lexer::new("Test test"))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);

        assert_eq!(
            FieldDefinition {
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint8),
                array: Some(ArrayInfo::Static(10)),
                name: Some(Ident::from_str("test").unwrap()),
            },
            FieldDefinitionParser::new()
                .parse(&mut errors, lexer::Lexer::new("uint8[10] test"))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn parse_const_definition() {
        let mut errors = Vec::new();

        assert_eq!(
            ConstDefinition {
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint2),
                name: Ident::from_str("HEALTH_OK").unwrap(),
                literal: Lit::Dec { sign: Sign::Implicit, value: String::from("0") },
            },
            ConstDefinitionParser::new()
                .parse(&mut errors, lexer::Lexer::new("uint2 HEALTH_OK = 0"))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);
    }


    #[test]
    fn parse_type_definition() {
        let mut errors = Vec::new();

        assert_eq!(
            TypeDefinition::Message(MessageDefinition(
                vec!(
                    Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: None, name: None }), comment: None },
                    Line::Empty,
                )
            )),
            TypeDefinitionParser::new()
                .parse(&mut errors, lexer::Lexer::new(
                    "void2

"
                ))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);

        assert_eq!(
            TypeDefinition::Message(MessageDefinition(
                vec!(
                    Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: None, name: None }), comment: None },
                )
            )),
            TypeDefinitionParser::new()
                .parse(&mut errors, lexer::Lexer::new(
                    "void2
"
                ))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);

        assert_eq!(
            TypeDefinition::Message(MessageDefinition(
                vec!(
                    Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: None, name: None }), comment: None },
                    Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), array: None, name: Some(Ident::from_str("vendor_specific_status_code").unwrap()) }), comment: None },
                    Line::Empty,
                )
            )),
            TypeDefinitionParser::new()
                .parse(&mut errors, lexer::Lexer::new(
                    "void2
uint16 vendor_specific_status_code

"
                ))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);

        assert_eq!(
            TypeDefinition::Message(MessageDefinition(
                vec!(
                    Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: None, name: None }), comment: None },
                    Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), array: None, name: Some(Ident::from_str("vendor_specific_status_code").unwrap()) }), comment: None },
                )
            )),
            TypeDefinitionParser::new()
                .parse(&mut errors, lexer::Lexer::new(
                    "void2
uint16 vendor_specific_status_code
"
                ))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);

        assert_eq!(
            TypeDefinition::Service(ServiceDefinition {
                request: MessageDefinition(vec![
                    Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void32), array: None, name: None }), comment: None },
                ]),
                response: MessageDefinition(vec![
                    Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: None, name: None }), comment: Some(Comment::from_str("# test comment").unwrap()) },
                ]),
            }),
            TypeDefinitionParser::new()
                .parse(&mut errors, lexer::Lexer::new(
                    "void32
---
void2 # test comment
"
                ))
                .unwrap()
        );

        assert_eq!(errors.len(), 0);
    }


    // Tests related to error handling


    #[test]
    fn parse_directive_error() {
        let mut errors = Vec::new();

        assert_eq!(
            Err(ParseError::new(ParseErrorKind::UnknownDirectiveName(Ident::from_str("unionasd").unwrap()), None)),
            DirectiveParser::new()
                .parse(&mut errors, lexer::Lexer::new("@unionasd"))
                .unwrap()
        );

        assert_eq!(
            Err(ParseError::new(ParseErrorKind::UnknownDirectiveName(Ident::from_str("bad_directive").unwrap()), None)),
            DirectiveParser::new()
                .parse(&mut errors, lexer::Lexer::new("@bad_directive 123 12345"))
                .unwrap()
        );
    }

    #[test]
    fn parse_line_error() {
        let mut errors = Vec::new();
        LineParser::new().parse(&mut errors, lexer::Lexer::new("@unionasd\n")).unwrap();
        assert_eq!(errors, vec![ParseError::new(ParseErrorKind::UnknownDirectiveName(Ident::from_str("unionasd").unwrap()), None)]);


        let mut errors = Vec::new();
        LineParser::new().parse(&mut errors, lexer::Lexer::new("@bad_directive 123 12345\n")).unwrap();
        assert_eq!(errors, vec![ParseError::new(ParseErrorKind::UnknownDirectiveName(Ident::from_str("bad_directive").unwrap()), None)]);

        let mut errors = Vec::new();
        LineParser::new().parse(&mut errors, lexer::Lexer::new("uint2 123\n")).unwrap();
        assert_eq!(errors, vec![ParseError::new(ParseErrorKind::UnexpectedToken(Token::Lit(Lit::Dec{sign: Sign::Implicit, value: String::from("123")})), Some(6))]);

    }

}