# BestMe Configuration

This directory contains configuration files for the BestMe application.

## Configuration Files

- `config.json`: Main configuration file for the application.

## Configuration Loading Order

The application searches for configuration files in the following order:

1. `<app_directory>/config/config.json`: Application-specific configuration
2. `~/.config/bestme/config.json`: User-specific configuration
3. Default configuration built into the application

If a configuration file is found, it's loaded and used. If not, the application falls back to the next location in the search order. 

## Configuration Structure

The configuration file contains settings for:

- General application settings
- Audio device settings
- Speech recognition settings
- Voice command settings

Example structure:

```json
{
  "version": "0.1.0",
  "general": {
    "theme": "system",
    "auto_start": false,
    "minimize_to_tray": true
  },
  "audio": {
    "input_device": null,
    "input_volume": 1.0,
    "speech": {
      "model_size": "Small",
      "model_path": null,
      "language": "auto",
      "auto_punctuate": true,
      "translate_to_english": false,
      "context_formatting": true,
      "segment_duration": 3.0,
      "save_transcription": true,
      "output_format": "txt",
      "buffer_size": 3.0
    },
    "voice_commands": {
      "enabled": true,
      "command_prefix": "hey computer",
      "require_prefix": true,
      "sensitivity": 0.7,
      "custom_commands": [
        ["Open Settings", {"Custom": "OpenSettings"}],
        ["Toggle Recording", {"Custom": "ToggleRecording"}]
      ]
    }
  }
}
```

For more detailed information about configuration options, see `docs/debugging.md`. 
