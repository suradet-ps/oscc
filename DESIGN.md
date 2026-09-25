# DESIGN.md - OSCC

> Visual design system for OSCC (One Stop Crisis Center / "ศูนย์พึ่งได้") case management.
> This file defines color, type, layout, and component tokens for the Leptos frontend
> (`app/style/`). For product scope, architecture, and workflow, see `AGENTS.md`.

## Overview

OSCC is used mid-shift, under pressure, by people making decisions that affect someone's
safety: an ER nurse registering a walk-in, a forensic physician recording an examination, a
social worker tracking a follow-up, the OSCC lead checking a deadline at 03:00. The screen is
therefore **a workbench, not a destination** — dense, quiet, keyboard-reachable, and honest
about what it does and does not know.

The system has one loud element, and it is not decorative: the **deadline rail**. Directly
under the case header, a row of timer chips shows the windows that actually change clinical
outcomes — HIV PEP (<= 72h), emergency contraception (<= 120h), STI/HBV prophylaxis,
follow-up appointments. Everything else in the interface stays calm and paper-like so the
rail is the only thing that can shout. Red and amber in this system mean *time is running
out*, and they are reserved for that meaning alone.

Because every record here is sensitive (health, sexual behavior, criminal record, minors),
the interface also has a second signature behaviour: **identity is masked by default**. Names
and full CIDs are hidden behind an explicit reveal action that requires a reason and is
recorded in the audit trail. The UI never treats unmasked identity as the normal state.

**Key characteristics:**
- Neutral warm paper workspace ({colors.canvas}), low-fatigue for long shifts
- Flat, hairline-bordered surfaces — structure comes from 1px borders, not shadows
- One loud element only: the deadline rail (timer chips in amber/red/green)
- Deep teal ({colors.brand}) chrome, kept separate from red/amber/green signal colors
- IBM Plex Sans Thai + IBM Plex Sans for bilingual clarity; IBM Plex Mono for case IDs,
  HN, CID, drug codes, timestamps
- Identity masked by default; reveal is explicit, reasoned, and audited
- Restrained corners ({rounded.md}, 6px) — a clinical tool, not a consumer app
- Compact, dense scale: this is read hundreds of times per shift, not once

### Design Principles

1. **Privacy by default, transparency by record** — Masked identity is the default state;
   revealing it is a deliberate, reasoned action; every reveal is visible in the case's
   audit trail to the people who are allowed to see it.
2. **Urgency must be legible without focus** — A deadline chip is readable at arm's length,
   never colour-only (icon + label + countdown), and never animated in a way that delays
   reading it.
3. **Progressive disclosure** — Search first, then case, then sections. Intake form fields
   appear in the order the interview happens, not in database order.
4. **Density over whitespace** — This is a work tool. Compact spacing lets staff scan more
   cases per glance; there is no hero, no marketing copy, no onboarding carousel.
5. **Keyboard-first navigation** — Every action reachable via Tab/Enter, visible focus rings,
   Escape closes overlays. Staff may be typing while wearing gloves, on a shared PC.
6. **Native feel** — 200ms ease-out hover, 150ms focus transitions, no bounce/elastic
   animation; respects `prefers-reduced-motion`.
7. **Honest states** — A missed deadline says "missed". A disconnected server says
   "disconnected". Nothing is softened into a colour that implies an answer it does not have.

## Colors

### Brand & Chrome
- **Brand** ({colors.brand}): `#00695C` — deep teal. Primary buttons, focused input borders,
  selected rows, active nav item. Calm, clinical, and deliberately distinct from the
  red/amber/green signal palette.
- **Brand Dark** ({colors.brand-dark}): `#004D40` — pressed state of brand elements.
- **Brand Soft** ({colors.brand-soft}): `#E0F2F1` — tint for selected rows, follow-up chips,
  and brand-tinted confirmation surfaces.

### Surface — neutral warm paper
- **Canvas** ({colors.canvas}): `#FAFAFA` — app background.
- **Canvas Raised** ({colors.canvas-raised}): `#FFFFFF` — panels, case workspace, forms.
- **Surface Muted** ({colors.surface-muted}): `#F5F5F5` — hover backgrounds, neutral chips.
- **Hairline** ({colors.hairline}): `#E0E0E0` — 1px default border/divider.
- **Hairline Strong** ({colors.hairline-strong}): `#BDBDBD` — input borders, top-bar buttons.

