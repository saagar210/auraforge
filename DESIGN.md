# AuraForge Design System

> **Design Philosophy**: "The Forge" — A futuristic Mount Doom where powerful tools are crafted from raw ideas. Dark, atmospheric, warm. Ancient ceremony meets precise creation.

---

## Table of Contents

1. [Design Principles](#design-principles)
2. [Color System](#color-system)
3. [Typography](#typography)
4. [Spacing & Layout](#spacing--layout)
5. [Components](#components)
6. [Animations & Effects](#animations--effects)
7. [Icons](#icons)
8. [States & Feedback](#states--feedback)
9. [Responsive Behavior](#responsive-behavior)
10. [Implementation Notes](#implementation-notes)

---

## Design Principles

### 1. **Atmosphere Over Decoration**
Every visual element serves the forge metaphor. No gratuitous ornamentation—effects should feel organic and purposeful, like heat rising or metal glowing.

### 2. **Warmth in Darkness**
The dark canvas isn't cold or sterile. Warm undertones throughout. The forge provides light and life in the void.

### 3. **Power with Restraint**
The UI should feel powerful but controlled. Not chaotic flames—focused, intentional heat. The user is the master smith.

### 4. **Ceremony for Significance**
Key moments (starting a project, generating documents) deserve visual ceremony. Routine actions stay subtle.

### 5. **Readability is Sacred**
Despite the atmospheric styling, text must always be highly readable. No sacrificing function for form.

---

## Color System

### Core Palette: "Molten Core"

#### Backgrounds
| Token | Hex | RGB | Usage |
|-------|-----|-----|-------|
| `--bg-void` | `#0D0A0B` | `13, 10, 11` | App background, deepest layer |
| `--bg-elevated` | `#1A1517` | `26, 21, 23` | Cards, panels, sidebar |
| `--bg-surface` | `#252022` | `37, 32, 34` | Input fields, interactive surfaces |
| `--bg-surface-hover` | `#2F292B` | `47, 41, 43` | Hover states on surfaces |
| `--bg-warm` | `#1F1815` | `31, 24, 21` | AI message background (forge-lit) |

#### Accents
| Token | Hex | RGB | Usage |
|-------|-----|-----|-------|
| `--accent-primary` | `#FF6B35` | `255, 107, 53` | Primary buttons, key highlights |
| `--accent-gold` | `#D4AF37` | `212, 175, 55` | Special highlights, "the ring" |
| `--accent-glow` | `#E8A045` | `232, 160, 69` | Glows, focus rings, warm highlights |
| `--accent-ember` | `#D4483B` | `212, 72, 59` | Warnings, destructive actions |
| `--accent-success` | `#7CB342` | `124, 179, 66` | Success states, completion |

#### Text
| Token | Hex | RGB | Usage |
|-------|-----|-----|-------|
| `--text-primary` | `#F5F0EB` | `245, 240, 235` | Primary text, headings |
| `--text-secondary` | `#9A9295` | `154, 146, 149` | Secondary text, timestamps |
| `--text-muted` | `#6B6567` | `107, 101, 103` | Placeholders, disabled text |
| `--text-inverse` | `#0D0A0B` | `13, 10, 11` | Text on light backgrounds |

#### Borders & Dividers
| Token | Hex | RGB | Usage |
|-------|-----|-----|-------|
| `--border-subtle` | `#2A2527` | `42, 37, 39` | Subtle dividers |
| `--border-default` | `#3D3639` | `61, 54, 57` | Default borders |
| `--border-focus` | `#E8A045` | `232, 160, 69` | Focus states |
| `--border-glow` | `rgba(232, 160, 69, 0.3)` | — | Glow borders |

#### Semantic Colors
| Token | Hex | Usage |
|-------|-----|-------|
| `--status-thinking` | `#E8A045` | AI processing indicator |
| `--status-success` | `#7CB342` | Completion, success |
| `--status-warning` | `#FFB700` | Warnings |
| `--status-error` | `#D4483B` | Errors |

### Gradients

```css
/* Heat thermal gradient for background movement */
--gradient-thermal: radial-gradient(
  ellipse at var(--thermal-x, 50%) var(--thermal-y, 50%),
  rgba(26, 10, 10, 0.8) 0%,
  transparent 70%
);

/* Forge glow for buttons and highlights */
--gradient-forge-glow: radial-gradient(
  circle at center,
  rgba(232, 160, 69, 0.4) 0%,
  rgba(232, 160, 69, 0.1) 50%,
  transparent 70%
);

/* Molten border gradient */
--gradient-molten-border: linear-gradient(
  90deg,
  #D4483B 0%,
  #FF6B35 25%,
  #E8A045 50%,
  #FF6B35 75%,
  #D4483B 100%
);
```

### Shadows

```css
/* Subtle elevation */
--shadow-sm: 0 2px 4px rgba(0, 0, 0, 0.3);

/* Card elevation */
--shadow-md: 0 4px 12px rgba(0, 0, 0, 0.4);

/* Modal/overlay elevation */
--shadow-lg: 0 8px 24px rgba(0, 0, 0, 0.5);

/* Inner shadow for recessed elements */
--shadow-inner: inset 0 2px 4px rgba(0, 0, 0, 0.3);

/* Glow shadow for focus/active states */
--shadow-glow: 0 0 20px rgba(232, 160, 69, 0.3);
--shadow-glow-intense: 0 0 30px rgba(232, 160, 69, 0.5);
```

---

## Typography

### Font Stack

```css
/* Headings - Cinzel */
--font-heading: 'Cinzel', 'Times New Roman', serif;

/* Body - Inter */
--font-body: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;

/* Monospace - JetBrains Mono */
--font-mono: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
```

### Font Loading

```html
<!-- Google Fonts -->
<link href="https://fonts.googleapis.com/css2?family=Cinzel:wght@400;500;600;700&family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
```

### Type Scale

| Token | Size | Weight | Line Height | Letter Spacing | Font | Usage |
|-------|------|--------|-------------|----------------|------|-------|
| `--text-display` | 32px | 600 | 1.2 | 0.02em | Cinzel | Hero text, empty states |
| `--text-h1` | 24px | 600 | 1.3 | 0.01em | Cinzel | Page titles |
| `--text-h2` | 20px | 500 | 1.3 | 0.01em | Cinzel | Section headings |
| `--text-h3` | 16px | 500 | 1.4 | 0.01em | Cinzel | Card titles, labels |
| `--text-body` | 14px | 400 | 1.6 | 0 | Inter | Primary body text |
| `--text-body-lg` | 16px | 400 | 1.6 | 0 | Inter | Emphasized body text |
| `--text-small` | 13px | 400 | 1.5 | 0 | Inter | Secondary text |
| `--text-xs` | 11px | 500 | 1.4 | 0.02em | Inter | Timestamps, badges |
| `--text-mono` | 13px | 400 | 1.5 | 0 | JetBrains | Code blocks |

### Text Styles

```css
/* Heading with subtle glow */
.text-heading-glow {
  font-family: var(--font-heading);
  color: var(--text-primary);
  text-shadow: 0 0 30px rgba(232, 160, 69, 0.2);
}

/* Gold accent text */
.text-gold {
  color: var(--accent-gold);
}

/* Muted secondary */
.text-muted {
  color: var(--text-secondary);
}
```

---

## Spacing & Layout

### Spacing Scale

| Token | Value | Usage |
|-------|-------|-------|
| `--space-1` | 4px | Tight spacing, inline elements |
| `--space-2` | 8px | Related elements |
| `--space-3` | 12px | Default component padding |
| `--space-4` | 16px | Card padding, section gaps |
| `--space-5` | 20px | Larger gaps |
| `--space-6` | 24px | Section padding |
| `--space-8` | 32px | Major section gaps |
| `--space-10` | 40px | Page-level spacing |
| `--space-12` | 48px | Large separations |

### Layout Tokens

```css
/* Sidebar */
--sidebar-width: 260px;
--sidebar-collapsed-width: 60px;

/* Content area */
--content-max-width: 800px;
--chat-max-width: 720px;

/* Input area */
--input-height: 48px;
--input-max-height: 200px;

/* Border radius */
--radius-sm: 4px;
--radius-md: 8px;
--radius-lg: 12px;
--radius-xl: 16px;
--radius-full: 9999px;
```

### App Layout

```
┌──────────────────────────────────────────────────────────────────┐
│                        Native Title Bar                          │
├─────────────────┬────────────────────────────────────────────────┤
│                 │                                                │
│     SIDEBAR     │                 MAIN CONTENT                   │
│    (260px)      │                                                │
│                 │    ┌────────────────────────────────────┐     │
│  ┌───────────┐  │    │                                    │     │
│  │  Session  │  │    │         CHAT MESSAGES              │     │
│  │   List    │  │    │          (720px max)               │     │
│  │           │  │    │                                    │     │
│  │           │  │    │                                    │     │
│  └───────────┘  │    │                                    │     │
│                 │    └────────────────────────────────────┘     │
│  ┌───────────┐  │                                                │
│  │  + New    │  │    ┌────────────────────────────────────┐     │
│  │  Project  │  │    │          INPUT AREA                │     │
│  └───────────┘  │    └────────────────────────────────────┘     │
│                 │                                                │
│  ┌───────────┐  │              [ GENERATE BUTTON ]               │
│  │ Settings  │  │                                                │
│  └───────────┘  │                                                │
├─────────────────┴────────────────────────────────────────────────┤
│                    Ember Particle Layer                          │
└──────────────────────────────────────────────────────────────────┘
```

---

## Components

### 1. Sidebar

#### Container
```css
.sidebar {
  width: var(--sidebar-width);
  background: var(--bg-elevated);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  height: 100%;
}
```

#### Session List Item

**States:**
- Default: Dark background, subtle border
- Hover: Slight background lighten, border becomes visible
- Active: Gold-orange left border, warmer background, subtle glow

```css
.session-item {
  padding: var(--space-3) var(--space-4);
  margin: var(--space-1) var(--space-2);
  border-radius: var(--radius-md);
  border: 1px solid transparent;
  background: transparent;
  cursor: pointer;
  transition: all 0.2s ease;
}

.session-item:hover {
  background: var(--bg-surface);
  border-color: var(--border-subtle);
}

.session-item.active {
  background: var(--bg-warm);
  border-left: 3px solid var(--accent-glow);
  box-shadow: var(--shadow-glow);
}

.session-item__title {
  font-family: var(--font-heading);
  font-size: var(--text-h3);
  color: var(--text-primary);
  margin-bottom: var(--space-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.session-item__timestamp {
  font-size: var(--text-xs);
  color: var(--text-muted);
}
```

#### New Project Button

```css
.btn-new-project {
  margin: var(--space-4);
  padding: var(--space-3) var(--space-4);
  background: transparent;
  border: 1px solid var(--accent-gold);
  border-radius: var(--radius-md);
  color: var(--accent-gold);
  font-family: var(--font-heading);
  font-size: var(--text-h3);
  cursor: pointer;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}

.btn-new-project:hover {
  background: rgba(212, 175, 55, 0.1);
  box-shadow: var(--shadow-glow);
}

.btn-new-project:active {
  transform: scale(0.98);
}
```

#### Settings Button

```css
.btn-settings {
  margin: var(--space-4);
  margin-top: auto;
  padding: var(--space-2) var(--space-4);
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: var(--text-small);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  transition: color 0.2s ease;
}

.btn-settings:hover {
  color: var(--text-primary);
}
```

---

### 2. Chat Messages

#### Message Container

```css
.chat-container {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-6);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  max-width: var(--chat-max-width);
  margin: 0 auto;
}
```

#### User Message

```css
.message.user {
  align-self: flex-end;
  max-width: 85%;
  background: var(--bg-surface);
  border-radius: var(--radius-lg);
  border-bottom-right-radius: var(--radius-sm);
  padding: var(--space-3) var(--space-4);
  box-shadow: var(--shadow-sm);
  animation: message-in-right 0.3s ease;
}

.message.user .message__text {
  color: var(--text-primary);
  font-size: var(--text-body);
  line-height: 1.6;
}

@keyframes message-in-right {
  from {
    opacity: 0;
    transform: translateX(20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}
```

#### AI Message (AuraForge)

```css
.message.ai {
  align-self: flex-start;
  max-width: 85%;
  background: var(--bg-warm);
  border-radius: var(--radius-lg);
  border-bottom-left-radius: var(--radius-sm);
  padding: var(--space-3) var(--space-4);
  box-shadow: var(--shadow-sm);
  position: relative;
  animation: message-in-left 0.3s ease;
  
  /* Forge glow on left edge */
  border-left: 2px solid var(--accent-glow);
  box-shadow: 
    var(--shadow-sm),
    -4px 0 20px rgba(232, 160, 69, 0.15);
}

.message.ai .message__text {
  color: var(--text-primary);
  font-size: var(--text-body);
  line-height: 1.6;
}

/* Text inscribing animation for new messages */
.message.ai.streaming .message__text {
  animation: inscribe 0.05s ease;
}

@keyframes message-in-left {
  from {
    opacity: 0;
    transform: translateX(-20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes inscribe {
  from {
    opacity: 0.7;
  }
  to {
    opacity: 1;
  }
}
```

#### AI Avatar/Indicator

```css
.message.ai::before {
  content: '';
  position: absolute;
  left: -12px;
  top: var(--space-3);
  width: 8px;
  height: 8px;
  background: var(--accent-glow);
  border-radius: var(--radius-full);
  box-shadow: 0 0 10px var(--accent-glow);
}
```

#### Thinking Indicator

```css
.thinking-indicator {
  display: flex;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
  align-self: flex-start;
}

.thinking-dot {
  width: 8px;
  height: 12px;
  background: linear-gradient(
    to top,
    var(--accent-ember) 0%,
    var(--accent-primary) 50%,
    var(--accent-glow) 100%
  );
  border-radius: 2px 2px 4px 4px;
  animation: flame-flicker 0.8s ease-in-out infinite;
}

.thinking-dot:nth-child(2) {
  animation-delay: 0.2s;
}

.thinking-dot:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes flame-flicker {
  0%, 100% {
    transform: scaleY(1) scaleX(1);
    opacity: 0.8;
  }
  50% {
    transform: scaleY(1.3) scaleX(0.9);
    opacity: 1;
  }
}
```

---

### 3. Input Area

```css
.input-area {
  padding: var(--space-4) var(--space-6);
  background: var(--bg-elevated);
  border-top: 1px solid var(--border-subtle);
}

.input-wrapper {
  display: flex;
  align-items: flex-end;
  gap: var(--space-3);
  max-width: var(--chat-max-width);
  margin: 0 auto;
}

.input-field {
  flex: 1;
  min-height: var(--input-height);
  max-height: var(--input-max-height);
  padding: var(--space-3) var(--space-4);
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  color: var(--text-primary);
  font-family: var(--font-body);
  font-size: var(--text-body);
  resize: none;
  transition: all 0.2s ease;
}

.input-field::placeholder {
  color: var(--text-muted);
}

.input-field:focus {
  outline: none;
  border-color: var(--accent-glow);
  box-shadow: 0 0 0 3px rgba(232, 160, 69, 0.15);
}

.btn-send {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-full);
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.btn-send:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-send.has-content {
  background: var(--accent-primary);
  border-color: var(--accent-primary);
  color: var(--text-inverse);
  animation: pulse-glow 2s ease-in-out infinite;
}

.btn-send.has-content:hover {
  background: var(--accent-glow);
  transform: scale(1.05);
}

@keyframes pulse-glow {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(255, 107, 53, 0.4);
  }
  50% {
    box-shadow: 0 0 0 8px rgba(255, 107, 53, 0);
  }
}
```

---

### 4. Generate Button (Forge the Plan)

This is the ceremonial action—it deserves special treatment.

```css
.btn-generate {
  display: block;
  margin: var(--space-4) auto;
  padding: var(--space-4) var(--space-8);
  background: transparent;
  border: 2px solid var(--accent-gold);
  border-radius: var(--radius-lg);
  color: var(--accent-gold);
  font-family: var(--font-heading);
  font-size: var(--text-h2);
  font-weight: 500;
  letter-spacing: 0.05em;
  cursor: pointer;
  position: relative;
  overflow: hidden;
  transition: all 0.3s ease;
}

/* Subtle inner glow */
.btn-generate::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--gradient-forge-glow);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.btn-generate:hover::before {
  opacity: 1;
}

.btn-generate:hover {
  box-shadow: var(--shadow-glow-intense);
  transform: translateY(-2px);
}

.btn-generate:active {
  transform: translateY(0);
}

/* Disabled state */
.btn-generate:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

/* Generating state */
.btn-generate.generating {
  pointer-events: none;
}

.btn-generate.generating .btn-generate__text {
  opacity: 0;
}

.btn-generate.generating::after {
  content: '';
  position: absolute;
  inset: 4px;
  border: 2px solid transparent;
  border-top-color: var(--accent-gold);
  border-radius: var(--radius-md);
  animation: forge-spin 1s linear infinite;
}

@keyframes forge-spin {
  to {
    transform: rotate(360deg);
  }
}
```

#### Forging Progress Indicator

When generating, show progress with document names:

```css
.forging-progress {
  text-align: center;
  padding: var(--space-4);
}

.forging-progress__ring {
  width: 80px;
  height: 80px;
  margin: 0 auto var(--space-4);
  position: relative;
}

.forging-progress__ring svg {
  transform: rotate(-90deg);
}

.forging-progress__ring circle {
  fill: none;
  stroke-width: 4;
}

.forging-progress__ring .track {
  stroke: var(--border-subtle);
}

.forging-progress__ring .progress {
  stroke: var(--accent-gold);
  stroke-linecap: round;
  stroke-dasharray: 226;
  stroke-dashoffset: 226;
  transition: stroke-dashoffset 0.5s ease;
  filter: drop-shadow(0 0 6px var(--accent-gold));
}

.forging-progress__status {
  font-family: var(--font-heading);
  font-size: var(--text-h3);
  color: var(--accent-glow);
  margin-bottom: var(--space-2);
}

.forging-progress__detail {
  font-size: var(--text-small);
  color: var(--text-secondary);
}
```

---

### 5. Document Preview

#### Tab Navigation

```css
.doc-tabs {
  display: flex;
  gap: var(--space-1);
  padding: 0 var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-elevated);
}

.doc-tab {
  padding: var(--space-3) var(--space-4);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--text-secondary);
  font-family: var(--font-heading);
  font-size: var(--text-small);
  cursor: pointer;
  transition: all 0.2s ease;
}

.doc-tab:hover {
  color: var(--text-primary);
}

.doc-tab.active {
  color: var(--accent-gold);
  border-bottom-color: var(--accent-gold);
  text-shadow: 0 0 20px rgba(212, 175, 55, 0.3);
}
```

#### Document Content

```css
.doc-content {
  padding: var(--space-6);
  background: var(--bg-void);
  overflow-y: auto;
  max-height: calc(100vh - 200px);
}

.doc-content__inner {
  max-width: 720px;
  margin: 0 auto;
  color: var(--text-primary);
  font-family: var(--font-body);
  font-size: var(--text-body);
  line-height: 1.7;
}

/* Markdown styles */
.doc-content h1,
.doc-content h2,
.doc-content h3 {
  font-family: var(--font-heading);
  color: var(--text-primary);
  margin-top: var(--space-6);
  margin-bottom: var(--space-3);
}

.doc-content h1 {
  font-size: var(--text-h1);
  border-bottom: 1px solid var(--border-subtle);
  padding-bottom: var(--space-3);
}

.doc-content h2 {
  font-size: var(--text-h2);
}

.doc-content h3 {
  font-size: var(--text-h3);
}

.doc-content p {
  margin-bottom: var(--space-4);
}

.doc-content code {
  font-family: var(--font-mono);
  font-size: var(--text-mono);
  background: var(--bg-surface);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  color: var(--accent-glow);
}

.doc-content pre {
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  overflow-x: auto;
  margin: var(--space-4) 0;
}

.doc-content pre code {
  background: none;
  padding: 0;
  color: var(--text-primary);
}

.doc-content ul,
.doc-content ol {
  margin: var(--space-4) 0;
  padding-left: var(--space-6);
}

.doc-content li {
  margin-bottom: var(--space-2);
}

.doc-content blockquote {
  border-left: 3px solid var(--accent-glow);
  padding-left: var(--space-4);
  margin: var(--space-4) 0;
  color: var(--text-secondary);
  font-style: italic;
}
```

---

### 6. Empty State

```css
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--space-10);
  text-align: center;
}

.empty-state__icon {
  width: 120px;
  height: 120px;
  margin-bottom: var(--space-6);
  opacity: 0.8;
  /* Animated forge icon - see Animations section */
}

.empty-state__title {
  font-family: var(--font-heading);
  font-size: var(--text-display);
  color: var(--text-primary);
  margin-bottom: var(--space-3);
  text-shadow: 0 0 40px rgba(232, 160, 69, 0.2);
}

.empty-state__subtitle {
  font-size: var(--text-body-lg);
  color: var(--text-secondary);
  max-width: 400px;
  margin-bottom: var(--space-6);
}

.empty-state__action {
  /* Uses btn-new-project styles with larger sizing */
  padding: var(--space-4) var(--space-8);
  font-size: var(--text-h2);
}
```

---

### 7. Settings Modal

```css
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  animation: fade-in 0.2s ease;
}

.modal {
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  animation: modal-in 0.3s ease;
}

.modal__header {
  padding: var(--space-4) var(--space-6);
  border-bottom: 1px solid var(--border-subtle);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal__title {
  font-family: var(--font-heading);
  font-size: var(--text-h2);
  color: var(--text-primary);
}

.modal__close {
  width: 32px;
  height: 32px;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: var(--radius-md);
  transition: all 0.2s ease;
}

.modal__close:hover {
  background: var(--bg-surface);
  color: var(--text-primary);
}

.modal__body {
  padding: var(--space-6);
  overflow-y: auto;
}

@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes modal-in {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-10px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}
```

#### Form Elements

```css
.form-group {
  margin-bottom: var(--space-5);
}

.form-label {
  display: block;
  font-size: var(--text-small);
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: var(--space-2);
}

.form-input {
  width: 100%;
  padding: var(--space-3);
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-family: var(--font-body);
  font-size: var(--text-body);
  transition: all 0.2s ease;
}

.form-input:focus {
  outline: none;
  border-color: var(--accent-glow);
  box-shadow: 0 0 0 3px rgba(232, 160, 69, 0.15);
}

.form-input::placeholder {
  color: var(--text-muted);
}

/* Toggle switch */
.toggle {
  position: relative;
  width: 48px;
  height: 26px;
  background: var(--bg-surface);
  border-radius: var(--radius-full);
  cursor: pointer;
  transition: background 0.2s ease;
}

.toggle.active {
  background: var(--accent-primary);
}

.toggle__knob {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 20px;
  height: 20px;
  background: var(--text-primary);
  border-radius: var(--radius-full);
  transition: transform 0.2s ease;
}

.toggle.active .toggle__knob {
  transform: translateX(22px);
}
```

---

### 8. Toast Notifications

```css
.toast-container {
  position: fixed;
  bottom: var(--space-6);
  right: var(--space-6);
  z-index: 200;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.toast {
  padding: var(--space-3) var(--space-4);
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-md);
  display: flex;
  align-items: center;
  gap: var(--space-3);
  animation: toast-in 0.3s ease;
  max-width: 400px;
}

.toast.success {
  border-left: 3px solid var(--accent-success);
}

.toast.error {
  border-left: 3px solid var(--status-error);
}

.toast.warning {
  border-left: 3px solid var(--status-warning);
}

.toast__message {
  flex: 1;
  font-size: var(--text-small);
  color: var(--text-primary);
}

.toast__close {
  width: 24px;
  height: 24px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateX(20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes toast-out {
  to {
    opacity: 0;
    transform: translateX(20px);
  }
}
```

---

## Animations & Effects

### 1. Ambient Ember Particles

Floating ember particles that drift slowly upward, creating atmosphere.

```tsx
// React component approach
interface Ember {
  id: number;
  x: number;
  y: number;
  size: number;
  duration: number;
  delay: number;
}

const EmberParticles: React.FC = () => {
  const [embers, setEmbers] = useState<Ember[]>([]);
  
  useEffect(() => {
    // Generate 12-15 embers
    const newEmbers = Array.from({ length: 14 }, (_, i) => ({
      id: i,
      x: Math.random() * 100,
      y: 100 + Math.random() * 20, // Start below viewport
      size: 2 + Math.random() * 3,
      duration: 8 + Math.random() * 6, // 8-14 seconds
      delay: Math.random() * 8,
    }));
    setEmbers(newEmbers);
  }, []);

  return (
    <div className="ember-container">
      {embers.map(ember => (
        <div
          key={ember.id}
          className="ember"
          style={{
            '--x': `${ember.x}%`,
            '--size': `${ember.size}px`,
            '--duration': `${ember.duration}s`,
            '--delay': `${ember.delay}s`,
          } as React.CSSProperties}
        />
      ))}
    </div>
  );
};
```

```css
.ember-container {
  position: fixed;
  inset: 0;
  pointer-events: none;
  overflow: hidden;
  z-index: 0;
}

.ember {
  position: absolute;
  left: var(--x);
  bottom: -20px;
  width: var(--size);
  height: var(--size);
  background: radial-gradient(
    circle at center,
    var(--accent-glow) 0%,
    var(--accent-primary) 40%,
    transparent 70%
  );
  border-radius: 50%;
  animation: ember-rise var(--duration) ease-in-out infinite;
  animation-delay: var(--delay);
  opacity: 0;
}

@keyframes ember-rise {
  0% {
    transform: translateY(0) translateX(0);
    opacity: 0;
  }
  10% {
    opacity: 0.8;
  }
  90% {
    opacity: 0.3;
  }
  100% {
    transform: translateY(-100vh) translateX(20px);
    opacity: 0;
  }
}
```

### 2. Background Heat Thermals

Subtle moving gradient patches that simulate heat distortion.

```css
.thermal-background {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  background: var(--bg-void);
}

.thermal-patch {
  position: absolute;
  width: 40%;
  height: 40%;
  background: radial-gradient(
    ellipse at center,
    rgba(26, 10, 10, 0.6) 0%,
    transparent 70%
  );
  animation: thermal-drift 20s ease-in-out infinite;
  opacity: 0.5;
}

.thermal-patch:nth-child(1) {
  top: 20%;
  left: 10%;
  animation-delay: 0s;
}

.thermal-patch:nth-child(2) {
  top: 50%;
  right: 10%;
  animation-delay: -7s;
}

.thermal-patch:nth-child(3) {
  bottom: 10%;
  left: 30%;
  animation-delay: -14s;
}

@keyframes thermal-drift {
  0%, 100% {
    transform: translate(0, 0) scale(1);
  }
  25% {
    transform: translate(30px, -20px) scale(1.1);
  }
  50% {
    transform: translate(-20px, 30px) scale(0.95);
  }
  75% {
    transform: translate(20px, 10px) scale(1.05);
  }
}
```

### 3. Forge Button Glow Animation

```css
.btn-generate {
  /* Base styles from component section */
  position: relative;
}

/* Animated border glow */
.btn-generate::before {
  content: '';
  position: absolute;
  inset: -2px;
  background: var(--gradient-molten-border);
  background-size: 200% 100%;
  border-radius: calc(var(--radius-lg) + 2px);
  z-index: -1;
  opacity: 0;
  animation: molten-flow 3s linear infinite;
  transition: opacity 0.3s ease;
}

.btn-generate:hover::before {
  opacity: 1;
}

@keyframes molten-flow {
  0% {
    background-position: 0% 50%;
  }
  100% {
    background-position: 200% 50%;
  }
}
```

### 4. Document Forging Animation

When documents are being generated, show a "forging" sequence:

```css
.forging-animation {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--space-8);
}

/* Circular progress ring that fills like molten metal */
.forge-ring {
  width: 100px;
  height: 100px;
  position: relative;
  margin-bottom: var(--space-6);
}

.forge-ring__track {
  position: absolute;
  inset: 0;
  border: 4px solid var(--border-subtle);
  border-radius: 50%;
}

.forge-ring__progress {
  position: absolute;
  inset: 0;
  border: 4px solid transparent;
  border-top-color: var(--accent-gold);
  border-radius: 50%;
  animation: forge-spin 1s linear infinite;
  filter: drop-shadow(0 0 10px var(--accent-gold));
}

.forge-ring__center {
  position: absolute;
  inset: 15px;
  background: var(--bg-elevated);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Anvil/hammer icon that pulses */
.forge-ring__icon {
  width: 32px;
  height: 32px;
  color: var(--accent-glow);
  animation: forge-pulse 0.5s ease-in-out infinite;
}

@keyframes forge-spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes forge-pulse {
  0%, 100% {
    transform: scale(1);
    opacity: 0.8;
  }
  50% {
    transform: scale(1.1);
    opacity: 1;
  }
}

/* Status text cycles through document names */
.forge-status {
  font-family: var(--font-heading);
  font-size: var(--text-h3);
  color: var(--accent-glow);
  text-align: center;
  animation: status-glow 2s ease-in-out infinite;
}

@keyframes status-glow {
  0%, 100% {
    text-shadow: 0 0 10px rgba(232, 160, 69, 0.3);
  }
  50% {
    text-shadow: 0 0 20px rgba(232, 160, 69, 0.6);
  }
}
```

### 5. Web Search Indicator

When AuraForge is searching the web:

```css
.search-indicator {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-surface);
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  color: var(--text-secondary);
}

.search-indicator__icon {
  width: 14px;
  height: 14px;
  animation: search-pulse 1s ease-in-out infinite;
}

@keyframes search-pulse {
  0%, 100% {
    opacity: 0.5;
  }
  50% {
    opacity: 1;
  }
}
```

### 6. Success Flash

When documents are successfully generated:

```css
.success-flash {
  position: fixed;
  inset: 0;
  background: radial-gradient(
    circle at center,
    rgba(212, 175, 55, 0.3) 0%,
    transparent 70%
  );
  pointer-events: none;
  animation: flash 0.6s ease-out forwards;
  z-index: 100;
}

@keyframes flash {
  0% {
    opacity: 0;
    transform: scale(0.8);
  }
  30% {
    opacity: 1;
    transform: scale(1);
  }
  100% {
    opacity: 0;
    transform: scale(1.2);
  }
}
```

---

## Icons

### Icon System

Use **Lucide React** for consistent, clean icons that pair well with the aesthetic.

```bash
npm install lucide-react
```

### Icon Mapping

| Action | Icon | Notes |
|--------|------|-------|
| New project | `Plus` or `Flame` | Flame more thematic |
| Send message | `Send` or `ArrowUp` | Arrow feels snappier |
| Generate docs | `Hammer` or `Sparkles` | Hammer = forging |
| Settings | `Settings` or `Cog` | Standard |
| Close | `X` | Standard |
| Save/Export | `Download` or `FolderDown` | |
| Web search | `Globe` or `Search` | |
| Success | `Check` | |
| Error | `AlertCircle` | |
| Session | `MessageSquare` | |
| Delete | `Trash2` | |
| Edit/Rename | `Pencil` | |

### Icon Styling

```css
.icon {
  width: 20px;
  height: 20px;
  stroke-width: 1.5;
}

.icon-sm {
  width: 16px;
  height: 16px;
}

.icon-lg {
  width: 24px;
  height: 24px;
}

/* Glowing icon for emphasis */
.icon-glow {
  filter: drop-shadow(0 0 4px currentColor);
}
```

---

## States & Feedback

### Loading States

| Context | Treatment |
|---------|-----------|
| Initial app load | Full-screen forge icon with pulsing glow |
| Sending message | Send button shows spinner |
| AI thinking | Flame dots animation in chat |
| Generating docs | Forging animation with progress |
| Web searching | Inline indicator in message |
| Saving files | Toast notification |

### Error States

```css
.error-message {
  padding: var(--space-3) var(--space-4);
  background: rgba(212, 72, 59, 0.1);
  border: 1px solid var(--status-error);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
}

.error-message__icon {
  color: var(--status-error);
  flex-shrink: 0;
}

.error-message__text {
  font-size: var(--text-small);
}

.error-message__action {
  color: var(--accent-primary);
  background: none;
  border: none;
  cursor: pointer;
  font-size: var(--text-small);
  text-decoration: underline;
}
```

### Empty States

Each empty state should feel atmospheric, not sterile:

**No sessions yet:**
- Forge illustration (subtle, animated)
- "Your forge awaits"
- "What will you create?"
- Prominent "Begin" button

**No messages in session:**
- Smaller forge icon
- "Describe your vision..."
- Input field is pre-focused

**No documents generated:**
- "The plan awaits forging"
- Generate button pulsing gently

### Disabled States

```css
.disabled,
[disabled] {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}
```

### Focus States

All interactive elements must have visible focus states for accessibility:

```css
:focus-visible {
  outline: none;
  box-shadow: 0 0 0 2px var(--bg-void), 0 0 0 4px var(--accent-glow);
}
```

---

## Responsive Behavior

### Minimum Window Size

- **Minimum width**: 800px
- **Minimum height**: 600px

Tauri should enforce this in the window configuration.

### Sidebar Behavior

At narrower widths (if user resizes):
- Sidebar can collapse to icon-only mode (60px)
- Click to expand temporarily
- Or toggle to stay expanded

```css
/* Collapsed sidebar */
.sidebar.collapsed {
  width: var(--sidebar-collapsed-width);
}

.sidebar.collapsed .session-item__title,
.sidebar.collapsed .session-item__timestamp,
.sidebar.collapsed .btn-new-project span {
  display: none;
}
```

### Content Scaling

Chat messages and documents should remain readable at all sizes:
- Max content width: 720px
- Centered in available space
- Comfortable padding maintained

---

## Implementation Notes

### CSS Architecture

Recommend using **CSS Modules** or **Tailwind CSS** for component-scoped styles.

If using Tailwind, extend the theme with the design tokens:

```js
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        void: '#0D0A0B',
        elevated: '#1A1517',
        surface: '#252022',
        warm: '#1F1815',
        accent: {
          primary: '#FF6B35',
          gold: '#D4AF37',
          glow: '#E8A045',
          ember: '#D4483B',
        },
        // ... etc
      },
      fontFamily: {
        heading: ['Cinzel', 'serif'],
        body: ['Inter', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      // ... spacing, radius, shadows
    },
  },
};
```

### Performance Considerations

1. **Particle animations**: Use CSS animations, not JS. GPU-accelerated via `transform` and `opacity`.

2. **Thermal gradients**: Use `will-change: transform` sparingly on moving elements.

3. **Font loading**: Use `font-display: swap` to prevent FOIT.

4. **Reduce motion**: Respect `prefers-reduced-motion`:

```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

### Accessibility

1. **Color contrast**: All text meets WCAG AA (4.5:1 for body, 3:1 for large text)
2. **Focus indicators**: Visible on all interactive elements
3. **Screen reader**: Semantic HTML, ARIA labels where needed
4. **Keyboard navigation**: Full keyboard support, logical tab order

### Asset Checklist

Before implementation:
- [ ] Google Fonts loaded (Cinzel, Inter, JetBrains Mono)
- [ ] Lucide React installed
- [ ] Forge icon/illustration created (SVG)
- [ ] Favicon in forge theme
- [ ] App icon for macOS (512x512)

---

## Appendix: Color Preview

For quick visual reference, here's the complete palette:

```
BACKGROUNDS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Void        ████████  #0D0A0B
Elevated    ████████  #1A1517
Surface     ████████  #252022
Warm        ████████  #1F1815

ACCENTS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Primary     ████████  #FF6B35
Gold        ████████  #D4AF37
Glow        ████████  #E8A045
Ember       ████████  #D4483B
Success     ████████  #7CB342

TEXT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Primary     ████████  #F5F0EB
Secondary   ████████  #9A9295
Muted       ████████  #6B6567
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01-31 | Initial design system |

---

*"One ring to rule them all, one ring to find them, one ring to bring them all, and in the darkness bind them."*

*AuraForge: Where ideas are forged into plans.*
