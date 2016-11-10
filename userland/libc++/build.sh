#!/bin/bash

GCC_SRC_DIR=$1

#export CFLAGS="-fbracket-depth=1024"
#export CXXFLAGS="-fbracket-depth=1024"

export CFLAGS_FOR_TARGET='-g -Os -ffunction-sections -fdata-sections -fPIC -msingle-pic-base -mno-pic-data-is-text-relative -mthumb -mcpu=cortex-m0 -isystem /home/ppannuto/tock/userland/newlib/newlib-2.2.0.20150423/newlib/libc/include'
export CXXFLAGS_FOR_TARGET='-g -Os -ffunction-sections -fdata-sections -fPIC -msingle-pic-base -mno-pic-data-is-text-relative -mthumb -mcpu=cortex-m0 -isystem /home/ppannuto/tock/userland/newlib/newlib-2.2.0.20150423/newlib/libc/include'

#export LDFLAGS_FOR_TARGET='--specs=nosys.specs'

# 6.2.0:
$GCC_SRC_DIR/configure \
  --build=x86_64-linux-gnu \
  --host=x86_64-linux-gnu \
  --target=arm-none-eabi \
  --with-cpu=cortex-m0 \
  --disable-fpu \
  --with-newlib \
  --with-headers=/home/ppannuto/tock/userland/newlib/newlib-2.2.0.20150423/newlib/libc/include \
  --enable-languages="c c++" \

make -j

