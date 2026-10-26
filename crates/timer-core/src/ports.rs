use crate::{Session, Snapshot};
/// Durable state and completed sessions must be written in one transaction.
pub trait Repository {
    type Error;
    fn load_checkpoint(&self) -> Result<Option<Snapshot>, Self::Error>;
    fn persist(&mut self, state: &Snapshot, completed: &[Session]) -> Result<(), Self::Error>;
    fn sessions(&self) -> Result<Vec<Session>, Self::Error>;
    fn delete_session(&mut self, id: &str) -> Result<(), Self::Error>;
    fn update_session(&mut self, session: &Session) -> Result<(), Self::Error>;
}
