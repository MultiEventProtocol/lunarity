use toolshed::list::ListBuilder;

use ast::*;
use parser::Parser;
use lexer::Token;

impl<'ast> Parser<'ast> {
    pub fn function_definition(&mut self) -> Option<ContractPartNode<'ast>> {
        let start = self.lexer.start_then_consume();

        let name = match self.lexer.token {
            Token::Identifier => Some(self.str_node()),
            _                 => None,
        };

        self.expect(Token::ParenOpen);

        let params = self.parameter_list();

        self.expect(Token::ParenClose);

        let mut mutability = None;
        let mut visibility = None;

        loop {
            match self.lexer.token {
                Token::KeywordExternal => self.unique_flag(FunctionVisibility::External, &mut visibility),
                Token::KeywordPublic   => self.unique_flag(FunctionVisibility::Public, &mut visibility),
                Token::KeywordInternal => self.unique_flag(FunctionVisibility::Internal, &mut visibility),
                Token::KeywordPrivate  => self.unique_flag(FunctionVisibility::Private, &mut visibility),

                Token::KeywordPure     => self.unique_flag(StateMutability::Pure, &mut mutability),
                Token::KeywordConstant => self.unique_flag(StateMutability::Constant, &mut mutability),
                Token::KeywordView     => self.unique_flag(StateMutability::View, &mut mutability),
                Token::KeywordPayable  => self.unique_flag(StateMutability::Payable, &mut mutability),

                _ => break,
            }
        }

        let returns;

        if self.allow(Token::KeywordReturns) {
            self.expect(Token::ParenOpen);

            returns = self.parameter_list();

            self.expect(Token::ParenClose);
        } else {
            returns = NodeList::empty();
        }

        let end = self.expect_end(Token::Semicolon);

