use crate::{
    chunk::OpCode,
    function::Function,
    interner::Interner,
    scanner::{Scanner, Token, TokenType},
    value::Value,
};
use std::{collections::HashMap, convert::TryFrom};

pub const USIZE_COUNT: usize = u8::MAX as usize + 1;

#[derive(Debug, PartialEq, PartialOrd)]
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

pub type ParseFn<'src> = fn(&mut Parser<'src>, bool) -> ();

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

pub struct Local<'src> {
    name: Token<'src>,
    depth: i32,
}

impl<'src> Local<'src> {
    pub fn new(name: Token<'src>, depth: i32) -> Local<'src> {
        Local { name, depth }
    }
}

enum FunctionType {
    TypeFunction, // function code
    TypeScript,   // top-level code
}

pub struct Compiler<'src> {
    // the compiler creates function objects during compilation. Then, at runtime, they are simply invoked
    pub function: Function,
    f_type: FunctionType,

    locals: Vec<Local<'src>>, // tracks how many locals are in scope
    scope_depth: i32,         // # of blocks surrounding the current bit of code
}

impl<'src> Compiler<'src> {
    pub fn new() -> Compiler<'src> {
        // Setstack slot zero for the VM’s own internal use
        let mut locals = Vec::with_capacity(USIZE_COUNT);
        let dummy_token = Local::new(Token::new(TokenType::Eof, 0, ""), 0);
        locals.push(dummy_token);

