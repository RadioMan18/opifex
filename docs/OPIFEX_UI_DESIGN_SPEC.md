# Opifex UI Design Spec

## Purpose

This document defines the design system and user experience direction for Opifex as a world-class AI agent builder. The goal is to turn Opifex from a technically capable but visually generic tool into a premium, high-trust product that feels like a design studio for intelligent systems.

The product is not just a chat app. It is a structured workflow for:

- defining an agent
- interviewing for requirements
- turning conversation into a plan
- editing the plan visually and structurally
- validating logic, tools, and outputs
- saving and iterating on versions

The interface should feel precise, confident, and expert-level.

## Design Mission

Opifex should feel like:

- a premium AI systems workspace
- a guided design studio for agent logic
- a reliable planner, not a toy assistant
- a tool people trust to turn intent into structured execution

It should not feel like:

- a default SaaS dashboard
- a generic chatbot shell
- a raw technical graph editor without hierarchy
- a cluttered utility app with competing priorities

## Product Narrative

Opifex sits at the intersection of three products:

1. Conversational guidance
2. Structured planning
3. Visual system building

The app has two primary surfaces:

- the conversational Oppy workflow
- the block-based Agent Canvas workflow

These are not separate products. They are two views of the same system-building process:

- Chat is the storytelling layer
- Canvas is the system layer
- Plan panel is the artifact layer

The design must communicate this hierarchy clearly at every level.

## Core Experience Principles

### 1. Make the product feel expert

The UI should communicate competence, not novelty.

- confident visual hierarchy
- minimal visual noise
- readable panel structure
- deliberate motion
- exact spacing and alignment

### 2. Show structure, not chaos

The product contains a lot of logic, data, and workflows. The interface must convert complexity into clarity.

- group related actions
- separate primary and secondary controls
- treat the plan as a first-class artifact
- use consistent status patterns

### 3. The plan is the product

The plan panel is not an auxiliary area. It is the artifact users are building and reviewing.

It must feel like a premium design review surface:

- clear sections
- direct labels
- valid/invalid states
- assumptions and risks visible
- outputs and tools explained

### 4. Trust is a design problem

Users need to understand:

- what the AI inferred
- what the user confirmed
- what is missing
- what is risky
- what will happen next

The UI must make uncertainty visible without making the product feel fragile.

## Design Goals

### User goals

- create an agent without confusion
- understand how the system is being built
- see the plan evolve as they talk
- edit confidently and safely
- validate logic before execution
- save and compare versions

### Product goals

- increase confidence in agent building
- reduce cognitive overhead
- distinguish “conversation” from “system design” clearly
- present a premium, memorable visual identity
- support both expert and first-time users

## Design System Foundations

## 1. Visual Identity

Opifex should adopt a “premium technical editorial” aesthetic.

This direction balances:

- technical precision
- calming confidence
- high-trust clarity
- strong product identity

### Recommended tone

- elevated but not flashy
- serious but not sterile
- premium without being over-designed
- precise rather than decorative

### Avoid

- generic SaaS neutral gray patterns
- flat dashboard clutter
- visual noise from low-value controls
- competing accent colors
- decorative shadows that do not create hierarchy

## 2. Color System

### Basis

Use a disciplined dark-mode-first palette with layered neutrals and one strong accent system.

### Core palette

- Background: deep neutral black / graphite
- Panel surfaces: layered dark grays, slightly lifted from base
- Card surfaces: warm dark surfaces with subtle contrast
- Primary accent: electric blue, cyan, or cool signal hue
- Success: green for valid states
- Warning: amber for risky or incomplete states
- Error: red for destructive/invalid states
- Text primary: high contrast cool white / neutral white
- Text secondary: muted gray-blue
- Text tertiary: lower emphasis metadata

### Principles

- one clear accent system
- strong contrast for readability
- neutral base with controlled accent use
- avoid arbitrary color clutter
- use colors semantically, not decoratively

### Suggested palette direction

```text
Base background: #0B1020
Panel surface: #111827
Elevated card: #151E2D
Border: #243247
Text primary: #F3F7FF
Text secondary: #B4C0D2
Text tertiary: #7C8AA5
Accent: #3B82F6
Accent hover: #2563EB
Success: #22C55E
Warning: #F59E0B
Error: #EF4444
```

This palette is a starting point. The final brand should feel more premium and less generic than the current default neutral system.

## 3. Typography

### Type philosophy

Typography should feel precise, confidence-building, and editorial.

Recommended system:

- UI Sans: system sans or clean geometric sans
- headings: semibold / medium weight, tight tracking
- body: readable, generous line-height
- labels: uppercase and small, spaced for system structure
- metadata: compact and restrained

### Type scale

- Caption: 11px, uppercase, tracked
- Meta: 12px
- Body: 14px / 16px
- Subheading: 18px / 20px
- Section heading: 24px
- Panel heading: 28px
- Hero / key statement: 32px+

