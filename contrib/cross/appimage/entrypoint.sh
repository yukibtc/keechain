#!/bin/sh

cp /target/x86_64-unknown-linux-gnu/release/keechain /usr/bin/keechain
chmod a+x /usr/bin/keechain
chmod a+x /keechain.AppDir/AppRun
chmod a+x /keechain.AppDir/keechain.desktop
APPIMAGE_EXTRACT_AND_RUN=1 linuxdeploy -e /usr/bin/keechain -d /keechain.AppDir/keechain.desktop -i /keechain.AppDir/keechain.png --appdir /keechain.AppDir --output appimage
cp *.AppImage /output