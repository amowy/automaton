use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::fs::write;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

#[derive(Clone, Debug)]
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

    fn to_complete_automaton(&mut self) -> Automaton{
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
        self.clone()
    }

    fn remove_unreachable_states(&mut self) {
        let mut reachable_nodes: HashSet<String> = HashSet::new();
        let mut productive_nodes: Vec<String> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue = VecDeque::new();

        for node in &self.states {
            queue.push_back(node.clone());
            visited.insert(node.clone());
        }

        while let Some(current_node) = queue.pop_front() {
            reachable_nodes.insert(current_node.clone());
            for edge in &self.transitions {
                if edge.0.0 == current_node && visited.insert(edge.1.clone()) {
                    queue.push_back(edge.1.clone());
                }
            }
        }
        for terminal_node in &self.terminal_states {
            queue.push_back(terminal_node.clone());
            visited.insert(terminal_node.clone());
        }
    
        while let Some(current_node) = queue.pop_front() {
            if reachable_nodes.contains(&current_node) {
                productive_nodes.push(current_node.clone());
            }
            for ((from, _), to) in &self.transitions {
                if to == &current_node && visited.insert(from.clone()) {
                    queue.push_back(from.clone());
                }
            }
        }

        self.states = reachable_nodes.clone();
        self.terminal_states = productive_nodes.into_iter().collect();
        self.transitions.retain(|(from, _), _| reachable_nodes.contains(from));
    }

    pub fn minimize(&self) -> Automaton {
        /*
        makes funny errors
        */
        let mut partition: HashMap<Vec<String>, HashSet<String>> = HashMap::new();
        let non_terminal_states: HashSet<String> = self.states.difference(&self.terminal_states).cloned().collect();
        partition.insert(non_terminal_states.iter().cloned().collect::<Vec<String>>(), HashSet::new());
        partition.insert(self.terminal_states.iter().cloned().collect::<Vec<String>>(), HashSet::new());

        let mut worklist: Vec<HashSet<String>> = partition.values().cloned().collect();

        while let Some(block) = worklist.pop() {
            for symbol in &self.alphabet {
                let mut transitions: HashMap<Vec<String>, HashSet<String>> = HashMap::new();
                for state in &block {
                    if let Some(next_state) = self.transitions.get(&(state.clone(), symbol.clone())) {
                        let block_key = partition.keys().find(|&key| key.contains(next_state)).unwrap();
                        transitions.entry(block_key.clone()).or_insert_with(HashSet::new).insert(state.clone());
                    }
                }

                for (key, states) in transitions {
                    if states.len() < block.len() {
                        let new_block: HashSet<String> = states.into_iter().collect();
                        let remaining_states: HashSet<String> = block.difference(&new_block).cloned().collect();
                        partition.insert(new_block.iter().cloned().collect::<Vec<String>>(), HashSet::new());
                        partition.insert(remaining_states.iter().cloned().collect::<Vec<String>>(), HashSet::new());
                    
                        worklist.push(new_block.iter().cloned().collect());
                        worklist.push(remaining_states.iter().cloned().collect());
                    }
                }

            }
        }

        let mut minimized = Automaton::new();
        let mut state_mapping: HashMap<String, String> = HashMap::new();


        minimized.alphabet = self.alphabet.clone();
        for (block, _) in &partition {
            let representative = block.iter().next().unwrap();
            minimized.states.insert(representative.clone());
            if self.start_states.contains(representative) {
                minimized.start_states.insert(representative.clone());
            }
            if self.terminal_states.contains(representative) {
                minimized.terminal_states.insert(representative.clone());
            }
            state_mapping.insert(representative.clone(), representative.clone());

            for state in block {
                for symbol in &self.alphabet {
                    if let Some(next_state) = self.transitions.get(&(state.clone(), symbol.clone())) {
                        let next_block = partition.keys().find(|&key| key.contains(next_state)).unwrap();
                        let next_representative = next_block.iter().next().unwrap();
                        minimized.transitions.insert((representative.clone(), symbol.clone()), next_representative.clone());
                    }
                }
            }
        }

        if minimized.start_states.is_empty() {
            if let Some(first_representative) = minimized.states.difference(&minimized.terminal_states).cloned().next() {
                minimized.start_states.insert(first_representative.clone());
            }
        }    

        minimized
    }

    pub fn minimize_table_method(&self) -> Automaton {
        let mut minimized = Automaton::new();
        minimized.alphabet = self.alphabet.clone();

        let mut distinguishable = HashMap::new();
        let state_list: Vec<String> = self.states.iter().cloned().collect();

        for i in 0..state_list.len() {
            for j in 0..i {
                let s1 = &state_list[i];
                let s2 = &state_list[j];
                let is_distinguishable = (self.terminal_states.contains(s1) && !self.terminal_states.contains(s2))
                    || (!self.terminal_states.contains(s1) && self.terminal_states.contains(s2));
                distinguishable.insert((s1.clone(), s2.clone()), is_distinguishable);
            }
        }

        let mut changes = true;
        while changes {
            changes = false;
            for i in 0..state_list.len() {
                for j in 0..i {
                    let s1 = &state_list[i];
                    let s2 = &state_list[j];

                    if !distinguishable[&(s1.clone(), s2.clone())] {
                        for symbol in &self.alphabet {
                            let t1 = self.transitions.get(&(s1.clone(), symbol.clone()));
                            let t2 = self.transitions.get(&(s2.clone(), symbol.clone()));
                            if let (Some(ns1), Some(ns2)) = (t1, t2) {
                                let (ns1, ns2) = if ns1 < ns2 {
                                    (ns1, ns2)
                                } else {
                                    (ns2, ns1)
                                };
                                if distinguishable.get(&(ns1.clone(), ns2.clone())) == Some(&true) {
                                    distinguishable.insert((s1.clone(), s2.clone()), true);
                                    changes = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut state_mapping: HashMap<String, String> = HashMap::new();
        for i in 0..state_list.len() {
            let s1 = &state_list[i];
            for j in 0..i {
                let s2 = &state_list[j];
                if !distinguishable[&(s1.clone(), s2.clone())] {
                    let representative = state_mapping.get(s1).unwrap_or(s1).clone();
                    state_mapping.insert(s2.clone(), representative.clone());
                }
            }
            state_mapping.entry(s1.clone()).or_insert(s1.clone());
        }

        for (state, repr) in &state_mapping {
            minimized.states.insert(repr.clone());
            if self.start_states.contains(state) {
                minimized.start_states.insert(repr.clone());
            }
            if self.terminal_states.contains(state) {
                minimized.terminal_states.insert(repr.clone());
            }
        }

        for ((from, symbol), to) in &self.transitions {
            let from_repr = state_mapping.get(from).unwrap();
            let to_repr = state_mapping.get(to).unwrap();
            minimized.transitions.insert((from_repr.clone(), symbol.clone()), to_repr.clone());
        }

        minimized
    }

    fn is_minimized(&self) -> bool {
        *self == self.minimize()
    }
}

impl fmt::Display for Automaton {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "States: {:?}\nAlphabet: {:?}\nStart States: {:?}\nTerminal States: {:?}\nTransitions: {:?}", 
               self.states, self.alphabet, self.start_states, self.terminal_states, self.transitions)
    }
}

impl PartialEq for Automaton {
    fn eq(&self, other: &Self) -> bool {
        let mut self_copy = self.clone();
        let mut other_copy = other.clone();
        self_copy.to_complete_automaton();
        other_copy.to_complete_automaton();
        let mut table: Vec<(String, String)> = vec![(self_copy.start_states.iter().next().unwrap().clone(), 
                                                    other_copy.start_states.iter().next().unwrap().clone())];
        if self_copy.terminal_states.contains(&table[0].0) != other_copy.terminal_states.contains(&table[0].1) {
            return false;
        }
        let mut table_index = 0;
        while table_index < table.len() {
            
            let (q, q_prime) = &table[table_index].clone();
            table_index += 1;
            for a in &self_copy.alphabet {
                let next_q = self_copy.transitions.get(&(q.clone(), a.clone()));
                let next_q_prime = other_copy.transitions.get(&(q_prime.clone(), a.clone()));
                if let (Some(next_q), Some(next_q_prime)) = (next_q, next_q_prime) {
                    if self_copy.terminal_states.contains(next_q) != other_copy.terminal_states.contains(next_q_prime) {
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
    /*
    let mut automaton1 = Automaton::new();
    let mut automaton2 = Automaton::new();
    automaton1.build_from_file("resources/form_I.B.1_a1.txt").expect("nem sikerult a filet olvasni");
    automaton2.build_from_file("resources/form_I.B.1_a2.txt").expect("nem sikerult a filet olvasni");
    automaton1.write_dot_code("resources/automaton1.dot").expect("couldnt build dot file");
    automaton2.write_dot_code("resources/automaton2.dot").expect("couldnt build dot file");
    //automaton1.to_complete_automaton();
    //automaton2.to_complete_automaton();
    automaton1.write_dot_code("resources/automaton1_complete.dot").expect("couldnt build dot file");
    automaton2.write_dot_code("resources/automaton2_complete.dot").expect("couldnt build dot file");
    if automaton1 == automaton2{
        println!("ekvivalens");
    } else {
        println!("nem ekvivalens");
    }
    if automaton2.is_minimized() {
        println!("minimized");
    } else {
        println!("not minimized");
        //automaton2.minimize().write_dot_code("resources/auto2_minimized.dot")?;
    }
    let mut minimize = automaton2.minimize();
    if minimize.is_minimized() {
        println!("legalabb ez megy");
    }
    minimize.write_dot_code("resources/auto2_minimized.dot")?;
    if minimize == automaton2 {
        println!("megy minden :D {:?}", automaton2);
    } 
    */
    let mut automaton2 = Automaton::new();
    automaton2.build_from_file("resources/form_I.B.1_a2.txt").expect("nem sikerult a filet olvasni");
    automaton2.write_dot_code("resources/automaton2_complete.dot").expect("couldnt build dot file");
    let minimize = automaton2.minimize();
    let mut minimize2 = automaton2.minimize_table_method();
    minimize.write_dot_code("resources/auto2_minimized.dot")?;
    minimize2.write_dot_code("resources/auto2_minimized2.dot")?;
    minimize2.to_complete_automaton().write_dot_code("resources/auto2_minimized2_comp.dot")?;
    if minimize2.to_complete_automaton() == automaton2.to_complete_automaton()  {
        println!("pls");
    }
    Ok(())
}
