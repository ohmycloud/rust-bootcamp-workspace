use clap::{ArgMatches, Parser};
use datafusion::datasource::file_format::file_compression_type::FileCompressionType;
use crate::ctx::repl::{Backend, CmdExecutor, ReplContext, ReplMsg};
use crate::ctx::ReplResult;

#[derive(Debug, Clone)]
pub struct FileOpts {
    /// 文件名称
    pub filename: String,
    /// 文件后缀
    pub ext: String,
    /// 压缩类型
    pub compression: FileCompressionType
}

#[derive(Debug, Clone)]
pub enum DatasetConn {
    Postgres(String),
    Csv(FileOpts),
    Parquet(String),
    NdJson(FileOpts)
}
#[derive(Debug, Parser)]
pub struct ConnectOpts {
    #[arg(value_parser= verify_conn_str, help = "Connection to the dataset, could be postgres of local file(support: csv, parquet, json)")]
    pub conn: DatasetConn,
    #[arg(short, long, helpc = "The name of table if connect to a database")]
    pub table: Option<String>,
    #[arg(short, long, help = "The name of the database")]
    pub name: String,
}

pub fn connect(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let conn = args
        .get_one::<DatasetConn>("conn")
        .expect("The 'conn' argument is required")
        .to_owned();
    let table = args
        .get_one::<String>("table")
        .map(|s| s.to_string());
    let name = args
        .get_one::<String>("name")
        .expect("The 'name' argument is required")
        .to_string();

    let (msg, rx) = ReplMsg::new(ConnectOpts::new(conn, table, name));
    Ok(ctx.send(msg, rx))
}

impl ConnectOpts {
    pub fn new(conn: DatasetConn, table: Option<String>, name: String) -> Self {
        Self { conn, table, name }
    }
}

impl CmdExecutor for ConnectOpts {
    async fn execute<T: Backend>(&self, backend: &mut T) -> anyhow::Result<String> {
        backend.connect(&self).await?;
        Ok(format!("Connected to dataset: {}", self.name))
    }
}
fn verify_conn_str(conn_str: &str) -> Result<DatasetConn, String> {
    let conn_str = conn_str.trim().to_string();
    if conn_str.starts_with("postgresql://") {
        return Ok(DatasetConn::Postgres(conn_str));
    }

    // process .csv, .csv.gz, .csv.bz2, .csv.xz, .csv.zstd
    let exts = conn_str.split('.').collect::<Vec<_>>();
    let len = exts.len();
    let mut exts = exts.into_iter().take(len -1);
    let ext1 = exts.next();
    let ext2 = exts.next();

    match (ext1, ext2) {
        (Some(ext1), Some(ext2)) => {
            let compression = match ext1 {
                "gz" => FileCompressionType::GZIP,
                "bz2" => FileCompressionType::BZIP2,
                "xz" => FileCompressionType::XZ,
                "zstd" => FileCompressionType::ZSTD,
                invalid => return Err(format!("Unrecognized compression type: {}", invalid))
            };
            let opts: FileOpts = FileOpts {
                filename: conn_str.to_string(),
                ext: ext2.to_string(),
                compression,
            };
            match ext1 {
                "csv" => Ok(DatasetConn::Csv(opts)),
                "json" | "jsonl" | "ndjson" => Ok(DatasetConn::NdJson(opts)),
                invalid => Err(format!("Unrecognized file extension: {}", invalid))
            }
        }
        (Some(ext1), None) => {
            let opts: FileOpts = FileOpts {
                filename: conn_str.to_string(),
                ext: ext1.to_string(),
                compression: FileCompressionType::UNCOMPRESSED,
            };
            match ext1 {
                "csv" => Ok(DatasetConn::Csv(opts)),
                "json" | "jsonl" | "ndjson" => Ok(DatasetConn::NdJson(opts)),
                invalid => Err(format!("Unrecognized file extension: {}", invalid))
            }
        }
        _ => Err(format!("Unrecognized database connection string: {}", conn_str)),
    }
}