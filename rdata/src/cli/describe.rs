use clap::{ArgMatches, Parser};
use crate::ctx::repl::{Backend, CmdExecutor, ReplContext, ReplMsg};
use crate::ctx::ReplResult;

#[derive(Debug, Parser)]
pub struct DescribeOpts {
    #[arg(help = "The name of the dataset")]
    pub name: String,
}

pub fn describe(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let name = args
        .get_one::<String>("name")
        .expect("Missing required name argument")
        .to_string();

    let (msg, rx) = ReplMsg::new(DescribeOpts::new(name));
    Ok(ctx.send(msg, rx))
}

impl DescribeOpts {
    pub fn new(name: String) -> Self {
        Self { name, }
    }
}

impl CmdExecutor for DescribeOpts {
    async fn execute<T: Backend>(&self, backend: &mut T) -> anyhow::Result<String> {
        let df = backend.schema(&self.name).await?;
        df.display().await
    }
}