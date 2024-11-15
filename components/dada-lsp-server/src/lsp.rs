use std::{
    marker::PhantomData,
    ops::ControlFlow,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::Scope,
};

use dada_util::Fallible;
use lsp_server::{Connection, Message, Notification};
use lsp_types::{notification, InitializeParams, InitializeResult, ServerCapabilities, ServerInfo};

/// LSP server handlers.
pub trait Lsp: Sized {
    /// The server is "forked" to handle incoming "read" requests (e.g., goto-def).
    /// "Read" requests are requests that do not modify document state.
    type Fork: LspFork;

    fn run() -> Fallible<()> {
        run_server::<Self>()
    }

    fn new(editor: Arc<dyn Editor>, params: InitializeParams) -> Fallible<Self>;

    fn server_capabilities(&mut self) -> Fallible<ServerCapabilities>;
    fn server_info(&mut self) -> Fallible<Option<ServerInfo>>;

    #[expect(dead_code)]
    fn fork(&mut self) -> Self::Fork;

    fn did_open(&mut self, item: lsp_types::DidOpenTextDocumentParams) -> Fallible<()>;
    fn did_change(&mut self, item: lsp_types::DidChangeTextDocumentParams) -> Fallible<()>;
}

#[expect(dead_code)]
pub trait LspFork: Sized {}

pub trait Editor {
    fn show_message(&self, message_type: lsp_types::MessageType, message: String) -> Fallible<()>;
}

impl Editor for Connection {
    fn show_message(&self, message_type: lsp_types::MessageType, message: String) -> Fallible<()> {
        let params = lsp_types::ShowMessageParams {
            typ: message_type,
            message,
        };

        self.send_notification::<notification::ShowMessage>(params)?;

        Ok(())
    }
}

trait EditorHelp {
    fn send_notification<N>(&self, params: N::Params) -> Fallible<()>
    where
        N: notification::Notification;
}

impl EditorHelp for Connection {
    fn send_notification<N>(&self, params: N::Params) -> Fallible<()>
    where
        N: notification::Notification,
    {
        self.sender.send(Message::Notification(Notification::new(
            N::METHOD.to_string(),
            params,
        )))?;
        Ok(())
    }
}

pub fn run_server<L: Lsp>() -> Fallible<()> {
    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    let connection = Arc::new(connection);
    let lsp = initialize_server::<L>(&connection)?;
    main_loop(&connection, lsp)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

static CANCEL: AtomicBool = AtomicBool::new(false);

fn not_canceled() -> bool {
    !CANCEL.load(Ordering::Relaxed)
}

fn initialize_server<L: Lsp>(connection: &Arc<Connection>) -> Fallible<L> {
    let connection = Arc::clone(connection);
    let (initialize_id, initialize_params) = connection.initialize_start_while(not_canceled)?;
    let initialize_params: InitializeParams = serde_json::from_value(initialize_params)?;

    let mut server = L::new(connection.clone(), initialize_params)?;

    let initialize_result = InitializeResult {
        capabilities: server.server_capabilities()?,
        server_info: server.server_info()?,
    };

    connection.initialize_finish_while(
        initialize_id,
        serde_json::to_value(initialize_result)?,
        not_canceled,
    )?;

    Ok(server)
}

fn main_loop<L: Lsp>(connection: &Arc<Connection>, lsp: L) -> Fallible<()> {
    std::thread::scope(|scope| {
        let mut notification_cast = Cast::new(scope, lsp)
            .on_notification::<notification::DidOpenTextDocument>(Lsp::did_open)
            .on_notification::<notification::DidChangeTextDocument>(Lsp::did_change);

        for message in &connection.receiver {
            // Check for shutdown requests:
            if let Message::Request(req) = &message {
                if connection.handle_shutdown(req)? {
                    break;
                }
            }

            // Otherwise:
            notification_cast.receive(message)?;
        }

        Ok(())
    })
}

struct Cast<'scope, 'env, C: 'scope> {
    #[expect(dead_code)]
    scope: &'scope Scope<'scope, 'env>,
    context: C,
    notification_arms: Vec<Box<dyn NotificationArm<C> + 'scope>>,
}

trait NotificationArm<C> {
    fn execute(
        &self,
        context: &mut C,
        notification: Notification,
    ) -> Fallible<ControlFlow<(), Notification>>;
}

impl<'env, 'scope, L: 'scope> Cast<'scope, 'env, L> {
    fn new(scope: &'scope Scope<'scope, 'env>, lsp: L) -> Self {
        Self {
            scope,
            context: lsp,
            notification_arms: vec![],
        }
    }

    fn on_notification<N>(
        mut self,
        execute: impl Fn(&mut L, N::Params) -> Fallible<()> + 'scope,
    ) -> Self
    where
        N: notification::Notification + 'static,
    {
        struct NotificationArmImpl<N, F, L> {
            notification: PhantomData<(N, L)>,
            execute: F,
        }

        impl<L, N, F> NotificationArm<L> for NotificationArmImpl<N, F, L>
        where
            N: notification::Notification,
            F: Fn(&mut L, N::Params) -> Fallible<()>,
        {
            fn execute(
                &self,
                lsp: &mut L,
                notification: Notification,
            ) -> Fallible<ControlFlow<(), Notification>> {
                if notification.method != N::METHOD {
                    return Ok(ControlFlow::Continue(notification));
                }

                let params: N::Params = serde_json::from_value(notification.params)?;
                (self.execute)(lsp, params)?;

                Ok(ControlFlow::Break(()))
            }
        }

        self.notification_arms.push(Box::new(NotificationArmImpl {
            notification: PhantomData::<(N, L)>,
            execute,
        }));

        self
    }

    fn receive(&mut self, message: Message) -> Fallible<()> {
        match message {
            Message::Request(_request) => Ok(()),
            Message::Response(_response) => Ok(()),
            Message::Notification(mut notification) => {
                for arm in &self.notification_arms {
                    match arm.execute(&mut self.context, notification)? {
                        ControlFlow::Break(()) => break,
                        ControlFlow::Continue(n) => notification = n,
                    }
                }
                Ok(())
            }
        }
    }
}
