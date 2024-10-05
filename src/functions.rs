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