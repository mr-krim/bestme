<script>
  // Simple Tabs component
</script>

<div class="tabs-container">
  <div class="tabs-header">
    <slot name="tabs"></slot>
  </div>
  <div class="tabs-content">
    <slot name="content"></slot>
  </div>
</div>

<style>
  .tabs-container {
    width: 100%;
  }
  
  .tabs-header {
    display: flex;
    border-bottom: 1px solid #e0e0e0;
    margin-bottom: 20px;
  }
  
  .tabs-content {
    padding: 0 10px;
  }
  
  :global(.tabs-header button) {
    padding: 10px 16px;
    border: none;
    background: none;
    cursor: pointer;
    font-size: 1rem;
    border-bottom: 2px solid transparent;
    transition: all 0.2s ease;
  }
  
  :global(.tabs-header button:hover) {
    background-color: #f5f5f5;
  }
  
  :global(.tabs-header button.active) {
    border-bottom: 2px solid #4299e1;
    font-weight: 500;
  }
</style> 
