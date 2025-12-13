use crate::ast::{ASTNode, BuiltinNumTypes};
use crate::lexer::Lexer;
use crate::symbols::BuiltinTypes;
use crate::token::{LocatedToken, Token};
use anyhow::Result;
use std::cell::RefCell;
use std::fmt;

#[derive(Debug, Clone)]
pub struct SyntaxError {
    title: String,
    detail: Option<String>,
    line: usize,
    column: usize,
    snippet: String,
}

impl SyntaxError {
    fn with_detail(
        location: &LocatedToken,
        title: impl Into<String>,
        detail: Option<String>,
    ) -> Self {
        Self {
            title: title.into(),
            detail,
            line: location.line,
            column: location.column,
            snippet: location.snippet.clone(),
        }
    }

    fn unexpected_token(location: &LocatedToken, expected: Option<&Token>) -> Self {
        let detail = match expected {
            Some(expected_token) => format!(
                "expected {}, found {}",
                expected_token,
                location.token.clone()
            ),
            None => format!("found {}", location.token.clone()),
        };
        Self::with_detail(location, "Unexpected token type", Some(detail))
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} at position {}:{}",
            self.title, self.line, self.column
        )?;
        if !self.snippet.is_empty() {
            writeln!(f, "{}", self.snippet)?;
            let caret_column = self.column.saturating_sub(1);
            writeln!(f, "{:>width$}^", "", width = caret_column)?;
        }
        if let Some(detail) = &self.detail {
            write!(f, "    {}", detail)?;
        }
        Ok(())
    }
}

