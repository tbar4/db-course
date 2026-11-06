use anyhow::result;
use std::path::Path;

pub trait Engine: Sized {
    fn open(path: &Path) -> Result<Self>;
    fn put(&mut self, k: &[u8], v: &[u8]) -> Result<()>;
    fn get(&self, k: &[u8]) -> Result<Option<Vec<u8>>>;
    fn delete(&mut self, k: &[u8]) -> Result<bool>;
    fn close(self) -> Result<()>;
}

/// One crash iteration: the workload runs in a child process, the parent
/// kills -9 at a random instant, reopens, and checks every acknowledged
/// write survived. Generic over the trait above.
pub fn crash_iteration<E: Engine>(workload: &Workload, seed: u64) -> Result<()> {
    let dir = tempfile::tempdir()?;
    let acked = run_child_until_killed::<E>(&dir, workload, seed)?;
    let store = E::open(&dir.path())?;
    for (k, v) in acked {
        assert_eq!(
            store.get(&k)?.as_deref(),
            Some(v.as_slice()),
            "lost acked write"
        );
    }
    Ok(())
}
