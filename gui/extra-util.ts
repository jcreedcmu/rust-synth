// x is a floating point number. We want to return an int, but have
// the function feel reasonably responsive even if x isn't that far
// from zero.
export function augment_and_snap(x: number) {
  const sgn = x > 0 ? 1 : -1;
  const abs = Math.abs(x);
  const snap = Math.floor(abs + 0.5);
  return snap * sgn;
}
