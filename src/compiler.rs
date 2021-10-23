use crate::{
    chunk::{Chunk, OpCode},
    scanner::{Scanner, Token, TokenType},
    value::Value,
};
use std::{collections::HashMap, convert::TryFrom};

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    // "When derived on enums, variants are ordered by their top-to-bottom discriminant order."
    // from lowest to highest
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}
impl Precedence {
    fn next(&self) -> Self {
        use Precedence::*;
        match *self {
            None => Assignment,
            Assignment => Or,
            Or => And,
            And => Equality,
            Equality => Comparison,
            Comparison => Term,
            Term => Factor,
            Factor => Unary,
            Unary => Call,
            Call => Primary,
            Primary => None,
        }
    }
}

pub type ParseFn<'src> = fn(&mut Parser<'src>) -> ();

struct ParseRule<'src> {
    prefix: Option<ParseFn<'src>>, // = how to parse the token if it is prefix
    infix: Option<ParseFn<'src>>,  // = same but the token is infix
    precedence: Precedence,
}

impl<'src> ParseRule<'src> {
    fn new(
        prefix: Option<ParseFn<'src>>,
        infix: Option<ParseFn<'src>>,
        precedence: Precedence,
    ) -> ParseRule<'src> {
        ParseRule {
            prefix,
            infix,
            precedence,
        }
    }
}

