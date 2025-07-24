# BestMe: Your Local AI-Powered Personal Assistant

## Vision
BestMe is an intelligent personal assistant that seamlessly integrates voice transcription with your digital workflow. More than just speech-to-text, BestMe understands context, executes commands, and enhances productivity by bridging the gap between your voice and your applications. Built with privacy-first principles, it operates locally by default while offering optional cloud AI enhancements for advanced features. BestMe transforms how you interact with your computer, making voice a first-class input method across all your applications.

## Core Principles
1. **Privacy First**: All core functionality runs locally, cloud features are optional and transparent
2. **Context Aware**: Understands what you're working on and adapts behavior accordingly
3. **Seamless Integration**: Injects text directly into any application, not just text fields
4. **Intelligent Assistant**: Goes beyond transcription to understand intent and execute actions
5. **Speed & Accuracy**: Real-time processing with minimal latency and high accuracy
6. **Beautiful UX**: Clean, intuitive interface that stays out of your way
7. **Cross-Platform**: Consistent experience across Windows, macOS, and Linux
8. **Extensible**: Plugin architecture for custom workflows and integrations

## Key Features

### Core Transcription
- **Real-time Speech-to-Text**: Industry-leading accuracy with Whisper AI
- **Multi-language Support**: 100+ languages with automatic detection
- **Smart Punctuation**: Context-aware punctuation and formatting
- **Voice Commands**: Natural language commands for editing and control

### Intelligent Integration
- **Direct Text Injection**: Type into any application, not just text fields
- **Application Context**: Adapts behavior based on active application
- **Smart Clipboard**: Enhanced clipboard with transcription history
- **Global Hotkeys**: System-wide shortcuts for instant access

### Personal Assistant Features
- **Conversation Memory**: Remembers context across sessions
- **Transcript Storage**: Local SQLite database with full-text search
- **Smart Retrieval**: "What did I say about X yesterday?"
- **Action Execution**: "Send this to John" triggers email with context

### AI Enhancement (Local + Optional Cloud)
- **Local AI**: Grammar correction, formatting, basic commands
- **Cloud AI**: Advanced summarization, style transformation, semantic understanding
- **Hybrid Mode**: Use cloud only when needed, with full transparency
- **Custom Models**: Support for OpenRouter/Requesty for model selection

### Privacy & Security
- **Local First**: All core features work offline
- **Encrypted Storage**: Sensitive data encrypted at rest
- **Audit Trail**: Track all AI interactions and data movement
- **Data Controls**: Fine-grained control over what gets processed where

## What Makes BestMe Different

### vs. Built-in Dictation Tools
- **Application Integration**: Works with ANY application, not just text fields
- **Persistent Memory**: Remembers your conversations and context
- **Intelligent Commands**: Natural language understanding, not just keywords
- **Offline AI**: Local processing for privacy and speed

### vs. Cloud Transcription Services
- **Privacy First**: Your voice never leaves your device unless you explicitly allow it
- **No Subscription Required**: Core features work forever without fees
- **Instant Response**: No network latency for basic operations
- **Customizable**: Add your own commands and workflows

### vs. Traditional Assistants (Siri, Alexa, Cortana)
- **Desktop Focused**: Designed for productivity, not general queries
- **Deep Integration**: Direct text injection into applications
- **Context Aware**: Understands your current task and application
- **Extensible**: Plugin system for custom integrations

## Use Cases

### For Professionals
- **Lawyers**: Dictate briefs with legal formatting and terminology
- **Doctors**: Medical transcription with HIPAA-compliant local storage
- **Writers**: Natural dictation with smart editing and formatting
- **Developers**: Code dictation with syntax awareness

### For Everyone
- **Email Composition**: "Reply to John's email about the meeting"
- **Note Taking**: "Save this idea to my project notes"
- **Accessibility**: Hands-free computer control for any application
- **Language Learning**: Practice pronunciation with instant feedback

# Product Requirements Document (PRD)
## BestMe – Cross-Platform Real-Time Transcription & Meeting Intelligence Tool

**Version:** 1.1  
**Date:** 2025-03-26

---

## 1. Overview

