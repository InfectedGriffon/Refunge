use clap::Parser;

#[derive(Parser, Debug, Default)]
pub struct Arguments {
    /// Starts the simulation paused
    #[arg(short = 'p')]
    pub start_paused: bool,

    /// Allow "put" commands to expand the grid
    #[arg(short)]
    pub expand: bool,

    /// Target file
    pub file: String
}
