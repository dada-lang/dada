macro_rules! thunk {
    (async move |$interpreter:ident| $body:expr) => {
        $crate::thunk::Thunk::new(move |$interpreter| Box::pin(async move { $body }))
    };
}
