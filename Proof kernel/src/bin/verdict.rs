use std::env;

use verdict::{evaluate_policy, Policy, PolicyEngine};

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
    let _ = policy.add_rule(|value| evaluate_policy(*value));
    println!("{:?}", PolicyEngine::evaluate(&policy, &value));
}
