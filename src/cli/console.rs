use std::io::{self, Write};
use crate::query::QueryExecutor;
use anyhow::Result;

pub fn run(query_executor: QueryExecutor) -> Result<()> {
    println!("Welcome to AI-First DB CLI");
    println!("Enter your queries or type 'exit' to quit");

    let mut user_id = String::new();
    print!("Enter your user ID: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut user_id)?;
    let user_id = user_id.trim();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        match query_executor.execute(input, user_id).await {
            Ok(result) => println!("Result: {}", result),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    println!("Goodbye!");
    Ok(())
}