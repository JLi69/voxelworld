pub type Face = [u8; 18];

#[rustfmt::skip]
pub const TOP_FACE: Face = [ 
    //Triangle 1
    1, 1, 1, 
    1, 1, 0,
    0, 1, 0, 

    //Triangle 2 
    0, 1, 1,
    1, 1, 1,
    0, 1, 0,
];

#[rustfmt::skip]
pub const BOTTOM_FACE: Face = [
    //Triangle 1
    0, 0, 0,
    1, 0, 0,
    1, 0, 1, 

    //Triangle 2
    1, 0, 1,
    0, 0, 1,
    0, 0, 0, 
];

#[rustfmt::skip]
pub const LEFT_FACE: Face = [
    //Triangle 1 
    0, 1, 1, 
    0, 1, 0,
    0, 0, 0,

    //Triangle 2
    0, 0, 1,
    0, 1, 1,
    0, 0, 0,
];

#[rustfmt::skip]
pub const RIGHT_FACE: Face = [
    //Triangle 1
    1, 0, 0,
    1, 1, 0,
    1, 1, 1, 

    //Triangle 2
    1, 0, 0,
    1, 1, 1, 
    1, 0, 1,
];

#[rustfmt::skip]
pub const FRONT_FACE: Face = [
    //Triangle 1 
    1, 1, 0, 
    1, 0, 0,
    0, 0, 0,

    //Triangle 2
    0, 1, 0,
    1, 1, 0,
    0, 0, 0,
];

#[rustfmt::skip]
pub const BACK_FACE: Face = [
    //Triangle 1
    0, 0, 1,
    1, 0, 1,
    1, 1, 1, 

    //Triangle 2
    0, 0, 1,
    1, 1, 1,
    0, 1, 1,
];

#[rustfmt::skip]
pub const DIAGONAL_FACE_1: Face = [
    //Triangle 1 
    1, 1, 1, 
    1, 0, 1,
    0, 0, 0,

    //Triangle 2 
    0, 0, 0,
    0, 1, 0,
    1, 1, 1,
];

#[rustfmt::skip]
pub const DIAGONAL_FACE_1_REVERSED: Face = [
    //Triangle 1
    0, 0, 0,
    1, 0, 1,
    1, 1, 1, 

    //Triangle 2 
    1, 1, 1,
    0, 1, 0,
    0, 0, 0,
];

#[rustfmt::skip]
pub const DIAGONAL_FACE_2: Face = [
    //Triangle 1 
    1, 1, 0, 
    1, 0, 0,
    0, 0, 1,

    //Triangle 2 
    0, 0, 1,
    0, 1, 1,
    1, 1, 0,
];

#[rustfmt::skip]
pub const DIAGONAL_FACE_2_REVERSED: Face = [
    //Triangle 1 
    0, 0, 1,
    1, 0, 0,
    1, 1, 0, 

    //Triangle 2 
    1, 1, 0,
    0, 1, 1,
    0, 0, 1,
];
