module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'scope-empty': [2, 'never'],
    // Enforce a readable, portable scope convention without maintaining a fragile static allowlist.
    'scope-case': [2, 'always', 'kebab-case'],
  },
};
