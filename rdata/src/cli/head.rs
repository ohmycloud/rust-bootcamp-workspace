use clap::{ArgMatches, Parser};
use crate::ctx::repl::{Backend, CmdExecutor, ReplContext, ReplMsg};
use crate::ctx::ReplResult;

#[derive(Debug, Parser)]
pub struct HeadOpts {
    #[arg(help = "The name of the dataset")]
    pub name: String,
    #[arg(short, long, help = "The number of rows to show")]
    pub numbers: Option<usize>,
}

pub fn head(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let name = args
        .get_one::<String>("name")
        .expect("Missing required name argument")
        .to_string();

    let numbers = args.get_one::<usize>("numbers").copied();

    let (msg, rx) = ReplMsg::new(HeadOpts::new(name, numbers));
    Ok(ctx.send(msg, rx))
}

impl HeadOpts {
    pub fn new(name: String, numbers: Option<usize>) -> Self {
        Self { name, numbers }
    }
}

impl CmdExecutor for HeadOpts {
    async fn execute<T: Backend>(&self, backend: &mut T) -> anyhow::Result<String> {
        let df = backend.head(&self.name, self.numbers.unwrap_or(20)).await?;
        df.display().await
    }
}