// Parse code to output OpCode to chunk
pub struct Parser<'src> {
    pub chunk: Chunk,
    current: Token<'src>,
    previous: Token<'src>,
    scanner: Scanner<'src>,
    rules: HashMap<TokenType, ParseRule<'src>>,
    had_error: bool,
    panic_mode: bool,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Parser<'src> {
        let mut rule_map = HashMap::new();
        rule_map.insert(
            TokenType::LeftParen,
            ParseRule::new(Some(Parser::grouping), None, Precedence::None),
        );
        rule_map.insert(
            TokenType::RightParen,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::LeftBrace,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::RightBrace,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Comma,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(TokenType::Dot, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(
            TokenType::Minus,
            ParseRule::new(Some(Parser::unary), Some(Parser::binary), Precedence::Term),
        );
        rule_map.insert(
            TokenType::Plus,
            ParseRule::new(None, Some(Parser::binary), Precedence::Term),
        );
        rule_map.insert(
            TokenType::Semicolon,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Slash,
            ParseRule::new(None, Some(Parser::binary), Precedence::Factor),
        );
        rule_map.insert(
            TokenType::Star,
            ParseRule::new(None, Some(Parser::binary), Precedence::Factor),
        );
        rule_map.insert(
            TokenType::Bang,
            ParseRule::new(Some(Parser::unary), None, Precedence::None),
        );
        rule_map.insert(
            TokenType::BangEqual,
            ParseRule::new(None, Some(Parser::binary), Precedence::Equality),
        );
        rule_map.insert(
            TokenType::Equal,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::EqualEqual,
            ParseRule::new(None, Some(Parser::binary), Precedence::Equality),
        );
        rule_map.insert(
            TokenType::Greater,
            ParseRule::new(None, Some(Parser::binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::GreaterEqual,
            ParseRule::new(None, Some(Parser::binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::Less,
            ParseRule::new(None, Some(Parser::binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::LessEqual,
            ParseRule::new(None, Some(Parser::binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::Identifier,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::String,
            ParseRule::new(Some(Parser::string), None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Number,
            ParseRule::new(Some(Parser::number), None, Precedence::None),
        );
        rule_map.insert(TokenType::And, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(
            TokenType::Class,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Else,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::False,
            ParseRule::new(Some(Parser::literal), None, Precedence::None),
        );
        rule_map.insert(TokenType::For, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(TokenType::Fun, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(TokenType::If, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(
            TokenType::Nil,
            ParseRule::new(Some(Parser::literal), None, Precedence::None),
        );
        rule_map.insert(TokenType::Or, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(
            TokenType::Print,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Return,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Super,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::This,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::True,
            ParseRule::new(Some(Parser::literal), None, Precedence::None),
        );
        rule_map.insert(TokenType::Var, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(
            TokenType::While,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Error,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(TokenType::Eof, ParseRule::new(None, None, Precedence::None));

        let dummy_token = Token::new(TokenType::Eof, 0, "");
        let dummy_token2 = Token::new(TokenType::Eof, 0, "");
        Parser {
            chunk: Chunk::new(),
            current: dummy_token,
            previous: dummy_token2,
            scanner: Scanner::new(src),
            rules: rule_map,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn compile(&mut self) -> bool {
        self.advance();
        self.expression(); // parse a single expression
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();
        !self.had_error
    }

    fn advance(&mut self) {
        self.previous = self.current;

        loop {
            self.current = self.scanner.scan_token();
            if self.current.token_type != TokenType::Error {
                break;
            };

            self.error_at_current(self.current.lexeme);
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.current.token_type == token_type {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn emit_byte(&mut self, byte: OpCode) {
        self.chunk.write(byte, self.previous.line);
    }

    fn emit_bytes(&mut self, byte1: OpCode, byte2: OpCode) {
        self.chunk.write(byte1, self.previous.line);
        self.chunk.write(byte2, self.previous.line);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let idx = self.chunk.add_constant(value);
        match u8::try_from(idx) {
            Ok(idx) => idx,
            Err(_) => {
                self.error("Too many constants in one chunk.");
                0
            }
        }
    }

    fn emit_constant(&mut self, val: Value) {
        let constant_idx = self.make_constant(val);
        self.emit_byte(OpCode::Constant(constant_idx));
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if cfg!(debug_assetions) && !self.had_error {
            crate::debug::disassemble_chunk(&self.chunk, "code");
        }
    }

    fn binary(&mut self) {
        let operator_type = self.previous.token_type;
        // let rule = self.get_rule(operator_type);
        self.parse_precedence(self.get_rule(operator_type).precedence.next());

        match operator_type {
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Not),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenType::Greater => self.emit_byte(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Greater, OpCode::Not),
            TokenType::Less => self.emit_byte(OpCode::Less),
            TokenType::LessEqual => self.emit_bytes(OpCode::Less, OpCode::Not),
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => {} // Unreachable.
        }
    }

    fn literal(&mut self) {
        match self.previous.token_type {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            TokenType::True => self.emit_byte(OpCode::True),
            _ => {} // Unreachable.
        }
    }

    fn grouping(&mut self) {
        // i.e. "(", grouping has no meaning for backend
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self) {
        self.emit_constant(Value::Number(
            self.previous
                .lexeme
                .parse()
                .expect("Cannot convert str to f64"),
        ));
    }

    fn string(&mut self) {
        let key = &self.previous.lexeme[1..self.previous.lexeme.len() - 1];
        let idx = self.chunk.interner.intern(key);
        self.emit_constant(Value::StringObj(idx));
    }

    fn unary(&mut self) {
        let operator_type = self.previous.token_type;

        // Compile the operand.
        self.parse_precedence(Precedence::Unary); // permit nested unary expressions

        // Emit the operator instruction.
        match operator_type {
            // operator_type is the previous token, e.g. "-" in "-50"
            TokenType::Bang => self.emit_byte(OpCode::Not),
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            _ => {} // Unreachable.
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        // read the next token and look up the corresponding ParseRule
        self.advance();
        let prefix_rule = self.get_rule(self.previous.token_type).prefix;
        match prefix_rule {
            Some(r) => r(self),
            None => {
                self.error("Expect expression.");
                return;
            }
        }

        while precedence <= self.get_rule(self.current.token_type).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.previous.token_type).infix;
            match infix_rule {
                Some(r) => r(self),
                None => {
                    self.error("Infix rule not found.");
                    return;
                }
            }
        }
    }

    fn get_rule(&self, token_type: TokenType) -> &ParseRule<'src> {
        return self
            .rules
            .get(&token_type)
            .expect("<TokenType, ParseRule> pair not found.");
    }

    fn expression(&mut self) {
        // parse the lowest precedence level,
        // which subsumes all of the higher-precedence expressions too
        self.parse_precedence(Precedence::Assignment);
    }

    fn error_at(&mut self, token: Token, message: &str) {
        // while panic mode, suppress any other detected errors
        if self.panic_mode {
            return;
        };
        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        if token.token_type == TokenType::Eof {
            eprint!(" at end");
        } else if token.token_type == TokenType::Error {
            // Nothing.
        } else {
            eprint!(" at {}'", token.lexeme);
        }

        eprintln!(": {}\n", message);
        self.had_error = true;
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.previous, message);
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current, message);
    }
}
