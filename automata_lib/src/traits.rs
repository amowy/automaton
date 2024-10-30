use std::fmt::Display;
use std::hash::Hash;
use std::fs::write;

pub trait AutomatonSymbol: Clone + Hash + Eq + Display {}
impl<T: Clone + Hash + Eq + Display> AutomatonSymbol for T {}

pub trait Automaton {
    fn build_dot_code(&self) -> String;
    
    fn build_from_file(&mut self, file_path: &str) -> std::io::Result<()>;

    fn write_dot_code(&self, file_path: &str) -> std::io::Result<()> {
        let dot_code = self.build_dot_code();
        write(file_path, dot_code)?;
        Ok(())
    }
}