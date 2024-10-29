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
    start_state: String,
    terminal_states: HashSet<String>,
    transitions: HashMap<(String, String), String>,
}

impl Automaton {
    fn new() -> Self {
        Automaton {
            states: HashSet::new(),
            alphabet: HashSet::new(),
            start_state: String::new(),
            terminal_states: HashSet::new(),
            transitions: HashMap::new(),
        }
    }

    fn build_from_file(&mut self, file_name: &str) -> io::Result<()> {
        let path = Path::new(file_name);
        let file = File::open(&path)?;
        let lines = io::BufReader::new(file).lines().collect::<Result<Vec<String>, _>>().expect("couldn't read from file");

        self.states = lines[0].split_whitespace().map(String::from).collect();
        self.alphabet = lines[1].split_whitespace().map(String::from).collect();
        
        // Assuming only one start state for DFA
        let start_states: HashSet<String> = lines[2].split_whitespace().map(String::from).collect();
        self.start_state = start_states.into_iter().next().expect("No start state provided");
        
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

        // Add start state indicator
        out_dot_code.push_str(&format!("    i{} [shape=point, style=invis];\n", self.start_state));

        // Add terminal states
        for state in &self.terminal_states {
            out_dot_code.push_str(&format!("    {} [shape=doublecircle];\n", state));
        }
        out_dot_code.push_str("\n");

        // Add start state transition
        out_dot_code.push_str(&format!("    i{} -> {};\n", self.start_state, self.start_state));

        // Create edge map for combining multiple transitions between same states
        let mut edge_map: HashMap<(&str, &str), Vec<&str>> = HashMap::new();
        
        for ((from, input), to) in &self.transitions {
            edge_map.entry((from.as_str(), to.as_str()))
                   .or_default()
                   .push(input.as_str());
        }

        // Add transitions
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

    fn to_complete_automaton(&mut self) -> Automaton {
        let mut complete_automaton = self.clone();
        let sink_state = "sink".to_string();
    
        if !complete_automaton.states.contains(&sink_state) {
            complete_automaton.states.insert(sink_state.clone());
        }
    
        for state in &complete_automaton.states {
            for symbol in &complete_automaton.alphabet {
                let key = (state.clone(), symbol.clone());
                if !complete_automaton.transitions.contains_key(&key) {
                    complete_automaton.transitions.insert(key, sink_state.clone());
                }
            }
        }
    
        complete_automaton
    }

    fn remove_unreachable_states(&mut self) {
        let mut reachable_states = HashSet::new();
        let mut queue = VecDeque::new();
    
        // Start from initial state
        queue.push_back(self.start_state.clone());
        reachable_states.insert(self.start_state.clone());
    
        // Perform BFS to find all reachable states
        while let Some(state) = queue.pop_front() {
            for symbol in &self.alphabet {
                if let Some(next_state) = self.transitions.get(&(state.clone(), symbol.clone())) {
                    if reachable_states.insert(next_state.clone()) {
                        queue.push_back(next_state.clone());
                    }
                }
            }
        }
    
        // Filter states and transitions
        self.states.retain(|state| reachable_states.contains(state));
        self.terminal_states.retain(|state| reachable_states.contains(state));
        self.transitions.retain(|(from, _), to| 
            reachable_states.contains(from) && reachable_states.contains(to));
    }

    fn minimize(&self) -> Automaton {
        // Initial partition: terminal and non-terminal states
        let mut partition: Vec<HashSet<String>> = vec![
            self.terminal_states.clone(),
            self.states.difference(&self.terminal_states).cloned().collect(),
        ];
        let mut worklist: Vec<HashSet<String>> = partition.clone();
    
        while let Some(a) = worklist.pop() {
            for c in &self.alphabet {
                let x: HashSet<String> = self.states.iter()
                    .filter(|state| {
                        if let Some(next_state) = self.transitions.get(&(state.to_string(), c.to_string())) {
                            a.contains(next_state)
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();
    
                let mut new_partition = Vec::new();
                for y in &partition {
                    let intersection: HashSet<_> = y.intersection(&x).cloned().collect();
                    let difference: HashSet<_> = y.difference(&x).cloned().collect();
    
                    if !intersection.is_empty() && !difference.is_empty() {
                        if worklist.contains(y) {
                            worklist.retain(|w| w != y);
                            worklist.push(intersection.clone());
                            worklist.push(difference.clone());
                        } else {
                            if intersection.len() <= difference.len() {
                                worklist.push(intersection.clone());
                            } else {
                                worklist.push(difference.clone());
                            }
                        }
                        new_partition.push(intersection);
                        new_partition.push(difference);
                    } else {
                        new_partition.push(y.clone());
                    }
                }
                partition = new_partition;
            }
        }
    
        // Construct minimized automaton
        let mut minimized = Automaton::new();
        minimized.alphabet = self.alphabet.clone();
    
        // Create state mapping
        let mut state_mapping: HashMap<String, String> = HashMap::new();
        for block in &partition {
            let representative = block.iter().next().unwrap().clone();
            minimized.states.insert(representative.clone());
            
            if block.contains(&self.start_state) {
                minimized.start_state = representative.clone();
            }
            
            if !block.is_disjoint(&self.terminal_states) {
                minimized.terminal_states.insert(representative.clone());
            }
    
            for state in block {
                state_mapping.insert(state.clone(), representative.clone());
            }
        }
    
        // Create transitions for minimized automaton
        for ((from, symbol), to) in &self.transitions {
            if let (Some(from_rep), Some(to_rep)) = (state_mapping.get(from), state_mapping.get(to)) {
                minimized.transitions.insert(
                    (from_rep.clone(), symbol.clone()),
                    to_rep.clone()
                );
            }
        }
    
        minimized
    }

    fn is_minimized(&self) -> bool {
        let minimized = self.minimize();
        self.states.len() == minimized.states.len()
    }
}

impl fmt::Display for Automaton {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "States: {:?}\nAlphabet: {:?}\nStart State: {}\nTerminal States: {:?}\nTransitions: {:?}", 
               self.states, self.alphabet, self.start_state, self.terminal_states, self.transitions)
    }
}

impl PartialEq for Automaton {
    fn eq(&self, other: &Self) -> bool {
        if self.alphabet != other.alphabet {
            return false;
        }

        let mut self_complete = self.clone().to_complete_automaton();
        let mut other_complete = other.clone().to_complete_automaton();

        let mut state_pairs = vec![(self_complete.start_state.clone(), other_complete.start_state.clone())];
        let mut visited_pairs = HashSet::new();
        let mut index = 0;

        while index < state_pairs.len() {
            let (q1, q2) = state_pairs[index].clone();
            index += 1;

            // Check if states are equivalent regarding acceptance
            if self_complete.terminal_states.contains(&q1) != other_complete.terminal_states.contains(&q2) {
                return false;
            }

            // Add pair to visited
            visited_pairs.insert((q1.clone(), q2.clone()));

            // Check transitions for each symbol
            for symbol in &self_complete.alphabet {
                let next_q1 = self_complete.transitions.get(&(q1.clone(), symbol.clone())).cloned();
                let next_q2 = other_complete.transitions.get(&(q2.clone(), symbol.clone())).cloned();

                match (next_q1, next_q2) {
                    (Some(s1), Some(s2)) => {
                        let pair = (s1.clone(), s2.clone());
                        if !visited_pairs.contains(&pair) {
                            state_pairs.push(pair);
                        }
                    }
                    _ => return false,
                }
            }
        }

        true
    }
}

fn main() -> io::Result<()> {
    let mut automaton = Automaton::new();
    automaton.build_from_file("resources/form_I.B.1_a1.txt")?;

    let mut automaton2 = Automaton::new();
    automaton2.build_from_file("resources/form_I.B.1_a2.txt")?;
    
    
    automaton.write_dot_code("b_4_1.dot")?;
    
    let minimized = automaton.minimize();
    automaton2.write_dot_code("b_4_1_2.dot")?;
    
    if automaton2 == automaton {
        println!("Automaton is already minimal!");
    }
    
    Ok(())
}