This document defines the requirements for a cross-platform real-time transcription tool that replicates and extends SuperWhisper's capabilities. Developed in Rust, the application will deliver low‑latency, highly accurate speech-to-text transcription along with advanced meeting intelligence features. Key functionalities include audio capture, AI-powered transcription, multi-language support, speaker diarization, meeting summarization, and seamless text integration—all while operating offline by default to protect user privacy. Optional cloud sync and collaboration features are available for team environments.

---

## 2. Functional Requirements

### 2.1. Voice Input & Audio Capture
- **Audio Capture:**  
  - Capture audio from a selected microphone using platform-independent audio APIs (e.g., cpal).
  - Support multiple input devices with configurable options.
- **Device Management:**  
  - Allow users to choose and switch between input devices.
  - Persist device settings across sessions.
- **Quality & Latency:**  
  - Support high sampling rates and minimal latency for real-time capture.

### 2.2. Real‑Time Transcription Engine
- **Transcription Engine:**  
  - Integrate an AI‑powered transcription engine (e.g., a Rust port of Whisper or similar) to convert speech to text in real time.
  - Allow switching between model sizes (base, standard, pro) to balance accuracy and performance.
- **Performance:**  
  - Achieve near‑instantaneous transcription with minimal delay.

### 2.3. Multi‑Language Support & Translation
- **Language Support:**  
  - Support transcription in over 100 languages.
  - Provide an option to translate non‑English input into English.
- **Language Selection:**  
  - Enable manual selection or auto-detection of input language.

### 2.4. Speaker Diarization & Meeting Segmentation
- **Speaker Identification:**  
  - Distinguish and label different speakers during a recording.
- **Meeting Segmentation:**  
  - Automatically segment recordings by speaker or topic to facilitate review and analysis.

### 2.5. Meeting Summaries & Action Items
- **Automatic Summarization:**  
  - Generate concise meeting summaries capturing key points.
- **Action Item Extraction:**  
  - Detect and list actionable items from the conversation.
- **Highlighting:**  
  - Allow users to mark and review important sections of the transcription.

### 2.6. Search, Annotation, and Collaboration
- **Search Capabilities:**  
  - Provide advanced search functionality within past transcriptions.
- **Annotation Tools:**  
  - Enable users to tag and annotate sections for quick reference.
- **Collaboration:**  
  - Optional cloud sync to store and share transcriptions securely among team members.
  - Provide collaboration features such as real-time editing and comments (opt-in with strict security controls).

### 2.7. User Interface & Interaction
- **UI Design:**  
  - Provide a lightweight, platform-native UI that follows design guidelines for each platform: 
    - Windows 11 style for Windows
    - macOS design guidelines for Apple devices
    - GTK or similar for Linux
  - Support both dark and light themes across all platforms.
- **Hotkeys & Shortcuts:**  
  - Implement global hotkeys/keyboard shortcuts for starting, pausing, and stopping dictation.
  - Allow customization of hotkey settings tailored to each platform's conventions.

### 2.8. Text Integration & Output
- **Seamless Integration:**  
  - Automatically insert transcribed text into the currently focused text field.
  - Alternatively, copy text to the clipboard.
- **Export Options:**  
  - Enable saving or exporting transcriptions in various formats (plain text, markdown, rich text).
  - Support periodic automatic saving or on-demand export.

### 2.9. Real-Time Analytics & Insights
- **Sentiment & Keyword Analysis:**  
  - Provide real-time sentiment analysis and keyword extraction.
- **Visual Feedback:**  
  - Display analytics dashboards or insights to help users quickly understand the conversation dynamics.

### 2.10. Error Handling & Feedback
- **Error Detection:**  
  - Detect issues such as microphone errors, transcription engine failures, or connectivity problems.
- **User Notifications:**  
  - Notify users via platform-native notifications and in-app alerts.
  - Provide clear troubleshooting steps and detailed logs for diagnostics.

---

## 3. Non‑Functional Requirements

### 3.1. Performance & Responsiveness
- **Low‑Latency Transcription:**  
  - Deliver real-time transcription speeds (targeting up to 3× faster than manual typing).
  - Ensure near-instantaneous updates with minimal delay.
- **Resource Optimization:**  
  - Optimize CPU and memory usage for smooth operation across platforms.

