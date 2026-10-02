// A deterministic generator for the tests, so a failing run can be repeated from its seed.
// xorshift32.
export class Rng {
  private s: number;
  constructor(seed: number) {
    this.s = seed >>> 0 || 0x9e3779b9;
  }
  next(): number {
    let x = this.s;
    x ^= x << 13;
    x >>>= 0;
    x ^= x >>> 17;
    x ^= x << 5;
    x >>>= 0;
    this.s = x;
    return x / 0x1_0000_0000;
  }
  int(n: number): number {
    return Math.floor(this.next() * n);
  }
}
