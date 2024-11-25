use std::collections::HashMap;

// Entry Type:
// 0 -> exact value
// 1 -> Lower bound (lower than the actual value)
// 2 -> upper bound (higher than the actual value)

// type  depth
// 0 0   0 0 0 0 0 0 

const INFO_DEPTH_MASK : u8 = 0x3F;

pub struct TTEntry{
    pub score: i16,
    pub info: u8,
    pub visited: u8,
}

impl TTEntry{
    pub fn new(score: i16, depth: u8, entry_type: u8) -> TTEntry{
        TTEntry{score: score, info: depth | entry_type << 6, visited:0}
    }

    pub fn depth(&self) -> u8{
        return self.info & INFO_DEPTH_MASK; 
    }

    pub fn entry_type(&self) -> u8{
        return self.info >> 6;
    }
}

// entry num = 1048576 (probably around 23mb)
const TT_SIZE: usize = 2 << 23; 

pub const UPPER_BOUND : u8 = 2;
pub const LOWER_BOUND : u8 = 1;
pub const EXACT_BOUND : u8 = 0;

pub struct TranspositionTable{
    pub table: HashMap<u64, TTEntry>,
}

impl TranspositionTable{
    pub fn new() -> TranspositionTable {
        TranspositionTable {table: HashMap::new() }
    }

    pub fn contains(&self, hash: &u64) -> bool{
        return self.table.contains_key(hash);
    }

    pub fn get_mut(&mut self, hash: &u64) -> &mut TTEntry{
        return self.table.get_mut(hash).unwrap();;
    }

    pub fn size(&self) -> usize{
        let length = self.table.len(); 
        return length;
    }

    pub fn exceed_size(&self) -> bool{
        println!("{} {}", self.size(), TT_SIZE);
        return self.size() > TT_SIZE;
    }

    pub fn capacity(&self) -> f32{
        return self.size() as f32 / TT_SIZE as f32;
    }

    pub fn add(&mut self, hash:u64, score:i16, depth:u8, node_type: u8){

        // self.table.entry(hash).and_modify(TTEntry::new(score, depth)).or_insert(TTEntry::new(score, depth));
        // updating the balue
        if self.table.contains_key(&hash){
            let tt_entry: &mut TTEntry = self.table.get_mut(&hash).unwrap();
            
            tt_entry.score = score;
            tt_entry.info = node_type << 6 | depth;
            tt_entry.visited += 1;
        }
        else{
            self.table.insert(
                hash,
                TTEntry::new(score, depth, node_type),
            );
        }
    }

    pub fn drain(&mut self){
        // shrink factor of 4
        // self.table.shrink_to(TT_SIZE / 4);
        self.table.retain(|_, k| k.visited > 2);
    }

    pub fn clear(&mut self){
        self.table.clear();
    }
}