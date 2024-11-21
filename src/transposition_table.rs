use std::collections::HashMap;

pub struct TranspositionTable{
    pub table: HashMap<u64, i16>,
}

impl TranspositionTable{
    pub fn new() -> TranspositionTable {
        TranspositionTable {table: HashMap::new() }
    }
}