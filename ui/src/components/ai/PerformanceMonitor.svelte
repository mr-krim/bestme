<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  
  // Performance data types
  interface PerformanceMetrics {
    timestamp: Date;
    tokens_per_second: number;
    latency_ms: number;
    memory_usage_mb: number;
    gpu_usage_percent?: number;
    cache_hit_rate?: number;
  }
  
  interface ModelPerformanceSummary {
    model_id: string;
    model_name: string;
    total_inferences: number;
    avg_tokens_per_second: number;
    p95_latency_ms: number;
    p99_latency_ms: number;
    max_memory_mb: number;
    errors_count: number;
    cache_hit_rate: number;
  }
  
  // Props
  export let modelId: string = '';
  export let refreshInterval: number = 1000; // ms
  export let historyLength: number = 60; // data points
  
  // State
  let metrics: PerformanceMetrics[] = [];
  let summary: ModelPerformanceSummary | null = null;
  let isMonitoring = false;
  let error = '';
  let updateInterval: NodeJS.Timeout | null = null;
  let unlistenMetrics: (() => void) | null = null;
  
  // Chart dimensions
  let chartWidth = 600;
  let chartHeight = 200;
  const chartPadding = { top: 20, right: 20, bottom: 40, left: 50 };
  
  // Lifecycle
  onMount(async () => {
    await startMonitoring();
    
    // Listen for real-time metrics
    unlistenMetrics = await listen('ai-performance-metrics', (event: any) => {
      const metric = event.payload as PerformanceMetrics;
      addMetric(metric);
    });
  });
  
  onDestroy(() => {
    stopMonitoring();
    if (unlistenMetrics) {
      unlistenMetrics();
    }
  });
  
  // Start monitoring
  async function startMonitoring() {
    if (!modelId) return;
    
    isMonitoring = true;
    error = '';
    
    // Initial load
    await loadPerformanceData();
    
    // Set up refresh interval
    updateInterval = setInterval(loadPerformanceData, refreshInterval);
  }
  
  // Stop monitoring
  function stopMonitoring() {
    isMonitoring = false;
    if (updateInterval) {
      clearInterval(updateInterval);
      updateInterval = null;
    }
  }
  
  // Load performance data
  async function loadPerformanceData() {
    try {
      // Get current metrics
      const currentMetrics = await invoke<PerformanceMetrics>('get_current_ai_metrics', { modelId });
      if (currentMetrics) {
        addMetric(currentMetrics);
      }
      
      // Get summary
      summary = await invoke<ModelPerformanceSummary>('get_ai_performance_summary', { modelId });
    } catch (e) {
      error = `Failed to load performance data: ${e}`;
    }
  }
  
  // Add metric to history
  function addMetric(metric: PerformanceMetrics) {
    metrics = [...metrics, metric].slice(-historyLength);
  }
  
  // Calculate chart paths
  $: tokensPath = calculatePath(metrics, m => m.tokens_per_second, 0, Math.max(...metrics.map(m => m.tokens_per_second)) * 1.1);
  $: latencyPath = calculatePath(metrics, m => m.latency_ms, 0, Math.max(...metrics.map(m => m.latency_ms)) * 1.1);
  $: memoryPath = calculatePath(metrics, m => m.memory_usage_mb, 0, Math.max(...metrics.map(m => m.memory_usage_mb)) * 1.1);
  
  function calculatePath(
    data: PerformanceMetrics[],
    accessor: (m: PerformanceMetrics) => number,
    minValue: number,
    maxValue: number
  ): string {
    if (data.length === 0) return '';
    
    const xScale = (chartWidth - chartPadding.left - chartPadding.right) / (historyLength - 1);
    const yScale = (chartHeight - chartPadding.top - chartPadding.bottom) / (maxValue - minValue);
    
    return data.map((metric, i) => {
      const x = chartPadding.left + i * xScale;
      const y = chartHeight - chartPadding.bottom - (accessor(metric) - minValue) * yScale;
      return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
    }).join(' ');
  }
  
  // Format numbers
  function formatNumber(value: number, decimals: number = 1): string {
    return value.toFixed(decimals);
  }
  
  // Get status color
  function getStatusColor(value: number, thresholds: { good: number; warning: number }): string {
    if (value <= thresholds.good) return '#28a745';
    if (value <= thresholds.warning) return '#ffc107';
    return '#dc3545';
  }
</script>

