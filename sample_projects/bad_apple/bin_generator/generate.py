# this script was ai generated, was going to do it myself but this is a sample script literally no one will see, i dont care enough
# i added all the modifications for compression myself tho

import cv2
import numpy as np

VIDEO = "48x36_20fps.mp4"
OUTPUT = "bad_apple_low.bin"

# rescale the video from 48x36 if you want
WIDTH, HEIGHT = 36, 27

# these settings only flip a bit if there value change is extreme, to stop pixels barely above gray and below gray as wasting a connection
COMPRESS = True
LOW_CHANGE = 79
HIGH_CHANGE = 175
# setting low to 63 and high to 191 results in going from 306k to 253k flips, which aint great but ig its something...

cap = cv2.VideoCapture(VIDEO)
assert cap.isOpened(), f"Could not open {VIDEO}"

prev_bits = None
flips = 0

frames_written = 0
with open(OUTPUT, "wb") as f:
    while True:
        ok, frame = cap.read()
        if not ok:
            break

        gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
        gray = cv2.resize(gray, (WIDTH, HEIGHT), interpolation=cv2.INTER_AREA)

        if prev_bits is None or not COMPRESS:
            bits = (gray > 127).astype(np.uint8).flatten()
        else:
            g = gray.flatten()
            bits = prev_bits.copy()
            bits[g > HIGH_CHANGE] = 1
            bits[g < LOW_CHANGE] = 0

        if prev_bits is not None:
            flips += np.count_nonzero(bits != prev_bits)
        prev_bits = bits

        # pack 8 bits per byte, MSB first
        packed = np.packbits(bits)
        f.write(packed.tobytes())
        frames_written += 1

cap.release()
print(f"Done: {frames_written} frames, {frames_written * 216} bytes -> {OUTPUT}")
print(f"flips: {flips}")
