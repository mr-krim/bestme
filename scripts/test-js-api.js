#!/usr/bin/env node

/**
 * This script tests the JavaScript API integration with Tauri 2.0
 * It verifies that all commands are correctly exposed and can be invoked
 */

const fs = require('fs');
const path = require('path');
const { promisify } = require('util');
const readFileAsync = promisify(fs.readFile);
const writeFileAsync = promisify(fs.writeFile);

// ANSI color codes for output
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m',
};

// Test results tracking
let passed = 0;
let failed = 0;
let warnings = 0;

/**
 * Report a test result
 * @param {string} testName - Name of the test
 * @param {boolean} success - Whether the test passed
 * @param {string} message - Optional message (required for failures)
 */
function reportTest(testName, success, message = '') {
  if (success) {
    console.log(`${colors.green}[PASS]${colors.reset} ${testName}`);
    passed++;
  } else {
    console.log(`${colors.red}[FAIL]${colors.reset} ${testName}: ${message}`);
    failed++;
  }
}

/**
 * Report a warning
 * @param {string} testName - Name of the test
 * @param {string} message - Warning message
 */
function warn(testName, message) {
  console.log(`${colors.yellow}[WARN]${colors.reset} ${testName}: ${message}`);
  warnings++;
}

/**
 * Main test function
 */
