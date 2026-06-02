# this script was ai generated, was going to do it myself but this is a sample script literally no one will see, i dont care enough

import cv2
import numpy as np

VIDEO = "48x36_20fps.mp4"
OUTPUT = "bad_apple.bin"

cap = cv2.VideoCapture(VIDEO)
assert cap.isOpened(), f"Could not open {VIDEO}"

frames_written = 0
with open(OUTPUT, "wb") as f:
    while True:
        ok, frame = cap.read()
        if not ok:
            break

        gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
        bits = (gray > 127).astype(np.uint8).flatten()  # 1728 bits, row-major

        # pack 8 bits per byte, MSB first
        packed = np.packbits(bits)
        f.write(packed.tobytes())
        frames_written += 1

cap.release()
print(f"Done: {frames_written} frames, {frames_written * 216} bytes -> {OUTPUT}")
