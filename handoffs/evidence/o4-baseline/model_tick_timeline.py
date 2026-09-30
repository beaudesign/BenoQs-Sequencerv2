"""Scratch model of the proposed clock arithmetic (NOT engine code): events are kept as integer tick positions and turned
into a sample offset only when the buffer that contains them is rendered, from the tempo (and, in host mode, the position)
at the start of that buffer. How far from the analytic answer does a 60 to 180 BPM ramp over 8 bars land, by buffer size?
Every tick of the ramp is a candidate note, so a note in the very first buffer is counted."""
import math
SR = 48000.0
def beats(t): return t + 0.0625 * t * t            # analytic: bpm(t) = 60 + 7.5 t
def bpm(t): return 60.0 + 7.5 * t
def time_of_beats(b): return (-1.0 + math.sqrt(1.0 + 4.0 * 0.0625 * b)) / (2.0 * 0.0625)
TICKS = list(range(0, 6144))                        # 8 bars of 4/4 at 192 PPQN

def run(n, mode, T_end=16.0):
    """mode: 'const' = tempo at buffer start only; 'slope' = plus the slope of the previous buffers, used only when the last two
    changes agree within 5%. Returns per-tick errors as (buffer_index, error_in_samples).
    host=True: each buffer is anchored on the host's exact position. host=False: the engine keeps its own position."""
    out = {}
    for host in (True, False):
        errs = []
        t0 = 0.0; ti = 0; k = 0; p0_own = 0.0
        hist = []                                   # (tempo at buffer start)
        while t0 < T_end and ti < len(TICKS):
            p0 = beats(t0) * 192.0 if host else p0_own
            b0 = bpm(t0)
            slope = 0.0
            if mode == 'slope' and len(hist) >= 2:
                s_last = (b0 - hist[-1]) / (n / SR)
                s_prev = (hist[-1] - hist[-2]) / (n / SR)
                if abs(s_last - s_prev) <= 0.05 * max(abs(s_last), abs(s_prev), 1e-9): slope = s_last
            def model_ticks(s): return p0 + 192.0 * (b0 * s / 60.0 + 0.5 * (slope / 60.0) * s * s)
            p_end = model_ticks(n / SR)
            while ti < len(TICKS) and TICKS[ti] < p_end:
                T = TICKS[ti]
                a = 0.5 * slope / 60.0 * 192.0; bq = b0 / 60.0 * 192.0; c = p0 - T
                s = -c / bq if abs(a) < 1e-12 else (-bq + math.sqrt(bq * bq - 4 * a * c)) / (2 * a)
                s = max(s, 0.0)
                errs.append((k, (t0 + s) * SR - time_of_beats(T / 192.0) * SR))
                ti += 1
            hist.append(b0); p0_own = p_end; t0 += n / SR; k += 1
        out[host] = errs
    return out

def worst(errs, skip=0): return max((abs(e) for k, e in errs if k >= skip), default=0.0)

print("60 to 180 BPM over 8 bars. Error of every tick's time before rounding to a whole sample, in samples (48 kHz: 1 sample = 20.8 us).")
print("'first' = the buffers where the slope is not known yet (the first two); 'then' = every buffer after them.")
print()
print("HOST POSITION MODE (each buffer anchored on the host's exact position)")
print("  buffer | tempo at start only: worst | with slope: first, then")
for n in [32, 64, 128, 256, 512, 1024, 2048, 4096]:
    c = run(n, 'const')[True]; s = run(n, 'slope')[True]
    print(f"  {n:>6} | {worst(c):>26.3f} | {worst([e for e in s if e[0] < 2]):>17.3f}, {worst(s, 2):.4f}")
print()
print("FREE-RUNNING (the engine keeps its own position; error can build up)")
print("  buffer | tempo at start only: worst, last note | with slope: worst overall, worst after the first two buffers, last note")
for n in [32, 64, 128, 256, 512, 1024, 2048, 4096]:
    c = run(n, 'const')[False]; s = run(n, 'slope')[False]
    print(f"  {n:>6} | {worst(c):>13.2f}, {c[-1][1]:>10.2f} | {worst(s):>13.3f}, {worst(s, 2):>10.3f}, {s[-1][1]:>8.3f}")
