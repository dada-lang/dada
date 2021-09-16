extern crate peg;

#[derive(Debug)]
pub enum Token {
    Leaf(usize, usize),
    Branch(Delimiter, Vec<Token>),
    Comment(usize, usize),
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

        rule nl() -> Token = s:position!() "\n" e:position!() { 
            Token::Leaf(s, e)
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

        rule comment() -> Token = s:position!() "//" [^'\n']* "\n" e:position!() {
            Token::Comment(s, e) 
        }

        rule ident() -> Token = s:position!() ['a'..='z' | 'A'..='Z' | '_' | '0' ..= '9' | '*' ]+ e:position!() {
            Token::Leaf(s, e)
        }

        rule comma() -> Token = s:position!() "," e:position!() {
            Token::Leaf(s, e)
        }

        rule string() -> Token = s:position!() ['"'] t:$([^'"']*) ['"'] e:position!() {
            Token::Leaf(s, e)
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
