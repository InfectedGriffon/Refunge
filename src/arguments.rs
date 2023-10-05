use clap::Parser;

#[derive(Parser, Debug, Default)]
pub struct Arguments {
    /// Starts the simulation paused
    #[arg(short, long)]
    pub paused: bool,

    /// Quiet mode (no interface)
    #[arg(short, long)]
    pub quiet: bool,

    /// start instruction at first non-# character
    #[arg(short, long)]
    pub script: bool,

    /// runs for this many ticks before stopping automatically
    #[arg(short, long = "max")]
    pub max_ticks: Option<u32>,

    /// immediately jump to this many ticks into the sim
    #[arg(short, long)]
    pub jump: Option<u32>,

    /// Target file
    pub file: String
}
