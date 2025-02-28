use clap::{ArgMatches, Parser};
use crate::ctx::repl::{Backend, CmdExecutor, ReplContext, ReplMsg};
use crate::ctx::ReplResult;

#[derive(Debug, Parser)]
pub struct SqlOpts {
    #[arg(help = "The SQL query to execute")]
    pub query: String,
}

pub fn query(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let query = args
        .get_one::<String>("query")
        .expect("Missing required query argument")
        .to_string();

    let (msg, rx) = ReplMsg::new(SqlOpts::new(query));
    Ok(ctx.send(msg, rx))
}

impl SqlOpts {
    pub fn new(query: String) -> Self {
        Self { query }
    }
}

impl CmdExecutor for SqlOpts {
    async fn execute<T: Backend>(&self, backend: &mut T) -> anyhow::Result<String> {
        let df = backend.sql(&self.query).await?;
        df.display().await
    }
}