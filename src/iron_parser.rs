//! Iron language parser
//!
//! Parses Iron tokens into an AST for transpilation to Rust.

use crate::iron_ast::*;
use crate::iron_tokenizer::{Token, Tokenizer};

pub struct IronParser {
    tokens: Vec<Token>,
    position: usize,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token, String),
    UnexpectedEndOfInput,
    InvalidSyntax(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(token, expected) => {
                write!(f, "Unexpected token {:?}, expected {}", token, expected)
            }
            ParseError::UnexpectedEndOfInput => {
                write!(f, "Unexpected end of input")
            }
            ParseError::InvalidSyntax(msg) => {
                write!(f, "Invalid syntax: {}", msg)
            }
        }
    }
}

impl std::error::Error for ParseError {}

impl IronParser {
    pub fn new(input: &str) -> Self {
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize();
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<IronFile, ParseError> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            let item = self.parse_item()?;
            items.push(item);
        }

        Ok(IronFile { items })
    }

    fn parse_item(&mut self) -> Result<IronItem, ParseError> {
        match self.peek() {
            Some(Token::Function) => self.parse_function(),
            Some(Token::Structure) => self.parse_struct(),
            Some(Token::Enumeration) => self.parse_enum(),
            Some(Token::Static) => self.parse_static(),
            Some(Token::Constant) => self.parse_const(),
            Some(Token::Type) => self.parse_type_alias(),
            Some(Token::Verbatim) => self.parse_verbatim_item(),
            Some(token) => Err(ParseError::UnexpectedToken(
                token.clone(),
                "function, structure, enumeration, static, constant, type, or verbatim".to_string(),
            )),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn parse_function(&mut self) -> Result<IronItem, ParseError> {
        self.expect(Token::Function)?;

        // Parse function name
        let name = self.expect_identifier()?;

        // Parse generics
        let generics = self.parse_generics_clause()?;

        self.skip_newlines();

        // Parse parameters (takes ...)
        let mut params = Vec::new();
        if self.match_token(Token::Takes) {
            params = self.parse_params()?;
        }

        // Parse return type
        self.skip_newlines();
        let return_type = if self.match_token(Token::Returns) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse body
        self.skip_newlines();
        self.expect(Token::Begin)?;
        let body = self.parse_block()?;
        self.expect(Token::End)?;

        // Expect "function" after end
        if !self.match_identifier("function") {
            // Could be another block type, just skip it
            self.advance();
        }

        Ok(IronItem::Function(IronFunction {
            name,
            generics,
            params,
            return_type,
            body,
        }))
    }

    fn parse_struct(&mut self) -> Result<IronItem, ParseError> {
        self.expect(Token::Structure)?;

        let name = self.expect_identifier()?;
        let generics = self.parse_generics_clause()?;

        self.expect(Token::With)?;
        self.expect(Token::Fields)?;

        let mut fields = Vec::new();
        while !self.check(Token::End) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(Token::End) {
                break;
            }

            let field_name = self.expect_identifier()?;
            self.expect(Token::Of)?;
            let ty = self.parse_type()?;

            fields.push(IronField {
                name: field_name,
                ty,
            });
        }

        self.expect(Token::End)?;
        self.advance(); // Skip "structure"

        Ok(IronItem::Struct(IronStruct {
            name,
            generics,
            fields,
        }))
    }

    fn parse_enum(&mut self) -> Result<IronItem, ParseError> {
        self.expect(Token::Enumeration)?;

        let name = self.expect_identifier()?;
        let generics = self.parse_generics_clause()?;

        self.expect(Token::With)?;
        self.expect(Token::Variants)?;

        let mut variants = Vec::new();
        while !self.check(Token::End) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(Token::End) {
                break;
            }

            let variant_name = self.expect_identifier()?;

            let data = if self.match_token(Token::Of) {
                let ty = self.parse_type()?;
                Some(IronVariantData::Type(ty))
            } else if self.match_token(Token::With) {
                let fields = self.parse_variant_fields()?;
                Some(IronVariantData::Fields(fields))
            } else {
                None
            };

            variants.push(IronVariant {
                name: variant_name,
                data,
            });
        }

        self.expect(Token::End)?;
        self.advance(); // Skip "enumeration"

        Ok(IronItem::Enum(IronEnum {
            name,
            generics,
            variants,
        }))
    }

    fn parse_static(&mut self) -> Result<IronItem, ParseError> {
        self.expect(Token::Static)?;

        let mutable = self.match_token(Token::Mutable);
        let name = self.expect_identifier()?;

        self.expect(Token::Of)?;
        let ty = self.parse_type()?;

        self.expect(Token::Begin)?;
        // For now, parse value as expression - this is simplified
        let value = IronExpr::Integer("0".to_string());
        self.expect(Token::End)?;
        self.advance(); // Skip "static"

        Ok(IronItem::Static(IronStatic {
            name,
            mutable,
            ty,
            value,
        }))
    }

    fn parse_const(&mut self) -> Result<IronItem, ParseError> {
        self.expect(Token::Constant)?;

        let name = self.expect_identifier()?;

        self.expect(Token::Of)?;
        let ty = self.parse_type()?;

        self.expect(Token::Begin)?;
        // For now, parse value as expression - this is simplified
        let value = IronExpr::Integer("0".to_string());
        self.expect(Token::End)?;
        self.advance(); // Skip "constant"

        Ok(IronItem::Const(IronConst { name, ty, value }))
    }

    fn parse_type_alias(&mut self) -> Result<IronItem, ParseError> {
        self.expect(Token::Type)?;

        let name = self.expect_identifier()?;
        let generics = self.parse_generics_clause()?;
        self.expect(Token::As)?;
        let ty = self.parse_type()?;

        Ok(IronItem::TypeAlias(IronTypeAlias { name, generics, ty }))
    }

    fn parse_verbatim_item(&mut self) -> Result<IronItem, ParseError> {
        self.expect(Token::Verbatim)?;

        if self.match_identifier("item") {
            // expected marker consumed
        }

        let payload = match self.peek() {
            Some(Token::String(value)) => {
                let value = value.clone();
                self.advance();
                value
            }
            Some(token) => {
                return Err(ParseError::UnexpectedToken(
                    token.clone(),
                    "string literal payload".to_string(),
                ));
            }
            None => return Err(ParseError::UnexpectedEndOfInput),
        };

        Ok(IronItem::Verbatim(payload))
    }

    fn parse_generics_clause(&mut self) -> Result<Vec<IronGeneric>, ParseError> {
        let mut generics = Vec::new();

        while let Some(&Token::With) = self.peek() {
            // Peek ahead to see if this "with" is followed by "generic"
            let next_is_generic = match self.tokens.get(self.position + 1) {
                Some(Token::Generic) => true,
                _ => false,
            };

            if !next_is_generic {
                break;
            }

            self.advance(); // Consume With
            self.expect(Token::Generic)?;
            self.expect(Token::Type)?;
            let name = self.expect_identifier()?;

            let mut bounds = Vec::new();
            if self.match_token(Token::Implementing) {
                loop {
                    let bound_name = self.expect_identifier()?;
                    bounds.push(IronBound {
                        trait_name: bound_name,
                    });

                    if !self.match_token(Token::And) {
                        break;
                    }
                }
            }

            generics.push(IronGeneric { name, bounds });
        }

        Ok(generics)
    }

    fn parse_params(&mut self) -> Result<Vec<IronParam>, ParseError> {
        let mut params = Vec::new();

        loop {
            let param_name = self.expect_identifier()?;
            self.expect(Token::Of)?;
            let ty = self.parse_type()?;

            params.push(IronParam {
                name: param_name,
                ty,
            });

            if !self.match_token(Token::And) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_variant_fields(&mut self) -> Result<Vec<IronField>, ParseError> {
        let mut fields = Vec::new();

        loop {
            let field_name = self.expect_identifier()?;
            self.expect(Token::Of)?;
            let ty = self.parse_type()?;

            fields.push(IronField {
                name: field_name,
                ty,
            });

            if !self.match_token(Token::And) {
                break;
            }
        }

        Ok(fields)
    }

    fn parse_type(&mut self) -> Result<IronType, ParseError> {
        // Complex type parsing
        if self.match_token(Token::Reference) {
            self.expect(Token::To)?;
            let inner = self.parse_type()?;
            return Ok(IronType::Reference(Box::new(inner)));
        }

        if self.match_token(Token::Mutable) {
            if self.check(Token::Reference) {
                self.advance();
                self.expect(Token::To)?;
                let inner = self.parse_type()?;
                return Ok(IronType::MutableReference(Box::new(inner)));
            } else if self.check(Token::Raw) {
                self.advance();
                self.expect(Token::Pointer)?;
                self.expect(Token::To)?;
                let inner = self.parse_type()?;
                return Ok(IronType::MutableRawPointer(Box::new(inner)));
            }
            // Could be "mutable" as part of another construct
        }

        if self.match_token(Token::Raw) {
            self.expect(Token::Pointer)?;
            self.expect(Token::To)?;
            let inner = self.parse_type()?;
            return Ok(IronType::RawPointer(Box::new(inner)));
        }

        if self.match_token(Token::Optional) {
            let inner = self.parse_type()?;
            return Ok(IronType::Optional(Box::new(inner)));
        }

        if self.match_token(Token::Result) {
            self.expect(Token::Of)?;
            let ok_type = self.parse_type()?;
            self.expect(Token::Or)?;
            self.expect(Token::Error)?;
            let err_type = self.parse_type()?;
            return Ok(IronType::Result(Box::new(ok_type), Box::new(err_type)));
        }

        if self.match_token(Token::List) {
            self.expect(Token::Of)?;
            let inner = self.parse_type()?;
            return Ok(IronType::List(Box::new(inner)));
        }

        if self.match_token(Token::Box) {
            self.expect(Token::Containing)?;
            let inner = self.parse_type()?;
            return Ok(IronType::BoxType(Box::new(inner)));
        }

        if self.match_token(Token::Tuple) {
            self.expect(Token::Of)?;
            let mut types = Vec::new();
            loop {
                types.push(self.parse_type()?);
                if !self.match_token(Token::And) {
                    break;
                }
            }
            return Ok(IronType::Tuple(types));
        }

        if self.match_token(Token::Array) {
            self.expect(Token::Of)?;
            let inner = self.parse_type()?;
            return Ok(IronType::Array(Box::new(inner)));
        }

        if self.match_token(Token::Slice) {
            self.expect(Token::Of)?;
            let inner = self.parse_type()?;
            return Ok(IronType::Slice(Box::new(inner)));
        }

        if self.match_token(Token::Function) {
            self.expect(Token::Taking)?;
            let mut params = Vec::new();
            loop {
                params.push(self.parse_type()?);
                if !self.match_token(Token::And) {
                    break;
                }
            }
            self.expect(Token::Returning)?;
            let ret = self.parse_type()?;
            return Ok(IronType::Function(params, Box::new(ret)));
        }

        if self.match_token(Token::Error) {
            return Ok(IronType::Named("error".to_string()));
        }

        // Simple type name
        let name = self.expect_identifier()?;

        if name == "string" && self.match_token(Token::Slice) {
            return Ok(IronType::Named("string slice".to_string()));
        }

        // Generic application for named types: MyType of A and B
        if self.match_token(Token::Of) {
            let mut args = Vec::new();
            loop {
                args.push(self.parse_type()?);
                if !self.match_token(Token::And) {
                    break;
                }
            }

            let rendered_args = args
                .iter()
                .map(Self::render_type_for_rust)
                .collect::<Vec<_>>()
                .join(", ");
            return Ok(IronType::Named(format!("{}<{}>", name, rendered_args)));
        }

        Ok(IronType::Named(name))
    }

    fn render_type_for_rust(ty: &IronType) -> String {
        match ty {
            IronType::Named(name) => match name.as_str() {
                "boolean" => "bool".to_string(),
                "character" => "char".to_string(),
                "string" => "String".to_string(),
                "string slice" => "str".to_string(),
                "list" => "Vec".to_string(),
                "optional" => "Option".to_string(),
                "result" => "Result".to_string(),
                "hash map" => "HashMap".to_string(),
                "box" => "Box".to_string(),
                "reference counted" => "Rc".to_string(),
                "atomic reference counted" => "Arc".to_string(),
                "unit" => "()".to_string(),
                "error" => "dyn std::error::Error".to_string(),
                _ => name.clone(),
            },
            IronType::Reference(inner) => format!("&{}", Self::render_type_for_rust(inner)),
            IronType::MutableReference(inner) => {
                format!("&mut {}", Self::render_type_for_rust(inner))
            }
            IronType::RawPointer(inner) => format!("*const {}", Self::render_type_for_rust(inner)),
            IronType::MutableRawPointer(inner) => {
                format!("*mut {}", Self::render_type_for_rust(inner))
            }
            IronType::Optional(inner) => format!("Option<{}>", Self::render_type_for_rust(inner)),
            IronType::Result(ok, err) => format!(
                "Result<{}, {}>",
                Self::render_type_for_rust(ok),
                Self::render_type_for_rust(err)
            ),
            IronType::List(inner) => format!("Vec<{}>", Self::render_type_for_rust(inner)),
            IronType::BoxType(inner) => format!("Box<{}>", Self::render_type_for_rust(inner)),
            IronType::Tuple(items) => {
                let rendered = items
                    .iter()
                    .map(Self::render_type_for_rust)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", rendered)
            }
            IronType::Array(inner) => format!("[{}]", Self::render_type_for_rust(inner)),
            IronType::Slice(inner) => format!("[{}]", Self::render_type_for_rust(inner)),
            IronType::Function(params, ret) => {
                let rendered_params = params
                    .iter()
                    .map(Self::render_type_for_rust)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "fn({}) -> {}",
                    rendered_params,
                    Self::render_type_for_rust(ret)
                )
            }
            IronType::Generic(name, _bounds) => name.clone(),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<IronStmt>, ParseError> {
        let mut stmts = Vec::new();

        while !self.check(Token::End) && !self.is_at_end() {
            self.skip_newlines();

            if self.check(Token::End) {
                break;
            }

            let stmt = self.parse_statement()?;
            stmts.push(stmt);
        }

        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<IronStmt, ParseError> {
        match self.peek() {
            Some(Token::Define) => self.parse_let(),
            Some(Token::Set) => self.parse_assign(),
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            Some(Token::For) => self.parse_for(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Exit) => self.parse_break(),
            Some(Token::Continue) => self.parse_continue(),
            _ => {
                let expr = self.parse_expression()?;
                Ok(IronStmt::Expr(expr))
            }
        }
    }

    fn parse_let(&mut self) -> Result<IronStmt, ParseError> {
        self.expect(Token::Define)?;

        let mutable = self.match_token(Token::Mutable);
        let name = self.expect_identifier()?;

        self.expect(Token::As)?;
        let value = self.parse_expression()?;

        Ok(IronStmt::Let {
            name,
            mutable,
            value,
        })
    }

    fn parse_assign(&mut self) -> Result<IronStmt, ParseError> {
        self.expect(Token::Set)?;

        let target = IronExpr::Identifier(self.expect_identifier()?);
        self.expect(Token::Equal)?;
        self.expect(Token::To)?;
        let value = self.parse_expression()?;

        Ok(IronStmt::Assign { target, value })
    }

    fn parse_if(&mut self) -> Result<IronStmt, ParseError> {
        self.expect(Token::If)?;

        let condition = self.parse_expression()?;
        self.expect(Token::Then)?;
        self.skip_newlines();
        self.expect(Token::Begin)?;

        let then_block = self.parse_block()?;
        self.expect(Token::End)?;
        self.advance(); // Skip "if"

        let else_block = if self.match_token(Token::Otherwise) {
            self.skip_newlines();
            self.expect(Token::Begin)?;
            let block = self.parse_block()?;
            self.expect(Token::End)?;
            self.advance(); // Skip "if"
            Some(block)
        } else {
            None
        };

        Ok(IronStmt::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_while(&mut self) -> Result<IronStmt, ParseError> {
        self.expect(Token::While)?;

        let condition = self.parse_expression()?;
        self.expect(Token::Repeat)?;
        self.skip_newlines();
        self.expect(Token::Begin)?;

        let body = self.parse_block()?;

        self.expect(Token::End)?;
        self.advance(); // Skip "while"

        Ok(IronStmt::While { condition, body })
    }

    fn parse_for(&mut self) -> Result<IronStmt, ParseError> {
        self.expect(Token::For)?;
        self.expect(Token::Each)?;

        let var = self.expect_identifier()?;
        self.expect(Token::In)?;
        let iterator = self.parse_expression()?;
        self.expect(Token::Repeat)?;
        self.skip_newlines();
        self.expect(Token::Begin)?;

        let body = self.parse_block()?;

        self.expect(Token::End)?;
        self.advance(); // Skip "for"

        Ok(IronStmt::For {
            var,
            iterator,
            body,
        })
    }

    fn parse_return(&mut self) -> Result<IronStmt, ParseError> {
        self.expect(Token::Return)?;

        let value = if self.check(Token::End) || self.check(Token::NewLine) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(IronStmt::Return(value))
    }

    fn parse_break(&mut self) -> Result<IronStmt, ParseError> {
        self.expect(Token::Exit)?;
        self.expect(Token::Loop)?;
        Ok(IronStmt::Break)
    }

    fn parse_continue(&mut self) -> Result<IronStmt, ParseError> {
        self.expect(Token::Continue)?;
        self.expect(Token::Loop)?;
        Ok(IronStmt::Continue)
    }

    fn parse_expression(&mut self) -> Result<IronExpr, ParseError> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, min_precedence: u8) -> Result<IronExpr, ParseError> {
        let mut left = self.parse_primary_expression()?;

        loop {
            self.skip_newlines();
            let Some(op) = self.peek_binary_op() else {
                break;
            };
            let precedence = self.get_precedence(&op);
            if precedence < min_precedence {
                break;
            }

            self.advance();

            // Consume additional tokens for multi-word operators
            match op {
                IronBinaryOp::Gt | IronBinaryOp::Lt => {
                    self.expect(Token::Than)?;
                    // Check for "or equal to"
                    if self.match_token(Token::Or) {
                        self.expect(Token::Equal)?;
                        self.expect(Token::To)?;
                    }
                }
                IronBinaryOp::Eq => {
                    self.expect(Token::To)?;
                }
                IronBinaryOp::Ne => {
                    self.expect(Token::Equal)?;
                    self.expect(Token::To)?;
                }
                _ => {}
            }

            self.skip_newlines();
            let right = self.parse_binary_expression(precedence + 1)?;

            left = IronExpr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary_expression(&mut self) -> Result<IronExpr, ParseError> {
        match self.peek() {
            Some(Token::Field) => {
                self.advance();
                let field_name = self.expect_identifier()?;
                self.expect(Token::Of)?;
                let base = self.parse_expression()?;
                Ok(IronExpr::FieldAccess {
                    base: Box::new(base),
                    field: field_name,
                })
            }
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();

                // Check for method call or field access
                if self.match_token(Token::Of) {
                    // Field access: field X of Y
                    if name == "field" {
                        let field_name = self.expect_identifier()?;
                        let base = self.parse_expression()?;
                        return Ok(IronExpr::FieldAccess {
                            base: Box::new(base),
                            field: field_name,
                        });
                    }
                }

                Ok(IronExpr::Identifier(name))
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(IronExpr::String(s))
            }
            Some(Token::Integer(n)) => {
                let n = n.clone();
                self.advance();
                Ok(IronExpr::Integer(n))
            }
            Some(Token::Float(n)) => {
                let n = n.clone();
                self.advance();
                Ok(IronExpr::Float(n))
            }
            Some(Token::Boolean(b)) => {
                let b = *b;
                self.advance();
                Ok(IronExpr::Boolean(b))
            }
            Some(Token::Some) => {
                self.advance();
                self.expect(Token::Of)?;
                let expr = self.parse_expression()?;
                Ok(IronExpr::Some(Box::new(expr)))
            }
            Some(Token::None) => {
                self.advance();
                Ok(IronExpr::None)
            }
            Some(Token::Ok) => {
                self.advance();
                self.expect(Token::Of)?;
                let expr = self.parse_expression()?;
                Ok(IronExpr::Ok(Box::new(expr)))
            }
            Some(Token::Error) => {
                self.advance();
                self.expect(Token::Of)?;
                let expr = self.parse_expression()?;
                Ok(IronExpr::Err(Box::new(expr)))
            }
            Some(Token::Array) => {
                self.advance();
                self.expect(Token::Of)?;

                let mut elems = Vec::new();
                loop {
                    elems.push(self.parse_expression()?);
                    if !self.match_token(Token::And) {
                        break;
                    }
                }

                Ok(IronExpr::Array(elems))
            }
            Some(Token::Tuple) => {
                self.advance();
                self.expect(Token::Of)?;

                let mut elems = Vec::new();
                loop {
                    elems.push(self.parse_expression()?);
                    if !self.match_token(Token::And) {
                        break;
                    }
                }

                Ok(IronExpr::Tuple(elems))
            }
            Some(Token::Range) => {
                self.advance();
                self.expect(Token::From)?;
                let start = self.parse_expression()?;
                self.expect(Token::To)?;

                let end = if self.check(Token::End) {
                    self.advance();
                    None
                } else {
                    Some(Box::new(self.parse_expression()?))
                };

                Ok(IronExpr::Range {
                    start: Some(Box::new(start)),
                    end,
                    inclusive: false,
                })
            }
            Some(Token::Inclusive) => {
                self.advance();
                self.expect(Token::Range)?;
                self.expect(Token::From)?;
                let start = self.parse_expression()?;
                self.expect(Token::To)?;

                let end = if self.check(Token::End) {
                    self.advance();
                    None
                } else {
                    Some(Box::new(self.parse_expression()?))
                };

                Ok(IronExpr::Range {
                    start: Some(Box::new(start)),
                    end,
                    inclusive: true,
                })
            }
            Some(Token::Index) => {
                self.advance();
                let base = self.parse_expression()?;
                self.expect(Token::At)?;
                let index = self.parse_expression()?;
                Ok(IronExpr::Index {
                    base: Box::new(base),
                    index: Box::new(index),
                })
            }
            Some(Token::Closure) => {
                self.advance();

                // Check for move keyword
                let _is_move = self.match_token(Token::Move);

                // Parse parameters
                let mut params = Vec::new();
                if self.match_token(Token::With) {
                    if self.match_token(Token::Parameters) {
                        // Parse parameter list
                        loop {
                            let param_name = self.expect_identifier()?;

                            // Check for parameter type
                            let param_ty = if self.check(Token::Of) {
                                // Parse type specification if present
                                self.advance(); // consume 'of'
                                self.parse_type()?
                            } else {
                                IronType::Named("unknown".to_string())
                            };

                            params.push(IronParam {
                                name: param_name,
                                ty: param_ty,
                            });

                            // Check if next token is "and"
                            if !self.check(Token::And) {
                                break;
                            }

                            // Peek ahead - is "and" followed by "body"?
                            if self.peek_next() == Some(&Token::Body) {
                                // This "and" is the connector to the body, not a param separator
                                break;
                            }

                            // Consume the "and" and continue to next parameter
                            self.advance();
                        }
                    } else if self.match_token(Token::Body) {
                        // Zero-parameter closure form: "closure with body ..."
                        let body_expr = self.parse_expression()?;
                        let body = vec![IronStmt::Expr(body_expr)];
                        return Ok(IronExpr::Closure { params, body });
                    }
                }

                // Parse body - expects "and body"
                self.expect(Token::And)?;
                self.expect(Token::Body)?;

                // For now, parse body as a single expression
                let body_expr = self.parse_expression()?;
                let body = vec![IronStmt::Expr(body_expr)];

                Ok(IronExpr::Closure { params, body })
            }
            Some(Token::Macro) => {
                self.advance();
                let name = self.expect_symbol_identifier()?;

                let (args, uses_brackets) = if self.match_token(Token::With) {
                    // Collect raw argument tokens preserving commas and structure
                    let mut arg_parts = Vec::new();
                    let mut uses_brackets = false;

                    while !self.is_at_end()
                        && !self.check(Token::NewLine)
                        && !self.check(Token::End)
                    {
                        // Check for bracket keyword which marks end of args
                        if self.check(Token::Bracket) {
                            uses_brackets = true;
                            self.advance();
                            break;
                        }

                        match self.peek() {
                            Some(Token::Comma) => {
                                arg_parts.push(",".to_string());
                                self.advance();
                            }
                            Some(Token::Identifier(s)) => {
                                arg_parts.push(s.clone());
                                self.advance();
                            }
                            Some(Token::Integer(n)) => {
                                arg_parts.push(n.clone());
                                self.advance();
                            }
                            Some(Token::Float(n)) => {
                                arg_parts.push(n.clone());
                                self.advance();
                            }
                            Some(Token::String(s)) => {
                                arg_parts.push(format!("\"{}\"", s));
                                self.advance();
                            }
                            _ => {
                                // Skip unknown tokens but preserve structure
                                self.advance();
                            }
                        }
                    }
                    (arg_parts.join(" "), uses_brackets)
                } else {
                    // Check for bracket suffix even without args
                    let uses_brackets = self.match_token(Token::Bracket);
                    (String::new(), uses_brackets)
                };

                Ok(IronExpr::Macro {
                    name,
                    args,
                    bracket: uses_brackets,
                })
            }
            Some(Token::Call) => {
                self.advance();

                // Check if this is "call associated function X on Y"
                if self.match_token(Token::Associated) {
                    self.expect(Token::Function)?;
                    let function_name = self.expect_symbol_identifier()?;
                    self.expect(Token::On)?;

                    let mut type_name = self.expect_symbol_identifier()?;
                    while let Some(segment) = self.take_symbol_identifier() {
                        type_name.push_str("::");
                        type_name.push_str(&segment);
                    }

                    let mut args = Vec::new();
                    if self.match_token(Token::With) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token(Token::And) {
                                break;
                            }
                        }
                    }

                    // Associated functions are different from method calls
                    let mut expr = IronExpr::AssociatedFunctionCall {
                        type_name,
                        function: function_name,
                        args,
                    };

                    // Check for "unwrap or return error" (try operator)
                    if self.match_token(Token::Unwrap) {
                        self.expect(Token::Or)?;
                        self.expect(Token::Return)?;
                        self.expect(Token::Error)?;
                        expr = IronExpr::Try {
                            expr: Box::new(expr),
                        };
                    }

                    return Ok(expr);
                }

                // Check if this is "call method X on Y"
                if self.match_token(Token::Method) {
                    let method_name = self.expect_symbol_identifier()?;
                    self.expect(Token::On)?;
                    let receiver = self.parse_expression()?;

                    let mut args = Vec::new();
                    if self.match_token(Token::With) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token(Token::And) {
                                break;
                            }
                        }
                    }

                    let mut expr = IronExpr::MethodCall {
                        receiver: Box::new(receiver),
                        method: method_name,
                        args,
                    };

                    // Check for "unwrap or return error" (try operator)
                    if self.match_token(Token::Unwrap) {
                        self.expect(Token::Or)?;
                        self.expect(Token::Return)?;
                        self.expect(Token::Error)?;
                        expr = IronExpr::Try {
                            expr: Box::new(expr),
                        };
                    }

                    return Ok(expr);
                }

                // Regular function call
                let func = self.parse_expression()?;
                let mut args = Vec::new();
                if self.match_token(Token::With) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.match_token(Token::And) {
                            break;
                        }
                    }
                }

                let mut expr = IronExpr::Call {
                    func: Box::new(func),
                    args,
                };

                // Check for "unwrap or return error" (try operator)
                if self.match_token(Token::Unwrap) {
                    self.expect(Token::Or)?;
                    self.expect(Token::Return)?;
                    self.expect(Token::Error)?;
                    expr = IronExpr::Try {
                        expr: Box::new(expr),
                    };
                }

                Ok(expr)
            }
            Some(Token::Create) => {
                self.advance();
                // Create struct: create TypeName [with field1 of value1 and field2 of value2]
                let type_name = self.expect_identifier()?;

                // Check for field initialization
                let fields = if self.match_token(Token::With) {
                    let mut fields = Vec::new();
                    loop {
                        let field_name = self.expect_identifier()?;
                        self.expect(Token::Of)?;
                        let value = self.parse_expression()?;
                        fields.push((
                            IronField {
                                name: field_name,
                                ty: IronType::Named("unknown".to_string()),
                            },
                            value,
                        ));
                        if !self.match_token(Token::And) {
                            break;
                        }
                    }
                    fields
                } else {
                    Vec::new()
                };

                Ok(IronExpr::Struct {
                    name: type_name,
                    fields,
                })
            }
            Some(token) => Err(ParseError::UnexpectedToken(
                token.clone(),
                "expression".to_string(),
            )),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    // Helper methods
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Some(Token::EndOfFile) | None)
    }

    fn check(&self, token: Token) -> bool {
        self.peek() == Some(&token)
    }

    fn match_token(&mut self, token: Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_identifier(&mut self, name: &str) -> bool {
        if let Some(Token::Identifier(n)) = self.peek() {
            if n == name {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expect(&mut self, token: Token) -> Result<(), ParseError> {
        if self.check(token.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(
                self.peek().cloned().unwrap_or(Token::EndOfFile),
                format!("{:?}", token),
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            Some(Token::From) => {
                self.advance();
                Ok("from".to_string())
            }
            Some(Token::Error) => {
                self.advance();
                Ok("error".to_string())
            }
            Some(token) => Err(ParseError::UnexpectedToken(
                token.clone(),
                "identifier".to_string(),
            )),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn expect_symbol_identifier(&mut self) -> Result<String, ParseError> {
        match self.take_symbol_identifier() {
            Some(name) => Ok(name),
            None => match self.peek() {
                Some(token) => Err(ParseError::UnexpectedToken(
                    token.clone(),
                    "identifier".to_string(),
                )),
                None => Err(ParseError::UnexpectedEndOfInput),
            },
        }
    }

    fn take_symbol_identifier(&mut self) -> Option<String> {
        match self.peek() {
            Some(Token::Identifier(_)) => {
                let name = match self.peek() {
                    Some(Token::Identifier(name)) => name.clone(),
                    _ => unreachable!(),
                };
                self.advance();
                Some(name)
            }
            Some(Token::From) => {
                self.advance();
                Some("from".to_string())
            }
            Some(Token::Error) => {
                self.advance();
                Some("error".to_string())
            }
            Some(Token::Ok) => {
                self.advance();
                Some("ok".to_string())
            }
            Some(Token::Some) => {
                self.advance();
                Some("some".to_string())
            }
            Some(Token::None) => {
                self.advance();
                Some("none".to_string())
            }
            Some(Token::Result) => {
                self.advance();
                Some("result".to_string())
            }
            Some(Token::Optional) => {
                self.advance();
                Some("optional".to_string())
            }
            Some(Token::List) => {
                self.advance();
                Some("list".to_string())
            }
            Some(Token::Box) => {
                self.advance();
                Some("box".to_string())
            }
            _ => None,
        }
    }

    fn skip_newlines(&mut self) {
        while let Some(token) = self.peek() {
            match token {
                Token::NewLine => self.advance(),
                Token::Indent(_) => self.advance(),
                _ => break,
            }
        }
    }

    fn peek_binary_op(&self) -> Option<IronBinaryOp> {
        match self.peek() {
            Some(Token::Plus) => Some(IronBinaryOp::Add),
            Some(Token::Minus) => Some(IronBinaryOp::Sub),
            Some(Token::Times) => Some(IronBinaryOp::Mul),
            Some(Token::Divided) => Some(IronBinaryOp::Div),
            Some(Token::Modulo) => Some(IronBinaryOp::Mod),
            Some(Token::And) => Some(IronBinaryOp::And),
            Some(Token::Or) => Some(IronBinaryOp::Or),
            Some(Token::Equal) => Some(IronBinaryOp::Eq),
            Some(Token::Greater) => {
                // Check for "greater than" or "greater than or equal to"
                if self.peek_next() == Some(&Token::Than) {
                    Some(IronBinaryOp::Gt)
                } else {
                    None
                }
            }
            Some(Token::Less) => {
                // Check for "less than" or "less than or equal to"
                if self.peek_next() == Some(&Token::Than) {
                    Some(IronBinaryOp::Lt)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    fn get_precedence(&self, op: &IronBinaryOp) -> u8 {
        match op {
            IronBinaryOp::Or => 1,
            IronBinaryOp::And => 2,
            IronBinaryOp::Eq | IronBinaryOp::Ne => 3,
            IronBinaryOp::Lt | IronBinaryOp::Le | IronBinaryOp::Gt | IronBinaryOp::Ge => 4,
            IronBinaryOp::Add | IronBinaryOp::Sub => 5,
            IronBinaryOp::Mul | IronBinaryOp::Div | IronBinaryOp::Mod => 6,
            _ => 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let input = r#"function hello
begin
    return 42
end function"#;

        let mut parser = IronParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_method_call_named_ok() {
        let input = r#"function to_option
    takes input of result of i32 or error string
    returns optional i32
begin
    call method ok on input
end function"#;

        let mut parser = IronParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_parse_macro_named_result() {
        let input = r#"function make_result
    returns optional i32
begin
    call method ok on macro result with 42
end function"#;

        let mut parser = IronParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_parse_multiline_or_expression() {
        let input = r#"function choose_option
    returns optional i32
begin
    call method ok on macro result with 42
    or
    call method ok on macro result with 0
end function"#;

        let mut parser = IronParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_parse_associated_call_with_keyword_path_segments() {
        let input = r#"function call_result_ok
    returns result of i32 or error string
begin
    call associated function ok on result with 42
end function"#;

        let mut parser = IronParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "{:?}", result.err());
    }
}
