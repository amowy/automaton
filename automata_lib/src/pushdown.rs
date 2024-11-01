/**
 * Gyorgy Matyas
 * gmim2236
 * 1.B.02
 * pda
 */

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::traits::Automaton;
use crate::utils::escape_dot_label;

#[derive(Debug)]
struct Transition {
    current_state: String,
    input_symbol: String,
    stack_symbol: String,
    new_stack_symbols: Vec<String>, //no hashset bcs we need order
    next_state: String, 
}

#[derive(Debug)]
pub struct PushdownAutomaton {
    states: HashSet<String>,
    input_symbols: HashSet<String>,
    stack_symbols: HashSet<String>,
    start_state: String,
    stack_start_symbol: String,
    terminal_states: HashSet<String>,
    transitions: Vec<Transition>,
}

impl PushdownAutomaton {
    pub fn new() -> Self {
        PushdownAutomaton {
            states: HashSet::new(),
            input_symbols: HashSet::new(),
            stack_symbols: HashSet::new(),
            start_state: String::new(),
            stack_start_symbol: String::new(),
            terminal_states: HashSet::new(),
            transitions: Vec::new(),
        }
    }

    #[doc = r"* Gyorgy Matyas
    * gmim2236
    * 1.B.02
    * pda"]
    pub fn accepts(&self, input: &str) -> bool {
        let stack = vec![self.stack_start_symbol.clone()];
        let input_chars: Vec<String> = input.chars().map(String::from).collect();
        
        // check for valid characters
        if !input_chars.iter().all(|c| self.input_symbols.contains(c)) {
            return false;
        }
        
        self.dfs_accept(&self.start_state, &input_chars, &stack, 0)
    }

    fn dfs_accept(
        &self,
        current_state: &str,
        input_chars: &[String],
        stack: &[String],
        iteration_count: usize,
    ) -> bool {
        const MAX_ITERATIONS: usize = 10000;
        
        if iteration_count >= MAX_ITERATIONS {
            return false;
        }

        // accept condition
        if input_chars.is_empty() && (self.terminal_states.contains(current_state) || stack.is_empty())  {
        //if self.terminal_states.contains(current_state) {
            return true;
        }

        // cant proceed if stack is empty
        if stack.is_empty() {
            return false;
        }

        let current_input = input_chars.first().map_or("eps", String::as_str);
        let current_stack_top = stack.last().expect("Stack should not be empty here");

        for transition in &self.transitions {
            if transition.current_state == current_state
                && (transition.input_symbol == current_input || transition.input_symbol == "eps")
                && transition.stack_symbol == *current_stack_top
            {
                // new stack
                let mut new_stack = stack[..stack.len()-1].to_vec();
                for symbol in transition.new_stack_symbols.iter().rev() {
                    if symbol != "eps" {
                        new_stack.push(symbol.clone());
                    }
                }

                //  new input
                let new_input = if transition.input_symbol != "eps" && !input_chars.is_empty() {
                    &input_chars[1..]
                } else {
                    input_chars
                };

                if self.dfs_accept(&transition.next_state, new_input, &new_stack, iteration_count + 1) {
                    return true;
                }
            }
        }
        false
    }

    pub fn check_for_file(&self, file_name: &str) -> io::Result<()> {
        let file = File::open(file_name)?;
        for line in io::BufReader::new(file).lines() {
            let line = line?;
            if self.accepts(&line) {
                println!("{} accepted", &line);
            } else {
                println!("{} declined", &line);
            }
        }
        Ok(())
    }
}

impl Automaton for PushdownAutomaton {
    fn build_dot_code(&self) -> String {
        let mut dot_content = String::new();

        dot_content.push_str("digraph PdAutomaton {\n");
        dot_content.push_str("\trankdir=LR;\n");
        dot_content.push_str("\tnode [shape=circle];\n");

        dot_content.push_str("\tstart [shape=point];\n");
        dot_content.push_str(&format!("\tstart -> {};\n", self.start_state));

        for terminal_state in &self.terminal_states {
            dot_content.push_str(&format!("\t{} [shape=doublecircle];\n", terminal_state));
        }

        for transition in &self.transitions {
            let stack_symbols = transition.new_stack_symbols.join(" ");
            dot_content.push_str(&format!(
                "\t{} -> {} [label=\"{}| {}-> {}\"];\n",
                transition.current_state,
                transition.next_state,
                escape_dot_label(&transition.input_symbol),
                escape_dot_label(&transition.stack_symbol),
                escape_dot_label(&stack_symbols)
            ));
        }

        dot_content.push_str("}\n");

        dot_content
    }

    fn build_from_file(&mut self, file_name: &str) -> io::Result<()> {
        let path = Path::new(file_name);
        let file = File::open(&path)?;
        let lines: Vec<String> = io::BufReader::new(file)
            .lines()
            .collect::<Result<_, _>>()?;

        if lines.len() < 6 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File does not contain enough lines",
            ));
        }

        //1. sor: allapotok, szokozokkel elvalasztva
        self.states = lines[0].split_whitespace().map(String::from).collect();
        //2. sor: bemeneti abece elemei, szokozokkel elvalasztva
        self.input_symbols = lines[1].split_whitespace().map(String::from).collect();
        //3. sor: veremabece elemei, szokozokkel elvalasztva
        self.stack_symbols = lines[2].split_whitespace().map(String::from).collect();
        // 4. sor: kezdoallapot
        self.start_state = lines[3].trim().to_string();
        // 5. sor: veremmemoria kezdojele
        self.stack_start_symbol = lines[4].trim().to_string();
        // 6. sor: vegallapotok, szokozokkel elvalasztva
        self.terminal_states = lines[5].split_whitespace().map(String::from).collect();
        
        //egy-egy atmenet
        self.transitions.clear();
        for line in &lines[6..] {
            let parts: Vec<String> = line.split_whitespace().map(String::from).collect();
            if parts.len() < 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid transition format: {}", line),
                ));
            }

            self.transitions.push(Transition {
                current_state: parts[0].clone(),
                input_symbol: parts[1].clone(),
                stack_symbol: parts[2].clone(),
                new_stack_symbols: parts[3..parts.len()-1].to_vec(),
                next_state: parts[parts.len()-1].clone(),
            });
        }

        Ok(())
    }
}