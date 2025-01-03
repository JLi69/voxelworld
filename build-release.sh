#!/bin/sh

VOXELWORLD_DIR=voxelworld-linux/

# creates a linux release of voxelworld (package is in .tar.gz format)
rm -rf release/
mkdir release/$VOXELWORLD_DIR -p
# Copy the binary over
cp target/release/voxelworld release/$VOXELWORLD_DIR
# Copy `cfg.impfile`
cp cfg.impfile release/$VOXELWORLD_DIR
# Copy assets/
cp -r assets/ release/$VOXELWORLD_DIR
cd release/ && tar -czf voxelworld-linux.tar.gz $VOXELWORLD_DIR