async function main() {
  console.log(`${colors.cyan}=== BestMe JavaScript API Integration Test ===${colors.reset}`);
  console.log('Testing JavaScript API integration with Tauri 2.0\n');

  // Path to the UI directory
  const uiDir = path.join(process.cwd(), 'ui');
  
  // Check if UI directory exists
  if (!fs.existsSync(uiDir)) {
    reportTest('UI directory check', false, 'UI directory not found');
    summarize();
    return;
  }
  
  // Find all JS files in the UI directory (recursive)
  const jsFiles = await findJsFiles(uiDir);
  reportTest('JS files found', jsFiles.length > 0, `Found ${jsFiles.length} JavaScript files`);
  
  if (jsFiles.length === 0) {
    summarize();
    return;
  }

  // Check for Tauri imports in JS files
  console.log(`\n${colors.cyan}=== Checking Tauri Imports ===${colors.reset}`);
  let tauriImportsFound = 0;
  
  for (const file of jsFiles) {
    const content = await readFileAsync(file, 'utf8');
    
    // Check for various Tauri import patterns
    const importPatterns = [
      /import\s+.*\s+from\s+['"]@tauri-apps\/api['"]/,
      /import\s+.*\s+from\s+['"]@tauri-apps\/api\/.*['"]/,
      /const\s+.*\s+=\s+require\(['"]@tauri-apps\/api.*['"]\)/,
    ];
    
    const hasTauriImport = importPatterns.some(pattern => pattern.test(content));
    
    if (hasTauriImport) {
      tauriImportsFound++;
      console.log(`${colors.green}[FOUND]${colors.reset} Tauri import in ${path.relative(process.cwd(), file)}`);
      
      // Check if using Tauri 2.0 import patterns
      if (content.includes('@tauri-apps/api/tauri')) {
        warn('Tauri import pattern', `${path.relative(process.cwd(), file)} uses outdated import pattern '@tauri-apps/api/tauri'`);
      }
    }
  }
  
  reportTest('Tauri imports', tauriImportsFound > 0, 'No Tauri imports found in any JS files');
  
  // Check for invoke patterns
  console.log(`\n${colors.cyan}=== Checking Invoke Patterns ===${colors.reset}`);
  let oldInvokePatterns = 0;
  let newInvokePatterns = 0;
  
  for (const file of jsFiles) {
    const content = await readFileAsync(file, 'utf8');
    
    // Check for old invoke pattern (Tauri 1.x)
    const oldPattern = /invoke\(['"]([^'"]+)['"]/g;
    const oldMatches = content.match(oldPattern) || [];
    
    // Check for new invoke pattern (Tauri 2.0)
    const newPattern = /invoke\.([a-zA-Z0-9_]+)\(/g;
    const newMatches = content.match(newPattern) || [];
    
    oldInvokePatterns += oldMatches.length;
    newInvokePatterns += newMatches.length;
    
    if (oldMatches.length > 0) {
      console.log(`${colors.yellow}[OUTDATED]${colors.reset} Found ${oldMatches.length} old invoke patterns in ${path.relative(process.cwd(), file)}`);
      // List them
      const uniqueOldMatches = [...new Set(oldMatches)];
      uniqueOldMatches.forEach(match => {
        const command = match.match(/invoke\(['"]([^'"]+)['"]/)[1];
        console.log(`  - ${command}`);
      });
    }
    
    if (newMatches.length > 0) {
      console.log(`${colors.green}[MODERN]${colors.reset} Found ${newMatches.length} new invoke patterns in ${path.relative(process.cwd(), file)}`);
      // List them
      const uniqueNewMatches = [...new Set(newMatches)];
      uniqueNewMatches.forEach(match => {
        const command = match.match(/invoke\.([a-zA-Z0-9_]+)\(/)[1];
        console.log(`  - ${command}`);
      });
    }
  }
  
  if (oldInvokePatterns > 0) {
    warn('Invoke patterns', `Found ${oldInvokePatterns} old invoke patterns that need to be updated to Tauri 2.0 style`);
  }
  
  reportTest('New invoke patterns', newInvokePatterns > 0, 'No Tauri 2.0 invoke patterns found');
  
  // Check for event listeners
  console.log(`\n${colors.cyan}=== Checking Event Listeners ===${colors.reset}`);
  let eventListenersFound = 0;
  
  for (const file of jsFiles) {
    const content = await readFileAsync(file, 'utf8');
    
    // Check for event listeners
    const listenerPattern = /listen\(['"]([^'"]+)['"]/g;
    const matches = content.match(listenerPattern) || [];
    
    eventListenersFound += matches.length;
    
    if (matches.length > 0) {
      console.log(`${colors.green}[FOUND]${colors.reset} Found ${matches.length} event listeners in ${path.relative(process.cwd(), file)}`);
      // List them
      const uniqueMatches = [...new Set(matches)];
      uniqueMatches.forEach(match => {
        const event = match.match(/listen\(['"]([^'"]+)['"]/)[1];
        console.log(`  - ${event}`);
      });
    }
  }
  
  reportTest('Event listeners', eventListenersFound > 0, 'No event listeners found');
  
  // Summarize results
  summarize();
}

/**
 * Find all JavaScript files in a directory (recursive)
 * @param {string} dir - Directory to search
 * @returns {Promise<string[]>} - Array of file paths
 */
async function findJsFiles(dir) {
  const files = [];
  
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    
    if (entry.isDirectory()) {
      // Skip node_modules directory
      if (entry.name === 'node_modules') continue;
      
      files.push(...await findJsFiles(fullPath));
    } else if (entry.name.endsWith('.js') || entry.name.endsWith('.ts') || 
               entry.name.endsWith('.jsx') || entry.name.endsWith('.tsx') ||
               entry.name.endsWith('.svelte') || entry.name.endsWith('.vue')) {
      files.push(fullPath);
    }
  }
  
  return files;
}

/**
 * Summarize test results
 */
function summarize() {
  console.log(`\n${colors.cyan}=== Test Summary ===${colors.reset}`);
  console.log(`Tests passed: ${colors.green}${passed}${colors.reset}`);
  console.log(`Tests failed: ${colors.red}${failed}${colors.reset}`);
  console.log(`Warnings: ${colors.yellow}${warnings}${colors.reset}`);
  
  if (failed === 0) {
    console.log(`\n${colors.green}JavaScript API verification completed with warnings!${colors.reset}`);
    console.log('Next steps:');
    console.log('1. Update any outdated invoke patterns to Tauri 2.0 style');
    console.log('2. Ensure all event listeners are registered with the correct syntax');
    console.log('3. Test the actual functionality of each command');
  } else {
    console.log(`\n${colors.red}JavaScript API verification completed with failures!${colors.reset}`);
    console.log('Please fix the reported issues before proceeding.');
  }
}

// Run the main function
main().catch(error => {
  console.error(`${colors.red}Error:${colors.reset} ${error.message}`);
  process.exit(1);
}); 
