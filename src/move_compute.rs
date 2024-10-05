
const ROOK_MAGIC_NUMBER_SIZE: i32 = 14;
const BISHOP_MAGIC_NUMBER_SIZE: i32 = 11;

const ROOK_CACHE_ENTRY_SIZE: i32 = 1 << ROOK_MAGIC_NUMBER_SIZE;
const BISHOP_CACHE_ENTRY_SIZE: i32 = 1 << BISHOP_MAGIC_NUMBER_SIZE;

const ROOK_CACHE_SIZE: i32 = ROOK_CACHE_ENTRY_SIZE * 64;
const BISHOP_CACHE_SIZE: i32 = BISHOP_CACHE_ENTRY_SIZE * 64;

const ROOK_MAGIC_NUMBER_PUSH: i32 = 64 - ROOK_MAGIC_NUMBER_SIZE;
const BISHOP_MAGIC_NUMBER_PUSH: i32 = 64 - BISHOP_MAGIC_NUMBER_SIZE;

// int rookMagicNumberNumSize = 1<<rookMagicNumberSize;
// int bishopMagicNumberNumSize = 1<<bishopMagicNumberSize;

// int rookMagicNumberPush = 64-rookMagicNumberSize;
// int bishopMagicNumberPush = 64-bishopMagicNumberSize;


// rook: size 14
const ROOK_MAGIC_NUMBERS:[u64; 64] = [
    5448404644668614887,
    7429541927353657341,
    8327997435217010691,
    8938367315950951252,
    1592624865783187151,
    14735398518802345370,
    7383800330718064983,
    12528622953315292328,
    8998764742762812572,
    1525762006122095376,
    783325121587688660,
    557101797949933871,
    2885169928671381696,
    8897354479193681189,
    7324409022409279275,
    16578604099016350383,
    596237921986050726,
    830651480625245235,
    3012074197939646231,
    17335198554382125613,
    17034243820374513941,
    3518569048515431009,
    5044764137268079380,
    3979835570931416807,
    6337069399971536039,
    7171604253467494139,
    17102821407206619022,
    8696639438320257623,
    10549485283534449238,
    13359384927437517500,
    6547948396914745147,
    538880567649511714,
    15290906821300541491,
    3736386743306093693,
    12050271774257187884,
    17908801163318505741,
    11290357538719332012,
    9776136378273040926,
    2969502550575125102,
    16861817747591319736,
    3153482815394150428,
    13227168744383770893,
    1269827028785269815,
    10127176719137706431,
    11594231665730612520,
    18038065282990883879,
    2480306424537900056,
    11051379579770875184,
    668926372545163101,
    6116844684778809798,
    13606018635905067366,
    10126631763441541386,
    9109328921465274348,
    15480569388139263100,
    8480304660104374877,
    3604968967170250906,
    11838826006976676654,
    15414647529949566389,
    8390624376644560101,
    3800585628858159354,
    4265391351849429650,
    12817858002029846986,
    2707709488875334082,
    5171488126276164202
];
    
    
// bishop: size 11
const BISHOP_MAGIC_NUMBERS:[u64; 64] = [
    4816202882398023937,
    13267235174689453647,
    6380310112784006984,
    17849528126728750400,
    359412490116528558,
    6647449365250821669,
    8681356469474740624,
    8728753678196081980,
    8348440887347500092,
    13710290208257272619,
    1045924646559550807,
    14893878697663905604,
    15964597724236281664,
    14306407576948803988,
    223300879286341787,
    17695811888542785386,
    13129716937366926425,
    3427267930971339510,
    2406511342307408714,
    12030132820667459508,
    14671230452877050860,
    4886863664326768075,
    10962767860182321371,
    9734134652846336201,
    15916511572152382939,
    6005610919717011374,
    15318553168282265942,
    8953410291336286988,
    1114719348885222519,
    7287024262585315011,
    10718942865949511186,
    17553951193964710703,
    12816472782639867024,
    15480269189612857715,
    15230257100345778826,
    7735371921360245178,
    7098328396060994145,
    8709087112944869388,
    1757543010345233921,
    3798420857244561904,
    4515144076193080075,
    5721330932381388700,
    7753228957941982476,
    7756531331913882602,
    17689033768369228966,
    15363965789115115445,
    15039615866405838874,
    1598940977997147763,
    12098379233742241649,
    3818770509561324908,
    12591460181784921765,
    8314903529799055937,
    11907703119236121047,
    5502215958626712043,
    16598400797456256977,
    16011309478981238564,
    11281171579754564078,
    15567475772665900064,
    9922726573404472482,
    9238776751486017804,
    12399267100135780775,
    14116791706745139553,
    3003651216789685006,
    3184243539290100210
];

