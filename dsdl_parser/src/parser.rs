#![allow(unused_imports)]
#![allow(dead_code)]

// The file compiled from lalrpop.rs
include!(concat!(env!("OUT_DIR"), "/src/parser.rs"));

#[cfg(test)]
mod tests {
    use *;
    use super::*;

    #[test]
    fn parse_array_info() {
        assert_eq!(
            ArrayInfo::Static(2),
            ArrayInfoParser::new()
                .parse(lexer::Lexer::new("[2]"))
                .unwrap()
        );

        assert_eq!(
            ArrayInfo::DynamicLess(2),
            ArrayInfoParser::new()
                .parse(lexer::Lexer::new("[<2]"))
                .unwrap()
        );

        assert_eq!(
            ArrayInfo::DynamicLeq(2),
            ArrayInfoParser::new()
                .parse(lexer::Lexer::new("[<=2]"))
                .unwrap()
        );
    }


    #[test]
    fn parse_directive() {
        assert_eq!(
            Directive::Union,
            DirectiveParser::new()
                .parse(lexer::Lexer::new("@union"))
                .unwrap()
        );
    }

    #[test]
    fn parse_field_definition() {
        assert_eq!(
            FieldDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Void2),
                array: None,
                name: None,
            },
            FieldDefinitionParser::new()
                .parse(lexer::Lexer::new("void2"))
                .unwrap()
        );

        assert_eq!(
            FieldDefinition {
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint32),
                array: None,
                name: Some(Ident::from_str("uptime_sec").unwrap()),
            },
            FieldDefinitionParser::new()
                .parse(lexer::Lexer::new("uint32 uptime_sec"))
                .unwrap()
        );

        assert_eq!(
            FieldDefinition {
                cast_mode: Some(CastMode::Truncated),
                field_type: Ty::Primitive(PrimitiveType::Uint32),
                array: None,
                name: Some(Ident::from_str("test").unwrap()),
            },
            FieldDefinitionParser::new()
                .parse(lexer::Lexer::new("truncated uint32 test"))
                .unwrap()
        );

        assert_eq!(
            FieldDefinition {
                cast_mode: None,
                field_type: Ty::Composite(CompositeType{namespace: None, name: Ident::from(String::from("Test"))}),
                array: None,
                name: Some(Ident::from_str("test").unwrap()),
            },
            FieldDefinitionParser::new()
                .parse(lexer::Lexer::new("Test test"))
                .unwrap()
        );

        assert_eq!(
            FieldDefinition {
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint8),
                array: Some(ArrayInfo::Static(10)),
                name: Some(Ident::from_str("test").unwrap()),
            },
            FieldDefinitionParser::new()
                .parse(lexer::Lexer::new("uint8[10] test"))
                .unwrap()
        );
    }

    #[test]
    fn parse_const_definition() {
        assert_eq!(
            ConstDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint2),
                name: Ident::from_str("HEALTH_OK").unwrap(),
                literal: Lit::Dec{sign: Sign::Implicit, value: String::from("0")},
            },
            ConstDefinitionParser::new()
                .parse(lexer::Lexer::new("uint2 HEALTH_OK = 0"))
                .unwrap()
        );
    }


    #[test]
    fn parse_type_definition() {
        assert_eq!(
                TypeDefinition::Message(MessageDefinition(
                    vec!(
                        Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: None, name: None }), comment: None },
                        Line::Empty,
                    )
            )),
            TypeDefinitionParser::new()
                .parse(lexer::Lexer::new(
"void2

"
                ))
                .unwrap()
        );
        assert_eq!(
                TypeDefinition::Message(MessageDefinition(
                    vec!(
                        Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: None, name: None }), comment: None },
                    )
            )),
            TypeDefinitionParser::new()
                .parse(lexer::Lexer::new(
"void2
"
                ))
                .unwrap()
        );

        assert_eq!(
                TypeDefinition::Message(MessageDefinition(
                    vec!(
                        Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: None, name: None }), comment: None },
                        Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), array: None, name: Some(Ident::from_str("vendor_specific_status_code").unwrap()) }), comment: None },
                        Line::Empty,
                    )
            )),
            TypeDefinitionParser::new()
                .parse(lexer::Lexer::new(
"void2
uint16 vendor_specific_status_code

"
                ))
                .unwrap()
        );

        assert_eq!(
                TypeDefinition::Message(MessageDefinition(
                    vec!(
                        Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: None, name: None }), comment: None },
                        Line::Definition { definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), array: None, name: Some(Ident::from_str("vendor_specific_status_code").unwrap()) }), comment: None },
                    )
            )),
            TypeDefinitionParser::new()
                .parse(lexer::Lexer::new(
"void2
uint16 vendor_specific_status_code
"
                ))
                .unwrap()
        );

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
                .parse(lexer::Lexer::new(
"void32
---
void2 # test comment
"
                ))
                .unwrap()
        );
    }

}
