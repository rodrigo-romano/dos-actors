/// Defines the data type associated with [Data] unique identifier type
pub trait UniqueIdentifier: Send + Sync {
    type Data;
}
