# BestMe Frontend

This directory contains the frontend code for the BestMe application, built with Tauri and Vite.

## Structure

- `package.json`: Frontend dependencies and build scripts
- `package-lock.json`: Lock file for package dependencies
- `vite.config.js`: Vite configuration for the frontend build
- `tsconfig.json`: TypeScript configuration
- `index.html`: Main entry point for the frontend application
- `src/`: Source code for the frontend
  - `components/`: UI components
  - `pages/`: Page layouts
  - `styles/`: CSS styles
  - `utils/`: Helper functions
  - `main.ts`: Main entry point for the frontend code
  - `app.ts`: Main application component

## Development

To run the frontend in development mode:

```bash
cd ui
npm install
npm run dev
```

To build the frontend for production:

```bash
cd ui
npm run build
```

## Tauri Integration

The frontend communicates with the Rust backend using Tauri's API. The main integration points are:

- `invoke`: To call Rust functions from JavaScript
- `listen`: To listen for events from Rust
- `once`: To listen for one-time events from Rust

Example:

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Call a Rust function
const result = await invoke('plugin:audio:get_devices');

// Listen for events
const unlisten = await listen('transcription-update', (event) => {
  console.log('Transcription update:', event);
});
``` 
