use std::io::{self, Write};
use crate::query::QueryExecutor;
use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Welcome to Whitematter CLI");
    println!("Enter your queries or type 'exit' to quit");

    let query_executor = create_query_executor()?;

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        let result = tokio::runtime::Runtime::new()?.block_on(query_executor.execute(input))?;
        println!("{}", result);
    }

    println!("Goodbye!");
    Ok(())
}

fn create_query_executor() -> Result<QueryExecutor> {
    // Initialize all components and create a QueryExecutor
    // This is a placeholder and should be implemented with actual component initialization
    unimplemented!()
}