### Text
- **Ink** ({colors.ink}): `#212121` — primary text.
- **Slate** ({colors.slate}): `#616161` — secondary text: labels, timestamps, metadata.
- **Steel** ({colors.steel}): `#9E9E9E` — tertiary text: placeholders, disabled labels.
- **On Brand** ({colors.on-brand}): `#FFFFFF` — text on brand buttons.
- **Muted Code** ({colors.muted-code}): `#757575` — case IDs, HN, CID when de-emphasized.

### Case status — quiet, cool tones (never urgency colours)
Status chips describe where a case is, not how urgent it is. They use cool/neutral tints so
they never compete with the deadline rail.
- **Intake** ({colors.status-intake}): bg `#ECEFF1`, text `#37474F`, border `#CFD8DC`.
- **Active** ({colors.status-active}): bg `#E3F2FD`, text `#1565C0`, border `#90CAF9`.
- **Follow-up** ({colors.status-followup}): bg `#E0F2F1`, text `#00695C`, border `#80CBC4`.
- **Closed** ({colors.status-closed}): bg `#F5F5F5`, text `#616161`, border `#E0E0E0`.

### Urgency — reserved, loud (timer chips only)
Treated as a **tinted system alert**: light background, coloured text, matching hairline
border. Green here means "done", not "good" — it appears only in the deadline rail.
- **Normal** ({colors.timer-normal}): bg `#F5F5F5`, text `#616161`, border `#E0E0E0` —
  deadline comfortably ahead.
- **Due Soon** ({colors.timer-soon}): bg `#FFF8E1`, text `#8a6420`, border `#FFE082` —
  inside the warning window (per-timer policy, e.g. <= 24h for PEP); clock icon.
- **Overdue** ({colors.timer-overdue}): bg `#FFEBEE`, text `#C62828`, border `#FFCDD2` —
  window missed; alert-triangle icon; also used by the missed-task row.
- **Done** ({colors.timer-done}): bg `#E8F5E9`, text `#2E7D32`, border `#A5D6A7` —
  recorded as given/scheduled; check icon.

### Privacy & access
- **Masked** ({colors.masked}): bg `#F5F5F5`, text `#616161`, border `#E0E0E0` — locked
  identity field, the default state.
- **Revealed** ({colors.revealed}): bg `#EDE7F6`, text `#5E35B1`, border `#B39DDB` —
  identity currently unmasked (audited) or a break-glass context; violet is used nowhere else.
- **Reveal Marker** ({colors.reveal-marker}): text `#5E35B1` on `{colors.revealed}` background
  with an eye icon — the persistent "เปิดเผยแล้ว · ถูกบันทึก" chip next to revealed fields.

### System / connection
- **Connected** ({colors.connected}): `#2E7D32` dot + text — API and HOSxP reachable.
- **Degraded** ({colors.degraded}): `#8a6420` dot + text — HOSxP unreachable; case data
  still works, patient lookup unavailable.
- **Disconnected** ({colors.disconnected}): `#C62828` dot + text — API unreachable; banner +
  paper-fallback instructions.

## Typography

### Font Family
- **IBM Plex Sans Thai** (primary, Thai UI text): humanist, good Thai glyph shapes at small
  sizes, open license.
- **IBM Plex Sans** (primary, Latin/numerals): same family and metrics, so mixed
  Thai/English strings (names, drug terms) do not clash.
- **IBM Plex Mono** (data): case IDs, HN, CID, drug codes, timers, timestamps — fixed width
  and unambiguous shapes (0 vs O, 1 vs l matter for a national ID).

### Hierarchy

