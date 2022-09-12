#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenType {
    // types
    String,
    Integer,
    Boolean,
    FloatingPoint,

    Identifier,

    AddOperation,
    SubOperation,
    MovRightOperation,
    MovLeftOperation,

    // symbols
    PrintOut,
    Colon,
    SemiColon,
    EndOfFile,
    FunctionCall,
    BracketOpen,
    BracketClose,
    CurlyBracketOpen,
    CurlyBracketClose,
    ParenthesisOpen,
    ParenthesisClose,
    SeparatorComma,
    ReturnTypeArrow,

    NullForParser,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub x: u32,
    pub y: u32,
}

impl Token {
    pub fn new(token_type: TokenType, value: String) -> Self {
        Self {
            token_type,
            value,
            x: 0,
            y: 0,
        }
    }
    pub fn is_string(&self) -> bool {
        self.token_type == TokenType::String
    }
    pub fn is_integer(&self) -> bool {
        self.token_type == TokenType::Integer
    }
    pub fn is_float(&self) -> bool {
        self.token_type == TokenType::FloatingPoint
    }
    pub fn is_bool(&self) -> bool {
        self.token_type == TokenType::Boolean
    }
    pub fn is_data_type(&self) -> bool {
        self.is_float() || self.is_bool() || self.is_string() || self.is_integer()
    }
    pub fn true_value(&self) -> String {
        if self.is_string() {
            format!("\"{}\"", self.value)
        } else {
            self.value.clone()
        }
    }
}

