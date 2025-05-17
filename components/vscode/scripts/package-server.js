#!/usr/bin/env node

/**
 * This script helps package the Dada language server binaries for different platforms.
 * It can be used in CI/CD pipelines to automate the process.
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Directories
const rootDir = path.resolve(__dirname, '..', '..', '..');
const binDir = path.resolve(__dirname, '..', 'bin');

// Platform configurations
const platforms = [
  {
    name: 'darwin-x64',
    displayName: 'macOS (Intel)',
    binaryName: 'dada-lsp-server',
    buildCommand: 'cargo build --release -p dada-lsp-server'
  },
  {
    name: 'darwin-arm64',
    displayName: 'macOS (Apple Silicon)',
    binaryName: 'dada-lsp-server',
    buildCommand: 'cargo build --release -p dada-lsp-server --target aarch64-apple-darwin'
  },
  {
    name: 'linux-x64',
    displayName: 'Linux (x64)',
    binaryName: 'dada-lsp-server',
    buildCommand: 'cargo build --release -p dada-lsp-server'
  },
  {
    name: 'win32-x64',
    displayName: 'Windows (x64)',
    binaryName: 'dada-lsp-server.exe',
    buildCommand: 'cargo build --release -p dada-lsp-server'
  }
];

// Ensure bin directories exist
for (const platform of platforms) {
  const platformDir = path.join(binDir, platform.name);
  if (!fs.existsSync(platformDir)) {
    fs.mkdirSync(platformDir, { recursive: true });
    console.log(`Created directory: ${platformDir}`);
  }
}

// Check if we're running in a CI environment
const isCI = process.env.CI === 'true';

// Determine which platform to build for
const currentPlatform = process.platform;
const currentArch = process.arch;

let platformsToBuild = platforms;

// If not in CI, only build for current platform
if (!isCI) {
  let platformName;
  
  if (currentPlatform === 'darwin') {
    platformName = `darwin-${currentArch}`;
  } else if (currentPlatform === 'linux') {
    platformName = 'linux-x64';
  } else if (currentPlatform === 'win32') {
    platformName = 'win32-x64';
  }
  
  platformsToBuild = platforms.filter(p => p.name === platformName);
  
  if (platformsToBuild.length === 0) {
    console.warn(`Warning: Unsupported platform ${currentPlatform}-${currentArch}`);
    console.warn('No binaries will be built. You can still use cargo run for development.');
    process.exit(0);
  }
}

// Build and package for each platform
for (const platform of platformsToBuild) {
  console.log(`Building for ${platform.displayName}...`);
  
  try {
    // Build the binary
    execSync(platform.buildCommand, { 
      cwd: rootDir,
      stdio: 'inherit'
    });
    
    // Determine source path
    let sourcePath;
    if (platform.name === 'darwin-arm64') {
      sourcePath = path.join(rootDir, 'target', 'aarch64-apple-darwin', 'release', platform.binaryName);
    } else {
      sourcePath = path.join(rootDir, 'target', 'release', platform.binaryName);
    }
    
    // Determine destination path
    const destPath = path.join(binDir, platform.name, platform.binaryName);
    
    // Copy the binary
    fs.copyFileSync(sourcePath, destPath);
    
    // Make executable on Unix platforms
    if (platform.name.startsWith('darwin') || platform.name.startsWith('linux')) {
      fs.chmodSync(destPath, '755');
    }
    
    console.log(`Successfully packaged ${platform.displayName} binary to ${destPath}`);
  } catch (error) {
    console.error(`Error building for ${platform.displayName}:`, error.message);
    if (!isCI) {
      console.log('You can still use cargo run for development.');
    } else {
      process.exit(1);
    }
  }
}

console.log('Packaging complete!');
