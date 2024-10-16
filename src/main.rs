use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::fs::write;
use std::collections::{HashMap, HashSet};


struct Automaton {
    states: HashSet<String>,
    alphabet: HashSet<String>,
    start_states: HashSet<String>,
    terminal_states: HashSet<String>,
    transitions: HashMap<(String, String), String>,
}

impl Automaton {
    fn new() -> Self {
        Automaton {
            states: HashSet::new(),
            alphabet: HashSet::new(),
            start_states: HashSet::new(),
            terminal_states: HashSet::new(),
            transitions: HashMap::new(),
        }
    }

    fn build_from_file(&mut self, file_name: &str) ->io::Result<()> {
        let path = Path::new(file_name);
        let file = File::open(&path)?;
        let lines = io::BufReader::new(file).lines().collect::<Result<Vec<String>, _>>().expect("couldnt read from file");

        self.states = lines[0].split_whitespace().map(String::from).collect();
        self.alphabet = lines[1].split_whitespace().map(String::from).collect();
        self.start_states = lines[2].split_whitespace().map(String::from).collect();
        self.terminal_states = lines[3].split_whitespace().map(String::from).collect();
        self.transitions.clear();
        
        for line in &lines[4..] {
            let parts: Vec<String> = line.split_whitespace().map(String::from).collect();
            if parts.len() == 3 {
                let from_state = parts[0].clone();
                let input = parts[1].clone();
                let to_state = parts[2].clone();
                self.transitions.insert((from_state, input), to_state);
            }
        }

        Ok(())
    }

    fn build_dot_code(&self) -> String {
        let mut out_dot_code = String::from(
            "digraph G {\n    ranksep=0.5;\n    nodesep=0.5;\n    rankdir=LR;\n    node [shape=\"circle\", fontsize=\"16\"];\n    fontsize=\"10\";\n    compound=true;\n\n"
        );

        for state in &self.states {
            if self.start_states.contains(state) {
                out_dot_code.push_str(&format!("    i{} [shape=point, style=invis];\n", state));
            }
        }

        for state in &self.states {
            if self.terminal_states.contains(state) {
                out_dot_code.push_str(&format!("    {} [shape=doublecircle];\n", state));
            }
        }
        out_dot_code.push_str("\n");

        for state in &self.start_states {
            out_dot_code.push_str(&format!("    i{} -> {};\n", state, state));
        }

        let mut edge_map: HashMap<(&str, &str), Vec<&str>> = HashMap::new();
        for ((from, input), to) in &self.transitions {
            let key = (from.as_str(), to.as_str());
            edge_map.entry(key).or_default().push(input.as_str());
        }

        for ((start, end), labels) in edge_map {
            let label_str = labels.join(", ");
            out_dot_code.push_str(&format!("    {} -> {} [label=\"{}\"];\n", start, end, label_str));
        }

        out_dot_code.push_str("}\n");
        out_dot_code
    }

    fn write_dot_code(&self, file_path: &str) -> io::Result<()> {
        let dot_code = self.build_dot_code();
        write(file_path, dot_code)?;
        Ok(())
    }

    fn to_complete_automaton(&mut self) {
        let sink_state = "sink".to_string();

        if !self.states.contains(&sink_state) {
            self.states.insert(sink_state.clone());
        }

        for state in self.states.clone() {
            for symbol in &self.alphabet {
                if !self.transitions.contains_key(&(state.clone(), symbol.clone())) {
                    self.transitions.insert((state.clone(), symbol.clone()), sink_state.clone());
                }
            }
        }

        for symbol in &self.alphabet {
            self.transitions.insert((sink_state.clone(), symbol.clone()), sink_state.clone());
        }
    }
}

impl PartialEq for Automaton {
    fn eq(&self, other: &Self) -> bool {
        let mut table: Vec<(String, String)> = vec![(self.start_states.iter().next().unwrap().clone(), 
                                                    other.start_states.iter().next().unwrap().clone())];
        if self.terminal_states.contains(&table[0].0) != other.terminal_states.contains(&table[0].1) {
            return false;
        }
        let mut table_index = 0;
        while table_index < table.len() {
            
            let (q, q_prime) = &table[table_index].clone();
            table_index += 1;
            for a in &self.alphabet {
                let next_q = self.transitions.get(&(q.clone(), a.clone()));
                let next_q_prime = other.transitions.get(&(q_prime.clone(), a.clone()));
                if let (Some(next_q), Some(next_q_prime)) = (next_q, next_q_prime) {
                    if self.terminal_states.contains(next_q) != other.terminal_states.contains(next_q_prime) {
                        return false;
                    }

                    if !table.contains(&(next_q.clone(), next_q_prime.clone())) {
                        table.push((next_q.clone(), next_q_prime.clone()));
                    }
                } else {
                    return false;
                }
            }
        }
                                                
        true
    }
}

fn main() -> io::Result<()> {
    let mut automaton1 = Automaton::new();
    let mut automaton2 = Automaton::new();
    automaton1.build_from_file("resources/form_I.B.1_a1.txt").expect("nem sikerult a filet olvasni");
    automaton2.build_from_file("resources/form_I.B.1_a2.txt").expect("nem sikerult a filet olvasni");
    automaton1.write_dot_code("resources/automaton1.dot").expect("couldnt build dot file");
    automaton2.write_dot_code("resources/automaton2.dot").expect("couldnt build dot file");
    automaton1.to_complete_automaton();
    automaton2.to_complete_automaton();
    automaton1.write_dot_code("resources/automaton1_complete.dot").expect("couldnt build dot file");
    automaton2.write_dot_code("resources/automaton2_complete.dot").expect("couldnt build dot file");
    if automaton1 == automaton2{
        println!("ekvivalens");
    } else {
        println!("nem ekvivalens");
    }
    Ok(())
}