| Token | Size | Weight | Line Height | Use |
|---|---|---|---|---|
| `{typography.case-id}` | 20px | 700 | 1.25 | Case ID in the case header (IBM Plex Mono) |
| `{typography.timer}` | 18px | 700 | 1.25 | Countdown inside timer chips (IBM Plex Mono) |
| `{typography.page-title}` | 17px | 600 | 1.30 | Queue/panel titles ("เคสของฉัน") |
| `{typography.heading}` | 15px | 600 | 1.30 | Section headings inside a case |
| `{typography.body}` | 15px | 400 | 1.50 | Primary body text, table cells |
| `{typography.body-medium}` | 15px | 500 | 1.50 | Emphasized body (case ID, name in a row) |
| `{typography.label}` | 13px | 600 | 1.40 | Field labels, form section labels |
| `{typography.caption}` | 12px | 400 | 1.40 | Timestamps, secondary metadata |
| `{typography.code}` | 13px | 400 | 1.45 | Case ID, HN, CID, drug codes (IBM Plex Mono) |
| `{typography.button}` | 14px | 600 | 1.30 | Button labels |

### Principles
- The scale is compact and dense; `{typography.case-id}` and `{typography.timer}` are the
  largest text on screen and stay that way.
- Weight carries emphasis: 400 reading, 500 emphasis, 600-700 headings, buttons, timers.
- Line height stays generous (1.40-1.50); staff read this under time pressure in imperfect
  lighting, sometimes on a wall-mounted panel.
- Case IDs, HN, CID, drug codes and countdowns always render in `{typography.code}` /
  `{typography.timer}` with `font-variant-numeric: tabular-nums`.

## Layout

### Spacing
- Base unit 4px, primary increment 8px.
- Tokens: `{spacing.xs}` (4px), `{spacing.sm}` (8px), `{spacing.md}` (12px),
  `{spacing.lg}` (16px), `{spacing.xl}` (24px), `{spacing.xxl}` (32px).
- No hero-scale token exists in this system. There is no hero.

### Workbench structure (case list)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Top bar (44px): OSCC · user + role · connection dot · clock · lock          │
├────────────────────┬────────────────────────────────────┬───────────────────┤
│ NAV RAIL (200px)   │ CASE LIST / WORKSPACE (fluid)      │ CONTEXT RAIL      │
│                    │                                    │ (320px, collapse) │
│ รับแจ้งใหม่ (4)    │ [ search cases... ]                │                   │
│ เคสของฉัน (7)      │ ┌ case-row ──────────────────────┐ │ ACCESS TRAIL      │
│ ติดตามวันนี้ (3)   │ │ OSCC-2026-0042  [Active]  2h   │ │ ใครเปิด/เปิดเผย   │
│ ทั้งหมด            │ │ next: PEP 41:12                │ │ เมื่อไหร่         │
│                    │ └────────────────────────────────┘ │                   │
│ แดชบอร์ด           │ ┌ case-row ──────────────────────┐ │ QUICK FACTS       │
│                    │ │ ...                            │ │ (non-identifying) │
│                    │ └────────────────────────────────┘ │                   │
└────────────────────┴────────────────────────────────────┴───────────────────┘
```

### Workbench structure (case detail)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Top bar                                                                     │
├────────────────────┬────────────────────────────────────┬───────────────────┤
│ NAV RAIL           │ CASE HEADER                        │ ACCESS TRAIL      │
│                    │ OSCC-2026-0042  [Active]            │                   │
│                    │ ผู้ป่วย: สม***** / 1-XXXX-XXXXX-XX-3 │ (who viewed /    │
│                    │ [เปิดเผยตัวตน] (audited)             │  revealed, when)  │
│                    ├────────────────────────────────────┤                   │
│                    │ DEADLINE RAIL  (signature)          │                   │
│                    │ [PEP 41:12 ↑] [EC 3d] [STI done]   │                   │
│                    ├────────────────────────────────────┤                   │
│                    │ SECTIONS                           │                   │
│                    │  รับแจ้ง · คัดกรอง · งาน · ส่งต่อ · │                   │
│                    │  การติดตาม · ประวัติการเข้าถึง       │                   │
└────────────────────┴────────────────────────────────────┴───────────────────┘
```

**Rationale:** the left rail is navigation (queues), the centre is the work, the right rail is
accountability (who has seen this case). Context is never hidden in a drawer; the audit trail
is intentionally adjacent to the record it describes.

### Regions
- **Top bar** — app identity, signed-in user + role, connection state, hospital clock
  (NTP-synced), lock button. Never shows patient identity.
