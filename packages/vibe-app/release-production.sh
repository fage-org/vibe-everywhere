#!/bin/sh
set -e

build_args="--no-wait"
if [ "${VIBE_EAS_INTERACTIVE:-0}" != "1" ]; then
  build_args="$build_args --non-interactive"
fi

ios_submit_args=""
if [ -n "${VIBE_IOS_AUTO_SUBMIT_PROFILE:-}" ]; then
  ios_submit_args="--auto-submit-with-profile=${VIBE_IOS_AUTO_SUBMIT_PROFILE}"
fi

android_submit_args=""
if [ -n "${VIBE_ANDROID_AUTO_SUBMIT_PROFILE:-}" ]; then
  android_submit_args="--auto-submit-with-profile=${VIBE_ANDROID_AUTO_SUBMIT_PROFILE}"
fi

eas build --profile production --platform ios $build_args $ios_submit_args
eas build --profile production --platform android $build_args $android_submit_args
