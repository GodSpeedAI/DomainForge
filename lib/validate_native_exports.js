// validate_native_exports.js
// Helper to verify native binding exports and throw a descriptive error when missing.
"use strict";

function validateNativeExports(nativeBinding, requiredExports) {
  const missing = requiredExports.filter((name) => !nativeBinding || !Object.prototype.hasOwnProperty.call(nativeBinding, name));
  if (missing.length > 0) {
    throw new Error(`Native binding missing required export(s): ${missing.join(', ')}. Ensure the native addon was built and exposes these symbol(s).`);
  }
}

module.exports = { validateNativeExports };
