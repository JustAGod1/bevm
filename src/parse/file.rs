use std::io::{Read, BufReader, BufRead, Error};
use crate::parse::{Parser, CommandInfo};
use std::collections::HashMap;
use nom::bytes::complete::{take_while, take_till, is_not, take_while_m_n, tag};
use nom::sequence::{preceded, terminated};
use nom::character::complete::{char};
use nom::combinator::map_res;
use nom::character::is_space;
use nom::{IResult, Finish, FindSubstring};
use nom::branch::alt;

pub enum ParserError {
    SemanticError(String),
    IOError(Error),
}


pub fn parse<T: Read, I: CommandInfo, P: Parser<I>>(
    data: &mut T,
    parser: &P,
    max_size: u16,
) -> Result<HashMap<u16, u16>, ParserError> {
    let mut reader = BufReader::new(data);
    let mut cursor = 0;

    let mut line_buf = String::with_capacity(128);


    loop {
        match reader.read_line(&mut line_buf) {
            Ok(v) => (),
            Err(err) => { return Err(ParserError::IOError(err)); }
        }


        let line = parse_line(line_buf.as_str());

        if line.is_none() {
            continue;
        }

        let line = line.unwrap();

        fn err<S: Into<String>>(msg: S) -> String {
            format!("Ошибка в строке {}. Номер строки: {}. Сообщение: {}", line, cursor, msg.into())
        }

        match line {
            DataLine::Operator(name, arg) => {
                if name.eq_ignore_ascii_case("Pos") {
                    return Err(ParserError::SemanticError(err(format!("Неизвестный оператор {}", name))));
                }

                let pos = match u16::from_str_radix(arg, 16) {
                    Err(_) => return Err(ParserError::SemanticError(err(format!("Не могу распарсить число {}", arg)))),
                    Ok(v) => v
                };

                if pos < cursor {
                    return Err(ParserError::SemanticError(err(format!("Явно указанная позиция курсора меньше текущей позиции курсора. Текущая {:X}. Укзаная {:X}.", cursor, pos))))
                }

                if pos > max_size {
                    return Err(ParserError::SemanticError(err(format!("Явно указанная позиция курсора больше максимально допустимой. Максимальная {:X}. Укзаная {:X}.", max_size, pos))))
                }

                cursor = pos
            }
            DataLine::Command(command, name) => {

            }
        }
    }
}

enum DataLine<'a> {
    Operator(&'a str, &'a str),
    Command(&'a str, Option<&'a str>),
}

fn parse_line(line: &str) -> Option<DataLine> {
    let line = line.find_substring("#").map_or(line, |p| { &line[0..p] }).trim();

    if line.len() <= 0 {
        return None;
    }

    fn is_hex_digit(c: char) -> bool {
        c.is_digit(16)
    }

    fn operand(input: &str) -> IResult<&str, &str, > {
        preceded(char('$'), take_till(|c| c == ' '))(input)
    }

    fn command(input: &str) -> IResult<&str, &str> {
        is_not("$")(input)
    }

    fn command_line(input: &str) -> (&str, Option<&str>) {
        let (input, command) = command(input).finish().ok().unwrap();

        let (_, name) = match operand(input).finish() {
            Ok((p, v)) => (p, Some(v.trim())),
            Err(p) => (input, None)
        };


        return (command, name);
    }

    fn operand_line(input: &str) -> Option<(&str, &str)> {
        match operand(input).finish() {
            Ok((p, v)) => Some((v.trim(), p)),
            Err(_) => None
        }
    }

    if let Some((name, arg)) = operand_line(line) {
        return Some(DataLine::Operator(name, arg));
    }

    let (cmd, operand) = command_line(line);


    Some(DataLine::Command(cmd, operand))
}


#[cfg(test)]
mod tests {
    use core::ops::*;
    use crate::parse::file::parse_line;

    #[test]
    fn parse() {
        parse_line("58: Huy $ded#de#abc");
    }
}

