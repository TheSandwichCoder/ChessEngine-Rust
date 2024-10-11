pub const ROOK_RAW_MAGICS : [u64;64] = [
    0xCE17D510001BA47E,
    0x1B896E7B1435250B,
    0x6299003152F607B7,
    0xE3BC45DACEE84B8C,
    0xBEE3DF90D3A427F2,
    0x43C8BBC8C9442928,
    0xA437C856DE9217FA,
    0x783A622B4A3C03BD,
    0xE545A82092D4028,
    0xB0FB6000B239A2C0,
    0x21A87D84142C6A20,
    0xC41DCBCF1E15E7E4,
    0x6634B5CE74B6322A,
    0xAAF1C2D63D50F93A,
    0xC848AF2FB160BCA9,
    0x59976A21B700437D,
    0x3EF076E08A8FBCFC,
    0x34C407C20748C9DF,
    0xE89949F6E1569D8C,
    0x91C87FF00440F67C,
    0x81B00B2166D61D9,
    0x1E96324215BE0AB3,
    0x8512C0097F4A9A50,
    0x17BC5058A75F06A,
    0x5F1CC30C4C2616B2,
    0x9A6824E2200F8502,
    0x25766AF0B8D71BA1,
    0x9785D69FFFAADFE,
    0xA2906BF5C0511141,
    0xCE3E4EA1B1BBC838,
    0xB14B0EE0BEDC71A7,
    0xA058D53EFFB9BB8A,
    0xC9D1813F60465AC9,
    0xAD1FDA6AD7C00255,
    0x728F2198E3A00108,
    0x8F1FF5BB524AE4D9,
    0x3F754090E540D641,
    0x25A7AD9E1A69E69E,
    0x1A652791477EB4C,
    0x1B1C1F5B431F3880,
    0x7447214FC39FFE3,
    0x7DF557F9EA7D5FEC,
    0x6F2323B4DA0DD68,
    0xE1F367BAF53535A9,
    0xBCD48549E91B7AD9,
    0x92B9002308942004,
    0x9D5271BE785722D0,
    0x4BB72F2CC282000C,
    0xDF9C1C7BB9748AC6,
    0xF779ECF3D657DBAE,
    0x2BC27FF5FFE73200,
    0xBC66164A066848A0,
    0xA56AE1237175AC8E,
    0xD17C9E70CDD6E3D7,
    0x2A87B0778A2400,
    0xCC6BE4A72E52FCCF,
    0x4AAE038502114222,
    0x9779F5AA73B7612,
    0x97B26E592E5E014E,
    0x64E251DDFE6927FE,
    0xA030D20F22460072,
    0xC452341FFF76D5A,
    0x6B93558A152D04F4,
    0x8F43F1850401A8CA
];

pub const ROOK_RAW_SHIFTS : [u8; 64] = [
    50,
    52,
    51,
    51,
    51,
    52,
    52,
    51,
    52,
    53,
    53,
    53,
    53,
    53,
    53,
    52,
    52,
    53,
    53,
    53,
    53,
    53,
    53,
    52,
    52,
    53,
    53,
    53,
    53,
    53,
    53,
    52,
    52,
    53,
    53,
    53,
    53,
    53,
    53,
    52,
    52,
    53,
    53,
    53,
    53,
    53,
    53,
    53,
    52,
    53,
    53,
    53,
    53,
    53,
    54,
    52,
    52,
    52,
    52,
    52,
    52,
    52,
    52,
    52
];

pub const ROOK_MOVE_CACHE_SIZE: usize = 214016;

pub const BISHOP_RAW_MAGICS : [u64;64] = [
    0xC3513C08156402D8,
    0x40B14395B68206BF,
    0xD38248C008E2A46,
    0x4EF8214AC55760FC,
    0x1AC49043BA09B798,
    0x48970320F0254AF5,
    0x6BB165C6B0401FFA,
    0xBD3ACB31E94CD314,
    0x5A693030559304,
    0x289AA0BA42004506,
    0x5339D0342382AABA,
    0x6E0BD7A1808BCAB1,
    0xC21DAD10C0A125B5,
    0x19A860602A0CE6F,
    0xA47B660210460924,
    0x814491F1040E2037,
    0xE84525718FAFED42,
    0xAFFB815F5EDBDEF6,
    0xF0A04667EF841B78,
    0x90828D724015AECF,
    0x36D52371F1D4B5E,
    0x7E173F027DD7A8CF,
    0xAF30989ACA182044,
    0xC20A00D947218820,
    0x19D00ABDC9618412,
    0x969824052A4C3837,
    0x81080287677E2039,
    0x8BC6398E14C9DA3E,
    0xBBA0EAB7FB2117B,
    0xC199B60793FCD90,
    0xA92804CDFF840480,
    0xE5F907C9CA060185,
    0xDB3860507B740434,
    0x10384117D6447830,
    0x7FCFE27006180781,
    0xF16289F622D3113E,
    0xC15A54C05BAA95A,
    0xD0D0CEE1007A00A9,
    0x1597E30102C9F811,
    0x66FC2BC6C32E050B,
    0x8134124E7033C034,
    0xD3EF10BA10256035,
    0x63D0E4FFAF7851B5,
    0x7AE1862164026802,
    0x31B9934171EFC8FC,
    0x2A80E0C7E02F4FA2,
    0xBC50ED281183CF05,
    0x5994011C0AA04B0D,
    0xDFC6DF08A00EF506,
    0x7DA5C2541460101E,
    0x995D8C8558281AB3,
    0x9B675A990848328A,
    0x33F2ED50206A0410,
    0xCD458205874856B,
    0x85315250310E0A67,
    0x58E4300D9602815C,
    0xB7DF51AA91070655,
    0xC4220E051086308F,
    0x7CDDFBEE154A0817,
    0xA5CBE53D7D8C0C1A,
    0x837E31CDE1F46408,
    0xAAAD695D70EE2A00,
    0x8A2B020180704EA,
    0x4B36F33C68020265,
];

pub const BISHOP_RAW_SHIFTS : [u8; 64] = [
    58,
    59,
    59,
    59,
    59,
    59,
    59,
    57,
    59,
    59,
    59,
    59,
    59,
    59,
    59,
    59,
    59,
    59,
    56,
    56,
    56,
    56,
    59,
    59,
    59,
    59,
    56,
    53,
    53,
    56,
    59,
    59,
    59,
    59,
    57,
    54,
    54,
    57,
    59,
    59,
    59,
    59,
    56,
    57,
    56,
    56,
    59,
    59,
    59,
    59,
    59,
    59,
    59,
    59,
    59,
    59,
    57,
    59,
    59,
    59,
    59,
    59,
    59,
    58,
];

pub const BISHOP_MOVE_CACHE_SIZE : usize = 10624;