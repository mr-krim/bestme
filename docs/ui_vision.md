# BestMe UI Vision & Progress

## 1. Core Application Structure (Completed)

*   **Layout:**
    *   Three-panel design: Left (Navigation), Middle (Context-aware lists), Main (Active content).
    *   Top bar: Global actions (recording, model/language selection, settings access).
    *   Bottom bar: Status information (version, online status, system metrics).
*   **Navigation:**
    *   `LeftPanel` items control `activePanel` state in `App.svelte`.
    *   `MiddlePanel` items control `selectedItemId` state in `App.svelte`.
*   **State Persistence:**
    *   `activePanel` and `selectedItemId` persisted to `localStorage`.
*   **Styling & Theme:**
    *   Basic light/dark theme switching implemented.
    *   CSS variables for theming.

## 2. Component Implementation (Ongoing)

*   **Core Shell:**
    *   `App.svelte`: Main application component, manages state, panel transitions (slide), and event handling.
*   **Layout Components:**
    *   `LeftPanel.svelte`: Navigation.
    *   `MiddlePanel.svelte`: Displays lists based on `activePanel`.
    *   `MainPanel.svelte`: Displays content based on `activePanel` and `selectedItem`.
    *   `TopBar.svelte`: Implemented with placeholders for model/language.
    *   `BottomBar.svelte` (formerly `StatusBar.svelte`): Displays system info.
*   **View Components (Content for MainPanel):**
    *   `TranscriptionView.svelte`: Basic placeholder for live transcription.
    *   `ChatView.svelte`: Basic structure for chat interface.
    *   `SettingsView.svelte`: Fetches and saves settings.
    *   `SavedTranscriptView.svelte`: **Newly created** to display details of a selected saved transcript (title, date, content) and includes a delete button. Implemented in `ui/src/views/SavedTranscriptView.svelte`.

## 3. Backend Integration (Partial)

*   **System Monitoring (Implemented):**
    *   `get_cpu_usage`, `get_memory_usage` (Rust/sysinfo) called by `App.svelte` and data passed to `BottomBar.svelte`.
*   **Configuration & Lists (Implemented):**
    *   `get_settings`, `save_all_settings`, `get_audio_devices`, `get_whisper_models`, `get_supported_languages` (Rust) used by `SettingsView.svelte` and `TopBar.svelte`.
*   **Online Status (Implemented):**
    *   `get_online_status` (Rust placeholder) called periodically by `App.svelte`.
*   **Transcription Lifecycle (Event-driven stubs):**
    *   `start_transcription`, `stop_transcription` invoke calls.
    *   Event listeners for `transcription:update`, `transcription:clear`, `transcribe:error`, `transcribe:started`, `transcribe:stopped`.
*   **Saved Transcripts (CRUD operations - partial):**
    *   `get_saved_transcript_list`: Fetches list for `MiddlePanel`.
    *   `get_saved_transcript`: **Implemented** in `App.svelte` to fetch full content for a selected item.
    *   `save_transcript`: Implemented.
    *   `delete_saved_transcript`: Implemented.
*   **Chat Sessions (CRUD operations - partial):**
    *   `get_chat_session_list`: Fetches list for `MiddlePanel`.
    *   `get_chat_session`: Fetches messages for a selected chat.
    *   `send_chat_message`: Implemented.
    *   `delete_chat_session`: Implemented.

## 4. Data Flow & State Management (Ongoing)

*   `App.svelte` acts as the central state manager.
*   Props down, events up for inter-component communication.
*   Reactive statements (`$:`) for deriving state and triggering data fetching:
    *   `middlePanelItems` reloaded when `activePanel` changes.
    *   `selectedItem` derived from `selectedItemId` and `middlePanelItems`.
    *   `selectedTranscriptContent` fetched when `selectedItemId` changes and `activePanel` is 'saved-transcripts'.
    *   `currentChatMessages` fetched when `selectedItemId` changes and `activePanel` is 'chat'.
