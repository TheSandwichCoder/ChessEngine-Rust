use crate::move_compute::*;
use crate::functions::*;

#[derive(Clone)]
pub struct HistoryHueristicTable{
    // [color][from][to]
    pub hh_table: [[[i16;64];64];2],
}

const MAX_HISTORY: i16 = 500;

impl HistoryHueristicTable{
    pub fn new() -> HistoryHueristicTable{
        HistoryHueristicTable{
            hh_table: [[[0;64];64];2]
        }
    }

    pub fn clear(&mut self){
        self.hh_table = [[[0;64];64];2];
    }

    pub fn update(&mut self, color: bool, mv: u16, bonus: i16){
        let color_index : usize;

        let from: usize = (mv & MOVE_DECODER_MASK) as usize;
        let to: usize = ((mv >> 6) & MOVE_DECODER_MASK) as usize;
        
        if color{
            color_index = 0;
        }
        else{
            color_index = 1;
        }

        let clamped_bonus = clamp_int(bonus, -MAX_HISTORY, MAX_HISTORY);

        let prev_value = self.hh_table[color_index][from][to];

        let difference = clamped_bonus - (prev_value * clamped_bonus.abs()) / MAX_HISTORY;

        self.hh_table[color_index][from][to] += difference;
    }
    
    pub fn age(&mut self){
        for color in 0..2{
            for from in 0..64{
                for to in 0..64{
                    self.hh_table[color][from][to] /= 2;
                }
            }
        }
    }
    
    pub fn get(&self, color: bool, mv: u16) -> i16{
        let color_index : usize;

        let from: usize = (mv & MOVE_DECODER_MASK) as usize;
        let to: usize = ((mv >> 6) & MOVE_DECODER_MASK) as usize;
        
        if color{
            color_index = 0;
        }
        else{
            color_index = 1;
        }

        return self.hh_table[color_index][from][to];
    }

    pub fn show_compressed(&self){
        let mut white_compressed = [0;64];
        let mut black_compressed = [0;64];

        // white
        for from_square in 0..64{
            let mut val_sum = 0;
            for to_square in 0..64{
                val_sum += self.hh_table[0][from_square][to_square];
            }
            white_compressed[from_square] = val_sum / 64;
        }

        // black
        for from_square in 0..64{
            let mut val_sum = 0;
            for to_square in 0..64{
                val_sum += self.hh_table[1][from_square][to_square];
            }
            black_compressed[from_square] = val_sum / 64;
        }

        println!("White");
        for i in 0..64{
            print!("{} ", white_compressed[i]);
            if i % 8 == 7{
                println!("");
            }
        }
        println!("");


        println!("Black");
        for i in 0..64{
            print!("{} ", black_compressed[i]);
            if i % 8 == 7{
                println!("");
            }
        }
        println!("");
    }
}