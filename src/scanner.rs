pub struct Scanner<'src> {
    start: usize, // beginning of the current lexeme being scanned
    current: usize,
    src: &'src str,
    line: usize,
}
impl<'src> Scanner<'src> {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            start: 0,
            current: 0,
            src: source,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token<'src> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();
        if is_alpha(c) {
            return self.identifier();
        };
        if is_digit(c) {
            return self.number();
        };

        match c {
            // note: compare c (u8) with ASCII letters (e.g. b'*')
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b';' => self.make_token(TokenType::Semicolon),
            b',' => self.make_token(TokenType::Comma),
            b'.' => self.make_token(TokenType::Dot),
            b'-' => self.make_token(TokenType::Minus),
            b'+' => self.make_token(TokenType::Plus),
            b'/' => self.make_token(TokenType::Slash),
            b'*' => self.make_token(TokenType::Star),
            b'!' if self.check_next(b'=') => self.make_token(TokenType::BangEqual),
            b'!' => self.make_token(TokenType::Bang),
            b'=' if self.check_next(b'=') => self.make_token(TokenType::EqualEqual),
            b'=' => self.make_token(TokenType::Equal),
            b'<' if self.check_next(b'=') => self.make_token(TokenType::LessEqual),
            b'<' => self.make_token(TokenType::Less),
            b'>' if self.check_next(b'=') => self.make_token(TokenType::GreaterEqual),
            b'>' => self.make_token(TokenType::Greater),
            b'"' => self.string(),
            _ => self.error_token("Unexpected character."),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current == self.src.len()
    }

    fn advance(&mut self) -> u8 {
        let c = self.src.as_bytes()[self.current];
        self.current += 1;
        c
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.src.as_bytes()[self.current]
        }
    }

    fn peek_next(&self) -> u8 {
        if self.current > self.src.len() - 2 {
            b'\0'
        } else {
            self.src.as_bytes()[self.current + 1]
        }
    }

    fn check_next(&mut self, expected: u8) -> bool {
        if self.is_at_end() || self.src.as_bytes()[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token<'src> {
        Token {
            token_type,
            // start: self.start,
            // length: self.current - self.start,
            line: self.line,
            lexeme: &self.src[self.start..self.current],
        }
    }

    fn error_token(&self, msg: &'static str) -> Token<'src> {
        // TODO: why need static lifetime?
        Token {
            token_type: TokenType::Error,
            // token.start = message;
            // token.length = (int)strlen(message);
            line: self.line,
            lexeme: msg,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                b' ' | b'\r' | b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'/' => {
                    if self.peek_next() == b'/' {
                        // A comment goes until the end of the line.
                        while self.peek() != b'\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                    break;
                }
                _ => return,
            }
        }
    }

    fn check_keyword(
        &self,
        start: usize,
        length: usize,
        rest: &str,
        token_type: TokenType,
    ) -> TokenType {
        if self.current - self.start == start + length
            && &self.src[self.start + start..self.start + start + length] == rest
        {
            return token_type;
        }
        TokenType::Identifier
    }

    fn identifier_type(&self) -> TokenType {
        match self.src.as_bytes()[self.start] {
            b'a' => self.check_keyword(1, 2, "nd", TokenType::And),
            b'c' => self.check_keyword(1, 4, "lass", TokenType::Class),
            b'e' => self.check_keyword(1, 3, "lse", TokenType::Else),
            b'f' if self.current - self.start > 1 => {
                // lexeme is more than 2 char
                match self.src.as_bytes()[self.start + 1] {
                    b'a' => self.check_keyword(2, 3, "lse", TokenType::False),
                    b'o' => self.check_keyword(2, 1, "r", TokenType::For),
                    b'u' => self.check_keyword(2, 1, "n", TokenType::Fun),
                    _ => TokenType::Identifier,
                }
            }
            b'i' => self.check_keyword(1, 1, "f", TokenType::If),
            b'n' => self.check_keyword(1, 2, "il", TokenType::Nil),
            b'o' => self.check_keyword(1, 1, "r", TokenType::Or),
            b'p' => self.check_keyword(1, 4, "rint", TokenType::Print),
            b'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
            b's' => self.check_keyword(1, 4, "uper", TokenType::Super),
            b'f' if self.current - self.start > 1 => {
                // lexeme is more than 2 char
                match self.src.as_bytes()[self.start + 1] {
                    b'h' => self.check_keyword(2, 2, "is", TokenType::This),
                    b'r' => self.check_keyword(2, 2, "ue", TokenType::True),
                    _ => TokenType::Identifier,
                }
            }
            b'v' => self.check_keyword(1, 2, "ar", TokenType::Var),
            b'w' => self.check_keyword(1, 4, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn identifier(&mut self) -> Token<'src> {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }
        self.make_token(self.identifier_type())
    }

    fn number(&mut self) -> Token<'src> {
        while is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == b'.' && is_digit(self.peek_next()) {
            // Consume the ".".
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn string(&mut self) -> Token<'src> {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1
            };
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        };

        // The closing quote.
        self.advance();
        self.make_token(TokenType::String)
    }
}

#[derive(Clone, Copy)]
pub struct Token<'src> {
    pub token_type: TokenType,
    // start: usize,
    // length: usize,
    pub line: usize,
    pub lexeme: &'src str,
}

impl<'src> Token<'src> {
    pub fn new(token_type: TokenType, line: usize, lexeme: &'src str) -> Token<'src> {
        Token {
            token_type,
            line,
            lexeme,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    Eof,
}

fn is_alpha(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}