- **Nav rail (200px fixed)** — queue counts and dashboard. Counts are non-identifying.
- **Centre** — either the case queue (case rows) or one open case. One case at a time; no
  split-pane case comparison in Phase 1 (it multiplies reveal risk).
- **Context rail (320px, collapsible)** — access trail and non-identifying quick facts.
  Collapsing it never hides a deadline; the rail belongs to the case workspace, not the
  context rail.

## Elevation & Depth

Elevation is minimal; surfaces are separated by hairlines and background colour.

| Level | Treatment | Use |
|---|---|---|
| 0 (flat) | No shadow; `{colors.hairline}` border | Nav rail, top bar, case rows, timer chips |
| 1 (canvas) | `background: {colors.canvas-raised}` | Case workspace, forms |
| 2 (modal) | `0 8px 24px rgba(23,33,43,0.16)` | Reveal dialog, break-glass dialog, confirmations |

The only floating elements are dialogs. Nothing hovers or lifts on hover.

## Shapes

| Token | Value | Use |
|---|---|---|
| `{rounded.sm}` | 4px | Small inner details, audit dots |
| `{rounded.md}` | 6px | Inputs, buttons, timer chips, case rows |
| `{rounded.lg}` | 8px | Panels, case header, dialogs |
| `{rounded.full}` | 9999px | Tiny chips only — status chips <= 20px tall |

Corners stay tight (4-8px). Pill shapes are limited to status chips; buttons and panels are
never pill-shaped.

## Components

### Top Bar

**`top-bar`** — Thin (44px), flat header.
- Background `{colors.canvas-raised}`, bottom border `1px solid {colors.hairline}`.
- **`top-bar__user`** — user name + role chip ("พยาบาล ER", "นิติเวช", "สังคมสงเคราะห์",
  "หัวหน้า OSCC"). Role is always visible so people know which permissions are active.
- **`top-bar__connection`** — `connection-status` indicator.
- **`top-bar__clock`** — hospital time, `{typography.code}`; the same clock that stamps
  deadlines. Timers always show their absolute due time in a tooltip.
- **`top-bar__lock`** — explicit lock button; locks are also automatic on idle.

### Connection Status

**`connection-status`** — 8px dot + label.
- Connected: `{colors.connected}`, "เชื่อมต่อแล้ว".
- Degraded: `{colors.degraded}`, "HOSxP ไม่ตอบสนอง" — patient lookup disabled, case data OK.
- Disconnected: `{colors.disconnected}`, "เซิร์ฟเวอร์ไม่ตอบสนอง" — a full-width banner
  appears with paper-fallback instructions; all write actions are disabled (Phase 1 has no
  offline queue).

### Session & Lock

**`session-warning`** — appears 60 seconds before session expiry: amber strip inside the top
bar area, countdown, "ต่อเวลา" (requires re-entry of password). No "remember me".
**`lock-screen`** — full-surface overlay, OSCC mark only, no case data, password field.
Locking must also clear any currently revealed identity back to masked.

### Nav Rail

**`nav-rail`** — fixed 200px, `{colors.canvas}`, right hairline.
**`nav-item`** — `{typography.body}`, padding `{spacing.sm} {spacing.md}`, rounded
`{rounded.md}`. Active: `{colors.brand-soft}` background, `{colors.brand}` text,
`{typography.body-medium}`.
**`nav-item__count`** — non-identifying count in `{typography.code}`; never a name.
Queue order is fixed: รับแจ้งใหม่, เคสของฉัน, ติดตามวันนี้, ทั้งหมด, แดชบอร์ด.

### Case Queue

**`case-row`** — one case per row, two lines.
- Line 1: `case_id` in `{typography.code}` (100px min-width) | `status-chip` | `age`
  (captions, relative: "2 ชม.") | owner role.
- Line 2: next deadline summary: `timer-chip--compact` (icon + label + countdown) or
  "ไม่มีกำหนด" in `{colors.steel}`.
- Border-bottom `1px solid {colors.hairline}`; hover `{colors.surface-muted}`.
- Rows are sorted by deadline urgency by default, never by patient name.
- A case row never shows a patient name, only the case ID and status.

