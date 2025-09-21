use clap::{Parser, Subcommand};
use dotenv::dotenv;
use sqlx::PgPool;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "db-helper")]
#[command(about = "A database helper tool for PostgreSQL with sqlx")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to migrations directory
    #[arg(short, long, global = true, default_value = "migrations")]
    migrations_path: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Reset database (drop, create, migrate, seed)
    Reset {
        /// Database name
        #[arg(short, long)]
        database: String,
        /// SQL file path for test data
        #[arg(short, long)]
        sql_file: Option<String>,
    },
    /// Drop database
    Drop {
        /// Database name
        #[arg(short, long)]
        database: String,
    },
    /// Create database and run migrations
    Setup {
        /// Database name
        #[arg(short, long)]
        database: String,
    },
    /// Seed database with test data
    Seed {
        /// Database name
        #[arg(short, long)]
        database: String,
        /// SQL file path
        #[arg(short, long)]
        sql_file: String,
    },
}

struct DatabaseHelper {
    base_url: String,
}

impl DatabaseHelper {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

        // 从 DATABASE_URL 中提取基础连接信息（去掉数据库名）
        let base_url = if let Some(pos) = database_url.rfind('/') {
            database_url[..pos].to_string()
        } else {
            return Err("Invalid DATABASE_URL format".into());
        };

