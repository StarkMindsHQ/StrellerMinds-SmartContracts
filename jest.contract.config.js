module.exports = {
  displayName: 'contract-tests',
  preset: 'ts-jest',
  testEnvironment: 'node',
  rootDir: '.',
  testMatch: [
    '**/test/contracts/**/*.spec.ts',
    '**/test/contracts/**/*.test.ts',
  ],
  moduleNameMapper: {
    '@shared/(.*)': '<rootDir>/src/shared/$1',
  },
  globals: {
    'ts-jest': {
      tsconfig: './tsconfig.json',
    },
  },
  testTimeout: 30000,
  maxWorkers: 1,
  verbose: true,
};
