pub struct Scanner<'src> {
    start: usize, // beginning of the current lexeme being scanned
    current: usize,
    src: &'src str,
    line: usize,
}
impl Scanner<'_> {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            start: 0,
            current: 0,
            src: source,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        self.error_token("Unexpected character.")
    }

    fn is_at_end(&self) -> bool {
        self.current == self.src.len()
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            // start: self.start,
            // length: self.current - self.start,
            line: self.line,
            lexeme: &self.src[self.start..self.current - self.start],
        }
    }

    fn error_token(&self, msg: &'static str) -> Token {
        // TODO: why need static lifetime?
        Token {
            token_type: TokenType::Error,
            // token.start = message;
            // token.length = (int)strlen(message);
            line: self.line,
            lexeme: msg,
        }
    }
}

pub struct Token<'src> {
    pub token_type: TokenType,
    // start: usize,
    // length: usize,
    pub line: usize,
    pub lexeme: &'src str,
}

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
