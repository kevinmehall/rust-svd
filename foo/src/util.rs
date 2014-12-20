#[inline]
pub fn or_tuples(l:(uint, uint), r:(uint, uint)) -> (uint, uint) {
    let (la, lb) = l;
    let (ra, rb) = r;
    (la | ra, lb | rb)
}

#[inline]
pub fn shift_tuple(pos:uint, l:(uint, uint)) -> (uint, uint) {
    let (la, lb) = l;
    (la << pos, lb << pos)
}
