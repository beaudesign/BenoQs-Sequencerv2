// The measuring half of the voice rig (SPEC-0004 section 5): what a circuit's output is measured with. A windowed spectrum,
// the frequency of a note, the centre of gravity of its spectrum, and how much of its energy is not on a harmonic (what aliasing
// is). Nothing here is part of a voice and nothing is imported by `src/`.

/** In place radix-2 FFT. `re` and `im` have the same power-of-two length. */
export function fft(re: Float64Array, im: Float64Array): void {
  const n = re.length;
  if (n & (n - 1)) throw new Error(`fft: ${n} is not a power of two`);
  for (let i = 1, j = 0; i < n; i++) {
    let bit = n >> 1;
    for (; j & bit; bit >>= 1) j ^= bit;
    j ^= bit;
    if (i < j) {
      [re[i], re[j]] = [re[j]!, re[i]!];
      [im[i], im[j]] = [im[j]!, im[i]!];
    }
  }
  for (let len = 2; len <= n; len <<= 1) {
    const ang = (-2 * Math.PI) / len;
    const wr = Math.cos(ang);
    const wi = Math.sin(ang);
    for (let i = 0; i < n; i += len) {
      let cr = 1;
      let ci = 0;
      for (let k = 0; k < len / 2; k++) {
        const a = i + k;
        const b = a + len / 2;
        const tr = re[b]! * cr - im[b]! * ci;
        const ti = re[b]! * ci + im[b]! * cr;
        re[b] = re[a]! - tr;
        im[b] = im[a]! - ti;
        re[a] = re[a]! + tr;
        im[a] = im[a]! + ti;
        const nr = cr * wr - ci * wi;
        ci = cr * wi + ci * wr;
        cr = nr;
      }
    }
  }
}

/** The largest power of two that is at most `n`. */
export function floorPow2(n: number): number {
  return 2 ** Math.floor(Math.log2(n));
}

/**
 * The power spectrum, bins 0 to n/2, of the first `n` samples of `x` under a four-term Blackman-Harris window: its leakage is below
 * -92 dB beyond four bins, so the energy that is not on a harmonic can be read down to about -90 dB (a Hann window leaves a floor near -63).
 */
export function powerSpectrum(x: ArrayLike<number>, n = floorPow2(x.length)): Float64Array {
  const re = new Float64Array(n);
  const im = new Float64Array(n);
  for (let i = 0; i < n; i++) {
    const t = (2 * Math.PI * i) / n;
    re[i] = (x[i] ?? 0) * (0.35875 - 0.48829 * Math.cos(t) + 0.14128 * Math.cos(2 * t) - 0.01168 * Math.cos(3 * t));
  }
  fft(re, im);
  const p = new Float64Array(n / 2 + 1);
  for (let k = 0; k < p.length; k++) p[k] = re[k]! * re[k]! + im[k]! * im[k]!;
  return p;
}

/** The Hann-windowed DFT of `n` samples of `x` from `from`, evaluated at any frequency `hz`, not only at a bin. */
function dftAt(x: ArrayLike<number>, from: number, n: number, hz: number, rate: number): { re: number; im: number } {
  const w = (2 * Math.PI * hz) / rate;
  let re = 0;
  let im = 0;
  for (let i = 0; i < n; i++) {
    const s = (x[from + i] ?? 0) * (0.5 - 0.5 * Math.cos((2 * Math.PI * i) / n));
    re += s * Math.cos(w * i);
    im -= s * Math.sin(w * i);
  }
  return { re, im };
}

/**
 * The frequency of the strongest component within a semitone of `expectedHz`, in hertz. The spectrum of the first `n` samples
 * finds the bin. The frequency within it comes from how far the phase at that bin turns between that window and one a quarter
 * of a window later (a phase vocoder), which does not depend on where the bins fall: for one dominant sinusoid with a slowly
 * changing level it is that sinusoid's frequency to a millionth of a hertz. Needs `n + n / 4` samples.
 */
export function fundamental(x: ArrayLike<number>, rate: number, expectedHz: number, n = 65_536): number {
  const hop = n / 4;
  if (x.length < n + hop) throw new Error(`fundamental: needs ${n + hop} samples, has ${x.length}`);
  const p = powerSpectrum(x, n);
  const binHz = rate / n;
  const lo = Math.max(1, Math.floor((expectedHz / 2 ** (1 / 12)) / binHz));
  const hi = Math.min(p.length - 2, Math.ceil((expectedHz * 2 ** (1 / 12)) / binHz));
  let best = lo;
  for (let k = lo; k <= hi; k++) if (p[k]! > p[best]!) best = k;
  const fk = best * binHz;
  const a = dftAt(x, 0, n, fk, rate);
  const b = dftAt(x, hop, n, fk, rate);
  // The kernel restarts at each window's first sample, so the phase advances by the whole of 2 pi f hop / rate; what is left after
  // taking off the advance of the bin's own frequency is the distance from the bin.
  let dphi = Math.atan2(b.im, b.re) - Math.atan2(a.im, a.re) - (2 * Math.PI * fk * hop) / rate;
  dphi -= 2 * Math.PI * Math.round(dphi / (2 * Math.PI));
  return fk + (dphi * rate) / (2 * Math.PI * hop);
}

/** Cents from `expected` to `measured`. */
export function cents(measured: number, expected: number): number {
  return 1200 * Math.log2(measured / expected);
}

/** The spectral centroid in hertz: the power-weighted mean frequency below Nyquist. */
export function centroid(x: ArrayLike<number>, rate: number, n = floorPow2(x.length)): number {
  const p = powerSpectrum(x, n);
  let num = 0;
  let den = 0;
  for (let k = 1; k < p.length; k++) {
    num += (k * rate * p[k]!) / n;
    den += p[k]!;
  }
  return den > 0 ? num / den : 0;
}

/**
 * How far below the fundamental the energy that is not on a harmonic of `f0` is, in decibels (more negative is cleaner).
 * "On a harmonic" is within `band` bins of h times f0, widened for a pitch that may wander by `driftCents`. A band-limited
 * oscillator has nothing else; a naive one folds its upper harmonics back below Nyquist onto frequencies that are not
 * harmonics, and that is the energy this counts.
 */
export function inharmonicDb(x: ArrayLike<number>, rate: number, f0: number, options: { n?: number; band?: number; driftCents?: number } = {}): number {
  const n = options.n ?? floorPow2(x.length);
  const band = options.band ?? 6;
  const drift = 2 ** ((options.driftCents ?? 0) / 1200) - 1;
  const p = powerSpectrum(x, n);
  const binHz = rate / n;
  const harmonic = new Uint8Array(p.length);
  let fundamentalPower = 0;
  for (let h = 1; h * f0 < rate / 2; h++) {
    const centre = (h * f0) / binHz;
    const half = band + (h * f0 * drift) / binHz;
    for (let k = Math.max(0, Math.floor(centre - half)); k <= Math.min(p.length - 1, Math.ceil(centre + half)); k++) {
      harmonic[k] = 1;
      if (h === 1) fundamentalPower += p[k]!;
    }
  }
  let other = 0;
  for (let k = 1; k < p.length; k++) if (!harmonic[k]) other += p[k]!;
  return 10 * Math.log10(Math.max(other, 1e-300) / fundamentalPower);
}
