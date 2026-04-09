#!/usr/bin/env sh
set -eu

node ./scripts/write-release-tauri-properties.mjs preview
yarn browser:build:preview
yarn tauri:build:preview
yarn mobile:android:preview:apk
node ./scripts/package-release-artifacts.mjs preview
