use crate::parse::Parser;
use core::ops::*;
use std::collections::HashMap;
use std::rc::Rc;
macro_rules! io_command {
    ($v:expr, $mask:expr, $mnemonic:expr) => {
        let _ : u16 = $mask;
        let _ : &str = $mnemonic;
        if $v.bitand($mask).bitand($mask) == $mask {
            return format!("{} {:0>2X}", $mnemonic, $v.bitand(0xFF))
        }
    };
}

macro_rules! address_command {
    ($v:expr, $mask:expr, $mnemonic:expr) => {
        let _ : u16 = $mask;
        let _ : &str = $mnemonic;
        if $v.bitand($mask).bitand($mask) == $mask {
            return format!("{} {:0>3X}", $mnemonic, $v.bitand(0x7FF))
        }
    };
}

macro_rules! simple_command {
    ($v:expr, $mask:expr, $mnemonic:expr) => {
        let _ : u16 = $mask;
        if $v.bitand($mask).bitand($mask) == $mask {
            return $mnemonic.to_string();
        }
    };
}

pub struct GeneralParser {
    sorted: Vec<Rc<dyn GeneralCommand>>,
    mnemonic_map: HashMap<String, Rc<dyn GeneralCommand>>
}

impl GeneralParser {

    fn register<T>(&mut self, command: T)
        where T:'static, T: GeneralCommand
    {
        let rc1 = Rc::new(command);
        let rc2 = rc1.clone();
        self.sorted.push(rc1);
        self.mnemonic_map.insert(rc2.mnemonic().to_string(), rc2);
    }

    pub fn new() -> GeneralParser {
        let mut parser = GeneralParser {
            sorted: vec![],
            mnemonic_map: HashMap::new()
        };

        parser.register(SimpleCommand::new(0xF700, "ROR"));
        parser.register(SimpleCommand::new(0xFB00, "DI"));
        parser.register(SimpleCommand::new(0xF300, "CLC"));
        parser.register(SimpleCommand::new(0xF500, "CMC"));
        parser.register(SimpleCommand::new(0xF600, "ROL"));
        parser.register(SimpleCommand::new(0xF900, "DEC"));
        parser.register(SimpleCommand::new(0xFA00, "EI"));
        parser.register(SimpleCommand::new(0xF200, "CLA"));
        parser.register(SimpleCommand::new(0xF400, "CMA"));
        parser.register(SimpleCommand::new(0xF800, "INC"));
        parser.register(SimpleCommand::new(0xF100, "NOP"));
        parser.register(AddressCommand::new_io(0xE300, "OUT"));
        parser.register(SimpleCommand::new(0xF000, "HLT"));
        parser.register(AddressCommand::new_io(0xE100, "TSF"));
        parser.register(AddressCommand::new_io(0xE200, "IN"));
        parser.register(AddressCommand::new_address(0xB000, "BEQ"));
        parser.register(AddressCommand::new_io(0xE000, "CLF"));
        parser.register(AddressCommand::new_address(0x3000, "MOV"));
        parser.register(AddressCommand::new_address(0x5000, "ADC"));
        parser.register(AddressCommand::new_address(0x6000, "SUB"));
        parser.register(AddressCommand::new_address(0x9000, "BPL"));
        parser.register(AddressCommand::new_address(0xA000, "BMI"));
        parser.register(AddressCommand::new_address(0xC000, "BR"));
        parser.register(AddressCommand::new_address(0x1000, "AND"));
        parser.register(AddressCommand::new_address(0x4000, "ADD"));
        parser.register(AddressCommand::new_address(0x8000, "BCS"));
        parser.register(AddressCommand::new_address(0x2000, "JSR"));
        parser.register(AddressCommand::new_address(0x0000, "ISZ"));

        parser
    }

}

impl Parser for GeneralParser {
    fn parse(&self, v: u16) -> String {
        for command in self.sorted.iter() {
            if command.mask().bitand(v).bitand(command.mask()) == command.mask() {
                return command.parse(v)
            }
        }
        panic!()
    }

    fn supports_rev_parse(&self) -> bool {
        true
    }

