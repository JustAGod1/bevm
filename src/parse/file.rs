use crate::parse::{CommandInfo, Parser};

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

pub fn parse_file<T: Read, I: CommandInfo, P: Parser<I>>(
    data: &mut T,
    parser: &P,
    max_size: u16,
) -> Result<Vec<(u16, u16)>, String> {
    let reader = BufReader::new(data);
    let mut cursor = 0;

    // let mut line_buf = String::with_capacity(128);

    let mut variables = HashMap::<String, u16>::new();
    let mut pre_result = Vec::<(u16, String, u16)>::new();

    for (line_num, line) in reader.lines().enumerate().take(u16::MAX.into()) {
        let line_num = line_num + 1;
        let line = line.map_err(|x| x.to_string())?;

        let Some(parsed) = parse_line(line.as_str()) else {
            continue;
        };

        macro_rules! err {
            ($msg:expr) => {
                format!("Ошибка в строке {}. Номер строки: {}. Сообщение: {}", line, line_num, $msg)
            };
        }
        match parsed {
            DataLine::Operator(name, arg) => {
                if name != "pos" {
                    return Err(err!(format!("Неизвестный оператор {}", name)));
                }

                let pos = match u16::from_str_radix(arg, 16) {
                    Err(_) => return Err(err!(format!("Не могу распарсить число {}", arg))),
                    Ok(v) => v,
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
                pre_result.push((cursor, command.to_string(), line_num as u16));

                if let Some(name) = name {
                    variables.insert(name.to_string(), cursor);
                }

                cursor += 1;

                if cursor > max_size {
                    return Err(err!(format!(
                        "Превышена максимальная позиция. Максимальная {:X}.",
                        max_size
                    )));
                }
            }
        }
    }

    let mut result = Vec::<(u16, u16)>::new();

    for (pos, cmd, line) in pre_result {
        let mut builder = String::new();
        let mut name = String::new();

        let mut var = false;

        for x in cmd.chars() {
            if !is_variable_name_char(x) && var {
                if !variables.contains_key(name.as_str()) {
                    return Err(format!(
                        "Ошибка в строке {}. Не могу найти переменную {}.",
                        line, name
                    ));
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
                return Err(format!(
                    "Ошибка в строке {}. Не могу найти переменную {}.",
                    line, name
                ));
            }
            builder.push_str(format!("{:X}", variables.get(name.as_str()).unwrap()).as_str());
        }

        let str = builder.as_str();

        if parser.supports_rev_parse() {
            match parser.rev_parse(str) {
                Ok(v) => result.push((pos, v)),
                Err(e) => match u16::from_str_radix(str, 16) {
                    Ok(v) => result.push((pos, v)),
                    Err(_) => return Err(format!("Ошибка в строке {}({}): {}", line, str, e)),
                },
            }
        } else {
            match u16::from_str_radix(str, 16) {
                Ok(v) => result.push((pos, v)),
                Err(_) => {
                    return Err(format!(
                        "Ошибка в строке {}({}): Не могу распарсить число.",
                        line, str
                    ))
                }
            }
        }
    }

    Ok(result)
}

#[derive(Debug, PartialEq)]
enum DataLine<'a> {
    Operator(&'a str, &'a str),
    Command(&'a str, Option<&'a str>),
}

fn is_variable_name_char(ch: char) -> bool {
    return ch.is_ascii_alphabetic() || ch.is_ascii_digit();
}

// Да я притащил либу для парсинга простой хуйни
fn parse_line(line: &str) -> Option<DataLine> {
    // remove comments
    // std::process::exit(500);
    let line = line.split_terminator('#').next().unwrap_or(line).trim();

    if line.len() == 0 {
        return None;
    };

    // if starts with $
    if line.starts_with('$') {
        let mut iter = (line[1..]).split_ascii_whitespace();
        let first = iter.next();
        let second = iter.next();
        return Some(DataLine::Operator(first?.trim(), second?.trim()));
    }

    // if not
    let mut iter = line.split('$');
    let first = iter.next();
    let second = iter.next();
    return Some(DataLine::Command(first?.trim(), second.map(|x| x.trim())));
}

#[cfg(test)]
mod tests {
    use crate::parse::file::{parse_line, DataLine};

    #[test]
    fn parse() {
        let res = parse_line("58: Huy $ded#de#abc");
        assert_eq!(res, Some(DataLine::Command("58: Huy", Some("ded"))));
    }

    #[test]
    fn parse_program_line_by_line() {
        let a = parse_line("$pos 10");
        assert_eq!(a, Some(DataLine::Operator("pos", "10")));
    }
    #[test]
    fn parse0() {
        let a = parse_line("CLA $start");
        assert_eq!(a, Some(DataLine::Command("CLA", Some("start"))));
    }
    #[test]
    fn parse1() {
        assert_eq!(
            parse_line("BMI %then"),
            Some(DataLine::Command("BMI %then", None))
        );
    }
    #[test]
    fn parse2() {
        assert_eq!(
            parse_line("BR %start"),
            Some(DataLine::Command("BR %start", None))
        );
    }
    #[test]
    fn parse3() {
        assert_eq!(parse_line("$pos 15"), Some(DataLine::Operator("pos", "15")));
    }
    #[test]
    fn parse4() {
        assert_eq!(
            parse_line("ISZ 2 $then"),
            Some(DataLine::Command("ISZ 2", Some("then")))
        );
    }
    #[test]
    fn parse5() {
        assert_eq!(
            parse_line("BR %start"),
            Some(DataLine::Command("BR %start", None))
        );
    }
}
