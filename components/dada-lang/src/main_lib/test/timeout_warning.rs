use std::{path::Path, time::Duration};

pub(super) fn timeout_warning<R>(test_path: &Path, op: impl FnOnce() -> R) -> R {
    let mut sec = 5;
    std::thread::scope(|scope| {
        let (tx, rx) = std::sync::mpsc::channel();
        scope.spawn(move || {
            loop {
                match rx.recv_timeout(Duration::from_secs(sec)) {
                    Ok(()) => return,
                    Err(_) => {
                        eprintln!("test `{test_path:?}` has been running for over {sec} seconds");
                        sec = (sec * 2).max(120);
                    }
                }
            }
        });

        let r = op();
        tx.send(()).unwrap();
        r
    })
}
