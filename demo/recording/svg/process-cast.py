#!/usr/bin/env python3
"""Post-process asciinema cast files for smooth SVG animation.

- Sets terminal width to 120 columns
- Compresses long pauses (max 1.5s between frames)
- Smooths timing so output flows instead of jumping
- Removes unnecessary blank frame bursts
"""
import json
import sys

def process_cast(input_path, output_path, max_pause=1.5, min_gap=0.03):
    with open(input_path, 'r') as f:
        lines = f.readlines()

    # Parse header
    header = json.loads(lines[0])
    header['width'] = 120
    header['height'] = 35

    # Parse events
    events = []
    for line in lines[1:]:
        line = line.strip()
        if not line:
            continue
        event = json.loads(line)
        events.append(event)

    if not events:
        return

    # Smooth timing
    smoothed = []
    new_time = 0.0

    for i, event in enumerate(events):
        if i == 0:
            new_time = event[0]
            smoothed.append([new_time, event[1], event[2]])
            continue

        gap = event[0] - events[i-1][0]

        # Compress long pauses
        if gap > max_pause:
            gap = max_pause

        # Ensure minimum gap between frames for smooth animation
        if gap < min_gap:
            gap = min_gap

        new_time += gap
        smoothed.append([round(new_time, 4), event[1], event[2]])

    # Write output
    with open(output_path, 'w') as f:
        f.write(json.dumps(header) + '\n')
        for event in smoothed:
            f.write(json.dumps(event) + '\n')

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} input.cast output.cast [max_pause]")
        sys.exit(1)

    max_pause = float(sys.argv[3]) if len(sys.argv) > 3 else 1.5
    process_cast(sys.argv[1], sys.argv[2], max_pause)
    print(f"Processed: {sys.argv[1]} -> {sys.argv[2]}")
