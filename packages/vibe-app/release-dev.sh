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

run_build() {
  profile="$1"
  platform="$2"
  extra_args="$3"
  eas build --profile "$profile" --platform "$platform" $build_args $extra_args
}

run_build development ios ""
run_build development android ""
run_build preview ios ""
run_build preview android ""
run_build development-store ios "$ios_submit_args"
run_build preview-store ios "$ios_submit_args"
