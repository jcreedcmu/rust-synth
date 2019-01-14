fn caller1<C>(k: &mut C)
where
  C: FnMut(&u8),
{
  k(&3);
}
