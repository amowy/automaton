use automata_lib::{self, NonDeterministicAutomaton, PushdownAutomaton, Automaton, DeterministicAutomaton};
use std::io::Result;


fn fel_01() -> Result<()>{
    println!("fel 01");
    let mut dfa1 = DeterministicAutomaton::new();
    let mut dfa2 = DeterministicAutomaton::new();
    println!("A");
    dfa1.build_from_file("resources/fel01/form_I.B.1_a1.txt")?;
    dfa2.build_from_file("resources/fel01/form_I.B.1_a2.txt")?;
    dfa1.write_dot_code("output/fel01/dfa1_a.dot")?;
    dfa2.write_dot_code("output/fel01/dfa2_a.dot")?;
    if dfa1 == dfa2 {
        println!("equals");
    } else {
        println!("not equals");
    }

    println!("B");
    dfa1.build_from_file("resources/fel01/form_I.B.1_b1.txt")?;
    dfa2.build_from_file("resources/fel01/form_I.B.1_b2.txt")?;
    dfa1.write_dot_code("output/fel01/dfa1_b.dot")?;
    dfa2.write_dot_code("output/fel01/dfa2_b.dot")?;
    if dfa1 == dfa2 {
        println!("equals");
    } else {
        println!("not equals");
    }

    println!("C");
    dfa1.build_from_file("resources/fel01/form_I.B.1_c1.txt")?;
    dfa2.build_from_file("resources/fel01/form_I.B.1_c2.txt")?;
    dfa1.write_dot_code("output/fel01/dfa1_c.dot")?;
    dfa2.write_dot_code("output/fel01/dfa2_c.dot")?;
    if dfa1 == dfa2 {
        println!("equals");
    } else {
        println!("not equals");
    }

    Ok(())
}

fn fel_02() -> Result<()>{
    println!("fel 02");
    println!("A");
    let mut pda = PushdownAutomaton::new();
    pda.build_from_file("resources/fel02/form_I.B.2.txt")?;
    pda.write_dot_code("output/fel02/pda.dot")?;
    pda.check_for_file("resources/fel02/form_I.B.2_szavak.txt")?;

    println!("B");
    pda.build_from_file("resources/fel02/form_I.B.2_b.txt")?;
    pda.write_dot_code("output/fel02/pda_b.dot")?;
    pda.check_for_file("resources/fel02/form_I.B.2_b_szavak.txt")?;

    Ok(())
}

fn fel_03() -> Result<()>{
    println!("fel 03");
    let mut dfa = DeterministicAutomaton::new();
    println!("A");

    dfa.build_from_file("resources/fel03/form_I.B.3.txt")?;
    dfa.write_dot_code("output/fel03/dfa_a.dot")?;
    let (res, minimized_dfa) = dfa.is_minimized();
    if res {
        println!("its minimalized");
        minimized_dfa.write_dot_code("output/fel03/dfa_minimized_a.dot")?;
    } else {
        minimized_dfa.write_dot_code("output/fel03/dfa_minimized_a.dot")?;
    }

    println!("B");

    dfa.build_from_file("resources/fel03/form_I.B.3_b.txt")?;
    dfa.write_dot_code("output/fel03/dfa_b.dot")?;
    let (res, minimized_dfa) = dfa.is_minimized();
    if res {
        println!("its minimalized");
        minimized_dfa.write_dot_code("output/fel03/dfa_minimized_b.dot")?;
    } else {
        minimized_dfa.write_dot_code("output/fel03/dfa_minimized_b.dot")?;
    }

    Ok(())
}

fn fel_04() -> Result<()>{
    println!("fel 04");
    let mut ndfa = NonDeterministicAutomaton::new();
    println!("A");

    ndfa.build_from_file("resources/fel04/form_I.B.4.txt")?;
    ndfa.write_dot_code("output/fel04/ndfa_a.dot")?;
    let mut dfa = ndfa.to_deterministic();
    dfa.write_dot_code("output/fel04/dfa_a.dot")?;

    println!("B");

    ndfa.build_from_file("resources/fel04/form_I.B.4_b.txt")?;
    ndfa.write_dot_code("output/fel04/ndfa_b.dot")?;
    dfa = ndfa.to_deterministic();
    dfa.write_dot_code("output/fel04/dfa_b.dot")?;

    Ok(())
}

fn main() -> Result<()> {
    fel_01()?;
    println!();

    fel_02()?;
    println!();
    fel_03()?;
    println!();
    fel_04()
}