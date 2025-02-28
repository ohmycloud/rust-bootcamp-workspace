use std::thread;
use crossbeam_channel as mpsc;
use enum_dispatch::enum_dispatch;
use reedline_repl_rs::CallBackMap;
use reedline_repl_rs::crossterm::ExecutableCommand;
use crate::cli;
use crate::cli::command::ReplCommand;
use crate::cli::connect::ConnectOpts;

pub type DisplayResult = anyhow::Result<impl ReplDisplay>;
pub type ReplCallBacks = CallBackMap<ReplContext, reedline_repl_rs::Error>;
pub trait Backend {
    async fn connect(&mut self, opts: &ConnectOpts) -> anyhow::Result<()>;
    async fn list(&self) -> DisplayResult;
    async fn schema(&self, name: &str) -> DisplayResult;
    async fn describe(&self, name: &str) -> anyhow::Result<String>;
    async fn head(&self, name: &str, size: usize) -> DisplayResult;
    async fn sql(&self, sql: &str) -> DisplayResult;
}

pub trait ReplDisplay {
    async fn display(self) -> anyhow::Result<String>;
}

#[enum_dispatch]
pub trait CmdExecutor {
    async fn execute<T: Backend>(&self, backend: &mut T) -> anyhow::Result<String>;
}

pub struct ReplMsg {
    cmd: ReplCommand,
    tx: oneshot::Sender<String>,
}

pub struct ReplContext {
    pub tx: mpsc::Sender<ReplMsg>,
}

impl ReplMsg {
    pub fn new(cmd: impl Into<ReplCommand>) -> (Self, oneshot::Receiver<String>) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                cmd: cmd.into(),
                tx,
            },
            rx
        )
    }
}

pub fn get_callbacks() -> ReplCallBacks {
    let mut callbacks = ReplCallBacks::new();
    callbacks.insert("connect".into(), cli::connect::connect);
    callbacks.insert("list".into(), cli::list::list);
    callbacks.insert("schema".into(), cli::schema::schema);
    callbacks.insert("describe".into(), cli::describe::describe);
    callbacks.insert("head".into(), cli::head::head);
    callbacks.insert("sql".into(), cli::sql::query);
    callbacks
}

impl ReplContext {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded::<ReplMsg>();
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        let mut backend = DataFusionBackend::new();
        thread::Builder::new()
            .name("ReplBackend".to_string())
            .spawn(move || {
                while let Ok(msg) = rx.recv() {
                    if let Err(e) = rt.block_on(async {
                        let ret = msg.cmd.execute(&mut backend).await?;
                        msg.tx.send(ret)?;
                        Ok::<_, anyhow::Error>(())
                    }) {
                        eprintln!("Failed to process command: {}", e);
                    }
                }
            }).unwrap();
        Self { tx }
    }

    pub fn send(&self, msg: ReplMsg, rx: oneshot::Receiver<String>) -> Option<String> {
        if let Err(e) = self.tx.send(msg) {
            eprintln!("Failed to send Error: {}", e);
            std::process::exit(1);
        }
        // If the oneshot receiver is dropped, return None, because server had an error on the command
        rx.recv().ok()
    }
}

