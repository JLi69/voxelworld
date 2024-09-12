pub type Face = [u8; 18];

pub const TOP_FACE: Face = [
    //Triangle 1
    0, 1, 0, 1, 1, 0, 1, 1, 1, //Triangle 2
    0, 1, 0, 1, 1, 1, 0, 1, 1,
];

pub const BOTTOM_FACE: Face = [
    //Triangle 1
    0, 0, 0, 1, 0, 0, 1, 0, 1, //Triangle 2
    0, 0, 0, 0, 0, 1, 1, 0, 1,
];

pub const LEFT_FACE: Face = [
    //Triangle 1
    0, 0, 0, 0, 1, 0, 0, 1, 1, //Triangle 2
    0, 0, 0, 0, 1, 1, 0, 0, 1,
];

pub const RIGHT_FACE: Face = [
    //Triangle 1
    1, 0, 0, 1, 1, 0, 1, 1, 1, //Triangle 2
    1, 0, 0, 1, 1, 1, 1, 0, 1,
];

pub const FRONT_FACE: Face = [
    //Triangle 1
    0, 0, 0, 1, 0, 0, 1, 1, 0, //Triangle 2
    0, 0, 0, 1, 1, 0, 0, 1, 0,
];

pub const BACK_FACE: Face = [
    //Triangle 1
    0, 0, 1, 1, 0, 1, 1, 1, 1, //Triangle 2
    0, 0, 1, 1, 1, 1, 0, 1, 1,
];
