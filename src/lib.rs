#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]
pub mod cell_impl;
pub mod check;
pub mod cli;
pub mod config;
pub mod extra_keys;
pub mod files;
pub mod hooks;
pub mod install;
pub mod record;
pub mod schema;
pub mod settings;
pub mod smudge;
pub mod strip;
pub mod utils;
#[cfg(test)]
pub(crate) mod test_helpers {
    use std::sync::LazyLock;
    use std::{env::set_current_dir, path::Path, sync::Mutex};
    pub static CWD_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    pub fn with_dir<P: AsRef<Path>, T: Sized>(dir: P, f: impl FnOnce() -> T) -> T {
        let _lock = CWD_MUTEX.lock().unwrap();
        let cur_dir = crate::files::get_cwd();
        dbg!(&cur_dir);
        set_current_dir(&dir).unwrap();
        dbg!(dir.as_ref());
        let res = f();
        dbg!(dir.as_ref());
        set_current_dir(cur_dir).unwrap();
        res
    }
}
