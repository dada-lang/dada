use dada_util::Fallible;

use wasmtime::*;

pub fn run_bytes(bytes: &[u8]) -> Fallible<()> {
    let engine = Engine::default();
    let module = Module::new(&engine, bytes)?;

    let linker = Linker::new(&engine);
    let mut store: Store<()> = Store::new(&engine, ());

    let instance = linker.instantiate(&mut store, &module)?;
    let main = instance.get_typed_func::<(i32,), ()>(&mut store, "main")?;

    main.call(&mut store, (0,))?;

    Ok(())
}
