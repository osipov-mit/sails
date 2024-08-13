use clap::Parser;
use sails_cli::program::ProgramGenerator;

#[derive(Parser)]
enum Commands {
    #[command(name = "new-program")]
    NewProgram {
        #[arg(help = "Path to the new program")]
        path: String,
        #[arg(short, long, help = "Name of the new program")]
        name: Option<String>,
        #[arg(long, help = "Generate client crate alongside the program")]
        with_client: bool,
        #[arg(
            long,
            help = "Generate program tests using 'gtest'. Implies '--with-client'"
        )]
        with_gtest: bool,
    },
}

fn main() {
    let command = Commands::parse();

    let result = match command {
        Commands::NewProgram {
            path,
            name,
            with_client,
            with_gtest,
        } => {
            let program_generator = ProgramGenerator::new(path)
                .with_name(name)
                .with_client(with_client)
                .with_gtest(with_gtest);
            program_generator.generate()
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }
}