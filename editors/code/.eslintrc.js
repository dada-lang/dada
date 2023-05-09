module.exports = {
  env: {
    es2021: true,
    node: true,
  },
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "prettier",
  ],
  overrides: [],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
    tsconfigRootDir: __dirname,
    project: "tsconfig.eslint.json",
  },
  root: true,
  plugins: ["@typescript-eslint"],
  rules: {
    "@typescript-eslint/member-delimiter-style": "error",
    "@typescript-eslint/no-unnecessary-type-assertion": "error",
    "@typescript-eslint/no-floating-promises": "error",
    camelcase: ["error"],
    eqeqeq: ["error", "always", { null: "ignore" }],
    curly: ["error"],
    "linebreak-style": ["error", "unix"],
    quotes: ["error", "double"],
    semi: ["error", "always"],
    "no-console": ["error", { allow: ["warn", "error"] }],
    "prefer-const": "error",
  },
};
