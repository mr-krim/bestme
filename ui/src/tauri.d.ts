// This is a placeholder type declaration file for Tauri
// In a real setup, these would be provided by the @tauri-apps/api package

declare module '@tauri-apps/api/tauri' {
  /**
   * Invoke a Tauri command
   */
  export function invoke<T = any>(command: string, args?: Record<string, unknown>): Promise<T>;
}

declare module '@tauri-apps/api/event' {
  interface TauriEvent {
    event: string;
    windowLabel: string;
    payload: any;
  }

  type EventCallback<T> = (event: TauriEvent & { payload: T }) => void;
  
  /**
   * Model download progress event payload
   */
  interface ModelDownloadProgressPayload {
    model: string;
    progress: number;
  }
  
  /**
   * Model download completion event payload
   */
  interface ModelDownloadCompletePayload {
    model: string;
    path: string;
  }

  /**
   * Listen to an event from the backend
   */
  function listen<T = any>(
    event: string, 
    handler: EventCallback<T>
  ): Promise<() => void>;

  /**
   * Unlisten to an event from the backend
   */
  function unlisten(event: string): Promise<void>;
} 
