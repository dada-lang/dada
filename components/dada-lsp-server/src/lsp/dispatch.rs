use std::{marker::PhantomData, ops::ControlFlow, sync::Arc, thread::Scope};

use dada_util::Fallible;
use lsp_server::{Connection, Message, Notification};
use lsp_types::notification;

use super::Editor;

pub(super) struct LspDispatch<'l, L: 'l> {
    connection: Arc<Connection>,
    lsp: L,
    notification_arms: Vec<Box<dyn NotificationArm<L> + 'l>>,
}

trait NotificationArm<C> {
    fn execute(
        &self,
        context: &mut C,
        editor: &dyn Editor,
        notification: Notification,
    ) -> Fallible<ControlFlow<(), Notification>>;
}

impl<'l, L: 'l> LspDispatch<'l, L> {
    pub fn new(connection: Connection, lsp: L) -> Self {
        Self {
            lsp,
            connection: Arc::new(connection),
            notification_arms: vec![],
        }
    }

    pub fn on_notification<N>(
        mut self,
        execute: impl Fn(&mut L, &dyn Editor, N::Params) -> Fallible<()> + 'l,
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
            F: Fn(&mut L, &dyn Editor, N::Params) -> Fallible<()>,
        {
            fn execute(
                &self,
                lsp: &mut L,
                editor: &dyn Editor,
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
        std::thread::scope(|scope| {
            let connection = self.connection.clone();
            for message in &connection.receiver {
                // Check for shutdown requests:
                if let Message::Request(req) = &message {
                    if self.connection.handle_shutdown(req)? {
                        break;
                    }
                }

                // Otherwise:
                self.receive(scope, message)?;
            }

            Ok(())
        })
    }

    fn receive(&mut self, _scope: &Scope<'_, '_>, message: Message) -> Fallible<()> {
        match message {
            Message::Request(_request) => Ok(()),
            Message::Response(_response) => Ok(()),
            Message::Notification(mut notification) => {
                let editor = LspDispatchEditor {
                    connection: &self.connection,
                };
                for arm in &self.notification_arms {
                    match arm.execute(&mut self.lsp, &editor, notification)? {
                        ControlFlow::Break(()) => break,
                        ControlFlow::Continue(n) => notification = n,
                    }
                }
                Ok(())
            }
        }
    }
}

struct LspDispatchEditor<'scope> {
    connection: &'scope Connection,
}

impl LspDispatchEditor<'_> {
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

impl Editor for LspDispatchEditor<'_> {
    fn show_message(&self, message_type: lsp_types::MessageType, message: String) -> Fallible<()> {
        let params = lsp_types::ShowMessageParams {
            typ: message_type,
            message,
        };

        self.send_notification::<notification::ShowMessage>(params)?;

        Ok(())
    }
}
