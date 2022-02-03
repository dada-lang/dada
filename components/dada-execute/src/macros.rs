macro_rules! thunk {
    ($description:expr, async move |$interpreter:ident| $body:expr) => {
        $crate::thunk::RustThunk::new($description, move |$interpreter| {
            Box::pin(async move { $body })
        })
    };
}
