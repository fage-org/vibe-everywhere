use super::db::{Database, DatabaseState};

impl Database {
    pub fn transaction<R>(&self, f: impl FnOnce(&mut DatabaseState) -> R) -> R {
        self.write(f)
    }
}
