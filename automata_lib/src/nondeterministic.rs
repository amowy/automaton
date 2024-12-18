/**
 * Gyorgy Matyas
 * gmim2236
 * 1.B.04
 * 1.A.03 (+)
 * 1.A.04 (+)
 * ndfa
 */

use std::collections::{HashMap, HashSet, VecDeque, BTreeSet};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::traits::Automaton;
use crate::deterministic::DeterministicAutomaton;

#[derive(Clone, Debug)]
pub struct NonDeterministicAutomaton {
    pub states: HashSet<String>,
    pub alphabet: HashSet<String>,
    pub start_states: HashSet<String>,
    pub terminal_states: HashSet<String>,
    pub transitions: HashMap<(String, String), HashSet<String>>,
}

impl NonDeterministicAutomaton {
    pub fn new() -> Self {
        NonDeterministicAutomaton {
            states: HashSet::new(),
            alphabet: HashSet::new(),
            start_states: HashSet::new(),
            terminal_states: HashSet::new(),
            transitions: HashMap::new(),
        }
    }

    pub fn add_transition(&mut self, from: String, symbol: String, to: String) {
        self.transitions
            .entry((from, symbol))
            .or_insert_with(HashSet::new)
            .insert(to);
    }

    #[doc = r"* Gyorgy Matyas
    * gmim2236
    * 1.B.04
    * pda"]
    pub fn to_deterministic(&self) -> DeterministicAutomaton{
        let mut dfa = DeterministicAutomaton::new();
        let transition_map = self.transitions.clone();

        // we need btreeset for hashing
        let mut dfa_state_index: HashMap<BTreeSet<String>, i32> = HashMap::new();
        let mut dfa_states = VecDeque::new();
        let mut dfa_transition_map: HashMap<(i32, String), i32> = HashMap::new();

        let mut state_counter = 0;
        
        let dfa_start_state: BTreeSet<String> = self.start_states.iter().cloned().collect();
        dfa_states.push_back(dfa_start_state.clone());
        dfa_state_index.insert(dfa_start_state.clone(), state_counter);
        
        state_counter += 1;
        while let Some(current_dfa_state) = dfa_states.pop_front() {
            let current_state_id = dfa_state_index[&current_dfa_state];

            for symbol in &self.alphabet {
                let mut next_state_set = BTreeSet::new();

                for state in &current_dfa_state {
                    if let Some(next_states) = transition_map.get(&(state.clone(), symbol.clone())) {
                        next_state_set.extend(next_states.clone());
                    }
                }

                if !next_state_set.is_empty() {
                    // check if the new state set is indexed
                    let next_state_id = if let Some(&id) = dfa_state_index.get(&next_state_set) {
                        id
                    } else {
                        dfa_state_index.insert(next_state_set.clone(), state_counter);
                        dfa_states.push_back(next_state_set.clone());
                        state_counter += 1;
                        state_counter - 1
                    };
                    
                    // add transition
                    dfa_transition_map.insert((current_state_id, symbol.clone()), next_state_id);
                }
            }
        }

        // collect dfa start state (should be only one)
        for (state_set, index) in &dfa_state_index {
            if state_set.iter().any(|s| self.terminal_states.contains(s)) {
                dfa.terminal_states.insert(index.to_string());
            }
        }

        // set up dfa
        dfa.start_states.insert(dfa_state_index[&dfa_start_state].to_string());
        dfa.transitions = dfa_transition_map.into_iter()
            .map(|((from, symbol), to)| ((from.to_string(), symbol), vec![to.to_string()].into_iter().collect()))
            .collect();
        
        for ((source, _), target) in &dfa.transitions {
            dfa.states.insert(source.clone());
            dfa.states.insert(target.clone());
        }
        dfa.alphabet = self.alphabet.clone();

        dfa
    }