const HORIZONTAL_SLICE_BITBOARD : u64 = 0xFF00000000000000;
const VERTICLE_SLICE_BITBOARD : u64 = 0x8080808080808080;
const EDGE_MASK: u64 = 0xFF818181818181FF;



const fn get_rook_blockers_mask() -> [u64; 64]{
    let mut bitboard_array: [u64; 64] = [0; 64];

    let strict_hor_slice_bitboard = 0x7E00000000000000;
    let strict_vert_slice_bitboard = 0x80808080808000;

    let mut i: i32 = 0;
    while i < 64{
        let x : i32 = i % 8;
        let y: i32 = i / 8;

        // dont question this dark magic
        bitboard_array[(63-i) as usize] = ((strict_hor_slice_bitboard >> (y * 8)) | (strict_vert_slice_bitboard >> x)) & !(1<<(63-i));
        i += 1;
    }

    return bitboard_array;
}

const fn get_bishop_blockers_mask() -> [u64; 64]{
    let mut bitboard_array: [u64; 64] = [0; 64];

    let mut i: i32 = 0;
    while i < 64{
        let mut bitboard: u64 = 0;

        let x: i32 = i % 8;
        let y: i32 = i / 8;

        let x_vec : [i32; 4] = [1, 1, -1, -1];
        let y_vec : [i32; 4] = [1, -1, 1, -1];

        let mut j: i32 = 0;
        while j < 4{
            let mut temp_x: i32 = x;
            let mut temp_y: i32 = y;

            while temp_x >= 0 && temp_y >= 0 && temp_x < 8 && temp_y < 8{
                bitboard |= 1 << (temp_y * 8 + temp_x);
                temp_x += x_vec[j as usize];
                temp_y += y_vec[j as usize];
            }
            j += 1;
        }
        
        // get rid of the piece pos
        bitboard &= !(1 << i);

        // get rid of the side pieces
        bitboard &= !(EDGE_MASK);

        bitboard_array[i as usize] = bitboard;
        i += 1;
    }

    return bitboard_array;
}

const fn get_queen_blockers_mask() -> [u64; 64]{
    let rook_blocker_mask: [u64; 64] = get_rook_blockers_mask();
    let bishop_blocker_mask: [u64; 64] = get_bishop_blockers_mask();
    let mut queen_blockers_mask: [u64; 64] = [0; 64];

    let mut i : i32 = 0;

    while i < 64{
        queen_blockers_mask[i as usize] = rook_blocker_mask[i as usize] | bishop_blocker_mask[i as usize];
        i += 1;
    }

    return queen_blockers_mask;
}

pub fn get_blocker_combinations(bitboard: u64) -> Vec<u64>{
    let mut temp_bitboard: u64 = bitboard;
    let mut bits_num: u64 = 0;

    while temp_bitboard != 0{
        let least_bit: u32 = temp_bitboard.trailing_zeros();
        temp_bitboard ^= 1 << least_bit;

        bits_num += 1;
    }

    let combination_num: u64 = 2 << bits_num;
    let mut blocker_combinations = Vec::new();

    let mut i : u64 = 0;
    while i < combination_num{
        let mut temp_bitboard: u64 = bitboard;
        let mut new_bitboard: u64 = 0;

        let mut j: u64 = 0;
        while j < bits_num{
            // get the pos of the bit
            let least_bit: u32 = temp_bitboard.trailing_zeros();
            
            // get the val of the bit
            let bit_val: u64 = i >> j & 1;

            new_bitboard |= bit_val << least_bit;
            
            temp_bitboard ^= 1 << least_bit;
            j += 1;
        }
        blocker_combinations.push(new_bitboard);
        
        i += 1;
    }

    return blocker_combinations;
}

