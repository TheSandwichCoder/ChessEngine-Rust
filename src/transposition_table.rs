use std::collections::HashMap;
use crate::functions::*;

// Entry Type:
// 0 -> exact value
// 1 -> Lower bound (lower than the actual value)
// 2 -> upper bound (higher than the actual value)

// type  depth
// 0 0   0 0 0 0 0 0 

const INFO_DEPTH_MASK : u8 = 0x3F;

#[derive(Clone)]
pub struct TTEntry{
    pub score: i16,
    pub info: u8,
    pub visited: u8,
    pub entry_type: u8,
    pub best_move: u16,
}

impl TTEntry{
    pub fn new(score: i16, depth: u8, entry_type: u8, best_move: u16) -> TTEntry{
        TTEntry{score:score, info:depth, visited: 0, entry_type: entry_type, best_move: best_move}

        // TTEntry{score: score, info: depth | (entry_type << 6), visited:0}

        // TTEntry{score: score, info: depth, visited: 0}
    }

    pub fn depth(&self) -> u8{
        return self.info;
        // return self.info & INFO_DEPTH_MASK; 
    }

    pub fn entry_type(&self) -> u8{
        // return self.info >> 6;
        return self.entry_type;
    }

    pub fn print_entry(&self){
        println!(
        "
        score: {}
        info: {}
        depth: {}
        visited: {}
        type: {}
        best move: {}
        ",self.score, self.info, self.depth(), self.visited, self.entry_type, get_move_string(self.best_move));
    }
}

// entry num = 1048576 (probably around 23mb)
const TT_SIZE: usize = 2 << 23; 

pub const UPPER_BOUND : u8 = 2;
pub const LOWER_BOUND : u8 = 1;
pub const EXACT_BOUND : u8 = 0;

pub const REPETITION_COUNT_HASHES : [u64; 4] = [0x0, 0x278C72C79F341B64, 0x45FB14AE2F9496D4, 0x5398422E049CBE50];

#[derive(Clone)]
pub struct TranspositionTable{
    pub table: HashMap<u64, TTEntry>,
}

impl TranspositionTable{
    pub fn new() -> TranspositionTable {
        TranspositionTable {table: HashMap::new() }
    }

    pub fn contains(&self, hash: u64, position_repitition_count:u8) -> bool{
        // if position_repitition_count as usize == 255{
        //     return false;
        // }

        let true_hash = hash ^ REPETITION_COUNT_HASHES[position_repitition_count as usize];

        return self.table.contains_key(&true_hash);
    }

    pub fn get_mut(&mut self, hash: u64, position_repitition_count:u8) -> &mut TTEntry{
        let true_hash = hash ^ REPETITION_COUNT_HASHES[position_repitition_count as usize];

        return self.table.get_mut(&true_hash).unwrap();
    }

    pub fn get(&self, hash: u64, position_repitition_count: u8) -> &TTEntry{
        let true_hash = hash ^ REPETITION_COUNT_HASHES[position_repitition_count as usize];

        return self.table.get(&true_hash).unwrap();
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

    pub fn add(&mut self, hash:u64, position_repitition_count:u8, score:i16, depth:u8, node_type: u8, best_move: u16){

        let true_hash = hash ^ REPETITION_COUNT_HASHES[position_repitition_count as usize];
        // self.table.entry(hash).and_modify(TTEntry::new(score, depth)).or_insert(TTEntry::new(score, depth));
        // updating the balue
        if self.table.contains_key(&true_hash){

            let tt_entry: &mut TTEntry = self.table.get_mut(&true_hash).unwrap();
            let tt_entry_depth: u8 = tt_entry.depth();

            

            tt_entry.score = score;
            // tt_entry.info = node_type << 6 | depth;
            tt_entry.info = depth;
            tt_entry.entry_type = node_type;

            if node_type == EXACT_BOUND{
                tt_entry.best_move = best_move;
            }

            if tt_entry.visited < 255{
                tt_entry.visited += 1;
            }
               
        }
        else{
            self.table.insert(
                true_hash,
                TTEntry::new(score, depth, node_type, best_move),
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