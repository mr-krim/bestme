# BestMe User Guide

Welcome to BestMe - your AI-powered speech transcription assistant!

## Table of Contents
1. [Getting Started](#getting-started)
2. [Basic Usage](#basic-usage)
3. [Voice Commands](#voice-commands)
4. [AI Features](#ai-features)
5. [Settings](#settings)
6. [Keyboard Shortcuts](#keyboard-shortcuts)
7. [Troubleshooting](#troubleshooting)

## Getting Started

### First Launch
When you first launch BestMe, you'll see:
- Main transcription window
- Audio level indicator
- Control buttons (Start/Stop recording)
- Settings button

### Initial Setup
1. **Select Audio Device**: Click Settings → Audio → Input Device
2. **Choose Whisper Model**: Settings → Speech → Model Size
   - `tiny`: Fastest, less accurate (39 MB)
   - `base`: Balanced (74 MB) - Recommended
   - `small`: Better accuracy (244 MB)
   - `medium`: High accuracy (769 MB)
   - `large`: Best accuracy (1.5 GB)
3. **Enable Voice Commands** (optional): Settings → Voice Commands → Enable

## Basic Usage

### Recording Audio
1. **Start Recording**:
   - Click the microphone button
   - Or press `Ctrl+Alt+R` (Windows/Linux) / `Cmd+Alt+R` (macOS)
   - Or say "Computer, start recording" (if voice commands enabled)

2. **Stop Recording**:
   - Click the stop button
   - Or press the same hotkey
   - Or say "Computer, stop recording"

3. **View Transcription**:
   - Text appears in real-time as you speak
   - Confidence indicators show accuracy
   - Timestamps available in detailed view

### Managing Transcripts
- **Clear**: Ctrl+L or "Computer, clear text"
- **Save**: Ctrl+S or File → Save
- **Export**: File → Export As → Choose format (TXT, JSON, SRT)
- **Copy**: Select text and Ctrl+C

## Voice Commands

### Setup
1. Enable in Settings → Voice Commands
2. Choose activation word (default: "Computer")
3. Adjust sensitivity if needed

### Available Commands
- **Recording Control**:
  - "Computer, start recording"
  - "Computer, stop recording"
  - "Computer, pause recording"
  
- **Text Management**:
  - "Computer, clear text"
  - "Computer, save file"
  - "Computer, copy all"
  
- **Navigation**:
  - "Computer, scroll up/down"
  - "Computer, go to top/bottom"
  
- **Editing**:
  - "Computer, undo"
  - "Computer, redo"
  - "Computer, select all"

## AI Features

### Text Enhancement
Select any text and right-click for AI options:

1. **Fix Grammar**: Corrects spelling and grammar mistakes
2. **Improve Style**: Makes text more professional and clear
3. **Summarize**: Creates concise summary
4. **Expand**: Adds more detail and context
5. **Translate**: Translate to 50+ languages

### AI Providers
Configure in Settings → AI:

- **Local Models** (Default):
  - No internet required
  - Privacy-focused
  - Runs on your device
  
- **Cloud Providers**:
  - OpenAI - GPT models
  - Anthropic - Claude models
  - OpenRouter - Multiple providers

### Custom Prompts
Create your own AI commands:
1. Settings → AI → Custom Prompts
2. Click "Add Prompt"
3. Enter name and instruction
4. Use via right-click menu

## Settings

### Audio Settings
- **Input Device**: Select microphone
- **Sample Rate**: 16000 Hz (recommended)
- **Voice Activity Detection**: Auto-detect speech
- **Noise Suppression**: Reduce background noise

### Speech Settings
- **Model Size**: Accuracy vs speed tradeoff
- **Language**: Auto-detect or select specific
- **Temperature**: Creativity in transcription
- **Beam Size**: Search breadth (higher = more accurate)

### AI Settings
- **Provider**: Local or cloud
- **Model**: Select AI model
- **API Keys**: For cloud providers
- **Max Tokens**: Response length limit

### Appearance
- **Theme**: Light/Dark/Auto
- **Font Size**: Adjust text size
- **Window Opacity**: Transparency
- **Always on Top**: Keep window visible

## Keyboard Shortcuts

### Global Hotkeys (work anywhere)
- `Ctrl+Alt+R`: Start/Stop recording
- `Ctrl+Alt+P`: Pause/Resume
- `Ctrl+Alt+M`: Mute/Unmute

### Application Shortcuts
- `Ctrl+S`: Save transcript
- `Ctrl+O`: Open file
- `Ctrl+N`: New transcript
- `Ctrl+L`: Clear text
- `Ctrl+Z`: Undo
- `Ctrl+Y`: Redo
- `Ctrl+A`: Select all
- `Ctrl+F`: Find text
- `F11`: Fullscreen
- `Esc`: Exit fullscreen

## Troubleshooting

### No Audio Input
1. Check microphone permissions
2. Verify device in Settings → Audio
3. Test with system recorder
4. Restart application

### Poor Transcription Quality
1. Use larger Whisper model
2. Reduce background noise
3. Speak clearly and closer to mic
4. Check language settings

### High CPU/Memory Usage
1. Use smaller model
2. Disable GPU if issues
3. Close other applications
4. Check Settings → Performance

### Voice Commands Not Working
1. Verify enabled in settings
2. Check activation word
3. Adjust sensitivity
4. Ensure clear pronunciation

### Application Crashes
1. Check logs in:
   - Windows: `%APPDATA%\bestme\logs`
   - macOS: `~/Library/Logs/bestme`
   - Linux: `~/.config/bestme/logs`
2. Update to latest version
3. Reset settings: Hold Shift while starting

## Tips & Tricks

1. **Batch Processing**: Drag & drop audio files for transcription
2. **Hotword Training**: Record custom activation phrase
3. **Shortcuts**: Create custom keyboard shortcuts
4. **Templates**: Save common formats for reuse
5. **Multi-language**: Switch languages mid-recording
6. **Privacy Mode**: Disable all network features

## Getting Help

- **Documentation**: Check `/docs` folder
- **Community**: Join our Discord/Forum
- **Bug Reports**: GitHub Issues
- **Feature Requests**: GitHub Discussions

---

Thank you for using BestMe! We're constantly improving based on your feedback.