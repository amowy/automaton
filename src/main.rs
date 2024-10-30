use automata_lib::{self, DeterministicAutomaton, PushdownAutomaton, Automaton};

fn main() -> std::io::Result<()> {
    // Example usage of both automaton types
    println!("valami");
    let mut dfa = DeterministicAutomaton::new();
    dfa.build_from_file("resources/form_I.B.1_a1.txt")?;
    dfa.write_dot_code("dfa.dot")?;

    let mut pda = PushdownAutomaton::new();
    pda.build_from_file("resources/form_I.B.2.txt")?;
    pda.write_dot_code("pda.dot")?;

    Ok(())
}