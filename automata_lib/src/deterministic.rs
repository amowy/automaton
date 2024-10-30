use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::traits::Automaton;

#[derive(Clone, Debug)]
pub struct DeterministicAutomaton {
    pub states: HashSet<String>,
    pub alphabet: HashSet<String>,
    pub start_states: HashSet<String>,
    pub terminal_states: HashSet<String>,
    pub transitions: HashMap<(String, String), String>,
}

impl DeterministicAutomaton {
    pub fn new() -> Self {
        DeterministicAutomaton {
            states: HashSet::new(),
            alphabet: HashSet::new(),
            start_states: HashSet::new(),
            terminal_states: HashSet::new(),
            transitions: HashMap::new(),
        }
    }

    pub fn to_complete_automaton(&mut self) -> DeterministicAutomaton {
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

    pub fn remove_unreachable_states(&mut self) {
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

    pub fn minimize(&self) -> DeterministicAutomaton {
        let mut partition: Vec<HashSet<String>> = vec![
            self.terminal_states.clone(), 
            self.states.difference(&self.terminal_states).cloned().collect()
        ];
    
        let mut worklist: Vec<HashSet<String>> = partition.clone();
    
        while !worklist.is_empty() {
            let a = worklist.remove(0);
    
            for c in &self.alphabet {
                let x: HashSet<String> = self.states.iter()
                    .filter(|&state| self.transitions.get(&(state.clone(), c.clone())).map_or(false, |next| a.contains(next)))
                    .cloned()
                    .collect();

                let mut new_partitions = Vec::new();
    
                for y in partition.iter() {
                    if !y.is_disjoint(&x) && !y.is_disjoint(&(y.difference(&x).cloned().collect())) {
                        let intersection = y.intersection(&x).cloned().collect::<HashSet<String>>();
                        let difference = y.difference(&x).cloned().collect::<HashSet<String>>();
    
                        new_partitions.push(intersection.clone());
                        new_partitions.push(difference.clone());
    
                        if worklist.contains(y) {
                            worklist.retain(|w| w != y);
                            worklist.push(intersection.clone());
                            worklist.push(difference.clone());
                        } else {
                            if intersection.clone().len() <= difference.clone().len() {
                                worklist.push(intersection.clone());
                            } else {
                                worklist.push(difference);
                            }
                        }
                    } else {
                        new_partitions.push(y.clone());
                    }
                }

                partition = new_partitions;
            }
        }
    
        let mut minimized = DeterministicAutomaton::new();
        minimized.alphabet = self.alphabet.clone();
        
        let mut state_mapping: HashMap<String, String> = HashMap::new();
    
        for block in &partition {
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
                        let next_block = partition.iter().find(|b| b.contains(next_state)).unwrap();
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
    
    pub fn is_minimized(&self) -> bool {
        *self == self.minimize()
    }
}

impl Automaton for DeterministicAutomaton {

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

}

impl fmt::Display for DeterministicAutomaton {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "States: {:?}\nAlphabet: {:?}\nStart States: {:?}\nTerminal States: {:?}\nTransitions: {:?}", 
               self.states, self.alphabet, self.start_states, self.terminal_states, self.transitions)
    }
}

impl PartialEq for DeterministicAutomaton {
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