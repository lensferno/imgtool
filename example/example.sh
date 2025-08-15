#!/bin/bash
set -v

TEST_IMG="81727447.png"
BASENAME="${TEST_IMG%.*}"

mkdir -p test_dir
cp "$TEST_IMG" "./test_dir"
cp "$TEST_IMG" "./test_dir/${BASENAME}-2.png"

./imgtool -i test_dir -o test_dir2

./imgtool -i test_dir -o test_dir --prefix prefix_ --suffix _suffix

cp "$TEST_IMG" "${BASENAME}-3.png"

# At this point, `test_dir2` should exist and be a folder, otherwise it will be treated as a file and output.
mkdir -p test_dir2
./imgtool -i "${BASENAME}-3.png" -o test_dir2
./imgtool -i "$TEST_IMG" -o test_dir2  --prefix prefix_ --suffix _suffix

./imgtool -i "$TEST_IMG" -o "${BASENAME}-default.png"

./imgtool -i "$TEST_IMG" -o "${BASENAME}-png-q-90.png"        --png-params quality=90
./imgtool -i "$TEST_IMG" -o "${BASENAME}-png-q-60-zopfli.png" --png-params quality=60,force_zopfli=true

./imgtool -i "$TEST_IMG" -o "${BASENAME}-lossless.png" --lossless

./imgtool -i "$TEST_IMG" -o "${BASENAME}.jpg" -t jpg
./imgtool -i "$TEST_IMG" -o "${BASENAME}-jpeg-q90.jpg" -t jpg --jpeg-params quality=90

./imgtool -i "$TEST_IMG" -o "${BASENAME}-short_edge-300.png"              --resize-args short_edge:edge_size=300
./imgtool -i "$TEST_IMG" -o "${BASENAME}-short_edge-300.webp"             --resize-args short_edge:edge_size=300 -t webp
./imgtool -i "$TEST_IMG" -o "${BASENAME}-long_edge-600.png"               --resize-args long_edge:edge_size=600
./imgtool -i "$TEST_IMG" -o "${BASENAME}-size-800x600.png"                --resize-args size:w=800,h=600
./imgtool -i "$TEST_IMG" -o "${BASENAME}-scale-0.5.png"                   --resize-args scale:ratio=0.5
./imgtool -i "$TEST_IMG" -o "${BASENAME}-scale-0.5wx0.2h.png"             --resize-args scale:w=0.5,h=0.2
./imgtool -i "$TEST_IMG" -o "${BASENAME}-width-600.png"                   --resize-args width:w=600
./imgtool -i "$TEST_IMG" -o "${BASENAME}-height-400.png"                  --resize-args height:h=400
./imgtool -i "$TEST_IMG" -o "${BASENAME}-height-400-no-keep-ratio.png"    --resize-args height:h=400,keep_aspect_ratio=false