    #[doc = r"* Gyorgy Matyas
    * gmim2236
    * 1.A.04"]
    pub fn accepts(&self, word: &str) -> bool {
        fn accepts_helper(
            current_states: &HashSet<String>,
            terminal_states: &HashSet<String>,
            transitions: &HashMap<(String, String), HashSet<String>>,
            word: &str,
            index: usize,
            iteration: usize,
        ) -> bool {
            const MAX_ITERATION: usize = 1000;
            if index == word.len() {
                return current_states.iter().any(|state| terminal_states.contains(state));
            }

            if iteration == MAX_ITERATION {
                return false;
            }

            let symbol = &word[index..index + 1];
            let mut next_states = HashSet::new();
            let mut step = 1;

            for state in current_states {
                if let Some(next) = transitions.get(&(state.clone(), symbol.to_string())) {
                    next_states.extend(next.clone());
                }
                if let Some(epsilon_next) = transitions.get(&(state.clone(), "eps".to_string())) {
                    next_states.extend(epsilon_next.clone());
                    step = 0;
                }
            }

            if next_states.is_empty() {
                return false;
            }

            accepts_helper(&next_states, terminal_states, transitions, word, index + step, iteration + 1)
        }

        let start_set = &self.start_states;
        let terminal_set = &self.terminal_states;

        accepts_helper(start_set, terminal_set, &self.transitions, word, 0, 0)
    }

    pub fn try_all_words(&self, words_list: Vec<String>) {
        for word in &words_list {
            if self.accepts(word) {
                println!("{:?} accepted", word);
            } else {
                println!("{:?} declined", word);
            }
        }
    }

    pub fn remove_unreachable_states(&mut self) {
        let mut reachable_nodes: HashSet<String> = HashSet::new();
        let mut productive_nodes: HashSet<String> = HashSet::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue = VecDeque::new();

        // marking all start states as reachable
        for start_state in &self.start_states {
            queue.push_back(start_state.clone());
            visited.insert(start_state.clone());
        }

        // find all reachable nodes
        while let Some(current_node) = queue.pop_front() {
            reachable_nodes.insert(current_node.clone());
            
            for ((from, _), to_states) in &self.transitions {
                if from == &current_node {
                    for to in to_states {
                        if visited.insert(to.clone()) {
                            queue.push_back(to.clone());
                        }
                    }
                }
            }
        }

        // second pass: reverse bfs to seatch for the productive nodes, knowing the reachable nodes
        visited.clear();
        for terminal_state in &self.terminal_states {
            queue.push_back(terminal_state.clone());
            visited.insert(terminal_state.clone());
        }

        while let Some(current_node) = queue.pop_front() {
            if reachable_nodes.contains(&current_node) {
                productive_nodes.insert(current_node.clone());
            }
            for ((from, _), to_states) in &self.transitions {
                if to_states.contains(&current_node) && visited.insert(from.clone()) {
                    queue.push_back(from.clone());
                }
            }
        }

        // include only reachable and productive
        self.states = &reachable_nodes & &productive_nodes;
        self.terminal_states.retain(|state| productive_nodes.contains(state));
        self.transitions.retain(|(from, _), to_states| reachable_nodes.contains(from) && to_states.iter().all(|to| productive_nodes.contains(to)));
    }
}

impl Automaton for NonDeterministicAutomaton {
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
        for ((from, symbol), to_states) in &self.transitions {
            for to in to_states {
                let key = (from.as_str(), to.as_str());
                edge_map.entry(key).or_default().push(symbol.as_str());
            }
        }

        for ((start, end), labels) in edge_map {
            let label_str = labels.join(", ");
            out_dot_code.push_str(&format!("    {} -> {} [label=\"{}\"];\n", start, end, label_str));
        }

        out_dot_code.push_str("}\n");
        out_dot_code
    }

    fn build_from_file(&mut self, file_name: &str) -> io::Result<()> {
        let path = Path::new(file_name);
        let file = File::open(&path)?;
        let lines = io::BufReader::new(file).lines().collect::<Result<Vec<String>, _>>()?;

        self.states = lines[0].split_whitespace().map(String::from).collect();
        self.alphabet = lines[1].split_whitespace().map(String::from).collect();
        self.start_states = lines[2].split_whitespace().map(String::from).collect();
        self.terminal_states = lines[3].split_whitespace().map(String::from).collect();
        self.transitions.clear();
        
        for line in &lines[4..] {
            let parts: Vec<String> = line.split_whitespace().map(String::from).collect();
            if parts.len() == 3 {
                self.add_transition(parts[0].clone(), parts[1].clone(), parts[2].clone());
            }
        }

        Ok(())
    }
}

impl fmt::Display for NonDeterministicAutomaton {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "States: {:?}\nAlphabet: {:?}\nStart States: {:?}\nTerminal States: {:?}\nTransitions: {:?}",
            self.states, self.alphabet, self.start_states, self.terminal_states, self.transitions
        )
    }
}