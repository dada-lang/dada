//! The "kernel" is the interface from the interpreter to the outside world.

use dada_ir::func::Function;
use parking_lot::Mutex;

#[async_trait::async_trait]
pub trait Kernel: Send + Sync {
    /// Implementation for the `print` intrinsic, that prints a line of text.
    async fn print(&self, text: &str) -> eyre::Result<()>;

    /// Prints a newline.
    async fn print_newline(&self) -> eyre::Result<()> {
        self.print("\n").await
    }
}

pub struct BufferKernel {
    buffer: Mutex<String>,
}

impl BufferKernel {
    pub fn new() -> Self {
        Self {
            buffer: Default::default(),
        }
    }

    pub async fn interpret(&self, db: &dyn crate::Db, function: Function) -> eyre::Result<()> {
        crate::interpret(function, db, self).await
    }

    pub async fn interpret_and_buffer(&self, db: &dyn crate::Db, function: Function) {
        match crate::interpret(function, db, self).await {
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
