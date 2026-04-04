use super::db::Database;

pub fn allocate_user_seq(db: &Database, account_id: &str) -> Option<u64> {
    db.allocate_account_seq(account_id)
}

pub fn allocate_session_seq(db: &Database, session_id: &str) -> Option<u64> {
    db.allocate_session_seq(session_id)
}

pub fn allocate_session_seq_batch(
    db: &Database,
    session_id: &str,
    count: usize,
) -> Option<Vec<u64>> {
    db.allocate_session_seq_batch(session_id, count)
}