        Ok(Self { base_url })
    }

    fn get_database_url(&self, db_name: &str) -> String {
        format!("{}/{}", self.base_url, db_name)
    }

    fn run_sqlx_command(
        &self,
        args: &[&str],
        db_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database_url = self.get_database_url(db_name);

        let mut cmd = Command::new("sqlx");
        cmd.args(args).env("DATABASE_URL", &database_url);

        println!("执行命令: sqlx {}", args.join(" "));

        let output = cmd.output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("{}", stdout);
            }
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("命令执行失败: {}", stderr);
            Err(format!("sqlx 命令失败: {}", stderr).into())
        }
    }

    fn drop_database(&self, db_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("正在删除数据库: {}", db_name);
        self.run_sqlx_command(&["database", "drop", "-y"], db_name)?;
        println!("数据库 {} 删除成功", db_name);
        Ok(())
    }

    fn create_database(&self, db_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("正在创建数据库: {}", db_name);
        self.run_sqlx_command(&["database", "create"], db_name)?;
        println!("数据库 {} 创建成功", db_name);
        Ok(())
    }

    fn search_and_run_migrations(&self, db_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_dir = env::current_dir()?;

        // 最多向上搜索 3 层目录
        for level in 0..3 {
            let migrations_path = current_dir.join("migrations");
            println!("搜索第 {} 层: {:?}", level + 1, migrations_path);

            if migrations_path.exists() {
                println!("找到 migrations 目录: {:?}", migrations_path);
                let migrations_str = migrations_path.to_string_lossy();
                self.run_sqlx_command(&["migrate", "run", "--source", &migrations_str], db_name)?;
                println!("迁移执行成功");
                return Ok(());
            }

            // 向上一层目录
            if !current_dir.pop() {
                break; // 已经到达根目录
            }
        }

        Err("未找到 migrations 目录，已搜索 5 层父目录".into())
    }

    fn run_migrations(
        &self,
        db_name: &str,
        migrations_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("正在运行数据库迁移...");
        println!("当前工作目录: {:?}", env::current_dir()?);
        println!("指定的 migrations 路径: {}", migrations_path);

        let migrations_path_buf = PathBuf::from(migrations_path);
        let absolute_path = if migrations_path_buf.is_absolute() {
            migrations_path_buf
        } else {
            env::current_dir()?.join(migrations_path_buf)
        };

        println!("解析后的绝对路径: {:?}", absolute_path);

        // 检查路径是否存在
        if !absolute_path.exists() {
            // 如果用户指定的路径不存在，且是默认的 "migrations"，尝试自动搜索
            if migrations_path == "migrations" {
                println!("默认 migrations 目录不存在，尝试向上搜索...");
                return self.search_and_run_migrations(db_name);
            } else {
                return Err(format!(
                    "指定的 migrations 目录不存在: {} (绝对路径: {:?})",
                    migrations_path, absolute_path
                )
                .into());
            }
        }

        println!("使用 migrations 目录: {:?}", absolute_path);
        let migrations_str = absolute_path.to_string_lossy();
        self.run_sqlx_command(&["migrate", "run", "--source", &migrations_str], db_name)?;
        println!("迁移执行成功");
        Ok(())
    }

    fn setup_database(
        &self,
        db_name: &str,
        migrations_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("正在设置数据库: {}", db_name);

        // 创建数据库并运行迁移
        self.create_database(db_name)?;
        self.run_migrations(db_name, migrations_path)?;

        println!("数据库 {} 设置完成！", db_name);
        Ok(())
    }

    async fn seed_database(
        &self,
        db_name: &str,
        sql_file: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("正在向数据库导入测试数据: {}", sql_file);

        if !Path::new(sql_file).exists() {
            return Err(format!("SQL 文件不存在: {}", sql_file).into());
        }

        let database_url = self.get_database_url(db_name);
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database pool");

        let sql_content = fs::read_to_string(sql_file)?;

        // 直接执行整个 SQL 文件内容
        match sqlx::raw_sql(&sql_content).execute(&pool).await {
            Ok(result) => {
                println!("测试数据导入成功，影响了 {} 行", result.rows_affected());
            }
            Err(e) => {
                eprintln!("批量执行失败: {}", e);
                eprintln!("尝试逐行执行...");

                // 如果批量执行失败，尝试按语句分割执行
                let mut success_count = 0;
                for (i, statement) in sql_content.split(';').enumerate() {
                    let statement = statement.trim();
                    if !statement.is_empty() && !statement.starts_with("--") {
                        match sqlx::query(statement).execute(&pool).await {
                            Ok(result) => {
                                println!(
                                    "语句 {} 执行成功，影响了 {} 行",
                                    i + 1,
                                    result.rows_affected()
                                );
                                success_count += 1;
                            }
                            Err(e) => {
                                eprintln!("语句 {} 执行失败: {}", i + 1, e);
                                eprintln!("语句内容: {}", statement);
                                pool.close().await;
                                return Err(e.into());
                            }
                        }
                    }
                }
                println!("成功执行了 {} 条语句", success_count);
            }
        }

        pool.close().await;
        println!("测试数据导入完成");
        Ok(())
    }

    async fn reset_database(
        &self,
        db_name: &str,
        sql_file: Option<&str>,
        migrations_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("正在重置数据库: {}", db_name);

        // 1. 删除数据库（忽略错误，可能数据库不存在）
        if let Err(e) = self.drop_database(db_name) {
            println!("删除数据库时出现警告: {} (可能数据库不存在)", e);
        }

        // 2. 创建数据库并运行迁移
        self.setup_database(db_name, migrations_path)?;

        // 3. 导入测试数据（如果提供）
        if let Some(sql_file) = sql_file {
            self.seed_database(db_name, sql_file).await?;
        }

        println!("数据库 {} 重置完成！", db_name);
        Ok(())
    }

    fn check_sqlx_cli(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("sqlx").arg("--version").output();

        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("找到 sqlx-cli: {}", version.trim());
                Ok(())
            }
            _ => {
                eprintln!("错误: 未找到 sqlx CLI 工具");
                eprintln!("请安装 sqlx-cli:");
                eprintln!("  cargo install sqlx-cli --no-default-features --features postgres");
                Err("sqlx CLI 未安装".into())
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let db_helper = DatabaseHelper::new()?;

    // 检查 sqlx CLI 是否安装
    db_helper.check_sqlx_cli()?;

    match &cli.command {
        Commands::Reset { database, sql_file } => {
            db_helper
                .reset_database(database, sql_file.as_deref(), &cli.migrations_path)
                .await?;
        }
        Commands::Drop { database } => {
            db_helper.drop_database(database)?;
        }
        Commands::Setup { database } => {
            db_helper.setup_database(database, &cli.migrations_path)?;
        }
        Commands::Seed { database, sql_file } => {
            db_helper.seed_database(database, sql_file).await?;
        }
    }

    Ok(())
}
