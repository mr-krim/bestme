# CLAUDE.md - BestMe Project Context

This file provides essential context for Claude to understand and work effectively on the BestMe project.

## Project Overview

BestMe is a modern, cross-platform speech-to-text application built with:
- **Backend**: Rust with Tauri 2.0 framework
- **Frontend**: Svelte with TypeScript
- **Audio**: Whisper AI for transcription
- **AI Enhancement**: ONNX Runtime with local/cloud models
- **Platforms**: Windows, macOS, and Linux

## Key Architecture Decisions

1. **Tauri 2.0**: Using the latest Tauri version for better performance and security
2. **Plugin Architecture**: Custom Tauri plugins for audio, transcription, voice commands, and AI
3. **Async Rust**: Heavy use of Tokio for concurrent operations
4. **Modular Design**: Separation between core library (`src/`) and Tauri app (`src-tauri/`)
5. **AI Integration**: ONNX Runtime for cross-platform inference with GPU acceleration

## Current Development State (January 2025)

### Completed ✅
- Core application framework
- Audio capture and transcription
- Voice commands with text injection
- GPU acceleration for Whisper
- Complete AI model integration (15/20 tasks done)
- Telemetry and monitoring
- Conversation context management
- Comprehensive testing infrastructure

### In Progress 🚧
- Error recovery and resilience (High Priority)
- Model-specific UI components
- Automatic model selection
- Model update checking
- Custom user model support

### AI Implementation Status
- **Phase 2**: ✅ Local/Cloud AI Integration Complete
- **Phase 2.5**: ✅ Real Model Support Complete
- **Phase 3**: ✅ Performance Optimization Complete
- **Phase 4**: ✅ Infrastructure (Telemetry, Context, Testing) Complete
- **Remaining**: 5 tasks focusing on resilience and UI

## Important Commands

### Development
```bash
# Run in development mode
cargo tauri dev

# Run with voice commands enabled
./scripts/run_voice.sh

# Run with debug logging
./scripts/run_debug.sh

# Build for production
cargo tauri build
```

### Testing (Always Run Before Commits!)
```bash
# Run all Rust tests
cargo test

# Run UI tests
cd ui && npm test

# Run integration tests
cargo test --test '*' --features integration-tests

# Check test coverage
cargo tarpaulin --out Html

# Run specific test suite
cargo test audio:: # Test audio module
cargo test ai:: # Test AI module
```

### Code Quality
```bash
# Format Rust code
cargo fmt

# Check for common issues
cargo clippy -- -D warnings

# Fix common issues
cargo fix --lib -p bestme

# Check for security issues
cargo audit

# Update dependencies to latest versions
cargo update
cd ui && npm update
```

### Dependency Management
```bash
# Check for outdated dependencies
cargo outdated
cd ui && npm outdated

# Update all dependencies to latest
cargo update
cd ui && npm update

# Verify dependency tree
cargo tree
cd ui && npm ls
```

## Project Structure

```
bestme/
├── src/                    # Core Rust library
│   ├── audio/             # Audio processing modules
│   ├── ai/                # AI integration modules
│   │   ├── local/         # Local model inference
│   │   ├── cloud/         # Cloud API clients
│   │   ├── context/       # Conversation management
│   │   ├── telemetry/     # Metrics and tracing
│   │   └── benchmarks/    # Performance testing
│   ├── config.rs          # Configuration management
│   └── gui/               # GUI components
├── src-tauri/             # Tauri application
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   └── plugin/        # Custom plugins
│   └── tauri.conf.json    # Tauri config
├── ui/                    # Svelte frontend
│   ├── src/
│   │   ├── App.svelte     # Main component
│   │   ├── components/    # UI components
│   │   └── views/         # View components
│   └── package.json       # Frontend deps
└── scripts/               # Development scripts
```

## Key Files to Know

### Core AI System
1. **`src/ai/services/model_service.rs`** - Model management and loading
2. **`src/ai/local/onnx_runtime.rs`** - ONNX inference implementation
3. **`src/ai/context/conversation.rs`** - Multi-turn conversation support
4. **`src/ai/telemetry/metrics.rs`** - Performance monitoring