**`case-search`** — filters by case ID, HN (exact digits only), or date range. HN search
is allowed (staff have it from the form) but results still render masked.

### Case Header

**`case-header`** — top of the workspace.
- `case_id` in `{typography.case-id}`, `status-chip`, incident type in `{typography.body}`.
- Identity line: `identity-field` for name + CID, masked by default.
- Right side: primary action by state ("ปิดเคส", "บันทึกการติดตาม") and overflow menu with
  audited actions only.

### Identity Field & Reveal

**`identity-field`** — masked identity display.
- Default: `{typography.body-medium}` showing `สม*****` / `1-XXXX-XXXXX-XX-3`, with a lock
  icon in `{colors.slate}` and a secondary "เปิดเผยตัวตน" button.
- Revealed: value in full, plus a persistent `reveal-marker` chip ("เปิดเผยแล้ว · ถูกบันทึก")
  in `{colors.revealed}` colours; a "ซ่อน" button re-masks immediately.
- Auto re-mask after 60 seconds of inactivity on the case, on navigation, and on lock.
- Copy is disabled while masked; copying a revealed value requires the reveal flow and is
  itself an audited action.

**`reveal-dialog`** — mandatory reason before any reveal.
- Title "ขอเปิดเผยตัวตนผู้ป่วย"; reason select (การรักษา / ประสานงานคดี / ความปลอดภัย /
  อื่น ๆ) + optional note (max 200 chars, never logged as PII in app logs; stored in audit).
- Notice line: "การเปิดเผยนี้จะถูกบันทึกและมองเห็นได้โดยหัวหน้า OSCC".
- Buttons: secondary "ยกเลิก", primary "ยืนยันและเปิดเผย".

**`break-glass-banner`** — when a case is open under break-glass, a persistent
`{colors.revealed}` banner states who opened it, why, and when, with a link to the access
trail. It cannot be dismissed while the case is open.

### Deadline Rail (signature)

**`deadline-rail`** — full-width strip directly under the case header; the loud element.
- Contains one `timer-chip` per active deadline, ordered by due time.
- Sticky within the workspace; never scrolls out of view, never hidden by collapsing a panel.

**`timer-chip`** — rounded `{rounded.md}`, padding `{spacing.sm} {spacing.md}`.
- Icon (clock / alert-triangle / check) + label ("PEP", "EC", "STI", "HBV", "นัดติดตาม") +
  countdown in `{typography.timer}` + absolute due time in `{typography.caption}`.
- States per the urgency palette: `timer-normal`, `timer-soon`, `timer-overdue`,
  `timer-done`. Never colour-only: icon and label always change with state.
- Actions: "บันทึกแล้ว" (opens a small record dialog) and "มอบหมาย" (assign to another role).
  Both are audited. A missed timer can be acknowledged with a reason, which changes it to
  `timer-done` with a visible "บันทึกย้อนหลัง" marker rather than hiding it.

**`timer-chip--compact`** — icon + label + countdown only, used inside case rows.

### Intake Form

**`form-section`** — a card on `{colors.canvas-raised}`, border `{colors.hairline}`,
rounded `{rounded.lg}`, padding `{spacing.lg}`, section heading in `{typography.heading}`.
Sections follow the interview order: ข้อมูลการรับแจ้ง -> ข้อมูลผู้ป่วย -> การคัดกรอง ->
ความปลอดภัย -> แผนเบื้องต้น.

**`form-field`** — label `{typography.label}` above input; required fields marked with a red
asterisk **plus** the word "จำเป็น" (never colour-only).
- Inputs 40px tall, border `{colors.hairline-strong}`, focus `2px solid {colors.brand}`.
- Free-text fields (if enabled) show a character counter and a hint: "อย่าบันทึกข้อมูล
  เกินจำเป็น".
- Validation is inline, next to the field, in `{colors.timer-overdue}` text colour with an
  icon; the save button stays enabled and moves focus to the first error.

**`form-save-bar`** — sticky bottom bar with the save action, autosave timestamp, and the
case ID. Draft state is visible ("บันทึกร่างเมื่อ 14:32") because intake is interrupted.

### Tasks & Follow-up

