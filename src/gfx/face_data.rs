pub type Face = [u8; 12];

#[rustfmt::skip]
pub const FACE_INDICES: [u32; 6] = [
    //Triangle 1
    0, 1, 2,
    //Triangle 2
    3, 0, 2,
];

#[rustfmt::skip]
pub const TOP_FACE: Face = [ 
    1, 1, 1, 
    1, 1, 0,
    0, 1, 0, 
    0, 1, 1,
];

#[rustfmt::skip]
pub const BOTTOM_FACE: Face = [
    0, 0, 0,
    1, 0, 0,
    1, 0, 1, 
    0, 0, 1,
];

#[rustfmt::skip]
pub const LEFT_FACE: Face = [
    0, 1, 1, 
    0, 1, 0,
    0, 0, 0,
    0, 0, 1,
];

#[rustfmt::skip]
pub const RIGHT_FACE: Face = [
    1, 0, 0,
    1, 1, 0,
    1, 1, 1, 
    1, 0, 1,
];

#[rustfmt::skip]
pub const FRONT_FACE: Face = [
    1, 1, 0, 
    1, 0, 0,
    0, 0, 0,
    0, 1, 0,
];

#[rustfmt::skip]
pub const BACK_FACE: Face = [
    0, 0, 1,
    1, 0, 1,
    1, 1, 1, 
    0, 1, 1,
];

#[rustfmt::skip]
pub const DIAGONAL_FACE_1: Face = [
    1, 1, 1, 
    1, 0, 1,
    0, 0, 0,
    0, 1, 0,
];

#[rustfmt::skip]
pub const DIAGONAL_FACE_1_REVERSED: Face = [
    0, 0, 0,
    1, 0, 1,
    1, 1, 1, 
    0, 1, 0,
];

#[rustfmt::skip]
pub const DIAGONAL_FACE_2: Face = [
    1, 1, 0, 
    1, 0, 0,
    0, 0, 1,
    0, 1, 1,
];

#[rustfmt::skip]
pub const DIAGONAL_FACE_2_REVERSED: Face = [
    0, 0, 1,
    1, 0, 0,
    1, 1, 0, 
    0, 1, 1,
];
