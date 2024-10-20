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

pub fn print_binary(n: u16){
    for i in 0..16{
        let true_i = 15 - i;

        if n >> true_i & 1 == 1{
            print!("1");
        }
        else{
            print!("0");
        }
    }
    println!("");
}

pub fn print_move(mv: &u16){
    let from_pos: u16 = mv & 0x3F;
    let to_pos: u16 = mv >> 6 & 0x3F;
    let special: u16 = mv >> 12;

    println!("{}{} | {}",num_to_coord(from_pos), num_to_coord(to_pos), special);
}

pub fn print_moves(move_vec: &Vec<u16>){
    for mv in move_vec{
        print_move(&mv);
    } 
}

pub fn get_move_string(mv: u16) -> String{
    let from_pos: u16 = mv & 0x3F;
    let to_pos: u16 = mv >> 6 & 0x3F;

    return format!("{}{}", num_to_coord(from_pos), num_to_coord(to_pos));
}

pub fn num_to_coord(square: u16) -> String{
    if square > 63 {
        return String::from("Invalid");
    }

    let file = (square % 8) as u8;  // Get the column index (0-7)
    let rank = 8 - (square / 8);    // Get the row (1-8)

    // Convert file to 'a'-'h'
    let file_char = (file + b'a') as char;
    
    format!("{}{}", file_char, rank)
}

pub fn coord_to_number(coordinate: &str) -> u8 {
    // Ensure the input is exactly 2 characters long (e.g., A1, H8)
    if coordinate.len() != 2 {
        return 0;
    }

    // Get the column character (A-H) and the row character (1-8)
    let column_char: char = coordinate.chars().nth(0).expect("should be char");
    let row_char: char = coordinate.chars().nth(1).expect("should be char");

    let column_index : u8 = ('h' as u8)-(column_char as u8);
    let row_index : u8 = ('8' as u8) - (row_char as u8);

    return row_index * 8 + (7-column_index);
}
