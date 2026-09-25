// The lint of the layer and its tests: ESLint's recommended rules, typescript-eslint's strict and stylistic rules with type
// information, the identifiers of the Google TypeScript Style Guide, and the TSDoc syntax of every comment.

import eslint from '@eslint/js';
import tsdoc from 'eslint-plugin-tsdoc';
import { defineConfig } from 'eslint/config';
import tseslint from 'typescript-eslint';

export default defineConfig(
    { ignores: ['coverage/', 'dist/', 'src/internal/'] },
    eslint.configs.recommended,
    tseslint.configs.strictTypeChecked,
    tseslint.configs.stylisticTypeChecked,
    {
        languageOptions: {
            parserOptions: {
                // The build script and this file are in no TypeScript project, and the tests are in their own, `test/tsconfig.json`.
                projectService: { allowDefaultProject: ['build.js', 'eslint.config.js'] },
                tsconfigRootDir: import.meta.dirname,
            },
        },
        linterOptions: { reportUnusedDisableDirectives: 'error', reportUnusedInlineConfigs: 'error' },
        plugins: { tsdoc },
        rules: {
            // ts.dev: `T[]` for a simple element type, `Array<T>` for any other.
            '@typescript-eslint/array-type': ['error', { default: 'array-simple', readonly: 'array-simple' }],
            '@typescript-eslint/explicit-module-boundary-types': 'error',
            // ts.dev's identifiers: UpperCamelCase for types; CONSTANT_CASE for enum members, and allowed for module constants
            // and static read-only properties; lowerCamelCase for everything else, but for the default import of a package
            // that names it in UpperCamelCase; no leading or trailing underscore.
            '@typescript-eslint/naming-convention': [
                'error',
                { selector: 'default', format: ['camelCase'], leadingUnderscore: 'forbid', trailingUnderscore: 'forbid' },
                { selector: 'import', format: ['camelCase', 'PascalCase'] },
                { selector: 'typeLike', format: ['PascalCase'] },
                { selector: 'enumMember', format: ['UPPER_CASE'] },
                { selector: 'variable', modifiers: ['const', 'global'], format: ['camelCase', 'UPPER_CASE'] },
                { selector: 'classProperty', modifiers: ['static', 'readonly'], format: ['camelCase', 'UPPER_CASE'] },
            ],
            '@typescript-eslint/prefer-readonly': 'error',
            '@typescript-eslint/switch-exhaustiveness-check': [
                'error',
                { considerDefaultExhaustiveForUnions: false, requireDefaultForNonUnion: true },
            ],
            curly: ['error', 'all'],
            eqeqeq: ['error', 'always'],
            'tsdoc/syntax': 'error',
            // The constant comes first in a comparison, as in every language of the repository (AGENTS.md).
            yoda: ['error', 'always'],
        },
    },
    { files: ['build.js', 'eslint.config.js'], extends: [tseslint.configs.disableTypeChecked] },
);
