use std::{
    marker::PhantomData,
    ops::ControlFlow,
    sync::{mpsc::Sender, Arc},
    thread::Scope,
};

use dada_util::Fallible;
use lsp_server::{Connection, Message, Notification};
use lsp_types::{notification, PublishDiagnosticsParams};

use super::{Editor, Lsp};

pub(super) struct LspDispatch<'l, L: Lsp + 'l> {
    connection: Arc<Connection>,
    lsp: L,
    notification_arms: Vec<Box<dyn NotificationArm<L> + 'l>>,
}

trait NotificationArm<L> {
    fn execute(
        &self,
        context: &mut L,
        editor: &mut dyn Editor<L>,
        notification: Notification,
    ) -> Fallible<ControlFlow<(), Notification>>;
}

impl<'l, L: Lsp + 'l> LspDispatch<'l, L> {
    pub fn new(connection: Connection, lsp: L) -> Self {
        Self {
            lsp,
            connection: Arc::new(connection),
            notification_arms: vec![],
        }
    }

    pub fn on_notification<N>(
        mut self,
        execute: impl Fn(&mut L, &mut dyn Editor<L>, N::Params) -> Fallible<()> + 'l,
    ) -> Self
    where
        N: notification::Notification + 'l,
    {
        struct NotificationArmImpl<N, F, L> {
            notification: PhantomData<(N, L)>,
            execute: F,
        }

        impl<L, N, F> NotificationArm<L> for NotificationArmImpl<N, F, L>
        where
            N: notification::Notification,
            F: Fn(&mut L, &mut dyn Editor<L>, N::Params) -> Fallible<()>,
        {
            fn execute(
                &self,
                lsp: &mut L,
                editor: &mut dyn Editor<L>,
                notification: Notification,
            ) -> Fallible<ControlFlow<(), Notification>> {
                if notification.method != N::METHOD {
                    return Ok(ControlFlow::Continue(notification));
                }

                let params: N::Params = serde_json::from_value(notification.params)?;
                (self.execute)(lsp, editor, params)?;

                Ok(ControlFlow::Break(()))
            }
        }

        self.notification_arms.push(Box::new(NotificationArmImpl {
            notification: PhantomData::<(N, L)>,
            execute,
        }));

        self
    }

    /// Start receiving and dispatch messages. Blocks until a shutdown request is received.
    pub fn execute(mut self) -> Fallible<()> {
        let (spawned_tasks_tx, spawned_tasks_rx) = std::sync::mpsc::channel::<SpawnedTask<L>>();
        let (errors_tx, errors_rx) = std::sync::mpsc::channel::<dada_util::Error>();
        let connection = self.connection.clone();
        std::thread::scope(|scope| {
            for message in &connection.receiver {
                // Check for shutdown requests:
                if let Message::Request(req) = &message {
                    if self.connection.handle_shutdown(req)? {
                        break;
                    }
                }

                // Otherwise:
                self.receive(scope, spawned_tasks_tx.clone(), message)?;

                while let Ok(task) = spawned_tasks_rx.try_recv() {
                    scope.spawn({
                        let fork: <L as Lsp>::Fork = self.lsp.fork();
                        let spawned_tasks_tx = spawned_tasks_tx.clone();
                        let errors_tx = errors_tx.clone();
                        let connection = &connection;
                        move || {
                            let mut editor = LspDispatchEditor {
                                connection,
                                spawned_tasks_tx,
                            };
                            match (task.task)(&fork, &mut editor) {
                                Ok(()) => (),
                                Err(err) => errors_tx.send(err).unwrap(),
                            }
                        }
                    });
                }

                while let Ok(err) = errors_rx.try_recv() {
                    return Err(err);
                }
            }

            Ok(())
        })
    }

    /// Given a message, find the handler (if any) and invoke it.
    fn receive(
        &mut self,
        _scope: &Scope<'_, '_>,
        spawned_tasks_tx: Sender<SpawnedTask<L>>,
        message: Message,
    ) -> Fallible<()> {
        match message {
            Message::Request(_request) => Ok(()),
            Message::Response(_response) => Ok(()),
            Message::Notification(mut notification) => {
                let mut editor = LspDispatchEditor {
                    connection: &self.connection,
                    spawned_tasks_tx,
                };
                for arm in &self.notification_arms {
                    match arm.execute(&mut self.lsp, &mut editor, notification)? {
                        ControlFlow::Break(()) => break,
                        ControlFlow::Continue(n) => notification = n,
                    }
                }
                Ok(())
            }
        }
    }
}

struct LspDispatchEditor<'scope, L: Lsp> {
    connection: &'scope Connection,
    spawned_tasks_tx: Sender<SpawnedTask<L>>,
}

impl<L: Lsp> LspDispatchEditor<'_, L> {
    fn send_notification<N>(&self, params: N::Params) -> Fallible<()>
    where
        N: notification::Notification,
    {
        self.connection
            .sender
            .send(Message::Notification(Notification::new(
                N::METHOD.to_string(),
                params,
            )))?;
        Ok(())
    }
}

impl<L: Lsp> Editor<L> for LspDispatchEditor<'_, L> {
    fn show_message(
        &mut self,
        message_type: lsp_types::MessageType,
        message: String,
    ) -> Fallible<()> {
        let params = lsp_types::ShowMessageParams {
            typ: message_type,
            message,
        };

        self.send_notification::<notification::ShowMessage>(params)?;

        Ok(())
    }

    fn publish_diagnostics(&mut self, params: PublishDiagnosticsParams) -> Fallible<()> {
        Ok(self.send_notification::<notification::PublishDiagnostics>(params)?)
    }

    fn spawn(
        &mut self,
        task: Box<dyn FnOnce(&<L as Lsp>::Fork, &mut dyn Editor<L>) -> Fallible<()> + Send>,
    ) {
        self.spawned_tasks_tx.send(SpawnedTask { task }).unwrap();
    }
}

struct SpawnedTask<L: Lsp> {
    task: Box<dyn FnOnce(&<L as Lsp>::Fork, &mut dyn Editor<L>) -> Fallible<()> + Send>,
}

impl<L: Lsp> SpawnedTask<L> {}
