#!/bin/bash

nsf_file="$1"
raw_video_file="$(basename "$nsf_file" .nsf)_video.raw"
raw_audio_file="$(basename "$nsf_file" .nsf)_audio.raw"
final_output="$(basename "$nsf_file" .nsf)_discord.mp4"

requested_duration_seconds="${2:-60}"
fade_duration="${3:-4}"
video_end_padding="${4:-1}"

((actual_duration_seconds=$requested_duration_seconds+$video_end_padding+$fade_duration))
((duration_frames=$actual_duration_seconds*60))

((fade_start=$requested_duration_seconds))

echo "=== User Provided Options ==="
echo "NSF: $nsf_file"
echo "Song Duration: $requested_duration_seconds"
echo "Fade Duration: $fade_duration"
echo "End Video Padding: $video_end_padding"
echo ""
echo "=== Derived Options ==="
echo "Raw Video: $raw_video_file"
echo "Raw Audio: $raw_audio_file"
echo "Final: $final_output"
echo "Actual Duration (Seconds): $actual_duration_seconds"
echo "Actual Duration (Frames): $duration_frames"
echo "Fade Start: $fade_start"
echo ""

echo "=== Capturing $duration_frames of Piano Roll output from $nsf_file ... ==="
cargo run --release -- cartridge "$nsf_file" video pianoroll "$raw_video_file" audio "$raw_audio_file" frames $duration_frames

echo "=== Converting $raw_video_file and $raw_audio_file to $final_output ... ==="
# piano roll settings
ffmpeg \
  -y -f rawvideo -pix_fmt rgb24 -s 1280x720 -r 60.0988 -i "$raw_video_file" \
  -f s16be -i "$raw_audio_file" \
  -c:v libx264 -crf 21 -preset veryslow -pix_fmt yuv420p \
  -vf "scale=1280:720, scale=out_color_matrix=bt709, fps=fps=60, fade=t=out:st=$fade_start:d=$fade_duration" \
  -af "afade=t=out:st=$fade_start:d=$fade_duration" \
  -color_range 1 -colorspace bt709 -color_trc bt709 -color_primaries bt709 -movflags faststart \
  -c:a aac -b:a 192k \
  "$final_output"

echo "=== Cleaning up temporary files ... ==="
rm "$raw_video_file"
rm "$raw_audio_file"
