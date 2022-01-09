//! The "kernel" is the interface from the interpreter to the outside world.

use dada_ir::func::Function;
use parking_lot::Mutex;

use crate::value::Value;

#[async_trait::async_trait]
pub trait Kernel: Send + Sync {
    /// Implementation for the `print` intrinsic, that prints a line of text.
    async fn print(&self, text: &str) -> eyre::Result<()>;

    /// Prints a newline.
    async fn print_newline(&self) -> eyre::Result<()> {
        self.print("\n").await
    }
}

#[derive(Default)]
pub struct BufferKernel {
    buffer: Mutex<String>,
}

impl BufferKernel {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn interpret(
        &self,
        db: &dyn crate::Db,
        function: Function,
        arguments: Vec<Value>,
    ) -> eyre::Result<()> {
        crate::interpret(function, db, self, arguments).await
    }

    pub async fn interpret_and_buffer(
        &self,
        db: &dyn crate::Db,
        function: Function,
        arguments: Vec<Value>,
    ) {
        match crate::interpret(function, db, self, arguments).await {
            Ok(()) => {}
            Err(e) => {
                self.buffer.lock().push_str(&e.to_string());
            }
        }
    }

    pub fn into_buffer(self) -> String {
        Mutex::into_inner(self.buffer)
    }
}

#[async_trait::async_trait]
impl Kernel for BufferKernel {
    async fn print(&self, message: &str) -> eyre::Result<()> {
        self.buffer.lock().push_str(message);
        Ok(())
    }
}
