use crate::scanner::{Scanner, TokenType};

pub fn compile(source: &str) {
    let mut s = Scanner::new(source);
    let mut line = usize::MAX;
    loop {
        let token = s.scan_token();
        if token.line != line {
            print!("{} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }
        println!("'{}' {:?}", token.lexeme, token.token_type);

        if let TokenType::Eof = token.token_type {
            break;
        }
    }
}
