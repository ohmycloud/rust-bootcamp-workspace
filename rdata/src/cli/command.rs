use clap::Parser;
use crate::cli::connect::ConnectOpts;
use crate::cli::describe::DescribeOpts;
use crate::cli::head::HeadOpts;
use crate::cli::list::ListOpts;
use crate::cli::schema::SchemaOpts;
use crate::cli::sql::SqlOpts;

#[derive(Debug, Parser)]
#[enum_dispatch::CmdExecutor]
pub enum ReplCommand {
    #[command(name =  "connect", about = "Connect to a dataset and register it ot rdata")]
    Connect(ConnectOpts),
    #[command(name = "list", about = "List all registered datasets")]
    List(ListOpts),
    #[command(name = "schema", about = "Describe the schema of dataset")]
    Schema(SchemaOpts),
    #[command(name = "describe", about = "Describe a dataset")]
    Describe(DescribeOpts),
    #[command(name = "head", about = "Show first few rows of a dataset")]
    Head(HeadOpts),
    #[command(name = "sql", about = "Query a dataset using given SQL")]
    Sql(SqlOpts),
}