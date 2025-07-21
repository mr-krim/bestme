## Next Steps (Post-Settings UI Enhancement)

1.  **Refine Backend Settings Structure (Continuing Step 1 & 3):**
    *   Review and potentially refine the `Config`, `GeneralSettings`, `AudioSettings`, and `SpeechSettings` structs in Rust (`src/config.rs`, `src/settings.rs`, `src/audio.rs`) to ensure they accurately represent all the options shown in the UI and are easy to work with.
    *   Double-check the implementation of `get_settings` and `save_all_settings` in `src/commands.rs` to ensure they correctly map between the Rust structs and the data format expected/sent by the Svelte frontend.

2.  **Implement Core Logic Connections (Step 4 & 5):** 
    *   **Audio Device Selection:** Modify the audio capture logic to use the `input_device` from the configuration.
    *   **Transcription Settings Application:** Update the transcription process to use `model_size`, `language`, `auto_punctuate`, and `translate_to_english` from the configuration.
    *   **General Settings Application:** Ensure `auto_transcribe` and `offline_mode` flags control application behavior.

3.  **Testing:** Add backend tests to verify settings load/save correctly and influence application behavior. 

## Progress

1.  [x] **Foundation & Settings UI:**
    *   [x] Create basic Tauri app structure.
    *   [x] Design and implement Svelte components for settings UI (`GeneralSettings`, `AudioSettings`, `TranscriptionSettings`, `AiSettings`).
    *   [x] Implement state management in Svelte (`settingsStore.ts`).
    *   [x] Create `ConfigManager` in Rust to load/save settings.
    *   [x] Add Tauri commands (`get_settings`, `save_all_settings`) to bridge frontend and backend.
    *   [x] Ensure settings persist across restarts.
2.  [x] **Implement Core Logic Connections:**
    *   [x] **Audio Input:** Modify `AudioState` to read the `input_device` setting from `ConfigManager` and use it when initializing the audio stream.
    *   [x] **Transcription Parameters:** Modify `TranscribeState` (`process_audio_buffer`) to read `language` and `translate_to_english` settings from `ConfigManager` and apply them to `FullParams` for Whisper.
    *   [x] **General Settings Application:** Ensure `auto_transcribe` (in `main.rs` setup) and `offline_mode` (in `TranscribeState::download_model`) flags control application behavior.
3.  [ ] **Testing:**
    *   [ ] Add backend unit/integration tests (e.g., for `ConfigManager`, settings application in `AudioState`/`TranscribeState`).
        *   [x] Test `TranscribeState::download_model` offline mode.
        *   [ ] Test `TranscribeState::create_whisper_params` logic.
        *   [ ] Test `AudioState` device selection.
        *   [ ] Test `main.rs` auto-transcribe logic.
    *   [ ] Add frontend tests if necessary (e.g., settings save/load interaction).
4.  [ ] **Refinement & UI Polish:**
    *   [ ] Improve UI layout and styling.
    *   [ ] Add status indicators (e.g., recording status, model download progress).
    *   [ ] Implement error handling and display messages in the UI.
    *   [ ] Consider adding model download management/feedback in the UI.
5.  [ ] **Advanced Features (Optional):**
    *   [ ] Implement `auto_punctuate` (might require post-processing).
    *   [ ] Implement `context_formatting` (if feasible).
    *   [ ] Voice command integration improvements based on config.