impl std::error::Error for SyntaxError {}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: LocatedToken,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Result<Self> {
        let current_token = lexer.next_token()?;
        Ok(Parser {
            lexer,
            current_token,
        })
    }

    pub fn parse(&mut self) -> Result<ASTNode> {
        self.program()
    }

    fn current_kind(&self) -> Token {
        self.current_token.token.clone()
    }

    fn current_location(&self) -> &LocatedToken {
        &self.current_token
    }

    fn eat(&mut self, expected_type: Option<&Token>) -> Result<()> {
        if let Some(expected) = expected_type {
            if std::mem::discriminant(&self.current_token.token) != std::mem::discriminant(expected)
            {
                return Err(
                    SyntaxError::unexpected_token(&self.current_token, Some(expected)).into(),
                );
            }
        }
        self.current_token = (&mut self.lexer).next_token()?;
        Ok(())
    }

    fn program(&mut self) -> Result<ASTNode> {
        self.eat(Some(&Token::Program))?;
        let var_node = self.variable()?;
        let ASTNode::Var { name: program_name } = var_node else {
            let err = SyntaxError::with_detail(
                self.current_location(),
                "Invalid program declaration",
                Some("expected a program name after PROGRAM".into()),
            );
            return Err(err.into());
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

        while matches!(self.current_kind(), Token::Var | Token::Procedure) {
            if matches!(self.current_kind(), Token::Var) {
                self.eat(Some(&Token::Var))?;
                while matches!(self.current_kind(), Token::Id(_)) {
                    let vd = self.variable_declaration()?;
                    declarations.extend(vd);
                    self.eat(Some(&Token::Semi))?;
                }
            } else {
                self.eat(Some(&Token::Procedure))?;
                let Token::Id(procedure_name) = self.current_kind() else {
                    let err = SyntaxError::with_detail(
                        self.current_location(),
                        "Unexpected token type",
                        Some("expected identifier after PROCEDURE".into()),
                    );
                    return Err(err.into());
                };
                self.eat(Some(&Token::Id(String::new())))?;

                let mut params = vec![];
                if matches!(self.current_kind(), Token::LParenthesis) {
                    self.eat(Some(&Token::LParenthesis))?;
                    params = self.formal_parameter_list()?;
                    self.eat(Some(&Token::RParenthesis))?;
                }

                self.eat(Some(&Token::Semi))?;
                let block = self.block()?;
                self.eat(Some(&Token::Semi))?;
                declarations.push(Box::new(ASTNode::ProcedureDecl {
                    proc_name: procedure_name,
                    params,
                    block_node: Box::new(block),
                }));
            }
        }

        Ok(declarations)
    }

    fn formal_parameter_list(&mut self) -> Result<Vec<Box<ASTNode>>> {
        let mut params = self.formal_parameters()?;

        while matches!(self.current_kind(), Token::Semi) {
            self.eat(Some(&Token::Semi))?;
            let res = self.formal_parameters()?;
            params.extend(res);
        }

        Ok(params)
    }

    fn formal_parameters(&mut self) -> Result<Vec<Box<ASTNode>>> {
        let mut var_names = vec![];
        let Token::Id(var_name) = self.current_kind() else {
            let err = SyntaxError::with_detail(
                self.current_location(),
                "Unexpected token type",
                Some("expected identifier in parameter declaration".into()),
            );
            return Err(err.into());
        };
        var_names.push(var_name);

        self.eat(Some(&Token::Id(String::new())))?;

        while matches!(self.current_kind(), Token::Comma) {
            self.eat(Some(&Token::Comma))?;
            let Token::Id(var_name) = self.current_kind() else {
                let err = SyntaxError::with_detail(
                    self.current_location(),
                    "Unexpected token type",
                    Some("expected identifier after comma".into()),
                );
                return Err(err.into());
            };
            var_names.push(var_name);
            self.eat(Some(&Token::Id(String::new())))?;
        }

        self.eat(Some(&Token::Colon))?;
        let type_spec = self.type_spec()?;

        let result = var_names
            .iter()
            .map(|n| {
                Box::new(ASTNode::Param {
                    var_node: Box::new(ASTNode::Var { name: n.to_owned() }),
                    type_node: Box::new(type_spec.clone()),
                })
            })
            .collect();

        Ok(result)
    }

    fn proc_call_statement(&mut self) -> Result<ASTNode> {
        let Token::Id(proc_name) = self.current_kind() else {
            let err = SyntaxError::with_detail(
                self.current_location(),
                "Expected function name",
                Some("Expected function identifier before ()".into()),
            );
            return Err(err.into());
        };

        self.eat(Some(&Token::Id("".to_string())))?;
        self.eat(Some(&Token::LParenthesis))?;

        let mut argument_nodes = vec![];
        if !matches!(self.current_kind(), Token::RParenthesis,) {
            let expr = self.expr()?;
            argument_nodes.push(Box::new(expr));
        }

        while let Token::Comma = self.current_kind() {
            self.eat(Some(&Token::Comma))?;
            let expr = self.expr()?;
            argument_nodes.push(Box::new(expr));
        }

        self.eat(Some(&Token::RParenthesis))?;

        Ok(ASTNode::ProcedureCall {
            proc_name: proc_name,
            arguments: argument_nodes,
            proc_symbol: RefCell::new(None),
        })
    }

    fn variable_declaration(&mut self) -> Result<Vec<Box<ASTNode>>> {
        let mut var_names = vec![];
        let Token::Id(var_name) = self.current_kind() else {
            let err = SyntaxError::with_detail(
                self.current_location(),
                "Unexpected token type",
                Some("expected identifier in declaration".into()),
            );
            return Err(err.into());
        };
        var_names.push(var_name);

        self.eat(Some(&Token::Id(String::new())))?;

        while matches!(self.current_kind(), Token::Comma) {
            self.eat(Some(&Token::Comma))?;
            let Token::Id(var_name) = self.current_kind() else {
                let err = SyntaxError::with_detail(
                    self.current_location(),
                    "Unexpected token type",
                    Some("expected identifier after comma".into()),
                );
                return Err(err.into());
            };
            var_names.push(var_name);
            self.eat(Some(&Token::Id(String::new())))?;
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
        match self.current_kind() {
            Token::Integer => {
                self.eat(Some(&Token::Integer))?;
                Ok(ASTNode::Type {
                    value: BuiltinTypes::Integer.to_string(),
                })
            }
            Token::Real => {
                self.eat(Some(&Token::Real))?;
                Ok(ASTNode::Type {
                    value: BuiltinTypes::Real.to_string(),
                })
            }
            _ => Err(SyntaxError::with_detail(
                self.current_location(),
                "Unsupported variable type",
                Some(format!("found {}", self.current_location().token.clone())),
            )
            .into()),
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

        while matches!(self.current_kind(), Token::Semi) {
            self.eat(Some(&Token::Semi))?;
            statement_list.push(Box::new(self.statement()?));
        }

        if matches!(self.current_kind(), Token::Id(_)) {
            let err = SyntaxError::with_detail(
                self.current_location(),
                "Unexpected token type",
                Some("possible missing semicolon between statements".into()),
            );
            return Err(err.into());
        }

        Ok(statement_list)
    }

    fn statement(&mut self) -> Result<ASTNode> {
        match self.current_kind() {
            Token::Begin => self.compound_statement(),
            Token::Id(_) => {
                if let LocatedToken {
                    token: Token::LParenthesis,
                    ..
                } = self.lexer.peek_token()?
                {
                    self.proc_call_statement()
                } else {
                    self.assignment_statement()
                }
            }
            _ => self.empty(),
        }
    }

    fn assignment_statement(&mut self) -> Result<ASTNode> {
        let var_node = self.variable()?;
        let token = self.current_kind();
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
        let token = self.current_kind();
        if let Token::Id(name) = token.clone() {
            self.eat(Some(&token))?;
            Ok(ASTNode::Var { name })
        } else {
            let err = SyntaxError::with_detail(
                self.current_location(),
                "Unexpected token type",
                Some("expected identifier".into()),
            );
            Err(err.into())
        }
    }

    fn factor(&mut self) -> Result<ASTNode> {
        match self.current_kind() {
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
                    value: BuiltinNumTypes::I32(val),
                })
            }
            Token::RealConst(val) => {
                self.eat(Some(&Token::RealConst(0.0)))?;
                Ok(ASTNode::NumNode {
                    value: BuiltinNumTypes::F32(val),
                })
            }
            Token::LParenthesis => {
                self.eat(Some(&Token::LParenthesis))?;
                let result = self.expr()?;
                self.eat(Some(&Token::RParenthesis))?;
                Ok(result)
            }
            Token::Id(_) => self.variable(),
            _ => {
                let err = SyntaxError::with_detail(
                    self.current_location(),
                    "Unexpected token type",
                    Some("expected numeric literal or factor".into()),
                );
                Err(err.into())
            }
        }
    }

    fn term(&mut self) -> Result<ASTNode> {
        let mut result = self.factor()?;

        loop {
            let op = self.current_kind();

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
            let op = self.current_kind();

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