pub struct Lexer {
    text_to_lex: Vec<char>,
    index: i32,
    run: bool,
    current_char: char,
    x: i32,
    y: i32,
    tok_start_x: i32,
    tok_start_y: i32,
    current_tokens: Vec<Token>,
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            text_to_lex: vec![],
            index: -1,
            run: true,
            current_char: ' ',
            x: 0,
            y: 1,
            tok_start_x: 0,
            tok_start_y: 0,
            current_tokens: vec![],
        }
    }
    pub fn lex_string(text: String) -> Vec<Token> {
        Lexer::new().lex_text(text)
    }
    fn error(&self, msg: &str) -> !{
        panic!("at line {} char {} '{}'", self.y, self.x, msg)
    }
    fn pos_starter(&mut self) {
        self.tok_start_x = self.x;
        self.tok_start_y = self.y;
    }
    fn next_char(&mut self) -> bool {
        self.index += 1;
        if self.index >= self.text_to_lex.len() as i32 {
            false
        } else {
            self.current_char = self.text_to_lex[self.index as usize];
            if self.current_char == '\n' {
                self.y += 1;
                self.x = 0;
            } else {
                self.x += 1;
            }
            true
        }
    }
    fn get_next_char_ignore_space(&self) -> Option<char> {
        let mut ind = self.index as usize;
        ind += 1;
        let mut character = self.text_to_lex[ind];
        while character == ' ' {
            ind += 1;
            character = self.text_to_lex[ind];
        }
        Some(character)
    }
    fn get_char(&self, ahead: i32) -> Option<char> {
        Some(self.text_to_lex[(self.index + ahead) as usize])
    }
    fn get_next_char(&self) -> Option<char> {
        self.get_char(1)
    }
    fn add_base(&mut self, tok_type: TokenType, value: String) {
        let mut tok = Token::new(tok_type, value);
        tok.x = self.tok_start_x as u32;
        tok.y = self.tok_start_y as u32;
        self.current_tokens.push(tok);
    }
    fn add_special(&mut self, tok_type: TokenType) {
        self.add_base(tok_type, "".to_string());
    }
    fn add_special_bare(&mut self, tok_type: TokenType, value: String) {
        self.add_base(tok_type, value);
    }
    fn add_string(&mut self, value: String) {
        self.add_base(TokenType::String, value);
    }
    fn add_integer(&mut self, value: String) {
        self.add_base(TokenType::Integer, value);
    }
    fn add_float(&mut self, value: String) {
        self.add_base(TokenType::FloatingPoint, value);
    }
    fn unknown_length(&mut self, value: String) {
        match &*value {
            _ => self.add_base(TokenType::Identifier, value),
        }
    }
    fn lex(&mut self) -> Vec<Token> {
        /*
        hierarchy

        comment
        string

        special
        identifier
        number

        */
        let num: Vec<char> = "0123456789".chars().collect();
        let allowed_for_id: Vec<char> =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_1234567890"
                .chars()
                .collect();

        let mut unknown_length = "".to_string();
        let mut unknown_length_being_used = false;
        let mut str_on = false;
        let mut comment_on = false;
        let mut id_on = false;
        let mut num_on = false;

        let mut int = false;
        let mut float = false;

        while self.run {
            if !self.next_char() {
                self.run = false;
                break;
            }

            if self.current_char == '"' && !comment_on {
                if str_on {
                    str_on = false;
                    unknown_length_being_used = false;
                    self.add_string(unknown_length.clone());
                    unknown_length = "".to_string();
                } else {
                    self.pos_starter();
                    if unknown_length_being_used {
                        if num_on {
                            if int {
                                self.add_integer(unknown_length.clone());
                                int = false;
                            } else if float {
                                self.add_float(unknown_length.clone());
                                float = false;
                            } else {
                                unreachable!()
                            }
                            num_on = false;
                        } else if id_on {
                            self.unknown_length(unknown_length);
                            id_on = false;
                        } else {
                            self.error("shit something went wrong!");
                        }

                        unknown_length_being_used = false;
                        unknown_length = "".to_string();
                    }

                    str_on = true;
                    unknown_length_being_used = true;
                }
            } else if str_on || comment_on {
                if self.current_char == '\n' {
                    if str_on {
                        self.error("unclosed string at line");
                    } else {
                        comment_on = false;
                        unknown_length_being_used = false;
                        unknown_length = "".to_string();
                    }
                } else if self.current_char == '\\' && str_on {
                    self.next_char();
                    match self.current_char {
                        'n' => {
                            unknown_length += "\n";
                        }
                        '\\' => {
                            unknown_length += "\\";
                        }
                        't' => {
                            unknown_length += "\t";
                        }
                        ' ' => {}
                        _ => unimplemented!(),
                    }
                } else {
                    unknown_length += &self.current_char.to_string();
                }
            } else if num.contains(&self.current_char) && !id_on {
                if num_on {
                    unknown_length += &self.current_char.to_string();
                } else {
                    unknown_length += &self.current_char.to_string();
                    self.pos_starter();
                    num_on = true;
                    int = true;

                    if unknown_length_being_used {
                        self.error("shit something went wrong");
                    }

                    unknown_length_being_used = true;
                }
            } else if self.current_char == '.'
                && num_on
                && num.contains(&self.get_next_char_ignore_space().expect(&format!(
                "expected char! at line {} char {}",
                self.y, self.x
            )))
            {
                int = false;
                float = true;
                unknown_length += ".";
            } else if allowed_for_id.contains(&self.current_char) {
                // to avoid errors
                if num_on {
                    if int {
                        self.add_integer(unknown_length.clone());
                        int = false;
                    } else if float {
                        self.add_float(unknown_length.clone());
                        float = false;
                    } else {
                        self.error("what? how?!");
                    }
                    num_on = false;
                    unknown_length_being_used = false;
                    unknown_length = "".to_string();
                }

                if id_on {
                    unknown_length += &self.current_char.to_string();
                } else {
                    unknown_length += &self.current_char.to_string();
                    self.pos_starter();
                    id_on = true;

                    if unknown_length_being_used {
                        self.error("shit something went wrong!");
                    }

                    unknown_length_being_used = true;
                }
            } else {
                if num_on {
                    if int {
                        self.add_integer(unknown_length.clone());
                        int = false;
                    } else if float {
                        self.add_float(unknown_length.clone());
                        float = false;
                    } else {
                        self.error("what? how?!");
                    }
                    num_on = false;
                    unknown_length_being_used = false;
                    unknown_length = "".to_string();
                } else if id_on {
                    self.unknown_length(unknown_length);
                    id_on = false;
                    unknown_length_being_used = false;
                    unknown_length = "".to_string();
                }

                self.pos_starter();
                match self.current_char {
                    '+' => self.add_special(TokenType::AddOperation),
                    '-' => self.add_special(TokenType::SubOperation),
                    '>' => self.add_special(TokenType::MovRightOperation),
                    '<' => self.add_special(TokenType::MovLeftOperation),
                    '.' => self.add_special(TokenType::PrintOut),
                    '(' => self.add_special(TokenType::ParenthesisOpen),
                    ')' => self.add_special(TokenType::ParenthesisClose),
                    ',' => self.add_special(TokenType::SeparatorComma),
                    '{' => self.add_special(TokenType::CurlyBracketOpen),
                    '}' => self.add_special(TokenType::CurlyBracketClose),
                    '[' => self.add_special(TokenType::BracketOpen),
                    ']' => self.add_special(TokenType::BracketClose),
                    ':' => self.add_special(TokenType::Colon),
                    ';' => self.add_special(TokenType::SemiColon),
                    '&' => self.add_special(TokenType::FunctionCall),
                    '\n' | ' ' | '\t' => {}
                    '/' => {
                        match self.get_next_char() {
                            Some(char) => {
                                if char != '/'{
                                    self.error(&*format!("not  added -> / <-,", ))
                                }
                                comment_on = true;
                                self.next_char();
                            }
                            None => self.error(&*format!("not added -> / <-,", ))
                        }
                    }
                    _ => {
                        unimplemented!(
                            "not added -> {} <-, at line {} char {}",
                            self.current_char,
                            self.tok_start_y,
                            self.tok_start_x
                        )
                    }
                }
            }
        }
        if num_on {
            if int {
                self.add_integer(unknown_length.clone());
            } else if float {
                self.add_float(unknown_length.clone());
            } else {
                self.error("what? how?!")
            }
        } else if id_on {
            self.unknown_length(unknown_length)
        } else if str_on {
            self.error("unclosed string literal");
        }
        self.add_special(TokenType::EndOfFile);
        self.current_tokens.clone()
    }

    pub fn lex_text(&mut self, text: String) -> Vec<Token> {
        self.text_to_lex = text.chars().collect();
        self.lex()
    }
}
