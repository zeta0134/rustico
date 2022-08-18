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

raw_video_file="$(basename "$nsf_file" .nsf)_video.raw"
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
echo "Raw Video: $raw_video_file"
echo "Raw Audio: $raw_audio_file"
echo "Final: $final_output"
echo "Actual Duration (Seconds): $actual_duration_seconds"
echo "Actual Duration (Frames): $duration_frames"
echo "Fade Start: $fade_start"
echo "Render Width: $render_width"
echo "Video Width: $output_width"
echo "Config: $base_config"
echo ""

echo "=== Capturing $duration_frames of Piano Roll output from $nsf_file ... ==="
cargo run --release -- \
  cartridge "$nsf_file" \
  track "$track_index" \
  config "$base_config" \
  config "configs/piano_roll_colors.toml" \
  video pianoroll "$raw_video_file" audio "$raw_audio_file" \
  frames $duration_frames

echo "=== Converting $raw_video_file and $raw_audio_file to $final_output ... ==="
# piano roll settings
ffmpeg \
  -y -f rawvideo -pix_fmt rgb24 -s "${render_width}x${render_height}" -r 60.0988 -i "$raw_video_file" \
  -f s16be -i "$raw_audio_file" \
  -c:v libx264 -crf "$video_crf" -preset veryslow -pix_fmt yuv420p \
  -vf "scale=${output_width}:${output_height}:flags=neighbor, scale=out_color_matrix=bt709, fps=fps=60, fade=t=out:st=$fade_start:d=$fade_duration" \
  -af "loudnorm, afade=t=out:st=$fade_start:d=$fade_duration" \
  -color_range 1 -colorspace bt709 -color_trc bt709 -color_primaries bt709 -movflags faststart \
  -c:a aac -b:a "$audio_bitrate" \
  "$final_output"

echo "=== Cleaning up temporary files ... ==="
rm "$raw_video_file"
rm "$raw_audio_file"