### Audio & Transcription
1. **`src/audio/transcribe.rs`** - Whisper integration and transcription logic
2. **`src/audio/voice_commands.rs`** - Voice command detection and processing
3. **`src/audio/streaming_transcribe.rs`** - Real-time streaming pipeline

### Frontend
1. **`ui/src/App.svelte`** - Main frontend application logic
2. **`ui/src/views/TranscriptionView.svelte`** - Transcription interface
3. **`ui/src/components/VADVisualization.svelte`** - Voice activity display

## Common Issues and Solutions

### Build Errors
1. **Missing dependencies**: Run `cd ui && npm install`
2. **Rust errors**: Update with `rustup update`
3. **Tauri errors**: Ensure WebView2 (Windows) or webkit2gtk (Linux) is installed

### AI Model Issues
1. **Model not found**: Models download automatically on first use
2. **GPU not detected**: Check CUDA/Metal drivers are installed
3. **High memory usage**: Use quantized models or reduce batch size

### Audio Issues
1. **No audio devices**: Check system permissions
2. **High latency**: Try smaller Whisper models
3. **No transcription**: Verify model is downloaded

### Development Tips
1. Always run `npm install` in the `ui/` directory after pulling changes
2. Use `RUST_LOG=debug` for detailed logging
3. The Tauri dev tools can be opened with Ctrl+Shift+I

## Development Philosophy

### No Shortcuts - Proper Implementation
- **Always implement features properly** - no temporary hacks or workarounds
- **Write production-ready code** from the start
- **Consider edge cases** and error handling in initial implementation
- **Refactor when needed** rather than patching issues

### Dependency Management
- **Always use latest stable versions** of all packages and libraries
- **Update dependencies regularly** to get security fixes and improvements
- **Only downgrade if critical bugs** are discovered that block functionality
- **Document any version pins** with clear reasons in comments

### Testing is Mandatory
- **Write tests for ALL new features** - no exceptions
- **Run existing test suite** before making any changes
- **Add integration tests** for complex workflows
- **Test on all platforms** before considering a feature complete
- **Maintain test coverage** above 80% for critical paths

## Code Style Guidelines

### Rust
- Use `cargo fmt` before committing
- Prefer `Result<T>` over panicking
- Use `tokio` for async operations
- Document public APIs with `///` comments
- Write unit tests in the same file using `#[cfg(test)]`
- Use `cargo test` before every commit

### TypeScript/Svelte
- Use TypeScript for all new code
- Define types in `types.ts`
- Keep components focused and small
- Use stores for shared state
- Write component tests using Vitest
- Test user interactions, not implementation details

### Git Workflow
- Create feature branches from `main`
- Use descriptive commit messages
- Reference issues in commits
- **Run full test suite** before pushing
- **Update tests** when changing functionality

## AI-Specific Guidelines

### Model Management
- Models are stored in platform-specific app data directories
- Automatic optimization for models with >100ms latency
- GPU acceleration enabled by default when available
- Caching enabled for common phrases

### Performance Targets
- **Inference latency**: <50ms for text enhancement
- **Model loading**: <5 seconds for large models
- **Memory usage**: <4GB for typical usage
- **Cache hit rate**: >60% for common text

### Error Handling
- Always provide fallback options (GPU → CPU → Cache)
- Never expose raw errors to users
- Log detailed errors for debugging
- Implement retry logic for transient failures

## Current Priorities (January 2025)

### Immediate Tasks
1. **Error Recovery Implementation**
   - Create resilience module
   - Implement fallback chains
   - Add circuit breakers
   - Test failure scenarios

2. **UI Components for AI**
   - Model selection interface
   - Performance visualization
   - Download progress indicators
   - Configuration panels

### Near-term Goals
1. Complete remaining 5 AI tasks
2. Create comprehensive documentation
3. Performance optimization
4. Production deployment preparation

## Useful Context

### AI System Status
- 15/20 AI tasks completed (75%)
- ONNX Runtime fully integrated
- GPU acceleration working (CUDA, Metal, DirectML)
- Telemetry and monitoring active
- Conversation context management ready

### Recent Additions
- OpenTelemetry for metrics and tracing
- Memory-augmented conversation context
- 5 pre-built conversation templates
- 50+ unit tests, 10+ integration tests
- Comprehensive benchmarking suite