        Some(self.node_at(start, end, FunctionDefinition {
            name,
            params,
            visibility,
            mutability,
            returns,
            body: None,
        }))
    }

    #[inline]
    fn unique_flag<F>(&mut self, flag: F, at: &mut Option<Node<'ast, F>>)
    where
        F: Copy,
    {
        if at.is_some() {
            // TODO: More descriptive errors, something like "Can't redeclare visibility/mutability"
            return self.error();
        }

        *at = Some(self.node_at_token(flag));

        self.lexer.consume();
    }

    fn parameter_list(&mut self) -> ParameterList<'ast> {
        match self.parameter() {
            Some(param) => {
                let builder = ListBuilder::new(self.arena, param);

                while self.allow(Token::Comma) {
                    match self.parameter() {
                        Some(param) => builder.push(self.arena, param),
                        None        => self.error(),
                    }
                }

                builder.as_list()
            },
            None => NodeList::empty(),
        }
    }

    fn parameter(&mut self) -> Option<Node<'ast, Parameter<'ast>>> {
        let type_name = self.type_name()?;
        let name      = self.allow_str_node(Token::Identifier);

        let end = name.end().unwrap_or_else(|| type_name.end);

        Some(self.node_at(type_name.start, end, Parameter {
            type_name,
            name,
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::mock::{Mock, assert_units};

    #[test]
    fn empty_function() {
        let m = Mock::new();

        assert_units(r#"

            contract Foo {
                function();
                function bar();
            }

        "#, [
            m.node(14, 102, ContractDefinition {
                name: m.node(23, 26, "Foo"),
                inherits: NodeList::empty(),
                body: m.list([
                    m.node(45, 56, FunctionDefinition {
                        name: None,
                        params: NodeList::empty(),
                        visibility: None,
                        mutability: None,
                        returns: NodeList::empty(),
                        body: None,
                    }),
                    m.node(73, 88, FunctionDefinition {
                        name: m.node(82, 85, "bar"),
                        params: NodeList::empty(),
                        visibility: None,
                        mutability: None,
                        returns: NodeList::empty(),
                        body: None,
                    }),
                ]),
            }),
        ]);
    }

    #[test]
    fn function_parameters() {
        let m = Mock::new();

        assert_units(r#"

            contract Foo {
                function(uint56, bool);
            }

        "#, [
            m.node(14, 82, ContractDefinition {
                name: m.node(23, 26, "Foo"),
                inherits: NodeList::empty(),
                body: m.list([
                    m.node(45, 68, FunctionDefinition {
                        name: None,
                        params: m.list([
                            m.node(54, 60, Parameter {
                                type_name: m.node(54, 60, ElementaryTypeName::Uint(7)),
                                name: None,
                            }),
                            m.node(62, 66, Parameter {
                                type_name: m.node(62, 66, ElementaryTypeName::Bool),
                                name: None,
                            }),
                        ]),
                        visibility: None,
                        mutability: None,
                        returns: NodeList::empty(),
                        body: None,
                    }),
                ]),
            }),
        ]);
    }

    #[test]
    fn function_named_parameters() {
        let m = Mock::new();

        assert_units(r#"

            contract Foo {
                function doge(uint56 wow, bool moon);
            }

        "#, [
            m.node(14, 96, ContractDefinition {
                name: m.node(23, 26, "Foo"),
                inherits: NodeList::empty(),
                body: m.list([
                    m.node(45, 82, FunctionDefinition {
                        name: m.node(54, 58, "doge"),
                        params: m.list([
                            m.node(59, 69, Parameter {
                                type_name: m.node(59, 65, ElementaryTypeName::Uint(7)),
                                name: m.node(66, 69, "wow"),
                            }),
                            m.node(71, 80, Parameter {
                                type_name: m.node(71, 75, ElementaryTypeName::Bool),
                                name: m.node(76, 80, "moon"),
                            }),
                        ]),
                        visibility: None,
                        mutability: None,
                        returns: NodeList::empty(),
                        body: None,
                    }),
                ]),
            }),
        ]);
    }

    #[test]
    fn function_returns() {
        let m = Mock::new();

        assert_units(r#"

            contract Foo {
                function doge() returns (uint56, bool);
            }

        "#, [
            m.node(14, 98, ContractDefinition {
                name: m.node(23, 26, "Foo"),
                inherits: NodeList::empty(),
                body: m.list([
                    m.node(45, 84, FunctionDefinition {
                        name: m.node(54, 58, "doge"),
                        params: NodeList::empty(),
                        visibility: None,
                        mutability: None,
                        returns: m.list([
                            m.node(70, 76, Parameter {
                                type_name: m.node(70, 76, ElementaryTypeName::Uint(7)),
                                name: None,
                            }),
                            m.node(78, 82, Parameter {
                                type_name: m.node(78, 82, ElementaryTypeName::Bool),
                                name: None,
                            }),
                        ]),
                        body: None,
                    }),
                ]),
            }),
        ]);
    }

    #[test]
    fn function_mutability_and_visibility() {
        let m = Mock::new();

        assert_units(r#"

            contract Foo {
                function wow() pure external;
                function such() internal view;
                function very() private;
                function much() payable;
            }

        "#, [
            m.node(14, 217, ContractDefinition {
                name: m.node(23, 26, "Foo"),
                inherits: NodeList::empty(),
                body: m.list([
                    m.node(45, 74, FunctionDefinition {
                        name: m.node(54, 57, "wow"),
                        params: NodeList::empty(),
                        visibility: m.node(65, 73, FunctionVisibility::External),
                        mutability: m.node(60, 64, StateMutability::Pure),
                        returns: NodeList::empty(),
                        body: None,
                    }),
                    m.node(91, 121, FunctionDefinition {
                        name: m.node(100, 104, "such"),
                        params: NodeList::empty(),
                        visibility: m.node(107, 115, FunctionVisibility::Internal),
                        mutability: m.node(116, 120, StateMutability::View),
                        returns: NodeList::empty(),
                        body: None,
                    }),
                    m.node(138, 162, FunctionDefinition {
                        name: m.node(147, 151, "very"),
                        params: NodeList::empty(),
                        visibility: m.node(154, 161, FunctionVisibility::Private),
                        mutability: None,
                        returns: NodeList::empty(),
                        body: None,
                    }),
                    m.node(179, 203, FunctionDefinition {
                        name: m.node(188, 192, "much"),
                        params: NodeList::empty(),
                        visibility: None,
                        mutability: m.node(195, 202, StateMutability::Payable),
                        returns: NodeList::empty(),
                        body: None,
                    }),
                ]),
            }),
        ]);
    }

    #[test]
    fn function_flags_are_unique_per_kind() {
        use parser::parse;

        assert!(parse("contract Foo { function() public public; }").is_err());
        assert!(parse("contract Foo { function() pure pure; }").is_err());
        assert!(parse("contract Foo { function() internal external; }").is_err());
        assert!(parse("contract Foo { function() payable constant; }").is_err());
    }
}
