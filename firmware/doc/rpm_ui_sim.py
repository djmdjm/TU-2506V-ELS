#!/usr/bin/env python3

# Small tool to play around with filtering and display of a somewhat noisy
# RPM count.

import time
import math
import random

CENTRE=1400
RANGE1=80
FREQ1=4 #Hz
RANGE2=40
FREQ2=20 #Hz
NOISE_RANGE=10

LOOP_RATE=100 #Hz
DISPLAY_RATE=4 #Hz
FIR_SIZE=int(LOOP_RATE/5)

start=time.monotonic_ns()
last_display=0
fir=[]
while True:
	now = (time.monotonic_ns() - start) / 1000000000.0
	theta1 = (2.0 * math.pi * now * FREQ1) % (2.0 * math.pi)
	theta2 = (2.0 * math.pi * now * FREQ2) % (2.0 * math.pi)
	noise = random.normalvariate(0, 10)
	RPM=(math.sin(theta1) * RANGE1) + (math.sin(theta2) * RANGE2) + noise + CENTRE
	fir.insert(0, RPM)
	if len(fir) > FIR_SIZE:
		fir.pop()
	if now - last_display > 1.0/DISPLAY_RATE:
		mean = sum(fir) / len(fir)
		display = int(math.floor(mean + 0.5))
		print("\r", display, "     ", sep="", end="")
		last_display = now
	time.sleep(1/LOOP_RATE)

