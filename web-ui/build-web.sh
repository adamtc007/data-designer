#!/bin/bash

# Build script for Data Designer Web (WASM)
set -e

echo "🦀 Building Data Designer Web UI..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf dist/ pkg/

# Build the WASM package
echo "📦 Building WASM package..."
wasm-pack build --target web --out-dir pkg --dev

# Create dist directory
mkdir -p dist

# Copy WASM files
cp pkg/*.wasm dist/
cp pkg/*.js dist/

# Create index.html
echo "📄 Creating index.html..."
cat > dist/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Data Designer - Web Edition</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        html, body {
            margin: 0;
            padding: 0;
            height: 100%;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: #1e1e1e;
            color: #ffffff;
        }

        #loading_text {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            text-align: center;
            font-size: 18px;
        }

        .spinner {
            border: 4px solid #333;
            border-top: 4px solid #00aaff;
            border-radius: 50%;
            width: 40px;
            height: 40px;
            animation: spin 1s linear infinite;
            margin: 20px auto;
        }

        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }

        canvas {
            display: block;
            width: 100%;
            height: 100%;
        }
    </style>
</head>
<body>
    <div id="loading_text">
        <div class="spinner"></div>
        <p>🦀 Loading Data Designer Web Edition...</p>
        <p style="font-size: 14px; color: #888;">Built with Rust + WASM + egui</p>
    </div>

    <canvas id="the_canvas_id"></canvas>

    <script type="module">
        import init, { start } from './data_designer_web_ui.js';

        async function run() {
            try {
                await init();
                start("the_canvas_id");
            } catch (error) {
                console.error("Failed to start app:", error);
                document.getElementById("loading_text").innerHTML =
                    "<p style='color: #ff4444;'>❌ Failed to load application</p>" +
                    "<p style='font-size: 14px;'>Check browser console for details</p>";
            }
        }

        run();
    </script>
</body>
</html>
EOF

echo "✅ Build complete! Files in dist/ directory:"
ls -la dist/

echo ""
echo "🚀 To serve with miniserve:"
echo "   miniserve dist/ --port 8080 --index index.html"
echo ""
echo "🌐 Then open: http://localhost:8080"