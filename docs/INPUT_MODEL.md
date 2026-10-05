# MoonPub Input Model

This document answers one question only:

**What should input sources such as Feishu, photos, voice memos, and reading excerpts look like when they enter MoonPub?**

The goal is not to implement every input source immediately, but to abstract the currently stable Feishu path first, so that adding each new source later does not reinvent a set of fields and flows.

## Why do this now

The Feishu path is now stable enough to abstract:

- It can import raw material into Inbox
- It can preserve original source information
- It can continue to generate drafts
- It can output structured `action` / `next_command`

But if the input model is not closed now, later work on:

- Photo organization
- Voice notes
- Reading excerpts

can easily become:

- Each input path defines its own frontmatter
- Each path returns different JSON fields
- Plugins / agents write separate logic for every source

This directly breaks the "unified entry protocol" we are already building.

## One-sentence model

MoonPub's input model should be split into two layers:

1. **Raw material layer**
2. **Editable draft layer**

That means:

- Input sources first enter `Inbox/`
- Objects in `Inbox` keep the real source and original content
- Later, `draft-from-inbox` or an equivalent flow turns them into article drafts

## Currently recommended unified structure

### A. Raw material layer: Inbox Item

Every input source should eventually land as an Inbox file.

It should at least contain:

- `source`
- `status`
- `created`
- `type`

Current Feishu already does this:

```yaml
---
source: feishu-minutes
status: inbox
created: 2026-07-02
type: voice-note
minute_token: "obcn123"
source_url: "https://..."
original_file: "/path/to/transcript.txt"
---
```

This shows what the Inbox layer should bear:

- Describe who the source is
- Describe what kind of raw material it is
- Record an upstream traceable identifier
- Preserve the original text or original information

### B. Editable draft layer: Draft Item

The draft layer should no longer care "is this Feishu or photos", but should care about:

- Is this an article ready for editing and publishing?
- What stage is the current draft in?
- What are the next steps for preview and push?

The draft layer is better suited to carry:

- Article title
- Publishing-related fields in frontmatter
- Article body
- Follow-up actions for preview / render / push

## Recommended unified fields

### Minimum required Inbox fields

| Field | Purpose | Note |
|-------|---------|------|
| `source` | Source identifier | e.g. `feishu-minutes`, future `photos`, `voice-memo` |
| `status` | Current stage | Currently recommended fixed value `inbox` |
| `created` | Time entered the system | Not the original capture time, but the time it entered MoonPub |
| `type` | Material type | e.g. `voice-note`, `photo-note`, `reading-note` |

### Optional Inbox source fields

| Field | Purpose |
|-------|---------|
| `source_url` | Upstream link |
| `original_file` | Local original file path |
| `external_id` | Generic source primary key |
| `captured_at` | Original material occurrence time |
| `source_title` | Title in the source system |

### Source-specific fields

Source-specific fields are allowed, but should be minimized:

- Feishu currently uses `minute_token`
- Future photos may use `photo_group_id`
- Voice may use `recording_id`

Recommended principle:

- **The generic system only recognizes `external_id`**
- **Source-specific fields are reserved for concrete input sources**

So the more ideal future form is:

```yaml
external_id: "obcn123"
minute_token: "obcn123"
```

This way:

- Upper layers do not need to understand every platform's private ID
- Input source implementations can still keep source characteristics

## How current Feishu should align

Feishu is already very close to the unified model:

- `source: feishu-minutes`
- `status: inbox`
- `type: voice-note`
- `minute_token`
- `source_url`
- `original_file`

This is no longer just "future consideration"; it is starting to land:

1. This is MoonPub's first formal input model
2. Future new input sources should align with this field hierarchy
3. Feishu currently writes both generic `external_id` and source-specific `minute_token`
4. Existing Feishu reuse logic is starting to prefer `external_id` alignment while remaining compatible with old files that only have `minute_token`

## How future input sources map

### Photos

Recommended mapping:

- `source: photos`
- `type: photo-note`
- `external_id: <stable ID for the same day or same group of photos>`
- `captured_at`
- `original_file` or photo directory reference

Material body can contain:

- Photo list
- EXIF / time / location summary
- Factual information extracted by AI / tools

### Voice memos

Recommended mapping:

- `source: voice-memo`
- `type: voice-note`
- `external_id`
- `captured_at`
- `original_file`

Material body can contain:

- Original transcript
- Speaking segments
- Timestamp summary

### Reading excerpts

Recommended mapping:

- `source: wechat-read` or `reading-notes`
- `type: reading-note`
- `external_id`
- `source_title`
- `source_url`

Material body can contain:

- Original excerpt
- Annotation
- Chapter information

### WeChat Official Account archive

Recommended mapping:

- `source: wechat-mp-article`
- `type: archived-article`
- `external_id`
- `source_title`
- `source_url`
- `source_author`

Material body can contain:

- Original title, author, publish time
- Body Markdown / plain text
- Original URL
- Optional original HTML or structured metadata reference

This line is more of an "archive input source", not a publishing action. Safety boundaries are in [WECHAT_ARCHIVE_WORKFLOW.md](WECHAT_ARCHIVE_WORKFLOW.md) (English) / [WECHAT_ARCHIVE_WORKFLOW_ZH.md](WECHAT_ARCHIVE_WORKFLOW_ZH.md) (Chinese): by default only process public URLs explicitly provided by the user, do not automatically scrape historical lists, and do not save or submit cookies, `pass_ticket`, `uin`, tokens, or QR-code login information.

## Relationship to the Agent / plugin protocol

The input model does not replace the `workspace / status / check` protocol layer.

The relationship should be:

1. Input sources produce Inbox Items
2. `draft-from-inbox` turns Inbox Items into drafts
3. `workspace / status / check` lets the entry layer know the current state
4. `preview / push` continues the publishing action

That means:

- The input model solves "what the material looks like"
- The entry protocol solves "what to do now"

## Recommended landing order

### Step 1

Settle this input model document first.

### Step 2

Make the Feishu path the first input source explicitly aligned to this model.

### Step 3

When adding future photo / voice input sources:

- Prefer reusing `Inbox` layer fields
- Fill `external_id` where possible
- Avoid inventing temporary frontmatter

## Current conclusion

The most reasonable next step is not to code-implement every input source right now, but to make clear:

- **MoonPub's input model is Inbox Item-centric**
- **Feishu is the first formal input model**
- **Future photos / voice / excerpts should map to the same layer structure**

If this continues, the most worthwhile additions to this document are:

- Formal `external_id` introduction strategy
- First version field draft for the photo input source
- Which fields belong to raw material and which enter article draft frontmatter
