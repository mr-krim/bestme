# Tauri 1.x to 2.0 Migration Guide

This document outlines the process we followed to migrate our application from Tauri 1.x to Tauri 2.0, including key changes, challenges, and solutions.

## Overview

Tauri 2.0 introduces several breaking changes and improvements over Tauri 1.x. Our migration process focused on:

1. Updating dependencies and feature flags
2. Migrating the core application structure
3. Updating the plugin system
4. Modifying the frontend JavaScript API
5. Testing and validation

## 1. Dependencies Update

### Cargo.toml Changes

```rust
[dependencies]
# Tauri 1.x
# tauri = { version = "1.4", features = ["api-all", "system-tray"] }

# Tauri 2.0
tauri = { version = "2.0.0-beta.12", features = ["api-all", "tray-icon"] }
```

### Feature Flag Changes

Several feature flags were renamed in Tauri 2.0:

| Tauri 1.x | Tauri 2.0 |
|-----------|-----------|
| system-tray | tray-icon |
| window-data-url | webview-data-url |
| icon-ico / icon-png | image-ico / image-png |

## 2. Core Application Migration

### 2.1 Initialization Pattern

**Tauri 1.x:**
```rust
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Setup code
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running application");
}
```

**Tauri 2.0:**
```rust
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Setup code
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running application");
}
```

The initialization pattern remains similar, but internal API usage has changed.

### 2.2 Event System

**Tauri 1.x:**
```rust
app.emit_all("event-name", payload).unwrap();
```

**Tauri 2.0:**
```rust
app.emit_all("event-name", payload).unwrap();
```

While the API is similar, event handling in Tauri 2.0 has improved type safety and error handling.

### 2.3 Window Management

**Tauri 1.x:**
```rust
let window = app.get_window("main").unwrap();
```

**Tauri 2.0:**
```rust
let window = app.webview_window("main").unwrap();
```

`Window` has been renamed to `WebviewWindow` in Tauri 2.0.

## 3. Plugin System Migration

### 3.1 State Management

**Tauri 1.x:**
```rust
pub struct AppState<R: Runtime> {
    inner_state: Arc<Mutex<InnerState>>,
    _phantom: PhantomData<R>,
}
```

**Tauri 2.0:**
```rust
pub struct AppState {
    inner_state: Arc<Mutex<InnerState>>,
}
```

Removed generic `Runtime` parameter from state structures.

### 3.2 Plugin Implementation

**Tauri 1.x:**
```rust
impl<R: Runtime> Plugin<R> for MyPlugin<R> {
    fn name(&self) -> &'static str {
        "my-plugin"
    }

    fn initialize(&mut self, app: &AppHandle<R>) -> Result<(), Box<dyn Error>> {
        // Initialize
        Ok(())
    }
}
```

**Tauri 2.0:**
```rust
impl Plugin for MyPlugin {
    fn name(&self) -> &'static str {
        "my-plugin"
    }

    fn initialize(&mut self, app: &AppHandle) -> Result<(), Box<dyn Error>> {
        app.plugin(
            tauri::plugin::Builder::new("my-plugin")
                .js_init_script(include_str!("./my_plugin_init.js"))
                .setup(|app, _| {
                    // Setup code
                    Ok(())
                })
                .build(),
        )?;
        
        Ok(())
    }
}
```

Tauri 2.0 introduces a more structured plugin system with JavaScript initialization scripts.

### 3.3 Command Handlers

**Tauri 1.x:**
```rust
#[tauri::command]
fn my_command<R: Runtime>(state: tauri::State<'_, Arc<Mutex<AppState<R>>>>) -> Result<String, String> {
    // Command implementation
}
```

**Tauri 2.0:**
```rust
#[tauri::command]
async fn my_command(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    // Command implementation
}
```

Command handlers in Tauri 2.0 are now async by default.

## 4. Frontend Integration

### 4.1 JavaScript API

**Tauri 1.x:**
```javascript
import { invoke } from '@tauri-apps/api/tauri';
await invoke('my_command', { param: 'value' });
```

**Tauri 2.0:**
```javascript
import { invoke } from '@tauri-apps/api/tauri';
await invoke('plugin:my_plugin:my_command', { param: 'value' });
```

Tauri 2.0 uses a namespaced command format with plugin prefix.

### 4.2 JavaScript Initialization Scripts

Tauri 2.0 plugins use JavaScript initialization scripts to define their API:

```javascript
window.__TAURI_PLUGIN_MY_PLUGIN__ = {
  init() {
    console.log("Initializing My Plugin");
    
    // Listen for events
    window.__TAURI__.event.listen("my-plugin:event", (event) => {
      // Handle event
    });
    
    // Export API
    return {
      async doSomething(param) {
        return window.__TAURI__.invoke("plugin:my_plugin:do_something", { param });
      }
    };
  }
};
```

## 5. Event System Migration

### 5.1 Listening to Events

**Tauri 1.x:**
```javascript
import { listen } from '@tauri-apps/api/event';
const unlisten = await listen('event-name', (event) => {
  // Handle event
});
```

**Tauri 2.0:**
```javascript
import { listen } from '@tauri-apps/api/event';
const unlisten = await listen('event-name', (event) => {
  // Handle event
});
```

While the API is similar, event payloads and handling have improved type safety.

### 5.2 Emitting Events

**Tauri 1.x:**
```javascript
import { emit } from '@tauri-apps/api/event';
await emit('event-name', { data: 'value' });
```

**Tauri 2.0:**
```javascript
import { emit } from '@tauri-apps/api/event';
await emit('event-name', { data: 'value' });
```

The API remains consistent, but internally events are handled differently.

## Challenges and Solutions

### Challenge 1: Generic Runtime Parameter

The removal of the generic `Runtime` parameter simplified our code but required careful refactoring of all state structures.

**Solution:** We created new state structures without the generic parameter and updated all references throughout the codebase.

### Challenge 2: Async Command Handlers

Converting synchronous command handlers to async required careful attention to error handling.

**Solution:** We used `tokio::sync::mpsc` channels for asynchronous message passing and improved error handling with `anyhow::Result`.

### Challenge 3: JavaScript API Updates

Updating the JavaScript API to use the new plugin format required careful coordination with the backend.

**Solution:** We created comprehensive JavaScript initialization scripts that expose a consistent API while handling the internal changes.

## Lessons Learned

1. **Start with Core Components:** Begin migration with core components and then move to plugins and frontend.

2. **Use Feature Flags Wisely:** Feature flags can help manage the transition between versions.

3. **Embrace Async Patterns:** Tauri 2.0's async-first approach encourages better error handling and concurrency patterns.

4. **Test Thoroughly:** Comprehensive testing is essential to ensure compatibility and detect regression issues.

5. **Document Changes:** Maintain detailed documentation of all changes to help future developers understand the codebase.

## Conclusion

Migrating to Tauri 2.0 has improved our codebase in several ways:

1. **Cleaner Code Structure:** Removing conditional compilation and generic parameters has made the code more readable.

2. **Modern Async Patterns:** Using async/await provides more consistent error handling and better performance.

3. **Simplified State Management:** Removing generic parameters has simplified our state structures.

4. **Plugin Architecture:** The updated plugin system provides a cleaner separation of concerns.

5. **Unified JavaScript API:** Consistent API design across all plugins with proper Tauri 2.0 patterns.

The migration process, while challenging, has resulted in a more maintainable and future-proof application. 