        Compiler {
            function: Function::new(),
            f_type: FunctionType::TypeScript,
            locals,
            scope_depth: 0,
        }
    }
}
// Parse code to output OpCode to chunk
pub struct Parser<'src> {
    pub compiler: Compiler<'src>,
    interner: &'src mut Interner,
    current: Token<'src>,
    previous: Token<'src>,
    scanner: Scanner<'src>,
    rules: HashMap<TokenType, ParseRule<'src>>,
    had_error: bool,
    panic_mode: bool,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str, interner: &'src mut Interner) -> Parser<'src> {
        let mut rule_map = HashMap::new();
        rule_map.insert(
            TokenType::LeftParen,
            ParseRule::new(Some(Parser::rule_grouping), None, Precedence::None),
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
            ParseRule::new(
                Some(Parser::rule_unary),
                Some(Parser::rule_binary),
                Precedence::Term,
            ),
        );
        rule_map.insert(
            TokenType::Plus,
            ParseRule::new(None, Some(Parser::rule_binary), Precedence::Term),
        );
        rule_map.insert(
            TokenType::Semicolon,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Slash,
            ParseRule::new(None, Some(Parser::rule_binary), Precedence::Factor),
        );
        rule_map.insert(
            TokenType::Star,
            ParseRule::new(None, Some(Parser::rule_binary), Precedence::Factor),
        );
        rule_map.insert(
            TokenType::Bang,
            ParseRule::new(Some(Parser::rule_unary), None, Precedence::None),
        );
        rule_map.insert(
            TokenType::BangEqual,
            ParseRule::new(None, Some(Parser::rule_binary), Precedence::Equality),
        );
        rule_map.insert(
            TokenType::Equal,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::EqualEqual,
            ParseRule::new(None, Some(Parser::rule_binary), Precedence::Equality),
        );
        rule_map.insert(
            TokenType::Greater,
            ParseRule::new(None, Some(Parser::rule_binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::GreaterEqual,
            ParseRule::new(None, Some(Parser::rule_binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::Less,
            ParseRule::new(None, Some(Parser::rule_binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::LessEqual,
            ParseRule::new(None, Some(Parser::rule_binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::Identifier,
            ParseRule::new(Some(Parser::rule_variable), None, Precedence::None),
        );
        rule_map.insert(
            TokenType::String,
            ParseRule::new(Some(Parser::rule_string), None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Number,
            ParseRule::new(Some(Parser::rule_number), None, Precedence::None),
        );
        rule_map.insert(
            TokenType::And,
            ParseRule::new(None, Some(Parser::rule_and), Precedence::And),
        );
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
            ParseRule::new(Some(Parser::rule_literal), None, Precedence::None),
        );
        rule_map.insert(TokenType::For, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(TokenType::Fun, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(TokenType::If, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(
            TokenType::Nil,
            ParseRule::new(Some(Parser::rule_literal), None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Or,
            ParseRule::new(None, Some(Parser::rule_or), Precedence::Or),
        );
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
            ParseRule::new(Some(Parser::rule_literal), None, Precedence::None),
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
            compiler: Compiler::new(),
            interner,
            current: dummy_token,
            previous: dummy_token2,
            scanner: Scanner::new(src),
            rules: rule_map,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn compile(mut self) -> Option<Function> {
        self.advance();
        while !self.equal(TokenType::Eof) {
            self.declaration();
        }
        let had_error = self.had_error;
        let f = self.end_compiler();
        if had_error {
            None
        } else {
            Some(f)
        }
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

    fn check(&self, token_type: TokenType) -> bool {
        self.current.token_type == token_type
    }

    fn equal(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn emit_byte(&mut self, byte: OpCode) {
        self.compiler.function.chunk.write(byte, self.previous.line);
    }

    fn emit_bytes(&mut self, byte1: OpCode, byte2: OpCode) {
        self.compiler
            .function
            .chunk
            .write(byte1, self.previous.line);
        self.compiler
            .function
            .chunk
            .write(byte2, self.previous.line);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.compiler.function.chunk.code.len() - loop_start;
        if offset > USIZE_COUNT {
            self.error("Loop body too large.");
        }

        self.emit_byte(OpCode::Loop(offset));
    }

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.emit_byte(instruction);
        self.compiler.function.chunk.code.len() - 1
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    // The jump OpCode at chunk.code[offset] will jump to the
    // current location (i.e. chunk.code[len-1])
    fn patch_jump(&mut self, offset: usize) {
        // -1 because offset is 0-based index.
        let jump = self.compiler.function.chunk.code.len() - 1 - offset;

        if jump > USIZE_COUNT {
            self.error("Too much code to jump over.");
        }

        // Replaces the operand at the given location with the calculated jump offset
        match self.compiler.function.chunk.code[offset] {
            OpCode::Jump(ref mut o) | OpCode::JumpIfFalse(ref mut o) => *o = jump,
            _ => {
                self.error("Operand is not Jump!");
                println!("{:?}", self.compiler.function.chunk.code)
            }
        }
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let idx = self.compiler.function.chunk.add_constant(value);
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

    fn end_compiler(mut self) -> Function {
        self.emit_return();
        let f = self.compiler.function;
        #[cfg(feature = "debug_trace_execution")]
        if !self.had_error {
            match f.name {
                Some(name_idx) => {
                    crate::debug::disassemble_chunk(
                        &f.chunk,
                        self.interner.lookup(name_idx),
                        &self.interner,
                    );
                }
                None => {
                    crate::debug::disassemble_chunk(&f.chunk, "<script>", &self.interner);
                }
            }
        }
        f
    }

    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;

        while !self.compiler.locals.is_empty()
            && self.compiler.locals[self.compiler.locals.len() - 1].depth
                > self.compiler.scope_depth
        {
            // Remove the var from the stack
            self.emit_byte(OpCode::Pop);
            // Remove the var from local array
            self.compiler.locals.pop();
        }
    }

    fn rule_binary(&mut self, can_assign: bool) {
        let operator_type = self.previous.token_type;
        // let rule = self.get_rule(operator_type);
        self.parse_precedence(self.get_rule(operator_type).precedence.next());

        match operator_type {
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Not),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenType::Greater => self.emit_byte(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit_byte(OpCode::Less),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Not),
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => {} // Unreachable.
        }
    }

    fn rule_literal(&mut self, can_assign: bool) {
        match self.previous.token_type {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            TokenType::True => self.emit_byte(OpCode::True),
            _ => {} // Unreachable.
        }
    }

    fn rule_grouping(&mut self, can_assign: bool) {
        // i.e. "(", grouping has no meaning for backend
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn rule_number(&mut self, can_assign: bool) {
        self.emit_constant(Value::Number(
            self.previous
                .lexeme
                .parse()
                .expect("Cannot convert str to f64"),
        ));
    }

    fn rule_or(&mut self, can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse(0xff));
        let end_jump = self.emit_jump(OpCode::Jump(0xff));

        // if LHS is falsey, skip `end_jump`, in order to evaluate RHS expression
        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop);

        self.parse_precedence(Precedence::Or);
        // if LHS is truthy, `end_jump` will be run, skipping RHS expression
        self.patch_jump(end_jump);
    }

    fn rule_string(&mut self, can_assign: bool) {
        let key = &self.previous.lexeme[1..self.previous.lexeme.len() - 1];
        let idx = self.interner.intern(key);
        self.emit_constant(Value::StringObj(idx));
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let get_op;
        let set_op;
        if let Some(arg) = self.resolve_local(name) {
            let idx = arg as u8;
            get_op = OpCode::GetLocal(idx);
            set_op = OpCode::SetLocal(idx);
        } else {
            let idx = self.identifier_constant(name);
            get_op = OpCode::GetGlobal(idx);
            set_op = OpCode::SetGlobal(idx);
        }
        // look for an equals sign after the identifier
        if can_assign && self.equal(TokenType::Equal) {
            // If we find one, instead of emitting code for a variable access,
            // we compile the assigned value and then emit an assignment instruction.
            self.expression();
            self.emit_byte(set_op);
        } else {
            self.emit_byte(get_op);
        }
    }

    fn rule_variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous, can_assign);
    }

    fn rule_unary(&mut self, can_assign: bool) {
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

        // we look up a prefix parser for the current token.
        // The first token is always going to belong to some kind of prefix expression, by definition.
        #[cfg(feature = "debug_trace_execution")]
        println!("precedence {:?} ", precedence);
        let prefix_rule = self.get_rule(self.previous.token_type).prefix;
        #[cfg(feature = "debug_trace_execution")]
        println!("prefix_rule of {:?} ", self.previous.token_type);
        let can_assign = precedence <= Precedence::Assignment;
        match prefix_rule {
            Some(r) => r(self, can_assign),
            None => {
                self.error("Expect expression.");
                return;
            }
        }
        // After parsing that, which may consume more tokens, the prefix expression is done.

        // Now we look for an infix parser for the next token.
        // If we find one, it means the prefix expression we already compiled might be an operand for it.
        // But only if the call to `parsePrecedence()` has a precedence that is low enough to permit that infix operator.
        while precedence <= self.get_rule(self.current.token_type).precedence {
            self.advance();
            // we consume the operator and hand off control to the infix parser we found.
            // It consumes whatever other tokens it needs and returns back to `parsePrecedence()` (this function).
            let infix_rule = self.get_rule(self.previous.token_type).infix;
            #[cfg(feature = "debug_trace_execution")]
            println!("infix_rule of {:?} ", self.previous.token_type);
            match infix_rule {
                // Then we loop back around and see if the next token is also a valid infix operator
                // that can take the entire preceding expression as its operand.
                Some(r) => r(self, can_assign),
                None => {
                    self.error("Infix rule not found.");
                    return;
                }
            }
        }
        // If the next token is too low precedence, or isn’t an infix operator at all, we’re done.
        // i.e., we’ve parsed as much expression as we can.

        if can_assign && self.equal(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn identifier_constant(&mut self, name: Token) -> u8 {
        // Global variables are looked up by name at runtime.
        // Store the string in the constant table (instead of bytecode "stream") for instructions
        let identifier = self.interner.intern(name.lexeme);
        self.make_constant(Value::Identifier(identifier))
    }

    fn identifiers_equal(&self, a: &Token, b: &Token) -> bool {
        a.lexeme == b.lexeme
    }

    fn resolve_local(&mut self, name: Token) -> Option<usize> {
        for (i, local) in self.compiler.locals.iter().enumerate().rev() {
            if self.identifiers_equal(&name, &local.name) {
                if local.depth == -1 {
                    self.error("Cannot read local variable in its own initializer.");
                }
                return Some(i);
            }
        }
        None
    }

    // Initializes the next available Local
    fn add_local(&mut self, name: Token<'src>) {
        if self.compiler.locals.len() == USIZE_COUNT {
            self.error("Too many local variables in function.");
            return;
        }

        let local = Local::new(name, -1);
        self.compiler.locals.push(local);
    }

    fn declare_variable(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }

        let name = self.previous;
        // Check for redeclaring
        for local in self.compiler.locals.iter().rev() {
            // -1 = uninitialized
            if local.depth != -1 && local.depth < self.compiler.scope_depth {
                break;
            }

            if self.identifiers_equal(&name, &local.name) {
                self.error("Already a variable with this name in this scope.");
                break;
            }
        }

        self.add_local(name);
    }

    fn parse_variable(&mut self, err_msg: &str) -> u8 {
        self.consume(TokenType::Identifier, err_msg);

        self.declare_variable();
        if self.compiler.scope_depth > 0 {
            return 0;
        }

        self.identifier_constant(self.previous)
    }

    fn mark_initialized(&mut self) {
        let last = self.compiler.locals.last_mut().unwrap();
        last.depth = self.compiler.scope_depth;
    }

    fn define_variable(&mut self, global: u8) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_byte(OpCode::DefineGlobal(global));
    }

    fn rule_and(&mut self, can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse(0xff));

        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
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

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.equal(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    // Semantically, an expression statement evaluates the expression and discards the result.
    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");

        // Initializer clause
        if self.equal(TokenType::Semicolon) {
            // No initializer.
        } else if self.equal(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.compiler.function.chunk.code.len();

        // Condition clause (Optional)
        let mut exit_jump = None;
        if !self.equal(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            // Jump out of the loop if the condition is false.
            exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse(0xff)));
            self.emit_byte(OpCode::Pop); // Condition.
        }

        // Increment clause (Optional)
        if !self.equal(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump(0xff));
            let increment_start = self.compiler.function.chunk.code.len();
            self.expression();
            self.emit_byte(OpCode::Pop); // discard increment expression's value
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        // If there is a condition clause, patch jumpand pop condition value.
        if let Some(offset) = exit_jump {
            self.patch_jump(offset);
            self.emit_byte(OpCode::Pop); // Condition.
        }

        self.end_scope();
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse(0xff));
        self.emit_byte(OpCode::Pop); // pop the condition value, each statement is required to have zero stack effect
        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump(0xff));
        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop); // pop the condition value, each statement is required to have zero stack effect

        if self.equal(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print);
    }

    fn while_statement(&mut self) {
        let loop_start = self.compiler.function.chunk.code.len(); // start location of loop
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse(0xff));
        self.emit_byte(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop);
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.token_type != TokenType::Eof {
            if self.previous.token_type == TokenType::Semicolon {
                return;
            }
            match self.current.token_type {
                // TokenType::Class => ,
                // TokenType::Fun => ,
                // TokenType::Var => ,
                // TokenType::For => ,
                // TokenType::If => ,
                // TokenType::While => ,
                // TokenType::Print => ,
                TokenType::Return => {
                    return;
                }
                _ => (), // Do nothing.
            }
            self.advance();
        }
    }

    fn declaration(&mut self) {
        if self.equal(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.equal(TokenType::Print) {
            self.print_statement();
        } else if self.equal(TokenType::For) {
            self.for_statement();
        } else if self.equal(TokenType::If) {
            self.if_statement();
        } else if self.equal(TokenType::While) {
            self.while_statement();
        } else if self.equal(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
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
