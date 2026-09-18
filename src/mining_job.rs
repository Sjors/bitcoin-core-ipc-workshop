use bitcoin::BlockHash;

pub struct Tip {
    pub height: i32,
    pub hash: BlockHash,
}
