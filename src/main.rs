use crate::lexer::{Token, TokenType};

mod lexer;

const STACK_SIZE: usize = 1000;

struct VM {
    program: Vec<Token>,
    program_counter: usize,
    current_instruction: Token,

    loops: Vec<(usize, i32)>,
    stack_pointer: usize,
    stack: [i32; STACK_SIZE],
    stored: (Vec<(usize, i32)>, usize, [i32; STACK_SIZE])
}

impl VM {
    fn new(program: Vec<Token>) -> Self {
        Self {
            program,
            program_counter: 0,
            current_instruction: Token {
                token_type: TokenType::NullForParser,
                value: "".to_string(),
                x: 0,
                y: 0,
            },

            loops: vec![],
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            stored: (vec![], 0, [0; STACK_SIZE])
        }
    }

    fn error(&self, msg: &str) -> ! {
        panic!("{}", format!("at index {} '{}'", self.program_counter, msg))
    }
    fn store_state(&mut self){
        self.stored = (self.loops.clone(), self.stack_pointer, self.stack.clone())
    }
    fn load_state(&mut self){
        self.loops = self.stored.0.clone();
        self.stack_pointer = self.stored.1;
        self.stack = self.stored.2.clone();
    }

    fn next_instruction(&mut self) -> bool {
        self.program_counter += 1;
        if self.program_counter >= self.program.len() {
            false
        } else {
            self.current_instruction = self.program[self.program_counter].clone();
            true
        }
    }
    fn next_instruction_expect(&mut self, expected_type: TokenType, msg: &str) {
        if !self.next_instruction() || self.current_instruction.token_type != expected_type {
            self.error(msg)
        }
    }

    fn increment(&mut self) {
        self.stack[self.stack_pointer] += 1;
    }

    fn decrement(&mut self) {
        self.stack[self.stack_pointer] -= 1;
    }

    fn sp_left(&mut self) {
        if self.stack_pointer < 1 {
            self.error("invalid move, going to negatives")
        }
        self.stack_pointer -= 1;
    }
    fn sp_right(&mut self) {
        self.stack_pointer += 1;
        if self.stack_pointer >= self.stack.len() {
            self.error("move causes out of bounds, increase stack size")
        }
    }

    fn print_current(&self) {
        println!("{}", self.stack[self.stack_pointer]);
    }
    fn start_loop_handler(&mut self) {
        // println!(">> {} >> {:?}", self.program_counter, self.program[self.program_counter]);
        self.next_instruction();
        if self.current_instruction.token_type != TokenType::Integer {
            self.error("expected number of loop condition '[1]' while current_cell is not 0")
        }
        let cond_value = self.current_instruction.value.parse::<i32>().unwrap();
        if self.stack[self.stack_pointer] == cond_value {
            while self.current_instruction.token_type != TokenType::BracketClose {
                self.next_instruction();
            }
        } else {
            self.loops.push((self.program_counter, cond_value));
        }
    }
    fn end_loop_handler(&mut self) {
        if self.loops.len() == 0 {
            self.error("unexpected end loop, start not found");
        }
        let (loc, cond) = self.loops.pop().unwrap();
        if self.stack[self.stack_pointer] != cond {
            self.program_counter = loc;
            self.loops.push((loc, cond));
        }
    }
    fn condition_handler(&mut self){
        // (>>, "==", <<; ++)
        self.next_instruction();

        self.store_state();
        while self.current_instruction.token_type != TokenType::String {
            if !self.single(){
                self.error("unexpected end of program for lhs condition")
            }
            self.next_instruction();
        }
        let lhs = self.stack[self.stack_pointer];

        let condition = self.current_instruction.value.clone();
        self.next_instruction();

        self.load_state();
        self.current_instruction = self.program[self.program_counter].clone();
        while self.current_instruction.token_type != TokenType::SemiColon {
            if !self.single(){
                self.error("unexpected end of program for rhs condition")
            }
            self.next_instruction();
        }
        self.next_instruction();

        let rhs = self.stack[self.stack_pointer];
        self.load_state();

        let mut failed_condition = false;
        match &*condition {
            "==" => failed_condition = lhs != rhs,
            ">=" => failed_condition = !(lhs >= rhs),
            "<=" => failed_condition = !(lhs <= rhs),
            "!=" => failed_condition = lhs == rhs,
            ">" => failed_condition = lhs < rhs,
            "<" => failed_condition = lhs > rhs,
             _ => self.error("unknown conditional")
        }

        if failed_condition {
            while self.current_instruction.token_type != TokenType::ParenthesisClose {
                self.next_instruction();
                if self.current_instruction.token_type == TokenType::EndOfFile {
                    self.error("unclosed conditional, false, program ended")
                }
            }
        } else {
            while self.current_instruction.token_type != TokenType::ParenthesisClose {
                if !self.single(){
                    self.error("unclosed conditional, true, program ended")
                }
                self.next_instruction();
            }
        }
    }
    fn call_handler(&mut self){

    }
    fn single(&mut self) -> bool {
        match self.current_instruction.token_type {
            TokenType::MovLeftOperation => self.sp_left(),
            TokenType::MovRightOperation => self.sp_right(),
            TokenType::AddOperation => self.increment(),
            TokenType::SubOperation => self.decrement(),
            TokenType::PrintOut => self.print_current(),
            TokenType::EndOfFile => return false,
            TokenType::BracketOpen => self.start_loop_handler(),
            TokenType::BracketClose => self.end_loop_handler(),
            TokenType::ParenthesisOpen => self.condition_handler(),
            TokenType::ParenthesisClose => self.error("unexpected close of condition"),
            TokenType::FunctionCall => self.call_handler(),
            _ => self.error(&*format!("unknown instruction '{:?}' ", self.current_instruction)),
        }
        true
    }
    fn run(&mut self) {
        self.current_instruction = self.program[self.program_counter].clone();
        loop {
            if !self.single() { break; }
            self.next_instruction();
        }
    }


    fn run_from_str(program: &str) {
        let program = lexer::Lexer::lex_string(program.to_string());
        VM::new(program).run();
    }
    fn run_from_file(program_file: &str){
        match std::fs::read_to_string(program_file) {
            Ok(file_contents) => {
                Self::run_from_str(&*file_contents)
            }
            Err(err) => panic!("{}", err)
        }
    }
}


fn main() {
    VM::run_from_file("main.bf");
}