pub fn get_rook_legal_moves(square: i32, blockers: u64) -> u64{
    let mut bitboard: u64 = 0;

    let x: i32 = square % 8;
    let y: i32 = square / 8;

    let x_vec : [i32; 4] = [1, 0, -1, 0];
    let y_vec : [i32; 4] = [0, 1, 0, -1];

    let mut j: i32 = 0;

    // goes through the 4 directions
    while j < 4{
        let mut temp_x: i32 = x;
        let mut temp_y: i32 = y;

        while temp_x >= 0 && temp_y >= 0 && temp_x < 8 && temp_y < 8{
            let square_bitboard: u64 = 1 << (temp_y * 8 + temp_x);
            bitboard |= square_bitboard;
            temp_x += x_vec[j as usize];
            temp_y += y_vec[j as usize];

            // there is a blocker
            if blockers & square_bitboard != 0{
                break;
            }
        }
        j += 1;
    }
    
    // get rid of the piece pos
    bitboard &= !(1 << square);

    return bitboard;
}

pub fn get_bishop_legal_moves(square: i32, blockers: u64) -> u64{
    let mut bitboard: u64 = 0;

    let x: i32 = square % 8;
    let y: i32 = square / 8;

    let x_vec : [i32; 4] = [1, 1, -1, -1];
    let y_vec : [i32; 4] = [1, -1, 1, -1];

    let mut j: i32 = 0;

    // goes through the 4 directions
    while j < 4{
        let mut temp_x: i32 = x;
        let mut temp_y: i32 = y;

        while temp_x >= 0 && temp_y >= 0 && temp_x < 8 && temp_y < 8{
            let square_bitboard: u64 = 1 << (temp_y * 8 + temp_x);
            bitboard |= square_bitboard;
            temp_x += x_vec[j as usize];
            temp_y += y_vec[j as usize];

            // there is a blocker
            if blockers & square_bitboard != 0{
                break;
            }
        }
        j += 1;
    }
    
    // get rid of the piece pos
    bitboard &= !(1 << square);

    return bitboard;
}

// these are the masks to identify the important blockers
pub const ROOK_BLOCKERS_MASK: [u64; 64] = get_rook_blockers_mask();
pub const BISHOP_BLOCKERS_MASK: [u64; 64] = get_bishop_blockers_mask();
pub const QUEEN_BLOCKERS_MASK: [u64; 64] = get_queen_blockers_mask();

fn get_rook_move_cache_index(square: i32, blockers: u64) -> usize{
    //     index of the square                index in the sub array section
    return (((square * ROOK_CACHE_ENTRY_SIZE) as u64) + ((ROOK_MAGIC_NUMBERS[square as usize] * blockers) >> ROOK_MAGIC_NUMBER_PUSH)) as usize;
}

fn get_rook_legal_move_cache() -> [u64; ROOK_CACHE_SIZE as usize]{
    let mut rook_move_cache: [u64; ROOK_CACHE_SIZE as usize] = [0; ROOK_CACHE_SIZE as usize];

    let mut square: i32 = 0;
    
    while square < 64{
        let rook_blocker_mask: u64 = ROOK_BLOCKERS_MASK[square as usize];
        let rook_blocker_combinations: Vec<u64> = get_blocker_combinations(rook_blocker_mask);

        let rook_blocker_counter: i32 = 0;

        // goes through all the blocker combinations in initialises them
        while rook_blocker_counter < rook_blocker_combinations.len() as i32{
            let rook_blocker_combination: u64 = rook_blocker_combinations[rook_blocker_counter as usize];
            let rook_legal_move: u64 = get_rook_legal_moves(square, rook_blocker_combination);

            let move_cache_index: usize = get_rook_move_cache_index(square, rook_blocker_combination);

            // initialise it
            rook_move_cache[move_cache_index] = rook_legal_move;

            rook_blocker_counter += 1;
        }

        square += 1;
    }

    return rook_move_cache;
}

pub static ROOK_LEGAL_MOVE_CACHE: [u64; ROOK_CACHE_SIZE as usize] = get_rook_legal_move_cache();