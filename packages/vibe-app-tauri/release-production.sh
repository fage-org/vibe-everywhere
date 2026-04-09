#!/usr/bin/env sh
set -eu

node ./scripts/write-release-tauri-properties.mjs production-candidate
yarn browser:build
yarn tauri:build:production-candidate
yarn mobile:android:production-candidate:apk
node ./scripts/package-release-artifacts.mjs production-candidate