### Platform-Specific Notes
- The app uses Whisper models stored in the user's app data directory
- Configuration is stored in `config.json` in platform-specific locations
- The UI has light/dark theme support (basic implementation)
- System tray integration is functional on all platforms
- Audio visualization uses WebAudio API with data from Rust

## Mandatory Testing Protocol

### Before ANY Code Changes
1. **Run existing test suite**: `cargo test && cd ui && npm test`
2. **Ensure all tests pass** - never proceed with failing tests
3. **Check code coverage** - maintain 80%+ coverage on critical paths

### For New Features - ALL Required
- [ ] **Unit tests** for core logic
- [ ] **Integration tests** for component interaction  
- [ ] **End-to-end tests** for user workflows
- [ ] **Error handling tests** for edge cases
- [ ] **Performance tests** for real-time features

### Feature Testing Checklist
When making changes, test:
- [ ] Audio device switching (automated + manual)
- [ ] Start/stop transcription (with tests)
- [ ] Model downloading (mock and integration tests)
- [ ] Settings persistence (unit tests)
- [ ] Theme switching (component tests)
- [ ] Window resize behavior (UI tests)
- [ ] System tray functionality (integration tests)
- [ ] Voice command detection (extensive unit tests)
- [ ] Multi-platform behavior (CI/CD tests)

### Test Categories Required
1. **Unit Tests**: Individual functions and methods
2. **Integration Tests**: Component interactions
3. **UI Tests**: User interface components
4. **System Tests**: Full application workflows
5. **Performance Tests**: Real-time audio processing
6. **Platform Tests**: OS-specific functionality

## Resources

- [Project Docs](./docs/) - Comprehensive documentation
- [AI Status](./docs/ai-implementation-status.md) - Detailed AI progress
- [Development Status](./docs/DEVELOPMENT.md) - Overall progress
- [Tauri 2.0 Docs](https://tauri.app/) - Framework reference
- [ONNX Runtime](https://onnxruntime.ai/) - AI inference engine
- [Whisper](https://github.com/openai/whisper) - Speech recognition model

## Notes for Claude

### When Resuming Development
1. **Check git status** to see current branch and changes
2. **Read** `/docs/ai-implementation-status.md` for detailed checkpoint
3. **Review** recent changes in key files listed above
4. **Run tests** to ensure everything still works
5. **Continue** with the next high-priority task

### Core Principles - NEVER Compromise On These
1. **No Shortcuts**: Always implement proper, production-ready solutions
2. **Latest Dependencies**: Use newest stable versions unless critical bugs exist
3. **Testing is Mandatory**: Write tests for every feature, no exceptions
4. **Cross-Platform**: Code must work on Windows, macOS, and Linux
5. **Performance First**: This is real-time audio - performance matters
6. **Privacy by Design**: All transcription happens locally

### Development Requirements - Always Follow
1. **Check git status** for current branch before starting
2. **Run test suite** before making any changes
3. **Write tests first** for new features (TDD approach)
4. **Update dependencies** regularly using latest stable versions
5. **Document version pins** if forced to downgrade due to bugs
6. **Test on all platforms** when possible (use CI/CD if local testing limited)

### Implementation Standards
- **Error Handling**: Proper `Result<T>` patterns, no panics in production
- **Async Patterns**: Use Tokio properly with cleanup
- **Memory Management**: No leaks, proper resource disposal
- **Code Coverage**: Maintain 80%+ coverage on critical functionality
- **Integration Testing**: Test component interactions, not just units

### What to Always Do
- Check existing patterns before implementing new features
- Update documentation when adding significant functionality
- Consider error handling and edge cases from the start
- Write comprehensive tests covering happy path and edge cases
- Update dependencies to latest versions unless bugs prevent it
- Refactor code rather than adding patches or workarounds
- Add telemetry for new features

### What to Never Do
- Skip writing tests for new functionality
- Use outdated dependencies without documented reason
- Implement temporary hacks or shortcuts
- Proceed with failing tests
- Ignore cross-platform compatibility
- Add features without proper error handling
- Forget to update relevant documentation