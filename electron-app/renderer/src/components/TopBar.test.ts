import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';

import TopBar from './TopBar.svelte';

describe('TopBar Component', () => {
  it('renders without errors', () => {
    // Provide default/empty props to satisfy the component's requirements
    const props = {
      isRecording: false,
      aiModels: [], // Provide empty array for {#each}
      selectedWhisperModel: '', // Provide default string
      languages: [], // Provide empty array for {#each}
      selectedLanguage: '' // Provide default string
    };
    render(TopBar, { props });

    // Basic check: Expect the component container to be in the document
    expect(screen.getByRole('toolbar')).toBeTruthy(); 

    // Check if select elements are present (even if disabled/empty)
    expect(screen.getByRole('combobox', { name: /whisper model/i })).toBeTruthy(); // Assuming implicit label or aria-label
    expect(screen.getByRole('combobox', { name: /language/i })).toBeTruthy(); // Assuming implicit label or aria-label
  });

  // Add more tests here for specific functionality, like button clicks or prop handling
}); 
