macro_rules! thunk {
    (async move |$interpreter:ident, $stack_frame:ident| $body:expr) => {
        $crate::thunk::Thunk::new(move |$interpreter, $stack_frame| Box::pin(async move { $body }))
    };
}
