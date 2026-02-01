# AuraForge Quick Reference

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| **Cmd + N** | New session |
| **Cmd + G** | Forge the Plan (generate documents) |
| **Cmd + S** | Save documents to folder |
| **Cmd + ,** | Open settings |
| **Cmd + \\** | Toggle sidebar |
| **Cmd + Shift + P** | Toggle document preview |
| **Cmd + /** | Open help panel |
| **Escape** | Close any open panel |

## Workflow

```
Describe your idea  -->  Answer questions  -->  Forge the Plan  -->  Save to folder
```

1. Start a new session (Cmd+N)
2. Describe what you want to build
3. Answer the clarifying questions AuraForge asks
4. When ready, click **Forge the Plan** (Cmd+G)
5. Review the generated documents in the preview panel (Cmd+Shift+P)
6. Save to a folder on your machine (Cmd+S)

Or: Open `START_HERE.md` in your saved folder — it walks you through everything from there.

## Generated Documents

| File | What It Is |
|------|-----------|
| **START_HERE.md** | Quick-start guide — read this first, walks you through setup |
| **README.md** | Overview of your project plan |
| **SPEC.md** | Detailed technical specification |
| **CLAUDE.md** | Context file for Claude Code — drop it in your project root |
| **PROMPTS.md** | Step-by-step implementation prompts, copy-paste ready |
| **CONVERSATION.md** | Full planning conversation for reference |

## Settings

Open settings with **Cmd + ,** or the gear icon in the sidebar.

**Simple mode** (default):
- Choose AI model from installed models
- Toggle web search on/off
- Set default save location

**Advanced mode** (click "Show advanced settings"):
- Change LLM provider, base URL, temperature, max tokens
- Configure search provider (Tavily, DuckDuckGo, SearXNG)
- Toggle proactive search and conversation export

## Tips

- **Be specific** about what you're building — the more detail you give, the better the plan
- **Web search** enriches conversations with current best practices (optional, uses Tavily or DuckDuckGo)
- **Re-run setup** from Settings if you need to change your AI model
- All data stays on your machine — nothing is sent to external servers (except optional web search queries)
