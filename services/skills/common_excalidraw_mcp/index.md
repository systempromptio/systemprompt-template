----|-------|-------------|
| **Elements** | create, read, update, delete, batch | Create and manipulate shapes, text, arrows, lines |
| **Layout** | align, distribute, group, lock | Organize elements on the canvas |
| **Vision** | inspect, screenshot | AI can see and analyze the canvas state |
| **File I/O** | export, import, mermaid | Export/import `.excalidraw` JSON, convert Mermaid diagrams |
| **State** | snapshot, rollback, clear | Canvas state management and undo |

## Prerequisites

- **Node.js 18+** installed
- **npm** installed

## Setup (One-Time)

Run the automated setup script:

```bash
# From the skill directory
bash skills/excalidraw-mcp/scripts/setup.sh
```

This will:
1. Clone the `mcp_excalidraw` repository to `~/Agents/mcp_excalidraw`
2. Install dependencies and build the project
3. Add MCP configuration to your user-level `~/.claude/.mcp.json`

### Manual Setup

If you prefer manual installation:

```bash
# 1. Clone and build
git clone https://github.com/yctimlin/mcp_excalidraw.git ~/Agents/mcp_excalidraw
cd ~/Agents/mcp_excalidraw
npm ci && npm run build

# 2. Add to your ~/.claude/.mcp.json (create if it doesn't exist)
```

Add this to `~/.claude/.mcp.json`:

```json
{
  "mcpServers": {
    "excalidraw": {
      "command": "node",
      "args": ["<HOME>/Agents/mcp_excalidraw/dist/index.js"],
      "env": {
        "EXPRESS_SERVER_URL": "http://localhost:3000"
      }
    }
  }
}
```

Replace `<HOME>` with your actual home directory path.

## Usage

### 1. Start the Canvas Server

Before using Excalidraw tools, start the canvas web server in a separate terminal:

```bash
cd ~/Agents/mcp_excalidraw && npm run canvas
```

This opens a browser with the Excalidraw canvas at `http://localhost:3000`.

### 2. Use via Claude

Once the canvas server is running and the MCP is configured, Claude has access to all Excalidraw tools. Examples:

- "Draw a flowchart showing the user login process"
- "Create an architecture diagram for our microservices"
- "Make a wireframe for the dashboard page"
- "Convert this Mermaid diagram to Excalidraw"
- "Export the current canvas as an .excalidraw file"

### 3. Canvas Server Port

The default port is `3000`. To use a different port:

```bash
PORT=3001 npm run canvas
```

Update `EXPRESS_SERVER_URL` in your `.mcp.json` accordingly.

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "MCP server not found" | Ensure `~/.claude/.mcp.json` has the correct path to `dist/index.js` |
| "Connection refused" | Start the canvas server first: `cd ~/Agents/mcp_excalidraw && npm run canvas` |
| Build fails | Ensure Node.js 18+ is installed: `node --version` |
| Port conflict | Use a different port: `PORT=3001 npm run canvas` |

## Source

- **Repository**: https://github.com/yctimlin/mcp_excalidraw
- **License**: MIT