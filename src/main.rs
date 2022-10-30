use runtime::tui;
use structopt::StructOpt;
use strum::EnumString;

mod filesystem;
mod memfs;
mod runtime;

#[derive(Debug, StructOpt, EnumString)]
#[structopt(help = "Runtime exposing the FS")]
enum Runtime {
    #[structopt(help = "Simple shell for traversing the filesystem")]
    Tui,
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long)]
    runtime: Runtime,
}

fn main() {
    let opt = Opt::from_args();
    match opt.runtime {
        Runtime::Tui => {
            let mut rt = tui::Tui::new(memfs::MemFs::new());

            rt.run();
        }
    }
}