### 3.2. Accuracy & Robustness
- **High Accuracy:**  
  - Use state‑of‑the‑art AI models to maintain high transcription accuracy, even in noisy environments.
- **Robustness:**  
  - Implement error recovery mechanisms to avoid crashes or data loss.

### 3.3. Security & Privacy
- **Offline Operation:**  
  - Default operation is entirely offline to ensure sensitive voice data remains on the user's device.
- **Data Protection:**  
  - Securely store any saved transcriptions and user settings.
- **Optional Cloud Sync:**  
  - When enabled, ensure cloud storage is encrypted and meets strict data protection standards.

### 3.4. Usability & Accessibility
- **Intuitive UI:**  
  - Design a streamlined, easy-to-navigate interface that minimizes configuration effort.
- **Accessibility:**  
  - Include features such as voice navigation, high‑contrast mode, and adherence to accessibility guidelines for each platform.

### 3.5. Maintainability & Extensibility
- **Modular Codebase:**  
  - Develop the application in Rust with clear interfaces, modular components, and comprehensive testing.
  - Implement a clean abstraction layer between platform-agnostic core and platform-specific code.
- **Future Enhancements:**  
  - Design the architecture to easily integrate additional features (e.g., plugin support for third-party integrations).

### 3.6. Portability & Compatibility
- **Cross-Platform Compatibility:**  
  - Ensure full compatibility with Windows 11, macOS, and Linux systems.
  - Implement platform-specific integrations where necessary (audio capture, clipboard integration, global hotkeys).
- **Scalability:**  
  - Plan for future scalability as platforms and AI models evolve.

---

## 4. Platform-Specific Considerations

### 4.1. Cross-Platform UI Strategy with Tauri
- **Unified Web-Based UI:**
  - Leverage Tauri's webview-based architecture for consistent UI rendering
  - Use React/Vue/Svelte (to be decided) for frontend components
  - Apply platform-specific styling through CSS variables and adaptive themes
  - Use Tauri's plugin system for accessing platform-specific features
- **Responsive Design:**
  - Implement a responsive design that adapts to each platform's display conventions
  - Use CSS media queries and feature detection for platform-aware styling

### 4.2. Platform-Specific Integrations via Tauri Plugins
- **Audio Framework:**  
  - Use cpal in Rust backend with Tauri plugin bridge to frontend
  - Create custom Tauri plugins for any platform-specific audio features
- **System Integration:**  
  - Utilize Tauri's plugin API for clipboard operations, notifications, and hotkeys
  - Implement custom Rust-based plugins for platform-specific functionality
- **Installation:**
  - Leverage Tauri's bundler for platform-specific installers

### 4.3. Windows Implementation
- **Audio Framework:**  
  - Utilize Windows audio frameworks (like WASAPI) for high‑quality, low‑latency audio capture.
- **System Integration:**  
  - Seamlessly integrate with Windows clipboard, notifications, and system hotkey management.
- **UI Framework:**
  - Leverage Windows UI libraries for native look and feel.
- **Installation:**
  - Support Microsoft Store distribution and standalone installer.

### 4.4. macOS Implementation
- **Audio Framework:**
  - Use Core Audio for native macOS audio capture.
- **System Integration:**
  - Integrate with macOS clipboard, Notification Center, and keyboard shortcuts.
- **UI Framework:**
  - Follow macOS Human Interface Guidelines for a native experience.
- **Installation:**
  - Support App Store distribution and standalone installer.
  - Ensure proper code signing and notarization.

### 4.5. Linux Implementation
- **Audio Framework:**
  - Support PulseAudio and ALSA for audio capture.
- **System Integration:**
  - Integrate with desktop environment services where available.
- **UI Framework:**
  - Implement a UI compatible with popular desktop environments (GNOME, KDE).
- **Installation:**
  - Provide distribution packages (.deb, .rpm) and Flatpak/Snap options.

### 4.6. Cross-Platform Abstractions
- **Audio Processing:**  
  - Leverage cross-platform crates like cpal for audio operations with platform-specific optimizations.
- **User Interface:**  
  - Consider using libraries like iced, egui, or tauri to build a responsive cross-platform UI.
- **Asynchronous Processing:**  
  - Utilize async programming (using tokio) to manage real‑time processing efficiently.