**`task-row`** — one follow-up or timer task.
- Checkbox (40px hit target), task title, due time in `{typography.code}`, owner role chip.
- States: pending (neutral), done (`{colors.timer-done}` check + strikethrough title),
  missed (`{colors.timer-overdue}` icon + "เลยกำหนด" label + acknowledge action).
- Completing a task asks for the recorded value (e.g. dose given, appointment date), never a
  free-form "done".

**`referral-row`** — internal referral: target department, reason, status chip, last update.
External agencies do not exist in Phase 1; the component must not accept free-typed external
destinations.

### Audit Trail

**`audit-trail`** — the accountability component, always reachable from the context rail.
**`audit-entry`** — one line: actor (name + role), action ("เปิดเคส", "เปิดเผยตัวตน",
"พิมพ์", "แก้ไข"), target field, timestamp `{typography.code}`, and the reason when the action
was a reveal or break-glass.
- Append-only: no edit or delete affordances exist, ever.
- Chain-verified entries carry a small `audit-seal` chip; a failed verification renders the
  entry in `{colors.timer-overdue}` with "การตรวจสอบความสมบูรณ์ไม่ผ่าน" and is never hidden.
- The trail is readable by roles that can see the case; break-glass entries are additionally
  surfaced to the OSCC lead.

### Dialogs

**`dialog`** — the only floating surface (elevation 2). Width 480px, title
`{typography.heading}`, body `{typography.body}`, actions right-aligned.
- Destructive or privacy-affecting actions ("ปิดเคส", "เปิดเผย", "พิมพ์") use a confirm
  dialog that restates the consequence.
- Never render patient identity in a dialog that does not need it.

### Feedback

**`inline-alert`** — within a panel: icon + text, tinted like the urgency palette but with
system meaning only (e.g. save failed, HOSxP lookup failed).
**`toast`** — transient, bottom-right, 4s, never contains PII. "บันทึกแล้ว" not
"บันทึกเคสของคุณสมชายแล้ว".

### Empty & Loading States

**`empty-state`** — icon + one sentence + one action. Examples: "ยังไม่มีเคสในคิว",
"ไม่พบเคสที่ค้นหา".
**`loading-skeleton`** — hairline-pulse rows, no spinners inside case lists. Patient lookup
shows a distinct "กำลังค้นหา HOSxP..." row because that path can be slow or degraded.

### Dashboard (PII-free)

**`metric-card`** — large count in `{typography.timer}`, label in `{typography.label}`,
optional sparkline in `{colors.brand}`. Counts only; no names, no HN, no case links that
reveal identity.
- Any drill-down keeps the masked rule intact.
- Exports from the dashboard are aggregate-only and audited.

## Privacy & Audit UI Rules

These are not guidelines; they are the reason this design system exists.

1. Masked is the default and the fallback: on load, on navigation, on lock, and after 60
   seconds of inactivity.
2. No PII in window titles, taskbar text, tray notifications, toasts, or tooltips.
3. Colour is never the only signal: every state pairs an icon and a text label.
4. Reveal, break-glass, print, export and case close are always explicit two-step actions
   with a stated consequence.
5. Copy is disabled on masked fields; revealed copy is audited.
6. The access trail is visible next to the case, not buried in settings.
7. Screenshot protection (`SetWindowDisplayAffinity`) is enabled on case workspaces when the
   platform allows it; when it is off, say so in the security notes rather than pretending.
8. No dark patterns: no preselected reveal, no "don't ask again" on privacy dialogs.

## Printing

Out of scope for Phase 1 (internal paperless workflow). When printing is added (Phase 2),
it must follow: audited print action, watermark with case ID + user + timestamp on every
page, `@page` 14mm margins, black-on-white, and a `print-sheet` that is the only content
rendered in `@media print`. Do not add print affordances before the audit path exists.

## Do's and Don'ts

### Do
- Reserve `{colors.timer-*}` (amber/red/green) for deadlines and their acknowledgement only.
- Structure surfaces with hairlines and background colour, not shadows.
- Keep identity masked until an explicit, reasoned reveal; show the reveal marker afterwards.
- Use `{typography.code}` / `{typography.timer}` (monospace, tabular figures) for every case
  ID, HN, CID, drug code, and countdown.
