use std::env;

use verdict::{Policy, PolicyEngine, Verdict};

fn evaluate_integer(value: &i32) -> Verdict {
    if *value > 0 {
        Verdict::Verified
    } else {
        Verdict::Unverified
    }
}

fn main() {
    let Some(input) = env::args().nth(1) else {
        println!("Usage: verdict <integer>");
        return;
    };

    let Ok(value) = input.parse::<i32>() else {
        eprintln!("Invalid integer input: {input}");
        return;
    };

    let mut policy = Policy::<i32, 1>::new();
    let _ = policy.add_rule(evaluate_integer);
    println!("{:?}", PolicyEngine::evaluate(&policy, &value));
}
