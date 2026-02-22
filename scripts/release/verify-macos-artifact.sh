#!/usr/bin/env bash
set -euo pipefail

BUNDLE_ROOT="${1:-src-tauri/target/release/bundle}"
SIGNING_MODE="${2:-unsigned}"

APP_PATH="$(find "$BUNDLE_ROOT/macos" -maxdepth 2 -type d -name "*.app" | head -n 1 || true)"
DMG_PATH="$(find "$BUNDLE_ROOT/dmg" -maxdepth 2 -type f -name "*.dmg" | head -n 1 || true)"
APP_BINARY_PATH="$(find "$APP_PATH/Contents/MacOS" -maxdepth 1 -type f | head -n 1 || true)"

if [[ -z "${APP_PATH}" || -z "${DMG_PATH}" || -z "${APP_BINARY_PATH}" ]]; then
  echo "Expected app and dmg artifacts under ${BUNDLE_ROOT}, but one or both are missing."
  exit 1
fi

if ! command -v shasum >/dev/null 2>&1; then
  echo "shasum is required for checksum generation."
  exit 1
fi

if ! command -v codesign >/dev/null 2>&1; then
  echo "codesign is required for macOS signature verification."
  exit 1
fi

if ! command -v spctl >/dev/null 2>&1; then
  echo "spctl is required for Gatekeeper verification."
  exit 1
fi

if ! command -v xcrun >/dev/null 2>&1; then
  echo "xcrun is required for notarization/staple verification."
  exit 1
fi

SIGNING_IDENTITY="unsigned"
CODESIGN_STATUS="not-run"
GATEKEEPER_STATUS="not-run"
NOTARIZATION_STATUS="not-run"
STAPLER_STATUS="not-run"

if [[ "${SIGNING_MODE}" == "signed" ]]; then
  codesign --verify --deep --strict --verbose=2 "${APP_PATH}"
  CODESIGN_STATUS="pass"

  SIGNING_IDENTITY="$(codesign -dv --verbose=4 "${APP_PATH}" 2>&1 | awk -F= '/^Authority=/{print $2; exit}' || true)"
  if [[ -z "${SIGNING_IDENTITY}" ]]; then
    SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY:-unknown}"
  fi

  spctl --assess --type execute --verbose=4 "${APP_PATH}"
  GATEKEEPER_STATUS="pass"

  if xcrun stapler validate -v "${APP_PATH}"; then
    STAPLER_STATUS="pass"
    NOTARIZATION_STATUS="stapled-app"
  elif xcrun stapler validate -v "${DMG_PATH}"; then
    STAPLER_STATUS="pass"
    NOTARIZATION_STATUS="stapled-dmg"
  else
    STAPLER_STATUS="fail"
    NOTARIZATION_STATUS="missing-or-invalid"
    echo "Signed build failed notarization/staple validation."
    exit 1
  fi
fi

APP_PACKAGE_SHA256="$(tar -cf - -C "$(dirname "${APP_PATH}")" "$(basename "${APP_PATH}")" | shasum -a 256 | awk '{print $1}')"
APP_BINARY_SHA256="$(shasum -a 256 "${APP_BINARY_PATH}" | awk '{print $1}')"
DMG_SHA256="$(shasum -a 256 "${DMG_PATH}" | awk '{print $1}')"

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  {
    echo "artifact_app_path=${APP_PATH}"
    echo "artifact_app_binary_path=${APP_BINARY_PATH}"
    echo "artifact_dmg_path=${DMG_PATH}"
    echo "artifact_app_sha256=${APP_PACKAGE_SHA256}"
    echo "artifact_app_binary_sha256=${APP_BINARY_SHA256}"
    echo "artifact_dmg_sha256=${DMG_SHA256}"
    echo "signing_identity=${SIGNING_IDENTITY}"
    echo "codesign_status=${CODESIGN_STATUS}"
    echo "gatekeeper_status=${GATEKEEPER_STATUS}"
    echo "notarization_status=${NOTARIZATION_STATUS}"
    echo "stapler_status=${STAPLER_STATUS}"
  } >> "${GITHUB_OUTPUT}"
fi

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    echo "### Artifact Verification"
    echo ""
    echo "- App path: \`${APP_PATH}\`"
    echo "- App binary path: \`${APP_BINARY_PATH}\`"
    echo "- DMG path: \`${DMG_PATH}\`"
    echo "- App SHA256 (bundle tar stream): \`${APP_PACKAGE_SHA256}\`"
    echo "- App binary SHA256: \`${APP_BINARY_SHA256}\`"
    echo "- DMG SHA256: \`${DMG_SHA256}\`"
    echo "- Signing identity: \`${SIGNING_IDENTITY}\`"
    echo "- Codesign status: \`${CODESIGN_STATUS}\`"
    echo "- Gatekeeper status: \`${GATEKEEPER_STATUS}\`"
    echo "- Notarization status: \`${NOTARIZATION_STATUS}\`"
    echo "- Stapler status: \`${STAPLER_STATUS}\`"
  } >> "${GITHUB_STEP_SUMMARY}"
fi
