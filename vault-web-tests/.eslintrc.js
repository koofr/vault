const path = require('path');

module.exports = {
  extends: ['eslint:recommended', 'plugin:@typescript-eslint/recommended'],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 2020,
    project: path.join(__dirname, 'tsconfig.json'),
    tsconfigRootDir: __dirname,
  },
  plugins: ['@typescript-eslint'],
  rules: {
    '@typescript-eslint/no-floating-promises': 'error',
  },
  ignorePatterns: ['.eslintrc.js', 'scripts/generate-playwright-auth-user.js'],
};