- **Performance Patterns:**  
  - Apply performance‑oriented design patterns to maintain low latency under heavy usage.

---

## 5. Implementation Considerations & Open Issues

- **Tauri Configuration:**
  - Determine optimal Tauri configuration for performance and bundle size
  - Select appropriate web framework (React, Vue, Svelte) for frontend components
- **Plugin Development:**
  - Identify required custom Tauri plugins for BestMe functionality
  - Design plugin architecture for audio processing and system integration
- **Model Switching:**  
  - Define how dynamic switching between different model sizes will be managed during runtime
- **Device Management:**  
  - Implement device selection using cpal with Tauri plugin bridge
- **Error Logging:**  
  - Create unified logging system that spans Rust backend and frontend
- **Customization Options:**  
  - Design configuration system utilizing Tauri's storage capabilities
- **Integration Depth:**  
  - Evaluate which system integrations to implement via Tauri's API vs custom plugins
- **Mobile Strategy:**
  - Monitor Tauri Mobile development for future expansion to iOS and Android
  - Design architecture with mobile compatibility in mind 
- **Development Environment:**
  - Configure Docker environment optimized for Tauri development
  - Establish tooling for efficient Rust-to-Web development workflow

---

## 6. Future Enhancements

- **Advanced AI Features:**  
  - Explore AI‑powered text formatting, summarization, or context-aware transcription enhancements.
- **Plugin Support:**  
  - Design a plugin system to allow third‑party extensions and integrations.
- **Web Application:**
  - Consider a complementary web version for browser-based access.
- **Mobile Clients:**
  - Evaluate the feasibility of mobile applications for iOS and Android.

---

## 7. Personal Assistant Vision

### The Ultimate Goal
Transform BestMe from a transcription tool into an intelligent personal assistant that understands context, executes actions, and learns from your patterns. Think of it as having a highly capable assistant who:
- Listens to everything you say (with your permission)
- Understands what you're working on
- Can execute commands in any application
- Remembers past conversations and context
- Helps you be more productive without getting in the way

### Key Capabilities

#### 1. Universal Text Injection
- Works with ANY application - terminals, browsers, IDEs, office apps
- Smart injection based on context (paste for large text, type for small)
- Maintains formatting and special characters
- Handles code syntax, markdown, and rich text

#### 2. Intelligent Context Understanding
- Knows which application is active
- Understands the type of content (code, email, document)
- Adapts commands based on context
- Learns your preferences over time

#### 3. Conversation Memory & Retrieval
- "What did I say about the API redesign last Tuesday?"
- "Show me all my thoughts on the marketing campaign"
- "Find when I mentioned the budget numbers"
- Semantic search across all transcripts

#### 4. Smart Command Execution
- "Send this paragraph to Sarah" - finds Sarah's email, composes message
- "Add this to my todo list" - integrates with your task manager
- "Schedule a meeting about this" - creates calendar event with context
- "Save this code snippet" - stores with proper syntax highlighting

#### 5. AI-Powered Enhancement
- Local AI for instant corrections and formatting
- Optional cloud AI for advanced features:
  - "Make this more professional"
  - "Summarize the last hour"
  - "Extract action items from this meeting"
  - "Translate this to Spanish"

### Privacy & Trust
- All transcription happens locally - your voice never leaves your device
- Cloud features are opt-in with clear indicators
- Full audit trail of what data goes where
- Encrypted storage for sensitive information
- Easy data export and deletion

### The Experience
Imagine starting your day:
1. "BestMe, start listening" - begins ambient transcription
2. You dictate emails, and they appear perfectly formatted in Gmail
3. During a meeting, it captures everything and identifies speakers
4. "What were the action items?" - instantly lists them
5. "Draft a follow-up email" - creates it with meeting context
6. All searchable, all private, all under your control

## 8. Conclusion

BestMe represents the future of human-computer interaction - where voice becomes as powerful as keyboard and mouse. By combining state-of-the-art transcription, intelligent context understanding, and seamless application integration, BestMe transforms how you work. It's not just about converting speech to text; it's about having an intelligent assistant that amplifies your productivity while respecting your privacy. The journey from transcription tool to personal assistant starts with the solid foundation we've built and extends into a future where your computer truly understands and assists you.

