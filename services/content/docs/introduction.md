---
title: "Introduction to SystemPrompt"
description: "Learn how to build AI-powered applications with SystemPrompt"
author: "SystemPrompt Team"
slug: "introduction"
keywords: "systemprompt, ai, agents, introduction"
image: ""
kind: "guide"
public: true
tags: ["documentation", "getting-started"]
published_at: "2025-01-01"
---

# Introduction to SystemPrompt

SystemPrompt is an extensible framework for building AI-powered applications.

## Overview

SystemPrompt provides:

- **Agent Framework**: Build intelligent agents with custom skills
- **Extension System**: Add functionality through modular extensions
- **Content Management**: Built-in blog and documentation support
- **Link Analytics**: Track engagement with your content

## Quick Start

### Installation

```bash
# Clone the template
git clone --recursive https://github.com/systempromptio/systemprompt-template

# Enter the directory
cd systemprompt-template

# Run setup
just setup
```

### Starting the Server

```bash
# Start all services
just start

# Or run the server directly
cargo run
```

## Architecture

The template uses a modular architecture:

```
systemprompt-template/
├── extensions/          # Custom extensions
│   └── blog/           # Blog extension
├── services/           # Configuration and content
│   ├── agents/         # Agent definitions
│   ├── config/         # Service configuration
│   └── content/        # Blog and documentation
└── src/                # Application entry point
```

## Next Steps

- Configure your agents in `services/agents/`
- Add your content to `services/content/`
- Create custom extensions in `extensions/`
