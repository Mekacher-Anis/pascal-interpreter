use crate::ast::{ASTNode, ASTVarType};
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
        self.eat(Some(&Token::Program))?;
        let var_node = self.variable()?;
        let ASTNode::Var { name: program_name } = var_node else {
            return Err(anyhow!("Expected a program name after PROGRAM"));
        };
        self.eat(Some(&Token::Semi))?;
        let block = self.block()?;
        self.eat(Some(&Token::Dot))?;
        Ok(ASTNode::Program {
            name: program_name,
            block: Box::new(block),
        })
    }

    fn block(&mut self) -> Result<ASTNode> {
        let declarations = self.declarations()?;
        let cs = self.compound_statement()?;
        Ok(ASTNode::Block {
            declarations: declarations,
            compound_statement: Box::new(cs),
        })
    }

    fn declarations(&mut self) -> Result<Vec<Box<ASTNode>>> {
        let mut declarations = vec![];
        while let Token::Var = self.current_token {
            self.eat(Some(&Token::Var))?;
            while let Token::Id(_) = self.current_token {
                let vd = self.variable_declaration()?;
                declarations.extend(vd);
                self.eat(Some(&Token::Semi))?;
            }
        }

        Ok(declarations)
    }

    fn variable_declaration(&mut self) -> Result<Vec<Box<ASTNode>>> {
        let mut var_names = vec![];
        let Token::Id(var_name) = self.current_token.clone() else {
            return Err(anyhow!(
                "Expected at least one variable name in declaration."
            ));
        };
        var_names.push(var_name);

        self.eat(Some(&Token::Id("".to_owned())))?;

        while let Token::Comma = self.current_token {
            self.eat(Some(&Token::Comma))?;
            let Token::Id(var_name) = self.current_token.clone() else {
                return Err(anyhow!("Expected at least one variable name after comma."));
            };
            var_names.push(var_name);
            self.eat(Some(&Token::Id("".to_owned())))?;
        }

        self.eat(Some(&Token::Colon))?;
        let type_spec = self.type_spec()?;

        let result = var_names
            .iter()
            .map(|n| {
                Box::new(ASTNode::VarDecl {
                    var_node: Box::new(ASTNode::Var { name: n.to_owned() }),
                    type_node: Box::new(type_spec.clone()),
                })
            })
            .collect();

        Ok(result)
    }

    fn type_spec(&mut self) -> Result<ASTNode> {
        match self.current_token {
            Token::Integer => {
                self.eat(Some(&Token::Integer))?;
                Ok(ASTNode::Type {
                    token: self.current_token.clone(),
                    value: Token::Integer,
                })
            }
            Token::Real => {
                self.eat(Some(&Token::Real))?;
                Ok(ASTNode::Type {
                    token: self.current_token.clone(),
                    value: Token::Real,
                })
            }
            _ => Err(anyhow!("Unsupported variable type.")),
        }
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
                Ok(ASTNode::Var { name: name.clone() })
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
            Token::IntegerConst(val) => {
                self.eat(Some(&Token::IntegerConst(0)))?;
                Ok(ASTNode::NumNode {
                    token: Token::IntegerConst(val),
                    value: ASTVarType::I32(val),
                })
            }
            Token::RealConst(val) => {
                self.eat(Some(&Token::RealConst(0.0)))?;
                Ok(ASTNode::NumNode {
                    token: Token::RealConst(val),
                    value: ASTVarType::F32(val),
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
                Token::Asterisk | Token::FloatDiv | Token::IntegerDiv => {
                    self.eat(Some(&op))?;

                    let right_node = self.factor()?;

                    result = ASTNode::BinOpNode {
                        left: Box::new(result),
                        right: Box::new(right_node),
                        op,
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
