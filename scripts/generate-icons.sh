#!/bin/bash
# Generate icons for Windows, macOS, and Linux

echo "Generating icons for BestMe..."

# Create temporary SVG icon
cat > /tmp/bestme-icon.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <!-- Background Circle -->
  <circle cx="256" cy="256" r="240" fill="#1e40af" stroke="#3b82f6" stroke-width="16"/>
  
  <!-- Microphone Icon -->
  <g transform="translate(256, 256)">
    <!-- Mic Body -->
    <rect x="-40" y="-120" width="80" height="160" rx="40" fill="#ffffff"/>
    
    <!-- Mic Stand -->
    <rect x="-8" y="40" width="16" height="60" fill="#ffffff"/>
    
    <!-- Mic Base -->
    <rect x="-40" y="100" width="80" height="16" rx="8" fill="#ffffff"/>
    
    <!-- Sound Waves -->
    <path d="M -80,-80 Q -100,-60 -100,-40 Q -100,-20 -80,0" 
          fill="none" stroke="#60a5fa" stroke-width="8" stroke-linecap="round"/>
    <path d="M 80,-80 Q 100,-60 100,-40 Q 100,-20 80,0" 
          fill="none" stroke="#60a5fa" stroke-width="8" stroke-linecap="round"/>
    
    <!-- Inner Sound Waves -->
    <path d="M -60,-60 Q -70,-50 -70,-40 Q -70,-30 -60,-20" 
          fill="none" stroke="#93c5fd" stroke-width="6" stroke-linecap="round"/>
    <path d="M 60,-60 Q 70,-50 70,-40 Q 70,-30 60,-20" 
          fill="none" stroke="#93c5fd" stroke-width="6" stroke-linecap="round"/>
  </g>
  
  <!-- BM Text -->
  <text x="256" y="420" font-family="Arial, sans-serif" font-size="48" font-weight="bold" 
        text-anchor="middle" fill="#ffffff">BM</text>
</svg>
EOF

# Check if ImageMagick is installed
if ! command -v convert &> /dev/null; then
    echo "ImageMagick not found. Installing..."
    if command -v apt-get &> /dev/null; then
        sudo apt-get update && sudo apt-get install -y imagemagick
    else
        echo "Please install ImageMagick manually"
        exit 1
    fi
fi

# Generate PNG icons
echo "Generating PNG icons..."
convert -background none /tmp/bestme-icon.svg -resize 32x32 src-tauri/icons/32x32.png
convert -background none /tmp/bestme-icon.svg -resize 128x128 src-tauri/icons/128x128.png
convert -background none /tmp/bestme-icon.svg -resize 256x256 src-tauri/icons/128x128@2x.png
convert -background none /tmp/bestme-icon.svg -resize 512x512 src-tauri/icons/icon.png

# Generate Windows ICO file
echo "Generating Windows ICO..."
convert -background none /tmp/bestme-icon.svg -define icon:auto-resize=256,128,64,48,32,16 src-tauri/icons/icon.ico

# Generate macOS ICNS file
echo "Generating macOS ICNS..."
if command -v png2icns &> /dev/null; then
    png2icns src-tauri/icons/icon.icns src-tauri/icons/icon.png
else
    # Fallback: just copy PNG as placeholder
    cp src-tauri/icons/icon.png src-tauri/icons/icon.icns
    echo "Warning: png2icns not found. Using PNG as placeholder for ICNS"
fi

# Clean up
rm /tmp/bestme-icon.svg

echo "Icons generated successfully!"
ls -la src-tauri/icons/