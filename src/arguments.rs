#[derive(clap::Parser, Default)]
pub struct Arguments {
    /// run in quiet mode (no tui)
    #[arg(short, long)]
    pub quiet: bool,
    /// start on the first non-# line
    #[arg(short, long)]
    pub script: bool,
    /// Target file
    pub file: String,

    /// start interpretation paused
    #[arg(short, long, conflicts_with="quiet")]
    pub paused: bool,
    /// jump many ticks before starting tui
    #[arg(short, long, conflicts_with="quiet")]
    pub jump: Option<u32>,

    /// log the stack(s) after ending
    #[arg(short, long, requires="quiet")]
    pub log_stack: bool,
    /// end interpreting early
    #[arg(short, long, requires="quiet")]
    pub max_ticks: Option<u32>,
}