### Rules

- no weak body text
- do not use too many font weights
- keep hierarchy intentional
- labels should help structure, not decorate

## 4. Layout System

### Grid and spacing

Use a consistent spacing system based on 4px increments.

Examples:

- 4px = micro spacing
- 8px = control gaps
- 12px = compact component spacing
- 16px = standard UI spacing
- 24px = section spacing
- 32px = panel rhythm

### Layout structure

The app shell should use a clear 3-part structure:

1. top bar
2. main content
3. panel and inspector area

### Recommended shell structure

```text
┌──────────────────────────────────────────────────────┐
│ Top Bar                                             │
├──────────────────────────────────────────────────────┤
│ Left Pane | Center/Primary Workspace | Right Pane     │
│ Chat      | Agent Builder/CANVAS     | Plan / Info    │
└──────────────────────────────────────────────────────┘
```

The shell should feel architectural, not cramped.

## 5. Major UI Surfaces

## A. App Shell

The shell must establish the product’s central hierarchy.

### App Bar requirements

- strong brand anchor
- clean action grouping
- clearly separated primary vs secondary actions
- no crowded overflow area
- responsive behavior at narrow widths

### Current issue to fix

The current right-side action cluster is overcrowded. This is not a cosmetic tweak; it is a structural layout issue.

#### Problem

The current top bar includes too many visible actions in one row:

- API Keys
- Import
- New Agent
- Canvas/Oppy toggle
- Home
- Dark mode

This creates a crowded right edge and causes layout compression.

#### Recommended fix

Use a simplified structure:

- left: brand
- center: context / active workspace label (optional)
- right: compact action cluster

Priority-based layout:

Primary visible actions:
- New Agent
- Canvas / Oppy toggle

Secondary actions moved into utility menu:
- API Keys
- Import
- Home
- Theme toggle
- more actions later

This creates clarity and prevents the right side from feeling clipped or overloaded.

### Top bar layout recommendation

Use a grid-based layout:

```text
[top-left brand] [top-center context] [top-right actions]
```

CSS behavior:

- `display: grid`
- `grid-template-columns: auto 1fr auto`
- action group `justify-self: end`
- use compact button sizing
- `overflow: hidden` only at the final fallback stage
- avoid negative margins

### Utility menu pattern

The utility menu should be a compact kebab or ellipsis trigger with dropdown actions.

This is the correct pattern for:

- theme toggle
- import/export functions
- model settings
- account/provider actions
- advanced workflow controls

## B. Agent Library

The library is a foundational entry point and should feel like a curated workspace, not a raw list.

### Goals

- make it easy to choose or create an agent
- create confidence in the product’s structure
- support starter templates and user agents clearly

### Layout

- list or grid of agents
- strong card hierarchy
- quick metadata: name, description, updated time, status
- starter badges for built-ins
- primary CTA: Create Agent

### Card hierarchy

Each agent card should show:

- name
- brief description
- model or provider
- status or activity
- actions with minimal clutter

### Empty state

When no agents exist, show a strong empty-state card with:

- clear explanation of what an agent is
- a primary CTA to create one
- optional “Use example starter” CTA

## C. Conversational Workflow

The chat area should feel like a guided expert interview, not a plain message log.

### Goals

- ask meaningful clarifying questions
- show that the system is understanding intent
- make the process feel structured and deliberate
- create trust and momentum

### Interface pattern

- header with assistant identity (Oppy)
- message thread with distinct user vs assistant styles
- agent status and state information in the header
- clear “Build Agent” CTA once enough information is available
- typing and thinking states with motion to indicate intelligence

### Interaction states

- interviewing mode
- building mode
- build failed
- retry state
- ready to edit

### Message card treatment

User messages:
- compact, readable, aligned right or left depending on layout

Assistant messages:
- clearer emphasis
- include structured insight when available
- support short summaries and plan cards
- show plan summaries inline when relevant

### Required additions

- assistant states like “I understand the goal”
- plan summary cards embedded in messages
- prompt guidance for missing information
- stronger action affordances for build/retry

## D. Plan Panel

The plan panel is the most important surface in the product. It should feel like a premium review surface.

### Sections

The plan should be grouped logically into these blocks:

1. Objective and success criteria
2. Agent role and scope
3. Tools and capabilities
4. Knowledge and data sources
5. Workflow steps
6. Triggers and entry points
7. Output format and success conditions
8. Risks / assumptions / validation notes
9. Versions and save history

### Rules

- each section should be visually distinct
- use structured cards, not raw JSON dumps alone
- provide toggles for expanded/collapsed detail
- show completion states and warnings
- highlight missing or risky decisions

### Visual treatment

- neutral card backgrounds
- section headings with strong hierarchy
- status badges for complete/incomplete/risky
- mild emphasis for the active section
- compact yet readable metadata

### Required plan review states

