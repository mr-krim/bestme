# BestMe Quick Testing Guide

This guide helps you quickly test all the features we've built so far.

## 1. Starting the Application

```bash
# Run in development mode
cargo tauri dev

# Or with voice commands enabled
./scripts/run_voice.sh

# Or with debug logging
./scripts/run_debug.sh
```

## 2. Core Transcription Testing

### Basic Transcription
1. Click "Start Recording" button
2. Speak clearly: "Hello, this is a test of the BestMe transcription system."
3. Click "Stop Recording"
4. Verify text appears in the transcription area

### Test Different Models
1. Go to Settings → General
2. Try each Whisper model (tiny, base, small, medium, large)
3. Notice speed vs accuracy differences

### Test Languages
1. In Settings, change language from "auto" to specific languages
2. Try speaking in different languages
3. Test "Translate to English" option

### VAD (Voice Activity Detection)
1. Start recording
2. Speak, then pause for 3+ seconds
3. Notice the VAD visualization showing speech/silence
4. Check that processing stops during silence

## 3. Voice Commands Testing

### Basic Commands
While recording, try these voice commands:
- "Delete last word" - removes the last word
- "Delete last sentence" - removes the last sentence
- "Undo" - undoes the last action
- "Redo" - redoes an undone action
- "Capitalize that" - capitalizes the last word
- "New line" - adds a line break
- "Period" / "Comma" / "Question mark" - adds punctuation

### AI Voice Commands
1. Go to Settings → Voice tab
2. Enable "AI Voice Commands"
3. Try natural language commands:
   - "Delete the previous paragraph"
   - "Make this uppercase"
   - "Remove everything after the comma"

## 4. Text Injection Testing

### Basic Injection
1. Open a text editor (Notepad, TextEdit, etc.)
2. Click in the text area
3. Start recording in BestMe
4. Speak some text
5. Stop recording
6. Text should appear in the external editor

### Test Different Applications
Try injecting text into:
- Web browsers (Google Docs, Gmail)
- Code editors (VS Code, Sublime)
- Terminal/Command Prompt
- Office apps (Word, Excel)
- Chat applications

### Injection Settings
1. Go to Settings → General
2. Find Text Injection settings
3. Try different modes:
   - Type mode (character by character)
   - Paste mode (via clipboard)
   - Adjust typing delay

## 5. AI Enhancement Testing

### Grammar Correction
1. Go to Settings → AI tab
2. Enable text enhancement
3. Transcribe with intentional errors: "this are a test of grammer"
4. Check if AI corrects to: "This is a test of grammar"

### Summarization
1. Transcribe a longer paragraph (2-3 minutes of speaking)
2. In the UI, look for AI tools
3. Click "Summarize"
4. Try different summary lengths

### Translation
1. Transcribe text in any language
2. Use the translation feature
3. Select target language
4. Verify translation quality

## 6. Storage and Search

### Save Transcripts
1. After transcribing, the text auto-saves
2. Go to "Saved Transcripts" tab
3. Verify your transcripts are listed

### Search
1. In Saved Transcripts, use the search box
2. Search for specific words from your transcripts
3. Verify search results highlight matches

### Export
1. Select a transcript
2. Try export options:
   - Text (.txt)
   - Markdown (.md)
   - JSON (.json)
   - CSV (.csv)

## 7. GPU Acceleration Testing

### Check GPU Status
1. Go to Settings → GPU tab
2. See if GPU is detected
3. Note which backend (CUDA, Metal, ROCm, Vulkan)

### Performance Comparison
1. Disable GPU (if available)
2. Transcribe 1 minute of audio, note the time
3. Enable GPU
4. Transcribe same audio, compare times
5. GPU should be 2-10x faster

### Run Benchmark
```bash
cargo run --bin benchmark --release
```

## 8. Custom Vocabulary

1. Go to "Vocabulary" tab
2. Add technical terms or names:
   - Term: "BestMe"
   - Boost: 3.0
   - Category: Product Names
3. Test transcription with these terms
4. They should be recognized more accurately

## 9. Performance Monitoring

### Check Resource Usage
While transcribing:
1. Open Task Manager (Windows) or Activity Monitor (Mac)
2. Check BestMe's CPU usage
3. Check memory usage
4. GPU usage (if GPU enabled)

### Expected Performance
- CPU: 10-30% during transcription
- Memory: 200-500MB
- GPU: 20-80% (if enabled)
- Latency: <500ms for transcription

## 10. Common Issues to Check

### Audio Issues
- If no audio, check microphone permissions
- Try different audio devices in Settings
- Check audio level meter is responding

### Transcription Issues
- If no text appears, check model is downloaded
- If quality is poor, try larger model
- If slow, check GPU is enabled

### AI Features
- If AI features fail, check Settings → AI
- Ensure API keys are configured (if using cloud)
- Try local models first

## Quick Feature Checklist

- [ ] Basic transcription works
- [ ] Multiple Whisper models work
- [ ] Language detection works
- [ ] Voice commands recognized
- [ ] Text injection works
- [ ] AI grammar correction works
- [ ] Summarization works
- [ ] Translation works
- [ ] Transcripts save automatically
- [ ] Search finds results
- [ ] Export creates files
- [ ] GPU acceleration works (if available)
- [ ] Custom vocabulary improves accuracy
- [ ] VAD reduces unnecessary processing

## Reporting Issues

If you find issues:
1. Note the exact steps to reproduce
2. Check the logs (run with `./scripts/run_debug.sh`)
3. Include your system info:
   - OS version
   - GPU model (if applicable)
   - Whisper model used
   - Error messages

## Performance Benchmarks

Expected performance on modern hardware:
- **Transcription**: Real-time or faster
- **Text injection**: <50ms latency
- **AI enhancement**: <500ms for grammar
- **Voice commands**: <200ms recognition
- **Search**: <100ms for 1000 transcripts