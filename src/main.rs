mod folder_scanner;
use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    folder: String,
}

fn main() {
    let args = Args::parse();
    folder_scanner::scanner::scan(&args.folder);   
}
