#!/usr/bin/env bash
# Helper notes for a minimal Android + Termux host environment.
set -euo pipefail

echo "=== Termux-in-Physics-World: Android host setup ==="
echo
echo "1. Install Android Studio or command-line tools (sdkmanager, emulator, adb)."
echo "2. Create an AVD (x86_64 or arm64) with Google APIs."
echo "3. Start it:"
echo "     emulator -avd <YourAvdName>"
echo "4. Wait:"
echo "     adb wait-for-device"
echo "5. Install Termux (F-Droid APK recommended):"
echo "     adb install Termux.apk"
echo "6. From this repo:"
echo "     cargo run"
echo
echo "The physics app will stream the Android framebuffer onto the in-world screen"
echo "and launch com.termux when the package is present."
