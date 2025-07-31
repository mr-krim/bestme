import App from './App.svelte';

// Get the app element
const appElement = document.getElementById('app') as HTMLElement;

// Clear any loading content
appElement.innerHTML = '';

// Initialize the app
const app = new App({
  target: appElement,
});

export default app; 
