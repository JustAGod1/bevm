use imgui::Ui;

pub mod file;
pub mod general;
pub mod mc;

pub trait CommandInfo {
    fn file_string(&self) -> String;
    fn mnemonic(&self) -> String;
    fn draw_highlight(&self, ui: &Ui);
}

pub trait Parser<T: CommandInfo> {
    fn parse(&self, opcode: u16) -> T;

    fn supports_rev_parse(&self) -> bool;

    fn rev_parse(&self, str: &str) -> Result<u16, String>;
}
