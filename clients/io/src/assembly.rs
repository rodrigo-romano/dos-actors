pub trait Assembly {
    const N: usize = 7;
    const SIDS: [u8; 7] = [1, 2, 3, 4, 5, 6, 7];

    fn position<const ID: u8>() -> Option<usize> {
        <Self as Assembly>::SIDS
            .into_iter()
            .position(|sid| sid == ID)
    }
}