- Keep the type scale compact (15px body, 13px labels); density is what makes this usable
  at 03:00.
- Provide visible focus rings (`2px solid {colors.brand}` on `:focus-visible`) on every
  interactive element.
- Use `200ms ease-out` for hover, `150ms ease-out` for focus.
- Show role chips on the top bar and on task rows so permission context is always visible.

### Don't
- Don't use red/amber/green for decoration, case status, or brand chrome.
- Don't show a patient name in a case row, queue count, notification, or dashboard.
- Don't auto-reveal identity, remember a reveal, or make reveal a single click.
- Don't add a hero, marketing copy, illustration, or promotional banner.
- Don't add hover animations, glow, or elevation to flat surfaces.
- Don't use pill-shaped buttons; pills are for status chips only.
- Don't animate anything on the deadline rail in a way that delays reading it.
- Don't introduce a second chrome accent beyond `{colors.brand}`; solve new needs with
  weight, size, and spacing.
- Don't hide keyboard focus indicators or rely on pointer-only interactions.
- Don't put case data in browser storage, client logs, or any persisted cache.

## Window & Degraded Behavior

OSCC is a fixed-purpose Tauri desktop app, used on shared department PCs with varying
monitors.

| Window width | Behavior |
|---|---|
| < 900px | **Single column:** nav rail collapses to icons, context rail becomes a drawer, case workspace full width. Deadline rail stays sticky. |
| 900-1279px | **Two columns:** nav rail + workspace; context rail collapses by default. |
| >= 1280px | **Full workbench:** nav rail 200px, workspace fluid, context rail 320px. |

Minimum supported window size: 1024x600 (typical ER/office monitors). Case workspace must
remain usable at 1024x600 without horizontal scrolling.

### Focus Management
- **Focus ring:** `2px solid {colors.brand}` on `:focus-visible`.
- **Tab order:** nav rail -> case list -> case header -> deadline rail -> sections -> context
  rail. Dialogs trap focus and return it to the invoking control.
- **Escape:** closes dialogs and drawers; returns focus to the last active control.
- **Lock shortcut:** `Ctrl+L` locks immediately; `Ctrl+K` focuses case search.

### Motion
- Hover `background-color 200ms ease-out`; focus `border-color 150ms ease-out`.
- `@media (prefers-reduced-motion: reduce)` disables all transitions.
- Never bounce/elastic; never fade a deadline chip in for more than 120ms.

### Downtime & Degraded
- API unreachable: full-width `inline-alert` with paper-fallback instructions; write actions
  disabled; the message is explicit ("ระบบไม่ได้บันทึกงานนี้") rather than optimistic.
- HOSxP unreachable: `connection-status` degraded; intake can continue with manual
  demographics entry, flagged as "กรอกเอง" in the audit trail, and linked later.
- Never silently queue writes in Phase 1; staff must know what is and is not recorded.

## Iteration Guide

1. Any new component must declare which existing token set it uses — no ad-hoc hex values.
2. If a new state seems to need a new colour, first try to express it with weight, border, or
   spacing. This system has deliberately few named colours.
3. The urgency palette (red/amber/green) is reserved for deadlines and their
   acknowledgement; the masked/revealed pair is reserved for privacy state. Neither rule is
   relaxed under UI pressure.
4. Default to `{typography.body}` for prose and `{typography.code}` for anything
   ID-shaped.
5. Re-check every new screen against three questions: is identity masked by default, is the
   deadline rail visible, and would a new staff member understand every state without a
   legend?
6. Changes to privacy/audit patterns require updating `AGENTS.md` §2 and this file in the
   same commit.

## Known Gaps

- Dark mode is out of scope for Phase 1; token names can carry a future dark variant without
  renaming, but values are not defined.
- The app icon and wordmark are not final; `{colors.brand}` is provisional to the OSCC app
  and has not been checked against hospital brand guidance.
- Print/watermark specification is deferred to Phase 2 (see Printing).
- High-contrast / forced-colors mode has not been designed; the urgency palette must be
  re-verified there before any pilot on Windows high-contrast settings.
- No usability testing with ER staff has happened yet; the cross-shift test (night shift,
  03:00, 60% screen brightness) is an open action.
