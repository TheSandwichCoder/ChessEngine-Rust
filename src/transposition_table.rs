use std::collections::HashMap;
use crate::functions::*;
use crate::app_settings::TRANSPOSITION_TABLE_SIZE;

// Entry Type:
// 0 -> exact value
// 1 -> Lower bound (lower than the actual value)
// 2 -> upper bound (higher than the actual value)

// type  depth
// 0 0   0 0 0 0 0 0 

const INFO_DEPTH_MASK : u8 = 0x3F;

#[derive(Clone, Copy)]
pub struct TTEntry{
    pub score: i16,
    pub info: u8,
    pub best_move: u16,
    pub hash: u64,
}

impl TTEntry{
    pub fn new(score: i16, depth: u8, entry_type: u8, best_move: u16, hash: u64) -> TTEntry{
        // TTEntry{score:score, info:depth, visited: 0, entry_type: entry_type, best_move: best_move}

        TTEntry{score: score, info: depth | (entry_type << 6), best_move: best_move, hash: hash}

        // TTEntry{score: score, info: depth, visited: 0}
    }

    pub fn null() -> TTEntry{
        TTEntry{score: 0, info:0, best_move: 0, hash: 0}
    }

    pub fn depth(&self) -> u8{
        // return self.info;
        return self.info & INFO_DEPTH_MASK; 
    }

    pub fn entry_type(&self) -> u8{
        return self.info >> 6;
        // return self.entry_type;
    }

    pub fn print_entry(&self){
        println!(
        "
        score: {}
        info: {}
        depth: {}
        type: {}
        best move: {}
        hash: {}
        ",self.score, self.info, self.depth(), self.entry_type(), get_move_string(self.best_move), self.hash);
    }
}

// entry num = 4194304 (probably around 58mb)
const TT_SIZE: usize = TRANSPOSITION_TABLE_SIZE;

pub const UPPER_BOUND : u8 = 2;
pub const LOWER_BOUND : u8 = 1;
pub const EXACT_BOUND : u8 = 0;

pub const REPETITION_COUNT_HASHES : [u64; 4] = [0x0, 0x278C72C79F341B64, 0x45FB14AE2F9496D4, 0x5398422E049CBE50];

#[derive(Clone)]
pub struct TranspositionTable{
    pub table: Box<[TTEntry]>,
    pub entry_num: u32,
}

impl TranspositionTable{
    pub fn new() -> TranspositionTable {
        
        let vec = vec![TTEntry::null() ; TT_SIZE];

        // Convert the Vec into a Box<[T]> (heap-allocated array)
        

        TranspositionTable {table: vec.into_boxed_slice(), entry_num: 0}
    }

    pub fn contains(&self, hash: u64) -> bool{
        let table_index = hash as usize % TT_SIZE;

        if self.table[table_index].hash == hash{
            return true;
        }
        else if self.table[table_index + 1].hash == hash{
            return true
        }

        return false;
    }

    pub fn get(&self, hash: u64) -> &TTEntry{
        let table_index = hash as usize % TT_SIZE;

        let tt_entry = &self.table[table_index];

        if tt_entry.hash == hash || table_index + 1 == TT_SIZE{
            return tt_entry;
        }
        
        return &self.table[table_index + 1];
    }

    pub fn add(&mut self, hash:u64, score:i16, depth:u8, node_type: u8, best_move: u16){

        let table_index = hash as usize % TT_SIZE;

        let mut tt_entry = &mut self.table[table_index];

        if tt_entry.hash != hash && table_index + 1 != TT_SIZE{
            tt_entry = &mut self.table[table_index + 1];
        }
        
            // if tt_entry.hash != hash && tt_entry.hash != 0{
            //     println!("overwrite");
            // }
        if tt_entry.hash != hash && tt_entry.hash == 0{
            self.entry_num += 1;
        }
        

        tt_entry.score = score;
        tt_entry.info = node_type << 6 | depth;
        tt_entry.hash = hash;
        tt_entry.best_move = best_move;

        
        
    }

    pub fn exceed_size(&self) -> bool{
        return false; // trust in the process
    }

    pub fn capacity(&self) -> f32{
        return self.entry_num as f32 / TT_SIZE as f32;
    }

    pub fn drain(&mut self){
        // shrink factor of 4
        // self.table.shrink_to(TT_SIZE / 4);
        // self.table.retain(|_, k| k.visited > 2);
        
        // trust in the process
    }

    pub fn clear(&mut self){
        for entry in self.table.iter_mut() {
            *entry = TTEntry::null();
        }
    }
}