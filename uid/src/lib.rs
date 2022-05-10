/// Defines the data type associated with unique identifier data type
pub trait UniqueIdentifier: Send + Sync {
    type Data;
}

#[cfg(test)]
mod tests {
    use super::*;
    use uid_derive::UID;

    #[test]
    fn uid() {
        enum U {}
        impl UniqueIdentifier for U {
            type Data = f64;
        }
        let _: <U as UniqueIdentifier>::Data = 1f64;
    }

    #[test]
    fn derive() {
        #[derive(UID)]
        enum U {}
        let _: <U as UniqueIdentifier>::Data = vec![1f64];
    }

    #[test]
    fn derive_uid() {
        #[derive(UID)]
        #[uid(data = "Vec<f32>")]
        enum U {}
        let _: <U as UniqueIdentifier>::Data = vec![1f32];
    }

    #[test]
    fn derive_uid_err() {
        #[derive(UID)]
        #[uid(dat = "Vec<f32>")]
        enum U {}
        let _: <U as UniqueIdentifier>::Data = vec![1f32];
    }
}
