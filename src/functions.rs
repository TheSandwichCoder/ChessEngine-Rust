pub fn print_bitboard(bitboard: u64){
    // LSB is first MSB is last

    for i in 0..64{
        if i % 8 == 0{
            println!("");
        }

        let j = i;
        print!("{} ",(bitboard >> j) & 1);
    }
    println!("");
}

pub fn print_move(mv: &u16){
    let from_pos: u16 = mv & 0x3F;
    let to_pos: u16 = mv >> 6 & 0x3F;
    let special: u16 = mv >> 12;

    println!("{} -> {} | {}",from_pos, to_pos, special)
}

pub fn print_moves(move_vec: &Vec<u16>){
    for mv in move_vec{
        print_move(&mv);
    } 
}