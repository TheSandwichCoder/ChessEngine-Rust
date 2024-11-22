use std::collections::HashMap;

pub struct TTEntry{
    pub score: i16,
    pub depth: u8,
    pub visited: u8,
}

impl TTEntry{
    pub fn new(score: i16, depth: u8) -> TTEntry{
        TTEntry{score: score, depth: depth, visited:0}
    }
}

// entry num = 1048576 (probably around 23mb)
const TT_SIZE: usize = 2 << 23; 

pub struct TranspositionTable{
    pub table: HashMap<u64, TTEntry>,
}

impl TranspositionTable{
    pub fn new() -> TranspositionTable {
        TranspositionTable {table: HashMap::new() }
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

    pub fn add(&mut self, hash:u64, score:i16, depth:u8){

        // self.table.entry(hash).and_modify(TTEntry::new(score, depth)).or_insert(TTEntry::new(score, depth));
        // updating the balue
        if self.table.contains_key(&hash){
            let tt_entry: &mut TTEntry = self.table.get_mut(&hash).unwrap();
            
            tt_entry.score = score;
            tt_entry.depth = depth;
            tt_entry.visited += 1;
        }
        else{
            self.table.insert(
                hash,
                TTEntry::new(score, depth),
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