- complete and ready
- missing information
- risky assumption
- tool not configured
- incompatible values

## E. Agent Canvas

The canvas is the system-building surface and should feel like a true design workspace.

### Core clarity

The canvas must clearly communicate:

- node categories
- connection flow
- selected node state
- edit context
- system path

### Layout structure

- left palette or block library
- center canvas workspace
- right inspector / tool drawer

### Visual treatment

- blocks should be clearly identifiable by category
- active node selection should be unmistakable
- connectors should be clean and legible
- dragging should feel precise and light
- empty canvas should feel intentional

### Block palette design

The Block Palette should be structured, not a wall of labels.

Use categories such as:

- Triggers
- LLM
- Tools
- Agents
- Outputs
- Control Flow
- Utilities

Each block should show:

- icon or shape
- name
- short description
- category

## F. Inspector and Tools Drawer

This is where the user edits the selected system block.

### Requirements

- clean property editing
- semantic grouping
- clear defaults
- visible state metadata
- tool configuration clarity
- rationale where relevant

### Visual approach

- use structured sections, not endless forms
- reduce cognitive load with logical grouping
- keep important controls visible and high affordance
- make validation obvious

## 6. Interaction and Motion Design

### Motion philosophy

Motion should support understanding, not decoration.

Use motion for:

- panel transitions
- selection feedback
- assistant response states
- status changes
- task progress

Avoid:

- heavy bounce effects
- constant motion across the UI
- decorative animation for animation’s sake

### Recommended motion patterns

- subtle fade for panels
- gentle slide for drawers
- soft transitions on chips and controls
- moderate response on active states
- restrained loading animations

## 7. State Model and Feedback

The product needs a comprehensive state language.

### State categories

- idle
- active
- loading
- selected
- warning
- error
- complete
- archived or saved
- unsaved
- disabled

### Feedback patterns

- status pill
- inline warning banner
- transient toast for save/export/success
- non-blocking validation messages
- destructive confirmations when required

### Important principle

Every state should be understandable without reading code.

## 8. Accessibility Requirements

Opifex must feel premium and accessible at the same time.

Required:

- sufficient contrast in all states
- visible focus states
- keyboard navigation support
- readable text sizing
- semantic structure for panels and buttons
- support for reduced motion
- clear labels on all controls
- keyboard-friendly canvas interactions

## 9. Responsive Behavior

Although Opifex is a desktop application, the interface still needs sensible responsive behavior.

### Requirements

- maintain shell integrity at narrow window widths
- collapse less important controls into utility menus
- keep plan panel and canvas readable at reduced widths
- ensure the app works cleanly on large displays without empty, stretched surfaces

## 10. Design System Components

### Buttons

- primary: create / build / confirm actions
- secondary: utilities / non-blocking actions
- tertiary: minimal action chips
- destructive: delete / reset / remove

### Status pills

- ready
- building
- warning
- failed
- saved
- draft

### Inputs

- text inputs
- textarea
- select dropdowns
- segmented control
- toggle switches

### Cards

- plan sections
- agent cards
- library tiles
- messages
- status summaries

### Modals

- create agent
- rename agent
- save version
- API key settings
- destructive confirmation

### Empty states

- no agents yet
- no messages yet
- no plan sections yet
- no versions saved

## 11. Immediate Design Fixes (Highest Priority)

These are the most important changes to ship first:

1. Simplify the top bar action grouping
2. Move low-priority controls into a utility menu
3. Reduce visual crowding on the right edge
4. Establish a clear hierarchy between chat, canvas, and plan
5. Give the plan panel a premium review treatment
6. Improve state and confidence communication
7. Create a consistent component system for all surfaces

## 12. Design Roadmap

### Phase 1: Foundation

- define palette, type, spacing, and states
- reset button hierarchy
- simplify top bar and shell layout
- unify panel styling

### Phase 2: Product clarity

- redesign agent library
- improve chat experience
- redesign plan panel
- add stronger empty states and onboarding

### Phase 3: System-building polish

- improved canvas visual hierarchy
- inspector / tool drawer refinement
- richer block categories and controls
- stronger version and review affordances

### Phase 4: Premium polish

- motion refinement
- higher-end status communication
- accessibility pass
- final density and interaction tuning

## 13. Final Design Standard

The final Opifex experience should feel like this:

- premium but not flashy
- technical but not cold
- guided but not simplistic
- structured but not rigid
- expert but not intimidating

It should make users feel:

- I understand how this system works
- I trust the logic being built
- I can shape the agent with confidence
- I am working in a serious system-building studio

## Summary

Opifex already has a strong product idea. The main opportunity is not feature expansion; it is product expression.

The UI should move from:

- generic utility app

to:

- premium AI system studio

The design system must align the chat interface, canvas interface, and plan artifact into one coherent product language. The top bar simplification is the first visible step toward that goal, but the real prize is a design system that makes the entire product feel coherent, premium, and trustworthy.
