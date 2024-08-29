use salsa::Event;

#[derive(Default)]
#[salsa::db]
pub(crate) struct Database {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for Database {
    fn salsa_event(&self, _event: &dyn Fn() -> Event) {}
}
