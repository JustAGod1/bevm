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



pub fn parse_file<T: Read, I: CommandInfo, P: Parser<I>>(
    data: &mut T,
    parser: &P,
    max_size: u16,
) -> Result<Vec<(u16, u16)>, String> {
    let mut reader = BufReader::new(data);
    let mut cursor = 0;

    let mut line_buf = String::with_capacity(128);

    let mut variables = HashMap::<String, u16>::new();
    let mut pre_result = Vec::<(u16, String, u16)>::new();

    let mut line_num = 0;

    loop {
        line_num += 1;
        line_buf.clear();
        match reader.read_line(&mut line_buf) {
            Ok(v) => if v == 0 { break; } else { () },
            Err(err) => {
                return Err(err.to_string());
            }
        }

        let line = line_buf.to_string();

        let parsed = parse_line(line.as_str());

        if parsed.is_none() {
            continue;
        }

        let parsed = parsed.unwrap();

        macro_rules! err {
            ($msg:expr) => {
                format!("Ошибка в строке {}. Номер строки: {}. Сообщение: {}", line_buf, line_num, $msg)
            };
        }


        match parsed {
            DataLine::Operator(name, arg) => {
                if name != "pos" {
                    return Err(err!(format!("Неизвестный оператор {}", name)));
                }

                let pos = match u16::from_str_radix(arg, 16) {
                    Err(_) => return Err(err!(format!("Не могу распарсить число {}", arg))),
                    Ok(v) => v
                };

                if pos < cursor {
                    return Err(err!(format!("Явно указанная позиция курсора меньше текущей позиции курсора. Текущая {:X}. Укзаная {:X}.", cursor, pos)));
                }

                if pos > max_size {
                    return Err(err!(format!("Явно указанная позиция курсора больше максимально допустимой. Максимальная {:X}. Укзаная {:X}.", max_size, pos)));
                }

                cursor = pos
            }
            DataLine::Command(command, name) => {
                pre_result.push((cursor, command.to_string(), line_num));

                if let Some(name) = name {
                    variables.insert(name.to_string(), cursor);
                }

                cursor += 1;

                if cursor > max_size {
                    return Err(err!(format!("Превышена максимальная позиция. Максимальная {:X}.", max_size)));
                }
            }
        }
    };

    let mut result = Vec::<(u16, u16)>::new();

    for (pos, cmd, line) in pre_result {
        let mut builder = String::new();
        let mut name = String::new();

        let mut var = false;

        for x in cmd.chars() {
            if (x == ' ' || x == '%') && var {
                if !variables.contains_key(name.as_str()) {
                    return Err(format!("Ошибка в строке {}. Не могу найти переменную {}.", line, name));
                }
                builder.push_str(format!("{:X}", variables.get(name.as_str()).unwrap()).as_str());
                if x != '%' {
                    builder.push(x);
                    var = false;
                }
                name = String::new();
            } else if x == '%' {
                var = true
            } else {
                if var {
                    name.push(x)
                } else {
                    builder.push(x)

                }

            }
        }

        if var {
            if !variables.contains_key(name.as_str()) {
                return Err(format!("Ошибка в строке {}. Не могу найти переменную {}.", line, name));
            }
            builder.push_str(format!("{:X}", variables.get(name.as_str()).unwrap()).as_str());
        }

        let str = builder.as_str();

        if parser.supports_rev_parse() {
            match parser.rev_parse(str) {
                Ok(v) => result.push((pos, v)),
                Err(e) => {
                    match u16::from_str_radix(str, 16) {
                        Ok(v) => result.push((pos, v)),
                        Err(_) => return Err(format!("Ошибка в строке {}({}): {}", line, str, e))
                    }
                }
            }
        } else {
            match u16::from_str_radix(str, 16) {
                Ok(v) => result.push((pos, v)),
                Err(_) => return Err(format!("Ошибка в строке {}({}): Не могу распарсить число.", line, str))
            }

        }

    }


    Ok(result)
}

enum DataLine<'a> {
    Operator(&'a str, &'a str),
    Command(&'a str, Option<&'a str>),
}

// Да я притащил либу для парсинга простой хуйни
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
        return Some(DataLine::Operator(name.trim(), arg.trim()));
    }

    let (cmd, operand) = command_line(line);


    Some(DataLine::Command(cmd.trim(), operand.map(|s| s.trim())))
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

