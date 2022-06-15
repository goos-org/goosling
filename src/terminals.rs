use limine::LimineTerminal;

pub struct Terminal<'a, TermWrite: Fn(&LimineTerminal, &str)> {
    limine_terminal: &'a LimineTerminal,
    term_write: TermWrite,
}
impl<'a, TermWrite: Fn(&LimineTerminal, &str)> Terminal<'a, TermWrite> {
    pub fn new(limine_terminal: &'a LimineTerminal, term_write: TermWrite) -> Self {
        Terminal {
            limine_terminal,
            term_write,
        }
    }
    pub fn info(&self, string: &str) {
        (self.term_write)(self.limine_terminal, "[INFO] ");
        (self.term_write)(self.limine_terminal, string);
        (self.term_write)(self.limine_terminal, "\n");
    }
    pub fn info_raw(&self, string: &str) {
        (self.term_write)(self.limine_terminal, "[INFO] ");
        (self.term_write)(self.limine_terminal, string);
    }
    pub fn ok(&self, string: &str) {
        (self.term_write)(self.limine_terminal, "[ OK ] ");
        (self.term_write)(self.limine_terminal, string);
        (self.term_write)(self.limine_terminal, "\n");
    }
    pub fn ok_raw(&self, string: &str) {
        (self.term_write)(self.limine_terminal, "[ OK ] ");
        (self.term_write)(self.limine_terminal, string);
    }
    pub fn fail(&self, string: &str) {
        (self.term_write)(self.limine_terminal, "[FAIL] ");
        (self.term_write)(self.limine_terminal, string);
        (self.term_write)(self.limine_terminal, "\n");
    }
    pub fn fail_raw(&self, string: &str) {
        (self.term_write)(self.limine_terminal, "[FAIL] ");
        (self.term_write)(self.limine_terminal, string);
    }
    pub fn print(&self, string: &str) {
        (self.term_write)(self.limine_terminal, string);
    }
    pub fn println(&self, string: &str) {
        (self.term_write)(self.limine_terminal, string);
        (self.term_write)(self.limine_terminal, "\n");
    }
}
