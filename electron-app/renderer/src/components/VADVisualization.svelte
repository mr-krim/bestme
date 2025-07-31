<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  // Using Electron API events instead of Tauri
  
  // VAD states
  let isDetectingSpeech = false;
  let isSpeaking = false;
  let vadEnabled = false;
  let speechLevel = 0;
  
  // Animation
  let animationFrame: number;
  let targetLevel = 0;
  
  // Event unlisteners
  let unlistenVadState: (() => void) | null = null;
  let unlistenAudioLevel: (() => void) | null = null;
  
  onMount(async () => {
    // Listen for VAD state changes
    unlistenVadState = await window.electronAPI.listen('vad:state', (event) => {
      const payload = event.payload as any;
      vadEnabled = payload.enabled;
      isDetectingSpeech = payload.detecting;
      isSpeaking = payload.speaking;
    });
    
    // Listen for audio level updates
    unlistenAudioLevel = await window.electronAPI.listen('audio:level', (event) => {
      const payload = event.payload as any;
      targetLevel = payload.level || 0;
    });
    
    // Start animation loop
    animate();
  });
  
  onDestroy(() => {
    // Clean up listeners
    if (unlistenVadState) unlistenVadState();
    if (unlistenAudioLevel) unlistenAudioLevel();
    
    // Cancel animation
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
    }
  });
  
  function animate() {
    // Smooth animation for level changes
    speechLevel += (targetLevel - speechLevel) * 0.3;
    
    animationFrame = requestAnimationFrame(animate);
  }
  
  $: statusText = vadEnabled 
    ? (isSpeaking ? 'Speaking' : (isDetectingSpeech ? 'Listening' : 'Silent'))
    : 'VAD Disabled';
  
  $: statusClass = vadEnabled
    ? (isSpeaking ? 'speaking' : (isDetectingSpeech ? 'listening' : 'silent'))
    : 'disabled';
</script>

<div class="vad-visualization">
  <div class="status-indicator {statusClass}">
    <div class="status-dot"></div>
    <span class="status-text">{statusText}</span>
  </div>
  
  {#if vadEnabled}
    <div class="level-meter">
      <div class="level-bar" style="width: {speechLevel * 100}%"></div>
      <div class="threshold-line"></div>
    </div>
  {/if}
</div>

<style>
  .vad-visualization {
    display: flex;
    align-items: center;
    gap: 15px;
    padding: 10px;
    background: var(--vad-bg, #f5f5f5);
    border-radius: 8px;
    margin: 10px 0;
  }
  
  .status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 12px;
    border-radius: 20px;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.3s ease;
  }
  
  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    transition: all 0.3s ease;
  }
  
  .status-indicator.disabled {
    background: #e0e0e0;
    color: #666;
  }
  
  .status-indicator.disabled .status-dot {
    background: #999;
  }
  
  .status-indicator.silent {
    background: #e3f2fd;
    color: #1976d2;
  }
  
  .status-indicator.silent .status-dot {
    background: #1976d2;
  }
  
  .status-indicator.listening {
    background: #fff3e0;
    color: #f57c00;
  }
  
  .status-indicator.listening .status-dot {
    background: #f57c00;
    animation: pulse 1.5s infinite;
  }
  
  .status-indicator.speaking {
    background: #e8f5e9;
    color: #388e3c;
  }
  
  .status-indicator.speaking .status-dot {
    background: #388e3c;
    animation: pulse 0.8s infinite;
  }
  
  @keyframes pulse {
    0% {
      transform: scale(1);
      opacity: 1;
    }
    50% {
      transform: scale(1.2);
      opacity: 0.8;
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }
  
  .level-meter {
    flex: 1;
    height: 20px;
    background: #e0e0e0;
    border-radius: 10px;
    position: relative;
    overflow: hidden;
  }
  
  .level-bar {
    height: 100%;
    background: linear-gradient(to right, #4caf50, #8bc34a, #cddc39, #ffeb3b, #ff9800, #f44336);
    border-radius: 10px;
    transition: width 0.1s ease;
  }
  
  .threshold-line {
    position: absolute;
    top: 0;
    left: 50%;
    width: 2px;
    height: 100%;
    background: #333;
    opacity: 0.5;
  }
  
  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    .vad-visualization {
      --vad-bg: #2a2a2a;
    }
    
    .level-meter {
      background: #444;
    }
    
    .threshold-line {
      background: #fff;
      opacity: 0.3;
    }
    
    .status-indicator.disabled {
      background: #333;
      color: #999;
    }
    
    .status-indicator.silent {
      background: #1a237e;
      color: #64b5f6;
    }
    
    .status-indicator.listening {
      background: #e65100;
      color: #ffb74d;
    }
    
    .status-indicator.speaking {
      background: #1b5e20;
      color: #81c784;
    }
  }
</style>