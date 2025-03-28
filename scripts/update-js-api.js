#!/usr/bin/env node

/**
 * This script updates JavaScript API invoke patterns from Tauri 1.x to Tauri 2.0 style
 * It scans UI files and updates invoke calls to the new format
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

// Results tracking
let filesScanned = 0;
let filesModified = 0;
let patternsUpdated = 0;

/**
 * Main function
 */
async function main() {
  console.log(`${colors.cyan}=== BestMe JavaScript API Updater ===${colors.reset}`);
  console.log('Updating JavaScript API invoke patterns from Tauri 1.x to Tauri 2.0 style\n');

  // Path to the UI directory
  const uiDir = path.join(process.cwd(), 'ui');
  
  // Check if UI directory exists
  if (!fs.existsSync(uiDir)) {
    console.error(`${colors.red}Error:${colors.reset} UI directory not found`);
    process.exit(1);
  }
  
  // Find all JS/TS/Svelte files in the UI directory (recursive)
  const files = await findJsFiles(uiDir);
  console.log(`Found ${files.length} files to scan\n`);
  
  // Process each file
  for (const file of files) {
    await processFile(file);
  }
  
  // Summarize results
  console.log(`\n${colors.cyan}=== Update Summary ===${colors.reset}`);
  console.log(`Files scanned: ${filesScanned}`);
  console.log(`Files modified: ${filesModified}`);
  console.log(`Patterns updated: ${patternsUpdated}`);

  if (patternsUpdated > 0) {
    console.log(`\n${colors.green}JavaScript API update completed successfully!${colors.reset}`);
    console.log('Updated invoke patterns to Tauri 2.0 style.');
    console.log('Please manually verify the changes and test the application.');
  } else {
    console.log(`\n${colors.yellow}No changes made to any files.${colors.reset}`);
    console.log('Either no old invoke patterns were found or they were already updated.');
  }
}

/**
 * Find all JavaScript/TypeScript/Svelte files in a directory (recursive)
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
 * Process a single file to update invoke patterns
 * @param {string} file - Path to the file
 */
async function processFile(file) {
  filesScanned++;
  
  try {
    // Read file content
    let content = await readFileAsync(file, 'utf8');
    const originalContent = content;
    
    // First, update Tauri imports
    if (content.includes('@tauri-apps/api/tauri')) {
      console.log(`${colors.blue}Updating imports in${colors.reset} ${path.relative(process.cwd(), file)}`);
      
      // Update the import pattern
      content = content.replace(
        /import\s+\{\s*invoke\s*\}\s+from\s+['"]@tauri-apps\/api\/tauri['"]/g,
        "import { invoke } from '@tauri-apps/api'"
      );
      
      // Also update other patterns
      content = content.replace(
        /import\s+\*\s+as\s+tauri\s+from\s+['"]@tauri-apps\/api\/tauri['"]/g,
        "import * as tauri from '@tauri-apps/api'"
      );
    }
    
    // Process invoke patterns
    // Look for the pattern invoke('command-name', { params })
    const oldInvokePattern = /invoke\(['"]([^:'"]+:[^:'"]+:[^'"]+)['"](,\s*(\{[^}]*\}))?\)/g;
    
    // Update to invoke.namespace.command(params)
    let match;
    while ((match = oldInvokePattern.exec(originalContent)) !== null) {
      const fullCommand = match[1];
      const params = match[2] || '';
      
      // Split the command into namespace and command parts
      const parts = fullCommand.split(':');
      if (parts.length !== 3) {
        console.log(`${colors.yellow}Warning:${colors.reset} Unexpected command format: ${fullCommand}`);
        continue;
      }
      
      const plugin = parts[0];
      const namespace = parts[1];
      const command = parts[2];
      
      // Create the new invoke pattern
      const newInvoke = `invoke.${namespace}.${command}${params}`;
      
      // Replace in content
      content = content.replace(
        new RegExp(`invoke\\(['"]${fullCommand}['"]${params ? params.replace(/\{/g, '\\{').replace(/\}/g, '\\}') : ''}\\)`, 'g'),
        newInvoke
      );
      
      console.log(`${colors.green}Updating${colors.reset} ${fullCommand} -> ${namespace}.${command}`);
      patternsUpdated++;
    }
    
    // If content changed, write back to file
    if (content !== originalContent) {
      await writeFileAsync(file, content, 'utf8');
      filesModified++;
      console.log(`${colors.green}Updated${colors.reset} ${path.relative(process.cwd(), file)}\n`);
    }
  } catch (error) {
    console.error(`${colors.red}Error processing file ${file}:${colors.reset}`, error.message);
  }
}

// Run the main function
main().catch(error => {
  console.error(`${colors.red}Error:${colors.reset} ${error.message}`);
  process.exit(1);
}); 