    fn rev_parse(&self, str: String) -> Result<u16, String> {
        let mnemonic = str.split(" ").next();
        if mnemonic.is_none() {
            return Err(format!("Пустая строка получается"))
        }

        let mnemonic = mnemonic.unwrap();
        let command = self.mnemonic_map.get(mnemonic);
        if command.is_none() {
            return Err(format!("Неизвестная мнемоника {}", mnemonic))
        }

        command.unwrap().rev_parse(str.as_str())

    }
}


trait GeneralCommand {
    fn mnemonic(&self) -> &str;

    fn mask(&self) -> u16;

    fn parse(&self, data: u16) -> String;

    fn rev_parse(&self, s: &str) -> Result<u16, String>;
}

struct SimpleCommand {
    name: String,
    mask: u16
}

impl SimpleCommand {
    fn new<S : Into<String>>(mask: u16, name: S) -> SimpleCommand {
        SimpleCommand {
            name: name.into(),
            mask
        }
    }
}

impl GeneralCommand for SimpleCommand {
    fn mnemonic(&self) -> &str {
        self.name.as_str()
    }

    fn mask(&self) -> u16 {
        self.mask
    }

    fn parse(&self, data: u16) -> String {
        if data.bitand(self.mask).bitand(self.mask) != self.mask {
            panic!();
        }

        self.name.to_string()
    }

    fn rev_parse(&self, s: &str) -> Result<u16, String> {
        if s.trim() != self.name { panic!() }
        Ok(self.mask)
    }
}
struct AddressCommand {
    name: String,
    mask: u16,
    io: bool
}

impl AddressCommand {
    fn new_address<S: Into<String>>(mask: u16, name: S) -> AddressCommand {
        AddressCommand {
            name: name.into(),
            mask,
            io: false
        }
    }
    fn new_io<S: Into<String>>(mask: u16, name: S) -> AddressCommand {
        AddressCommand {
            name: name.into(),
            mask,
            io: true
        }
    }
}

impl GeneralCommand for AddressCommand {
    fn mnemonic(&self) -> &str {
        self.name.as_str()
    }

    fn mask(&self) -> u16 {
        self.mask
    }

    fn parse(&self, data: u16) -> String {
        if data.bitand(self.mask).bitand(self.mask) != self.mask {
            panic!();
        }

        if data.bitand(0x0800) != 0 {
            format!("{} ({:0>3X})", self.name, data.bitand(0x7FF))
        } else {
            format!("{} {:0>3X}", self.name, data.bitand(0x7FF))
        }

    }

    fn rev_parse(&self, s: &str) -> Result<u16, String> {
        let splited = s.trim().split(" ").collect::<Vec<&str>>();


        if splited.len() > 2 {
            return Err(format!("Неожиданные штуки:{}", splited.iter().skip(2).fold("".to_string(), |a,b| format!("{} {}", a, b))))
        }

        if splited.len() < 2 {
            return Err("Ожидалось два параметра".to_string())
        }

        let address = splited.get(1).unwrap().trim();


        let indirect = if !address.is_empty() {
            if address.chars().nth(0usize).unwrap() == '(' {
                if address.chars().nth(address.len() - 1).unwrap() != ')' {
                    return Err("Не закрытая скобка".to_string())
                } else {
                    true
                }
            } else {
                false
            }
        } else {
            false
        };

        let address = if indirect {
            address.get(1..address.len()-1).unwrap()
        } else { address};

        if let Ok(parsed) = u16::from_str_radix(address, 16) {
            let max = if self.io {
                0xFF
            } else {
                0x7FF
            };
            if parsed > max {
                if self.io {
                    Err("Максимально адресуемое ВУ 0xFF".to_string())
                } else {
                    Err("Максимально адресуема память 0x7FF".to_string())
                }
            } else {
                if indirect {
                    Ok(self.mask.bitor(parsed).bitor(0x0800))
                } else {
                    Ok(self.mask.bitor(parsed))

                }
            }
        } else {
            Err(format!("Ошибка во время парсинга числа {}", splited.get(1).unwrap()))
        }


    }
}

#[cfg(test)]
mod tests {
    use core::ops::*;
    use crate::parse::general::GeneralParser;
    use crate::parse::Parser;

    #[test]
    fn test() {
        let parser = GeneralParser::new();

        assert_eq!(parser.parse(0xF700), "ROR");
        assert_eq!(parser.parse(0x3024), "MOV 024");
    }
}
