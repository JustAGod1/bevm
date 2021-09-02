
pub mod mc;
pub mod general;

pub trait Parser {

    fn parse(&self, opcode: u16) -> String;

    fn supports_rev_parse(&self) -> bool;

    fn rev_parse(&self, str: String) -> Result<u16, String>;

}