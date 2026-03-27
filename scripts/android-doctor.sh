#!/usr/bin/env bash

set -u

failures=0
warnings=0

info() {
  printf '[info] %s\n' "$1"
}

pass() {
  printf '[ok] %s\n' "$1"
}

warn() {
  printf '[warn] %s\n' "$1"
  warnings=$((warnings + 1))
}

fail() {
  printf '[fail] %s\n' "$1"
  failures=$((failures + 1))
}

check_command() {
  local command_name="$1"
  if command -v "$command_name" >/dev/null 2>&1; then
    pass "$command_name found at $(command -v "$command_name")"
  else
    fail "$command_name not found in PATH"
  fi
}

SDK_ROOT="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
SDKMANAGER="$SDK_ROOT/cmdline-tools/latest/bin/sdkmanager"
EXPECTED_PLATFORM="$SDK_ROOT/platforms/android-36"
EXPECTED_BUILD_TOOLS="$SDK_ROOT/build-tools/35.0.0"
EXPECTED_NDK_VERSION="${ANDROID_NDK_VERSION:-25.2.9519653}"
EXPECTED_NDK_DIR="${NDK_HOME:-${ANDROID_NDK_HOME:-$SDK_ROOT/ndk/$EXPECTED_NDK_VERSION}}"

info "Checking Android environment for Vibe Everywhere"
info "SDK root: $SDK_ROOT"

check_command java
check_command cargo
check_command rustup
check_command npm

if [[ -n "${JAVA_HOME:-}" ]]; then
  pass "JAVA_HOME is set to $JAVA_HOME"
else
  warn "JAVA_HOME is not set; builds may still work if java is on PATH, but the project docs recommend exporting it"
fi

if [[ -d "$SDK_ROOT" ]]; then
  pass "Android SDK directory exists"
else
  fail "Android SDK directory is missing at $SDK_ROOT"
fi

if [[ -x "$SDKMANAGER" ]]; then
  pass "sdkmanager found at $SDKMANAGER"
else
  fail "sdkmanager missing at $SDKMANAGER"
fi

if [[ -d "$SDK_ROOT/platform-tools" ]]; then
  pass "platform-tools installed"
else
  fail "platform-tools missing from $SDK_ROOT"
fi

if [[ -d "$EXPECTED_PLATFORM" ]]; then
  pass "Android platform android-36 installed"
else
  fail "Android platform android-36 missing from $SDK_ROOT/platforms"
fi

if [[ -d "$EXPECTED_BUILD_TOOLS" ]]; then
  pass "Android build-tools 35.0.0 installed"
else
  fail "Android build-tools 35.0.0 missing from $SDK_ROOT/build-tools"
fi

complete_ndk_dir=""
incomplete_ndk_dirs=()
if [[ -d "$SDK_ROOT/ndk" ]]; then
  while IFS= read -r ndk_dir; do
    if [[ -f "$ndk_dir/source.properties" ]]; then
      if [[ -z "$complete_ndk_dir" ]]; then
        complete_ndk_dir="$ndk_dir"
      fi
    else
      incomplete_ndk_dirs+=("$ndk_dir")
    fi
  done < <(find "$SDK_ROOT/ndk" -mindepth 1 -maxdepth 1 -type d | sort)
else
  fail "No NDK directory exists at $SDK_ROOT/ndk"
fi

if [[ -f "$EXPECTED_NDK_DIR/source.properties" ]]; then
  ndk_revision="$(grep '^Pkg.Revision' "$EXPECTED_NDK_DIR/source.properties" 2>/dev/null | cut -d'=' -f2- | xargs)"
  pass "Expected NDK is complete at $EXPECTED_NDK_DIR${ndk_revision:+ (revision $ndk_revision)}"
else
  fail "Expected NDK metadata missing at $EXPECTED_NDK_DIR/source.properties"
  if [[ -d "$EXPECTED_NDK_DIR" ]]; then
    warn "That NDK directory exists but looks incomplete. This usually means sdkmanager left a partial install behind."
  fi
  if [[ -n "$complete_ndk_dir" ]]; then
    warn "A complete NDK was found at $complete_ndk_dir. Export NDK_HOME and ANDROID_NDK_HOME to that path or reinstall $EXPECTED_NDK_VERSION."
  fi
fi

if ((${#incomplete_ndk_dirs[@]})); then
  warn "Incomplete NDK directories detected: ${incomplete_ndk_dirs[*]}"
  warn "Tauri may auto-select the highest NDK version, so remove broken directories or set NDK_HOME explicitly."
fi

if rustup target list --installed | grep -qx 'aarch64-linux-android'; then
  pass "Rust target aarch64-linux-android is installed"
else
  fail "Rust target aarch64-linux-android is not installed"
fi

if ((failures > 0)); then
  printf '\nSuggested repair commands:\n'
  if [[ -x "$SDKMANAGER" ]]; then
    printf '  yes | "%s" --licenses\n' "$SDKMANAGER"
    printf '  "%s" --install "platform-tools" "platforms;android-36" "build-tools;35.0.0" "ndk;%s"\n' "$SDKMANAGER" "$EXPECTED_NDK_VERSION"
  fi
  printf '  export ANDROID_HOME="%s"\n' "$SDK_ROOT"
  printf '  export ANDROID_SDK_ROOT="$ANDROID_HOME"\n'
  printf '  export NDK_HOME="%s"\n' "$EXPECTED_NDK_DIR"
  printf '  export ANDROID_NDK_HOME="$NDK_HOME"\n'
  printf '  rustup target add aarch64-linux-android\n'
  exit 1
fi

printf '\nEnvironment looks ready for:\n'
printf '  cd apps/vibe-app && npm run android:build:debug:apk\n'

if ((warnings > 0)); then
  printf '\nWarnings were emitted above; resolve them before publishing release artifacts.\n'
fi
