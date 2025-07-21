import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';

import LeftPanel from './LeftPanel.svelte';

describe('LeftPanel Component', () => {
  it('renders navigation items', () => {
    const { getByText } = render(LeftPanel);

    // Check if some key items are rendered
    expect(getByText('Transcription')).toBeTruthy();
    expect(getByText('Chat')).toBeTruthy();
    expect(getByText('Settings')).toBeTruthy();
  });

  it('dispatches navigate event with correct panel id on click', async () => {
    const { getByText, component } = render(LeftPanel);
    const dispatch = vi.fn(); // Create a mock function
    component.$on('navigate', dispatch); // Listen for the event

    const chatButton = getByText('Chat');
    await fireEvent.click(chatButton);

    // Check if dispatch was called correctly
    expect(dispatch).toHaveBeenCalled();
    expect(dispatch).toHaveBeenCalledWith(expect.objectContaining({
      detail: { panel: 'chat' }
    }));

    // Check if activeItem prop updates (optional, depends on internal logic visibility)
    // You might need to export activeItem or test this via App.svelte interaction
  });

  it('dispatches navigate event on Enter key press', async () => {
    const { getByText, component } = render(LeftPanel);
    const dispatch = vi.fn();
    component.$on('navigate', dispatch);

    const settingsButton = getByText('Settings');
    await fireEvent.keyDown(settingsButton, { key: 'Enter', code: 'Enter' });

    expect(dispatch).toHaveBeenCalled();
    expect(dispatch).toHaveBeenCalledWith(expect.objectContaining({
      detail: { panel: 'settings' }
    }));
  });

  it('dispatches navigate event on Space key press', async () => {
    const { getByText, component } = render(LeftPanel);
    const dispatch = vi.fn();
    component.$on('navigate', dispatch);

    const helpButton = getByText('Help & Documentation');
    await fireEvent.keyDown(helpButton, { key: ' ', code: 'Space' });

    expect(dispatch).toHaveBeenCalled();
    expect(dispatch).toHaveBeenCalledWith(expect.objectContaining({
      detail: { panel: 'help' }
    }));
  });

}); 
 