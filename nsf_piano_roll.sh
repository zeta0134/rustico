#!/bin/bash

nsf_file=""
requested_duration_seconds="60"
render_height="720"
fade_duration="4"
video_end_padding="1"
video_crf="15"
audio_bitrate="384k"
video_scale="1"
track_index="1"

while (( $# > 1 )); do
  case $1 in
    --size) shift; render_height="$1" ;;
    --duration) shift; requested_duration_seconds="$1" ;;
    --fade-length) shift; fade_duration="$1" ;;
    --end-padding) shift; video_end_padding="$1" ;;
    --crf) shift; video_crf="$1" ;;
    --audio-bitrate) shift; audio_bitrate="$1" ;;
    --scale) shift; video_scale="$1" ;;
    --track) shift; track_index="$1" ;;
    *) nsf_file="$1" ;;
  esac;
  shift
done

intermediate_video_file="$(basename "$nsf_file" .nsf)_intermediate_video.mp4"
raw_audio_file="$(basename "$nsf_file" .nsf)_audio.raw"
final_output="${nsf_file%.*}.mp4"

((actual_duration_seconds=$requested_duration_seconds+$video_end_padding+$fade_duration))
((duration_frames=$actual_duration_seconds*60))

case "$render_height" in
  270) render_width=480 ;;
  480) render_width=854 ;;
  720) render_width=1280 ;;
  1080) render_width=1920 ;;
  *) echo "Invalid video height: $render_height, giving up."; exit -1 ;;
esac

base_config="configs/piano_roll_${render_height}p.toml"

output_height=$(( render_height*video_scale ))
output_width=$(( render_width*video_scale ))

((fade_start=$requested_duration_seconds))

echo "=== User Provided Options ==="
echo "NSF: $nsf_file"
echo "Song Duration: $requested_duration_seconds"
echo "Fade Duration: $fade_duration"
echo "End Video Padding: $video_end_padding"
echo "Render Height: $render_height"
echo "Video Height: $output_height"
echo "Video Quality (CRF): $video_crf"
echo "Audio Quality: $audio_bitrate"
echo ""
echo "=== Derived Options ==="
echo "Intermediate Video: $intermediate_video_file"
echo "Raw Audio: $raw_audio_file"
echo "Final: $final_output"
echo "Actual Duration (Seconds): $actual_duration_seconds"
echo "Actual Duration (Frames): $duration_frames"
echo "Fade Start: $fade_start"
echo "Render Width: $render_width"
echo "Video Width: $output_width"
echo "Config: $base_config"
echo ""

# Make a named pipe to funnel the video stream through. This prevents us from needing
# to store the uncompressed (and very large) generated video frames to disk while
# the render is going.

# Note that we *are* still writing the uncompressed audio to disk, as we can't reliably
# use two named pipes here without risking starvation.

rm __videopipe || true # if a previous run failed, the old pipe may still exist in a weird state
mkfifo __videopipe

echo "=== Capturing $duration_frames frames of Piano Roll output from $nsf_file ... ==="
cargo run --release -- \
  cartridge "$nsf_file" \
  track "$track_index" \
  config "$base_config" \
  config "configs/piano_roll_colors.toml" \
  video pianoroll __videopipe audio "$raw_audio_file" \
  frames $duration_frames &

echo "=== Converting captured video to $intermediate_video_file ... ==="
# piano roll settings
ffmpeg -y \
  -f rawvideo -pix_fmt rgba -s "${render_width}x${render_height}" -r 60.0988 -i __videopipe \
  -c:v libx264 -crf "$video_crf" -preset veryslow -pix_fmt yuv420p \
  -vf "scale=${output_width}:${output_height}:flags=neighbor, scale=out_color_matrix=bt709, fps=fps=60, fade=t=out:st=$fade_start:d=$fade_duration" \
  -color_range 1 -colorspace bt709 -color_trc bt709 -color_primaries bt709 -movflags faststart \
  "$intermediate_video_file"

echo "=== Combining intermediate video and captured audio $raw_audio_file ... ==="
ffmpeg -y \
  -i "$intermediate_video_file" \
  -f s16be -i "$raw_audio_file" \
  -af "afade=t=out:st=$fade_start:d=$fade_duration" \
  -c:a aac -b:a "$audio_bitrate" \
  "$final_output"

echo "=== Cleaning up temporary files ... ==="
rm "$intermediate_video_file"
rm "$raw_audio_file"
rm __videopipe

echo "=== Success! ==="