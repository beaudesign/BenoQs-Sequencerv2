// A WAV file from samples (SPEC-0004 S0): 16-bit PCM, mono, little endian, the plainest thing a phone will play.

/** Float samples in -1..1 as the bytes of a mono 16-bit PCM WAV file at `rate`. Values outside -1..1 are clipped. */
export function encodeWav(samples: ArrayLike<number>, rate: number): Uint8Array {
  const bytes = samples.length * 2;
  const out = new Uint8Array(44 + bytes);
  const v = new DataView(out.buffer);
  const tag = (at: number, text: string): void => {
    for (let i = 0; i < 4; i++) out[at + i] = text.charCodeAt(i);
  };
  tag(0, "RIFF");
  v.setUint32(4, 36 + bytes, true);
  tag(8, "WAVE");
  tag(12, "fmt ");
  v.setUint32(16, 16, true); // the size of what follows
  v.setUint16(20, 1, true); // PCM
  v.setUint16(22, 1, true); // one channel
  v.setUint32(24, rate, true);
  v.setUint32(28, rate * 2, true); // bytes a second
  v.setUint16(32, 2, true); // bytes a frame
  v.setUint16(34, 16, true);
  tag(36, "data");
  v.setUint32(40, bytes, true);
  for (let i = 0; i < samples.length; i++) v.setInt16(44 + 2 * i, Math.max(-32768, Math.min(32767, Math.round((samples[i] ?? 0) * 32767))), true);
  return out;
}
