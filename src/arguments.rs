use clap::Parser;

#[derive(Parser, Debug, Default)]
pub struct Arguments {
    /// Starts the simulation paused
    #[arg(short, long)]
    pub paused: bool,

    /// Allow "put" commands to expand the grid
    #[arg(short, long)]
    pub expand: bool,

    /// Quiet mode (no interface)
    #[arg(short, long)]
    pub quiet: bool,

    /// Target file
    pub file: String
}