*   `MainPanel.svelte` now receives `selectedItem` and `selectedTranscriptContent` to display details, specifically for `SavedTranscriptView`.

## 5. Testing (Setup & Initial Tests - Needs Attention)

*   **Framework:** Vitest and Svelte Testing Library.
*   **Configuration:** `vite.config.ts` and `tsconfig.json` updated for tests.
*   **Mocking:** Tauri `invoke` and `listen` are mocked; `localStorage` is mocked.
*   **Existing Tests:**
    *   `App.test.ts`: Initial rendering, panel switching, localStorage interaction, item selection reset.
    *   `LeftPanel.test.ts`: Rendering and event dispatch.
    *   `TopBar.test.ts`: Basic rendering.
*   **Current Status: Tests are failing.** Failures were observed in `App.test.ts`, `LeftPanel.test.ts` (implicitly via App), and `TopBar.test.ts`. Issues seem related to elements not being found, possibly due to async operations, mocking, or recent component changes not yet reflected correctly in tests or component logic.

## 6. Refactoring & General Improvements (Ongoing)

*   `.gitignore` updated.
*   Type safety improvements with `src/types.ts`.
*   Accessibility improvements (ARIA roles, keyboard navigation on clickable divs).

## 7. Current Focus & Next Steps

**Immediate Priority: Stabilize and Fix Tests**

1.  **Resolve Linter Errors:**
    *   **`ui/src/components/MainPanel.svelte`**: Address "Expected }" error (around Lines 88-89 in the template for `SavedTranscriptView`). This is critical as it's causing a cascading "no default export" error.
    *   **`ui/src/App.svelte`**: The "Module ... MainPanel.svelte has no default export" error (Line 13) should resolve once `MainPanel.svelte` is fixed.
2.  **Verify `SavedTranscriptView` Integration:**
    *   Manually run the application (`npm run tauri dev` in the `ui` directory or root if configured) to confirm that selecting a saved transcript correctly displays its title, date, and content in the `MainPanel` via `SavedTranscriptView`.
    *   Test the delete functionality from `SavedTranscriptView`.
3.  **Address Test Failures Systematically (After Linter Errors are Fixed):**
    *   Run `npm test` (in the `ui` directory).
    *   **`App.test.ts` failures:**
        *   `renders default panel (Transcription) on initial load`: Investigate why "Your transcription will appear here..." and "Recent Transcriptions" are not found.
        *   `switches panel to Chat...`: Investigate why chat-specific elements are not found.
        *   `loads initial panel from localStorage...`: Investigate why "Saved Transcripts" heading isn't found.
        *   Ensure `waitFor` is used effectively and mock responses for `invoke` cover all necessary commands triggered during these tests, especially list fetching for different panels.
    *   **`TopBar.test.ts` failure:**
        *   `renders without errors`: Fix the "Unable to find an accessible element with the role 'combobox' and name /whisper model/i" error. Ensure select elements have proper ARIA labels if not implicitly provided by a visible `<label>`.
    *   Review tests for `LeftPanel.test.ts` if failures persist after `App.test.ts` fixes.
4.  **Run `npm test`** again after fixes to confirm all tests pass.

**Once Tests are Green:**

5.  **Continue UI Implementation based on original vision and user priorities:**
    *   Implement `VoiceCommandView.svelte` and integrate into `MainPanel.svelte`.
    *   Implement any missing functionality in existing views (e.g., editing saved transcripts, advanced settings tabs).
    *   Flesh out placeholder functionality (e.g., actual transcription, voice command processing).
6.  **Iteratively Develop and Test:**
    *   Run the application frequently to catch runtime errors.
    *   Write new tests for new features and components.

## Open Questions / Future Considerations:

*   Move helper functions (like `formatDate` from `SavedTranscriptView.svelte`) to utility files (e.g., `src/utils.ts`).
*   Consider a more robust state management solution if `App.svelte` becomes too complex (e.g., Svelte stores).
*   Detailed error handling and user feedback for all backend interactions.
*   Full implementation of all backend commands currently stubbed.