<div class="performance-monitor">
  <div class="header">
    <h3>Performance Monitor</h3>
    {#if modelId}
      <span class="model-name">{modelId}</span>
    {/if}
    <div class="controls">
      {#if isMonitoring}
        <button on:click={stopMonitoring} class="stop">Stop</button>
      {:else}
        <button on:click={startMonitoring} class="start">Start</button>
      {/if}
    </div>
  </div>
  
  {#if error}
    <div class="error">{error}</div>
  {/if}
  
  {#if summary}
    <div class="summary-grid">
      <div class="summary-card">
        <div class="metric-value">{formatNumber(summary.avg_tokens_per_second)}</div>
        <div class="metric-label">Avg Tokens/sec</div>
      </div>
      <div class="summary-card">
        <div 
          class="metric-value"
          style="color: {getStatusColor(summary.p95_latency_ms, { good: 50, warning: 100 })}"
        >
          {formatNumber(summary.p95_latency_ms, 0)}ms
        </div>
        <div class="metric-label">P95 Latency</div>
      </div>
      <div class="summary-card">
        <div class="metric-value">{formatNumber(summary.max_memory_mb / 1024, 2)}GB</div>
        <div class="metric-label">Max Memory</div>
      </div>
      <div class="summary-card">
        <div class="metric-value">{formatNumber(summary.cache_hit_rate * 100, 0)}%</div>
        <div class="metric-label">Cache Hit Rate</div>
      </div>
      <div class="summary-card">
        <div class="metric-value">{summary.total_inferences}</div>
        <div class="metric-label">Total Inferences</div>
      </div>
      <div class="summary-card">
        <div 
          class="metric-value"
          style="color: {summary.errors_count > 0 ? '#dc3545' : '#28a745'}"
        >
          {summary.errors_count}
        </div>
        <div class="metric-label">Errors</div>
      </div>
    </div>
  {/if}
  
  {#if metrics.length > 0}
    <div class="charts">
      <div class="chart-container">
        <h4>Throughput (tokens/sec)</h4>
        <svg width={chartWidth} height={chartHeight}>
          <path 
            d={tokensPath}
            fill="none"
            stroke="#007bff"
            stroke-width="2"
          />
          <line 
            x1={chartPadding.left}
            y1={chartHeight - chartPadding.bottom}
            x2={chartWidth - chartPadding.right}
            y2={chartHeight - chartPadding.bottom}
            stroke="#ccc"
          />
          <line 
            x1={chartPadding.left}
            y1={chartPadding.top}
            x2={chartPadding.left}
            y2={chartHeight - chartPadding.bottom}
            stroke="#ccc"
          />
        </svg>
      </div>
      
      <div class="chart-container">
        <h4>Latency (ms)</h4>
        <svg width={chartWidth} height={chartHeight}>
          <path 
            d={latencyPath}
            fill="none"
            stroke="#dc3545"
            stroke-width="2"
          />
          <line 
            x1={chartPadding.left}
            y1={chartHeight - chartPadding.bottom}
            x2={chartWidth - chartPadding.right}
            y2={chartHeight - chartPadding.bottom}
            stroke="#ccc"
          />
          <line 
            x1={chartPadding.left}
            y1={chartPadding.top}
            x2={chartPadding.left}
            y2={chartHeight - chartPadding.bottom}
            stroke="#ccc"
          />
        </svg>
      </div>
      
      <div class="chart-container">
        <h4>Memory Usage (MB)</h4>
        <svg width={chartWidth} height={chartHeight}>
          <path 
            d={memoryPath}
            fill="none"
            stroke="#28a745"
            stroke-width="2"
          />
          <line 
            x1={chartPadding.left}
            y1={chartHeight - chartPadding.bottom}
            x2={chartWidth - chartPadding.right}
            y2={chartHeight - chartPadding.bottom}
            stroke="#ccc"
          />
          <line 
            x1={chartPadding.left}
            y1={chartPadding.top}
            x2={chartPadding.left}
            y2={chartHeight - chartPadding.bottom}
            stroke="#ccc"
          />
        </svg>
      </div>
    </div>
  {:else if isMonitoring}
    <div class="no-data">Collecting performance data...</div>
  {/if}
</div>

<style>
  .performance-monitor {
    padding: 20px;
  }
  
  .header {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 20px;
  }
  
  .header h3 {
    margin: 0;
  }
  
  .model-name {
    font-size: 0.9em;
    color: #666;
    background: #f0f0f0;
    padding: 4px 12px;
    border-radius: 20px;
  }
  
  .controls {
    margin-left: auto;
  }
  
  .controls button {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    font-size: 0.9em;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .controls button.start {
    background: #28a745;
    color: white;
  }
  
  .controls button.start:hover {
    background: #218838;
  }
  
  .controls button.stop {
    background: #dc3545;
    color: white;
  }
  
  .controls button.stop:hover {
    background: #c82333;
  }
  
  .error {
    background-color: #f8d7da;
    color: #721c24;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 15px;
    margin-bottom: 30px;
  }
  
  .summary-card {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    text-align: center;
  }
  
  .metric-value {
    font-size: 1.8em;
    font-weight: bold;
    margin-bottom: 5px;
  }
  
  .metric-label {
    font-size: 0.9em;
    color: #666;
  }
  
  .charts {
    display: flex;
    flex-direction: column;
    gap: 30px;
  }
  
  .chart-container h4 {
    margin: 0 0 10px 0;
    font-size: 1em;
    color: #333;
  }
  
  .chart-container svg {
    background: #fff;
    border: 1px solid #e0e0e0;
    border-radius: 4px;
  }
  
  .no-data {
    text-align: center;
    padding: 40px;
    color: #666;
  }
  
  @media (prefers-color-scheme: dark) {
    .model-name {
      background: #3a3a3a;
      color: #e0e0e0;
    }
    
    .summary-card {
      background: #2a2a2a;
      color: #e0e0e0;
    }
    
    .metric-label {
      color: #999;
    }
    
    .chart-container h4 {
      color: #e0e0e0;
    }
    
    .chart-container svg {
      background: #1a1a1a;
      border-color: #3a3a3a;
    }
  }
</style>