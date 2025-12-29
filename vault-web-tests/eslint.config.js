import eslint from '@eslint/js';
import { defineConfig, globalIgnores } from 'eslint/config';
import tseslint from 'typescript-eslint';

export default defineConfig(
  eslint.configs.recommended,
  tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        tsconfigRootDir: import.meta.dirname,
        projectService: true,
      },
    },
  },
  [
    {
      rules: {
        '@typescript-eslint/no-floating-promises': 'error',
      },
    },
    globalIgnores([
      'eslint.config.js',
      'scripts/generate-playwright-auth-user.js',
      'vault-wasm-nodejs/**/*',
    ]),
  ],
);
