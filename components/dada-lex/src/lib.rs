extern crate peg;

#[derive(Debug)]
pub enum Token {
    Leaf(String),
    Branch(Delimiter, Vec<Token>),
    Comment,
}

#[derive(Debug)]
pub enum Delimiter {
    Paren,
    CurlyBrace,
}

peg::parser! {
    grammar tokenizer() for str {
        pub rule tokens() -> Vec<Token> = n:token()**__ {
            n
        }

        rule nl() -> Token = ['\n'] { 
            Token::Leaf(format!("\n") )
        }

        rule token() -> Token = comment()
            / nl()
            / ident()
            / string()
            / curly_brace()
            / comma()
        
        rule paren() -> Token = ['('] _ t:token()**__ _ [')'] {
            Token::Branch(Delimiter::Paren, t)
        }

        rule curly_brace() -> Token = ['{'] _ t:token()**__ _ ['}'] {
            Token::Branch(Delimiter::CurlyBrace, t)
        }

        rule comment() -> Token = "//" [^'\n']* "\n" { Token::Comment }

        rule ident() -> Token = t:$("'"?['a'..='z' | 'A'..='Z' | '_' | '0' ..= '9' | '*' ]+) {
            Token::Leaf(t.to_string())
        }

        rule comma() -> Token = t:$(",") {
            Token::Leaf(t.to_string())
        }

        rule string() -> Token = ['"'] t:$([^'"']*) ['"'] {
            Token::Leaf(t.to_string())
        }

        rule _ = quiet!{[' ' | '\t']*}
        
        rule __ = quiet!{[' ' | '\t']+}
    }
}

#[test]
fn main_test() {
    let d = tokenizer::tokens("abc zzz { abc }");
    println!("{:#?}", d);
}
