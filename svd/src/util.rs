#[inline]
pub fn or_tuples(l:(usize, usize), r:(usize, usize)) -> (usize, usize) {
    let (la, lb) = l;
    let (ra, rb) = r;
    (la | ra, lb | rb)
}

#[inline]
pub fn shift_tuple(pos:usize, l:(usize, usize)) -> (usize, usize) {
    let (la, lb) = l;
    (la << pos, lb << pos)
}
