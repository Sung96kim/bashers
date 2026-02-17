use anyhow::Result;
use bashers::run;

fn main() -> Result<()> {
    run(std::env::args().collect())
}
