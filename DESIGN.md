# Design System: OP Security Telemetry & Interceptor Cockpit
**Project:** OP Security Proxy (Optimism Superchain Layer-2 JSON-RPC Middleware)

## 1. Visual Theme & Atmosphere
- **Atmosphere:** Tactical, Institutional, High-Density Dark-Mode Cockpit.
- **Vibe:** Swiss-inspired typographic discipline paired with high-contrast Cyberpunk/Bloomberg financial terminal precision.
- **Depth & Surfaces:** Multi-layer obsidian slate (`#0B0E14`, `#121824`, `#1A2333`) with 1px frosted translucent borders (`rgba(255, 255, 255, 0.08)`) and whisper-soft neon cyan & optimism red ambient glow.

## 2. Color Palette & Roles
* **Obsidian Canvas (`#0B0E14`):** Primary background surface for zero-distraction deep contrast.
* **Slate Container (`#121824`):** Secondary card and panel surfaces with hairline borders.
* **Optimism Crimson (`#FF0420`):** Core brand accent and alert indicator for blocked reverting transactions.
* **Superchain Cyan (`#00F0FF`):** High-contrast metric callouts, gas savings indicators, and live network telemetry.
* **Terminal Emerald (`#00E599`):** Safe execution passes and confirmed on-chain broadcast states.
* **Muted Mist (`#8A99AD`):** Secondary metadata, JSON-RPC parameters, and microcopy.

## 3. Typography Rules
* **Display & Navigation:** `Inter`, `-apple-system`, `sans-serif` (tight tracking, semi-bold 600).
* **Metrics & Code Data:** `JetBrains Mono`, `Fira Code`, `monospace` (tabular numbers, exact character alignment for hex hashes and gas units).

## 4. Component Stylings
* **Metric Cards:** Subtly rounded corners (`12px`), obsidian background, hairline neon border, top glow bar.
* **Playground Action Buttons:** Pill-shaped (`rounded-full`) with solid high-contrast fill and subtle active press scaling.
* **Live Feed Stream:** Monospace terminal list with color-coded badges (`BLOCKED`, `SIMULATED`, `PASSED`).
* **Input Forms:** Dark matte background (`#0E131F`), 1px subtle stroke, glowing cyan focus ring.

## 5. Layout Principles
* 12-column responsive CSS grid with 24px gutter.
* Sticky top navigation with live connection status pill (`● Connected to OP-Sec Proxy :3000`).
* Real-time metrics marquee above dual-column layout (Left: Live Interception Feed; Right: Interactive Sandbox Tester).
