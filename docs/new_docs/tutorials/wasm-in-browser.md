# Running DomainForge in the Browser (WASM)

DomainForge can run entirely client-side using WebAssembly. This allows for building interactive editors or visualizers without a backend.

## Prerequisites

- Rust with `wasm32-unknown-unknown` target
- `wasm-pack`

## Step 1: Build WASM

```bash
cd sea-core
wasm-pack build --target web --out-dir ../examples/wasm-demo/pkg
```

## Step 2: HTML Setup

Create `index.html`.

```html
<!DOCTYPE html>
<html>
<head>
    <title>SEA WASM Demo</title>
</head>
<body>
    <h1>SEA Parser</h1>
    <textarea id="input" rows="10" cols="50">
entity Web { type = "service" }
    </textarea>
    <button id="parseBtn">Parse</button>
    <pre id="output"></pre>

    <script type="module">
        import init, { parse_sea } from './pkg/sea_core.js';

        async function run() {
            await init(); // Initialize WASM module

            document.getElementById('parseBtn').onclick = () => {
                const input = document.getElementById('input').value;
                try {
                    const result = parse_sea(input);
                    document.getElementById('output').textContent =
                        JSON.stringify(result, null, 2);
                } catch (e) {
                    document.getElementById('output').textContent = "Error: " + e;
                }
            };
        }

        run();
    </script>
</body>
</html>
```

## Step 3: Serve

You need a local web server to serve the WASM file (due to CORS).

```bash
# Using python
python3 -m http.server
```

Open `http://localhost:8000` in your browser. Click "Parse". You should see the JSON representation of the parsed model.

## See Also

- [Cross-Language Binding Strategy](../explanations/cross-language-binding-strategy.md)
