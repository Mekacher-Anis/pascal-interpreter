use crate::ast::ASTNode;
use crate::lexer::Lexer;
use crate::token::Token;
use anyhow::{anyhow, Ok, Result};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Result<Self> {
        let current_token = lexer.next_token().unwrap_or(Token::Eof);
        Ok(Parser {
            lexer,
            current_token,
        })
    }

    pub fn parse(&mut self) -> Result<ASTNode> {
        self.program()
    }

    fn eat(&mut self, expected_type: Option<&Token>) -> Result<()> {
        if let Some(expected) = expected_type {
            if std::mem::discriminant(&self.current_token) != std::mem::discriminant(expected) {
                return Err(anyhow!(
                    "Expected {:?}, found {:?}",
                    expected,
                    self.current_token
                ));
            }
        }
        self.current_token = (&mut self.lexer).next_token()?;
        Ok(())
    }

    fn program(&mut self) -> Result<ASTNode> {
        let cs = self.compound_statement()?;
        self.eat(Some(&Token::Dot))?;
        Ok(cs)
    }

    fn compound_statement(&mut self) -> Result<ASTNode> {
        self.eat(Some(&Token::Begin))?;
        let statement_list = self.statement_list()?;
        self.eat(Some(&Token::End))?;
        Ok(ASTNode::Compound {
            children: statement_list,
        })
    }

    fn statement_list(&mut self) -> Result<Vec<Box<ASTNode>>> {
        let statement: ASTNode = self.statement()?;
        let mut statement_list = vec![Box::new(statement)];

        while let Token::Semi = self.current_token {
            self.eat(Some(&Token::Semi))?;
            statement_list.push(Box::new(self.statement()?));
        }

        if let Token::Id(_) = self.current_token {
            return Err(anyhow!("Didn't expect an id in the statement list"));
        }

        Ok(statement_list)
    }

    fn statement(&mut self) -> Result<ASTNode> {
        match self.current_token {
            Token::Begin => self.compound_statement(),
            Token::Id(_) => self.assignment_statement(),
            _ => self.empty(),
        }
    }

    fn assignment_statement(&mut self) -> Result<ASTNode> {
        let var_node = self.variable()?;
        let token = self.current_token.clone();
        self.eat(Some(&Token::Assign))?;
        let expr_node = self.expr()?;
        Ok(ASTNode::Assign {
            left: Box::new(var_node),
            right: Box::new(expr_node),
            token: token,
        })
    }

    fn empty(&mut self) -> Result<ASTNode> {
        Ok(ASTNode::NoOp)
    }

    fn variable(&mut self) -> Result<ASTNode> {
        let current_token: Token = self.current_token.clone();
        match &current_token {
            Token::Id(name) => {
                self.eat(Some(&current_token))?;
                Ok(ASTNode::Var {
                    name: name.clone(),
                    token: current_token,
                })
            }
            _ => Err(anyhow!("Expected an identifier node, found something else")),
        }
    }

    fn factor(&mut self) -> Result<ASTNode> {
        match self.current_token {
            Token::Plus => {
                self.eat(Some(&Token::Plus))?;
                Ok(ASTNode::UnaryOpNode {
                    token: Token::Plus,
                    expr: Box::new(self.factor()?),
                })
            }
            Token::Minus => {
                self.eat(Some(&Token::Minus))?;
                Ok(ASTNode::UnaryOpNode {
                    token: Token::Minus,
                    expr: Box::new(self.factor()?),
                })
            }
            Token::Integer(val) => {
                self.eat(Some(&Token::Integer(0)))?;
                Ok(ASTNode::NumNode {
                    token: Token::Integer(val),
                    value: val,
                })
            }
            Token::LParenthesis => {
                self.eat(Some(&Token::LParenthesis))?;
                let result = self.expr()?;
                self.eat(Some(&Token::RParenthesis))?;
                Ok(result)
            }
            Token::Id(_) => self.variable(),
            _ => return Err(anyhow!("Syntax error, expected an integer")),
        }
    }

    fn term(&mut self) -> Result<ASTNode> {
        let mut result = self.factor()?;

        loop {
            let op = self.current_token.clone();

            match op {
                Token::Eof => break,
                Token::Asterisk | Token::Slash => {
                    self.eat(Some(&op))?;

                    let right_node = self.factor()?;

                    match op {
                        Token::Asterisk | Token::Slash => {
                            result = ASTNode::BinOpNode {
                                left: Box::new(result),
                                right: Box::new(right_node),
                                op,
                            }
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }

        Ok(result)
    }

    fn expr(&mut self) -> Result<ASTNode> {
        let mut result = self.term()?;

        loop {
            let op = self.current_token.clone();

            match op {
                Token::Eof => break,
                Token::Plus | Token::Minus => {
                    self.eat(Some(&op))?;

                    let right = self.term()?;

                    match op {
                        Token::Plus | Token::Minus => {
                            result = ASTNode::BinOpNode {
                                left: Box::new(result),
                                right: Box::new(right),
                                op,
                            }
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_compound_statement() {
        let input = "BEGIN END.";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_expression() {
        let input = "1 + 2 * 3";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();
        let result = parser.expr();
        assert!(result.is_ok());
        // 1 + (2 * 3)
        if let ASTNode::BinOpNode { left, right, op } = result.unwrap() {
            assert_eq!(op, Token::Plus);
            if let ASTNode::NumNode { value, .. } = *left {
                assert_eq!(value, 1);
            } else {
                panic!("Expected NumNode(1)");
            }
            if let ASTNode::BinOpNode { left, right, op } = *right {
                assert_eq!(op, Token::Asterisk);
                if let ASTNode::NumNode { value, .. } = *left {
                    assert_eq!(value, 2);
                } else {
                    panic!("Expected NumNode(2)");
                }
                if let ASTNode::NumNode { value, .. } = *right {
                    assert_eq!(value, 3);
                } else {
                    panic!("Expected NumNode(3)");
                }
            } else {
                panic!("Expected BinOpNode(*)");
            }
        } else {
            panic!("Expected BinOpNode(+)");
        }
    }

    #[test]
    fn test_parenthesis() {
        let input = "(1 + 2) * 3";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();
        let result = parser.expr();
        assert!(result.is_ok());
        // (1 + 2) * 3
        if let ASTNode::BinOpNode { left, right, op } = result.unwrap() {
            assert_eq!(op, Token::Asterisk);
            if let ASTNode::BinOpNode { left, right, op } = *left {
                assert_eq!(op, Token::Plus);
                if let ASTNode::NumNode { value, .. } = *left {
                    assert_eq!(value, 1);
                } else {
                    panic!("Expected NumNode(1)");
                }
                if let ASTNode::NumNode { value, .. } = *right {
                    assert_eq!(value, 2);
                } else {
                    panic!("Expected NumNode(2)");
                }
            } else {
                panic!("Expected BinOpNode(+)");
            }
            if let ASTNode::NumNode { value, .. } = *right {
                assert_eq!(value, 3);
            } else {
                panic!("Expected NumNode(3)");
            }
        } else {
            panic!("Expected BinOpNode(*)");
        }
    }

    #[test]
    fn test_assignment() {
        let input = "BEGIN a := 1 END.";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();
        let result = parser.parse();
        assert!(result.is_ok());
        if let ASTNode::Compound { children } = result.unwrap() {
            assert_eq!(children.len(), 1);
            if let ASTNode::Assign { left, right, .. } = &*children[0] {
                if let ASTNode::Var { name: value, .. } = &**left {
                    assert_eq!(value, "a");
                } else {
                    panic!("Expected Var(a)");
                }
                if let ASTNode::NumNode { value, .. } = &**right {
                    assert_eq!(*value, 1);
                } else {
                    panic!("Expected NumNode(1)");
                }
            } else {
                panic!("Expected Assign");
            }
        } else {
            panic!("Expected Compound");
        }
    }

    #[test]
    fn test_unary_op() {
        let input = "-1";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();
        let result = parser.expr();
        assert!(result.is_ok());
        if let ASTNode::UnaryOpNode { token, expr } = result.unwrap() {
            assert_eq!(token, Token::Minus);
            if let ASTNode::NumNode { value, .. } = *expr {
                assert_eq!(value, 1);
            } else {
                panic!("Expected NumNode(1)");
            }
        } else {
            panic!("Expected UnaryOpNode");
        }
    }

    #[test]
    fn test_complex_nested_structure() {
        let input = "BEGIN
    BEGIN
        number := 2;
        a := number;
        b := 10 * a + 10 * number / 4;
        c := a - - b
    END;
    x := 11;
END.";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();
        let result = parser.parse();
        assert!(result.is_ok());
    }
}
