/// Defines the data type associated with unique identifier data type
pub trait UniqueIdentifier: Send + Sync {
    type Data;
}

pub use uid_derive::UID;

#[cfg(test)]
mod tests {
    use crate as uid;
    use uid_derive::UID;

    #[derive(UID)]
    #[uid(data = "u8")]
    pub enum A {}

    #[test]
    fn impl_uid() {
        enum U {}
        impl uid::UniqueIdentifier for U {
            type Data = f64;
        }
        let _: <U as uid::UniqueIdentifier>::Data = 1f64;
    }

    #[test]
    fn derive() {
        #[derive(UID)]
        enum U {}
        let _: <U as uid::UniqueIdentifier>::Data = vec![1f64];
    }

    #[test]
    fn derive_uid() {
        #[derive(UID)]
        #[uid(data = "Vec<f32>")]
        enum U {}
        let _: <U as uid::UniqueIdentifier>::Data = vec![1f32];
    }
}
