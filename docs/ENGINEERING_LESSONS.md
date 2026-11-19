# MoonPub Engineering Lessons

This document is MoonPub's long-term, searchable record of solved engineering problems.

It serves two purposes: check whether a conclusion already exists before debugging, and only cite verified real problems and trade-offs when writing public articles. It does not record tokens, QR codes, accounts, private articles, photos, or full logs.

## Usage Rules

Before debugging, search this document and `AGENTS.md` by keyword. After a problem is fixed and verified, add or update a record whenever any of the following applies:

- It affects a real user path, data safety, privacy, or publishing result.
- The root cause is not obvious and is likely to recur.
- The fix introduces a new boundary, command, configuration convention, or regression test.

Each record should keep at least: symptom and impact scope, confirmed root cause, final fix, prevention constraints, publicly reviewable evidence, and an optional article angle. If a fix is not verified yet, keep it in an issue, PR, or temporary debugging note instead of writing it here as an established conclusion.

## High-priority Lessons

### Config copy is user text too: a `<script>` in the footer becomes a real tag

**Symptom:** A probe article put `description: 描述 <script>x</script>` and `follow_text: 关注 <img src=x onerror=alert(2)>` in the `[footer]` config, plus `footer_qrcode_note: 扫码 <div>` and `footer_qrcode: https://x.example/" onerror="alert(1)` in frontmatter. The rendered HTML contained **real tags and real event handlers** — the `<script>` executes and `onerror` is injected onto the QR `<img>`. The local preview opens in a browser, so this is directly exploitable.

**Root cause:** `footer.rs` defined `html_escape()` but only applied it to three brand-card fields (`brand_name` / `brand_avatar` / `brand_bio`); six copy fields (`divider` / `title` / `description` / `rules` / `qrcode_note` / `follow_text`) were interpolated raw, and two URL fields (`qrcode` / `follow_image`) went into `src` with neither escaping nor scheme filtering. Earlier rounds audited the Markdown renderers; footer is a **separate path** (config → template) and fell outside that audit. Treating "it's config" as "it's trusted" is the mental root of this class.

**Fix:** (1) Escape all six copy fields; multiline ones (`description` / `rules` / `qrcode_note`) go through a new `escaped_multiline()` — **escape the text first, then replace newlines with `<br>`**; reversing it either escapes our own `<br>` or leaves an injection hole. (2) The http(s) branch of both URL fields uses `html_escape(safe_destination(x))`: check the scheme on the **original** URL first, then escape (the quote in `https://…" onerror=…` gets sealed inside the attribute value); local-path data URIs skip scheme filtering (base64 would become `#`) but are still escaped. (3) Add scheme filtering to the `avatar` `src`.

**Guarding against recurrence:** Process-level contract `escape_contract::footer_copy_and_urls_cannot_inject_markup` renders a real article with footer config + frontmatter overrides and asserts no `<script>` / `<div>` / `<img src=x` / `onerror="alert(1)`, with literal markup visible after unescaping and our own `<br>` intact. Controlled mutation (raw `description`, unescaped `qrcode`) turns it red immediately.

**Evidence:** `src/footer.rs` (`escaped_multiline()` + nine call sites), `tests/escape_contract.rs`. Before the fix the probe hit `<script>` / `<div>` / `onerror="alert(1)`; after, zero. Byte-for-byte comparison of 182 rendered documents: **zero changes** (the repo has no dangerous footer markup, confirming no false positives). 955 tests passing, coverage 79.41%.

### Escaping `href` / `src` is not enough: the scheme needs filtering too

**Symptom:** A probe article with `[click](javascript:alert(1))`, `![img](javascript:alert(3))` and a `:::figure` whose `image:` is `javascript:alert(5)` produced HTML with those targets verbatim. Escaping was correct (quotes could not break out of the attribute), but a browser still treats a well-formed `javascript:` as a scheme and executes it. The WeChat backend probably filters these; **`moonpub preview` does not** — a malicious link from fetched/RSS content executes the moment someone clicks it in the local preview.

**Root cause:** Every destination went through `html_escape` only. That prevents attribute break-outs; it cannot touch a scheme that is perfectly valid attribute content.

**Fix:** `src/html.rs::safe_destination()` is now the single gate: only `http` / `https` / `mailto` / `tel` and schemeless relative addresses pass; everything else becomes `#`. Scheme detection follows browser behaviour (leading whitespace/control characters ignored, case-insensitive). Entity-encoded schemes are already inert because `escape` turns their `&` into `&amp;`. Every `href=` / `src=` interpolation in `inline.rs` / `blocks.rs` / `plain.rs` now goes through it, including reference links, block images, `:::figure` `image:` and `:::compact-links` URLs.

**Guarding against recurrence:** Process-level contract `escape_contract::dangerous_url_schemes_never_reach_an_attribute` scans all attributes through the real binary; unit tests cover both dangerous and ordinary destinations. Controlled mutation: making `safe_destination()` a pass-through turns both the contract and the unit test red; restoring leaves sha256 identical.

**Evidence:** After the fix all `javascript:` / `vbscript:` / `data:` / `file:` targets in the probe became `#`, while `https://…` / `mailto:` / `./a.md` were untouched. Byte-for-byte comparison of 181 rendered documents against the previous binary: **0 changes** (the corpus contains no such schemes, so no regression). 954 tests passing, coverage 79.26% → 79.29%.

### The spaces between inline elements get deleted: words end up glued together

**Symptom:** A sentence that puts two inline elements side by side (an inline code span immediately followed by bold) reads `step_yulan调用它` in the published article. A list item where prose is followed by inline code and then bold reads `列表里a b 相邻` — even the space between prose and an element disappears. Rendering all 181 renderable documents in the repo surfaced **1753** whitespace-only text nodes sitting between two inline elements, across 116 documents.

**Root cause:** The WeChat backend does not delete all whitespace. Its actual rule is: a text node that still contains text keeps its spaces (converted to `&nbsp;`), while a node that is *entirely* whitespace is dropped. The published markup proves it — `没有&nbsp;</span><span …><span leaf="">before</span></span>` keeps both sides of the code span; meanwhile every standalone `</span> <span>` node inside code blocks vanished (count: 0). `inline_md()` pushed spaces through verbatim (`s.push(ch)`), so whenever two elements were adjacent that space became a standalone node.

**Fix:** In `inline_md()`, whitespace that follows a closing tag (previous char is `>`) **and** is followed by another inline element is emitted as `&nbsp;` (lookahead via `starts_inline_element()`). Whitespace between prose and an element stays a plain space — it is a legitimate break opportunity and the backend converts it for us. Four hand-assembled templates were fixed the same way: the emoji and text of `render_tip`, the columns of `compact-links`, the header/value pair of term-style tables, and the three dots in the code-block chrome (`display:inline-block` siblings separated by whitespace really do show a gap, so the local preview disagreed with the published article). Also fixed `:::tip`, whose body skipped `body_html()` and therefore displayed literal backticks.

**Guarding against recurrence:** Unit test `whitespace_after_an_inline_element_is_escaped` (requires `</span>&nbsp;<strong`, and forbids changing the plain space in `</strong> 后面`); process-level contract `no_blank_text_node_between_inline_elements` scans a whole rendered document with `blank_between_inline_tags()` for any `</span>` / `</strong>` / `</em>` / `</code>` / `</del>` / `</a>` followed by whitespace and then an inline opening tag; the contract's `drop_blank_text_nodes()` model was corrected too (only fully blank nodes are dropped; spaces around a node that has text are preserved).

**Evidence:** `src/markdown/inline.rs` (whitespace branch + `starts_inline_element()`), `src/illustrate.rs` (tip, chrome dots), `src/markdown/blocks.rs` (compact-links, tip via `body_html`), `src/markdown/plain.rs` (term tables), `tests/render_syntax_contract.rs`. Controlled mutation: forcing the condition to `&& false` turns both the unit test and the contract red; restoring it leaves `shasum -a256 -c` identical, 951 tests passing, coverage 79.26%. Old-vs-new binary over the corpus: 42 of 181 documents changed reader-visible text, **all of them restoring a space that used to be deleted** (`2026-08-10X 推文` → `2026-08-10 X 推文`), zero content changes, zero documents getting shorter.

### One `<div>` in the prose can eat half the article: user text must be escaped

**Symptom:** a probe article carrying `<div>` and `&` in every syntactic position (title, paragraph, bold, inline code, link text, list, quote, table) produced raw tags in the output HTML. Parsed as a browser would, the article reads **only “标题里的”** — the `<div>` inside the heading opens a real div that swallows everything after it. The same sweep found three **attribute** injections: `:::figure` with `image: /a.png" onerror="alert(1)` truncates `src`, and `:::book-info`'s `cover:` and `:::photo-grid`'s image URL have the same hole.

**Root cause:** the catch-all branch of `inline_md()` was `s.push(chars[i])`, writing prose into HTML verbatim; `render_book_info` / `render_callout` / `render_figure` / `render_cover` / `render_photo_grid` in `blocks.rs` and quote-card / divider / concept-card / emotion-card / timeline / comparison / tip in `illustrate.rs` interpolated props and body text the same way. Only inline code, link destinations and image `alt` were escaped before — the visible half was fixed, the invisible half was not. CommonMark passes raw HTML through by default, but WeChat does not support custom tags, so passing it through means losing content.

**Fix:** the `inline_md` catch-all now escapes `<` and `>`; `&` is escaped **only when it does not start a valid character reference** (`&nbsp;` / `&#160;` / `&#x1F600;` pass through — the author wrote them to see a space, and `&amp;nbsp;` would show seven literal characters; this is not an injection surface because a parsed reference never re-opens a tag). Every prop value and illustration string in `blocks.rs` / `illustrate.rs` is `html_escape` / `esc`-ed before it reaches `format!`.

**Prevention:** `tests/escape_contract.rs` adds two process-level contracts — (1) nine syntactic positions each carrying `<div>`: the output may contain no `<(div|b|i|script)` and the unescaped visible text must match exactly; (2) card props carrying `<div>` and `" onerror="`: no `onerror="` may survive (only `&quot;`). Controlled mutation: reverting `'<' => "&lt;"` to a bare `<` and dropping `html_escape(image)` in figure turns both contracts red; restoring them (`shasum -a256 -c` clean) turns everything green. One sentence: **text nodes and attributes are two separate paths, and escaping happens before `format!`, never as a later patch.**

**Evidence:** `src/markdown/inline.rs` (escape branch, `starts_char_ref()`, 3 unit tests), `src/markdown/blocks.rs` (5 prop escapes), `src/illustrate.rs` (7 illustration string escapes). Corpus of 512 documents, old vs new: 35 changed visible text and **every one of them is content the old build had swallowed** (e.g. `Scans HTML <img src> local paths` used to render as `Scans HTML  local paths`); none got shorter; the real articles under `ready/` are unchanged. Suite 945 → 949 passed, coverage 79.21% → 79.25%.

### `&nbsp;` is not a free pass: the test is whether a node still holds a visible character

**Symptom:** after the previous fix ("every space becomes `&nbsp;`") shipped, code blocks in the published article **still** smeared into `letSome` / `ifmeta` — only the shape changed, from "the whole line sticks together" to "only spaces that sit between tokens are lost". The user's diagnosis turned out to be the decisive clue: the backend does not delete *space characters*, it deletes *text nodes that contain nothing but whitespace*.

**Root cause:** the previous fix swapped the character but **not the position**. In `<span>let</span>&nbsp;<span>Some</span>` that `&nbsp;` is still a standalone text node. It decodes to U+00A0, which some whitespace predicates treat as blank (not only ASCII `<= ' '`), so the node is dropped wholesale. **Changing the character does not change the node structure.** The criterion must be "does this text node still contain a visible character after trimming", never "did I use an entity".

**Fix:** `xcode_highlight` now ends with `absorb_blank_nodes()`, which moves the whitespace **into** the neighbouring token's `<span>`: `</span>&nbsp;` → `&nbsp;</span>`, so the node becomes `let\xa0` and still trims to `let`. Three edges are handled separately: (1) leading indentation is absorbed only when a token follows immediately, otherwise a `&#8203;` anchor is added — blindly absorbing turns an indented `--limit` into `--` plus two trailing spaces; (2) blank lines get `&#8203;&nbsp;`, otherwise their only node is blank, gets deleted, and the line collapses to zero height; (3) inter-word whitespace always moves into the preceding token, never into a new node.

**Prevention:** (1) unit test `code_lines_never_leave_a_blank_text_node` decodes each line and asserts it is not whitespace-only; (2) `word_spaces_survive_wechat_blank_node_deletion` **simulates the backend action** — blank out every whitespace-only text node while keeping the tags, then assert `let Some` / `if meta` / `else {` and the four-space indent survive. One sentence: **do not ask "did I use `&nbsp;`", ask "does this node hold a character someone can see".** The previous test only forbade `</span> <span`, and `</span>&nbsp;<span` slipped right past it — **a test aimed at the wrong signature can be fooled by any fix that merely changes the signature**.

**Evidence:** the 16-line regression sample (`let Some(meta) = meta else { continue };`, 8-space indent, leading `--limit`, blank line, Chinese comment) matches the source character for character after the simulated deletion, with `letSome` / `ifmeta` counts at 0; re-running it against the `v0.4.19` release asset (sha256 `a921c9a9…`) yields `white-space` = `normal` in all 16 places, 0 `<pre>`, 0 `</span>&nbsp;<span`. Suite 965 → 967 passed.

### Code blocks smeared into one line in the published article: a space is a character, not styling

**Symptom:** in the published article (2026-11-18, 22:20) the code blocks no longer stack vertically — they **smear horizontally**: `npx tsx simple-indexer.ts` renders as `npxtsxsimple-indexer.ts`, `Found 20 signatures` as `Found20signatures`, `--limit 40` as `--limit40`. Indentation survives (the line starts with `&nbsp;`) but every space between words is gone, so commands can no longer be copied and run. Side note: that article's markup still carries `white-space:pre-wrap`, so it was published with the old `v0.4.15` binary (`#217` never reached the user's installed build). Even on `#217` the spaces would still vanish — these are two separate defects.

**Root cause:** the previous round removed `white-space:pre-wrap` and moved to "one `<p>` per line, indentation via `&nbsp;`", but **the space between words was still a bare ASCII space** (`out.push(' ')` in `xcode_highlight`, kept at the time as "a break opportunity for the browser"). The WeChat backend deletes **text nodes that contain only whitespace**: grepping the published HTML, `</span> <span` appears **0** times while `&nbsp;` appears **428** times — bare spaces all dead, `&nbsp;` all alive. Syntax highlighting wraps every token in its own `<span>`, so every inter-word space sits *between elements*. Spaces *inside* one text node survive (prose keeps `课程的索引器 demo` and `20 条签名` intact), which is why only code blocks were hit.

**Fix:** the space branch of `xcode_highlight` dropped the "leave one real space" path — **every space becomes `&nbsp;`**, indentation and word gaps alike. The cost is that a code line has no soft wrap points, so an over-wide line can only be broken at the container edge by `overflow-wrap:break-word` (the wrapper `section` uses `overflow:hidden`, so without that break the text would be clipped). Fixed in the same function, while we were there: the catch-all branch `out.push(c)` never escaped punctuation, so a code block teaching HTML would emit a real `<div>` that the browser swallowed, and `</T>` would be parsed as an end tag eating the rest of the line — punctuation now goes through `&lt;` / `&gt;` / `&amp;` / `&quot;`.

**Prevention:** (1) unit test `code_block_spaces_survive_whitespace_stripping`: the output may contain no `</span> <span` at all, and the four-space indent plus `pure Rust` must still read correctly after "whitespace-only text nodes" are deleted; (2) process-level contract `code_block_words_survive_blank_node_stripping` uses `drop_blank_text_nodes()` to bake the backend's deletion into the test — the command block must still read `npx tsx simple-indexer.ts \`, with the two-space indent and `--limit 40 --reset` on the second line intact; (3) unit test `code_block_escapes_markup_so_examples_survive`: `<div>` / `</div>` may never leak as real tags. One sentence: **a space must be a character, not styling** — "I'll leave a space so the browser can wrap" is never sound on any path where an upstream re-serialises the DOM.

**Evidence:** `src/illustrate.rs` (space branch, punctuation escaping, 3 unit tests), `tests/render_syntax_contract.rs` (`drop_blank_text_nodes()` plus the new contract), `tests/directive_body_contract.rs` (`visible_text()` now decodes `&nbsp;`). The real article `2026-11-12-solana-indexer-lab.md` (17 code blocks / 110 lines) rendered with the new binary matches the source **line by line: 0 mismatches, 0 leaked `<`**. Suite 943 → 945 passed, coverage 79.18% → 79.21%.

**Article angle:** "Your code block survived the copy-paste and died in the sanitizer." Nobody deletes a space on purpose; a whitespace-only text node just looks like noise to a rewriter. Anything whose meaning lives in whitespace has to be encoded as a character before it crosses a boundary.

### Code blocks toppling over in the WeChat editor: `white-space` is styling, not structure

**Symptom:** The code block looks perfect in the local preview, but after pushing to the WeChat draft editor it becomes "one token per line" — `const` / `opts` / `:` / `any` / `=` / `{` / `limit` / `:` stacked vertically, the whole block falling over like dominoes. The block was ordinary top-level body text, no `:::` wrapper anywhere, so this was not the card renderer.

**Root cause:** code lines relied on `white-space:pre-wrap` to keep their spaces and indentation. Line breaks were already guaranteed by "one `<p>` per line", so `pre-wrap` was redundant — and worse, it turns **every newline in the HTML source** into a real line break. The WeChat editor re-serialises the HTML we hand it (pushing newlines between elements); those newlines land inside a `pre-wrap` element and become visible breaks, one before every `<span>`. Locally there is nothing to see, because the html is written by us with no newlines between elements.

**Fix:** code line layout is now carried by "structure plus explicit encoding" and no longer touches `white-space` at all: line breaks stay one `<p>` per line; indentation and runs of spaces become `&nbsp;` (`xcode_highlight`'s space branch already did most of this, it just used to leave one real space per run as a break opportunity); tabs are expanded to four hard spaces before rendering (previously `pre-wrap` preserved them, now we must); and the `white-space:pre-wrap` declaration is gone from the style string. `word-break:normal;overflow-wrap:break-word` stays, so long lines still break when they must.

**Prevention:** one sentence: **layout information must come from structure or from explicit encoding, never from an upstream keeping a CSS property** — we cannot control how the WeChat editor serialises HTML, only whether "this line is still correct if every style were rewritten". Tests sit in `illustrate::tests`: the output must contain no `white-space` (its presence means we are relying on it again), 4- and 8-space indentation must be `&nbsp;`, and no tab may reach the output. The "is this a code block?" marker moved from `white-space:pre-wrap` to `font-family:Menlo` — using a property we are about to delete as the test marker means every test silently stops testing when we delete it.

**Evidence:** `src/illustrate.rs` (`render_code_block` drops `white-space:pre-wrap`, expands tabs, plus two new unit tests); the code-block markers in `plain.rs` / `blocks.rs` / `tests/directive_body_contract.rs` now use `font-family:Menlo`. Suite 940 → 943 passed, coverage 79.13% → 79.18%. Two pieces of evidence that the visuals do not move: (1) one article rendered by both binaries produces html that is **byte-identical** once the `white-space:pre-wrap;` declaration is stripped from the old one; (2) both versions screenshotted at 420px in headless Chrome produce **screenshots whose sha256 matches**. Corpus regression across 512 documents: 415 differ only by that style declaration, 1 (a JSON sample with tab indentation) has its tabs expanded to `&nbsp;`, the other 96 are unchanged — zero content differences.

> Follow-up that same evening: this only fixed the vertical stacking. The spaces between words were still bare spaces, so the published article smeared into one line — see the entry above. Both have to hold at once.

**Article angle:** "Styling should be decoration, not the beam that carries the structure." The moment a piece of layout lives only in a CSS property, you have handed it to someone else: browsers, the WeChat editor, any HTML re-serialisation can take it away. In places where spaces *are* the semantics — code blocks above all — write the information into the characters themselves.

### A `:::` block's body is still Markdown: code blocks inside cards get boiled into one inline span

**Symptom:** Put a code sample inside `:::card` / `:::callout` / `:::intro` and the output contains no code block at all: three **empty inline-code spans** appear (each ```` ``` ```` fence line becomes one empty span), the language tag `ts` and the whole snippet are crammed into a single `family:monospace` `<span>`, and every line break and indent is left for the browser to guess. The same root cause ate two more places: two paragraphs separated by a **blank line** inside `:::intro` were merged into one line (that is exactly the lead of the real article `ready/gzh-tools-roundup-2026.md`), and `parse_blocks()` scanned the raw string with `find("\n:::")`, so a line inside a code block teaching `:::callout` was treated as a real directive and cut the code block in half — the template examples in the repo's own `README.md` / `README_zh.md` / `docs/USER_GUIDE.md` were rendered as real cards with shattered fences.

**Root cause:** every "body" renderer in `src/markdown/blocks.rs` (`render_intro` / `render_callout` / `render_summary` / `render_letter_card` / `render_scene_card` / `render_closing_card` / `render_pull_quote` / `render_generic_fence`) called `inline_md(body)`: a whole **block-level** body handed to an **inline** renderer. The inline renderer only knows backticks, asterisks, and links, so a fence degrades into inline code and every block construct leaks literally; in HTML a `\n` is just whitespace, so two paragraphs naturally collapse into one line. `parse_blocks()` was a **string scan** that had no idea code fences exist — the same disease as the `~~~` bug fixed a round earlier: one syntax, only one place knows about it.

**Fix:** `blocks.rs` grew `body_html()`: the body first goes through `plain::body_needs_blocks()` (the same predicate the blockquote already uses), and when it contains a fence, list, heading, table, quote, or blank line it is handed back to `render_markdown_segment()`; only genuinely plain text keeps the old `inline_md` path, so the visual of `:::summary` does not change wholesale. When the body becomes block-level the wrapper follows: `body_wrapper()` emits `<section>` for block content and `<p>` for plain text (block elements nested inside `<p>` get auto-closed by the browser and split once more in WeChat). `parse_blocks()` was rewritten as a **line scanner that tracks fence state**: `fence_open()` / `fence_closes()` are now shared from `plain`, so a `:::` inside a code block — or inside a block body — is no longer read as a directive or a closer; closing requires the whole line to be a fence (trailing whitespace and extra colons allowed), so `::: followed by prose` does not close anything.

**Prevention:** two rules: (1) **a container's body is still Markdown** — for every "body" (`>` quotes, `:::` directives, any future container) ask "who catches block syntax?"; if nobody can answer, a `render_markdown_segment` call is missing; (2) **structural decisions must not scan raw strings** — `find("\n:::")` knows nothing about fences, inline code, or escaping, and any "does this line carry syntax?" question needs a line-by-line state machine. Tests sit on two layers: three unit tests in `blocks.rs` (a code fence inside five different `:::` blocks must stay a code block; block bodies must not be wrapped in `<p>`; plain bodies keep the old layout — the last one is the reverse guard so a bug fix does not restyle every card) and three in `parser.rs` (a `:::` inside a fence does not split the document; a fence inside a block body keeps its lines; a closer with trailing text does not close). `tests/directive_body_contract.rs` adds four real-binary contracts. Two controlled mutations: `body_html` back to `inline_md` → 4 red across unit and process layers; fence state disabled → 3 red.

**Evidence:** `src/markdown/blocks.rs` (`body_html` / `body_wrapper` plus three new unit tests), `src/markdown/parser.rs` (`parse_blocks` rewritten as a fence-aware line scanner plus three tests), `src/markdown/plain.rs` (`fence_open` / `fence_closes` exposed as `pub(super)`, `quote_body_needs_blocks` renamed to `body_needs_blocks` and shared), plus the new `tests/directive_body_contract.rs`. Real binary: a ```ts block inside `:::card` now renders one monospace `<p>` per line with no `ts` in the visible text; a sample containing a `:::` line keeps all three lines; a code block inside `:::summary` is wrapped in `<section>`. Both binaries rendered 512 real documents; six changed, all correct: README / README_zh / USER_GUIDE / LAYOUT_RECIPES_ZH / MOONPUB_INTRO_ARTICLE_ZH stopped executing the `:::` examples that live inside fences, and `ready/gzh-tools-roundup-2026.md` stopped merging the two paragraphs of its `:::intro`. Suite 930 → 940 passed, coverage 78.85% → 79.13%.

**Article angle:** "A container's body is still a document." We spent several rounds giving the top-level body a block parser, then forgot that every container (quote, card, directive) grows its own little body — a container is not just padding and colors, it also has to decide who parses its content. Forget that and you end up raising a second, half-crippled Markdown dialect inside every container.

### One file, two spellings: the path the tool prints cannot be pasted back into its own command

**Symptom:** Walking the first-run path in a temp workspace, `moonpub new 测试文章` printed:

```
created
  /private/tmp/pathtest/Articles/drafts/测试文章.md
next: moonpub render /private/tmp/pathtest/Articles/drafts/测试文章.md
```

and the very next `moonpub status` printed `Articles/drafts/测试文章.md`. Same file, two strings one second apart. `render` / `check` / `preview` / `preflight` / `push` all followed the first convention (even `check`'s `next:` line was absolute); only `status` / `workspace` used the second — so `status --json`'s `next_command` was a pasteable relative path while `check --json`'s was absolute. Two entry points of the same command, two different strings. macOS adds a second layer: absolute paths go through symlink resolution, so the user types `/tmp/x` and the tool prints `/private/tmp/x`; the strings never match, and `diff`, `grep`, and eyeballing all stop working.

**Root cause:** Printing was **scattered** — every command handed its `PathBuf` to `.display()` and nobody owned the rule "one file, one spelling". Absolute paths appeared only because the workspace root happens to be absolute (`current_dir()`'s result); `status` printed relative paths only because it held bare filenames and prefixed them by hand. Both conventions were accidents of convenience; neither was a decision.

**Fix:** Added `src/human_path.rs::display_path(root, path)` as the single exit: a path inside the workspace prints in **workspace-relative form** (`Articles/drafts/x.md`, character-for-character what `status` lists); a path outside it keeps its absolute form (explicitly rendering someone else's file is legitimate, and truncating it into `../../` only makes it harder to verify); the workspace root itself never prints as an empty string, and `/tmp/demo2` is not mistaken for a child of `/tmp/demo` (`strip_prefix` compares path components, not string prefixes). Separators are normalised to `/`: `Path::display` emits backslashes on Windows, so the same command looked different on the two platforms while running fine on both. `new` / `render` / `check` (text and JSON) / `preview` (text and JSON) / `preflight` (text and JSON) / `push`'s `moved to` / `cover` / `humanize` now all print through it — no decision logic changed, only `root` was threaded to the print sites.

**Prevention:** Two sentences: (1) **a token the tool prints must be a legal argument to the tool itself** (the path from `new` pastes straight into `check`); (2) **one file may have exactly one spelling across every command and every mode** (text, JSON, listings, `next:` guidance share it). The tests sit at the process boundary (`tests/workspace_path_contract.rs`): `new`'s path carries no workspace prefix and runs through `check`; `render` / `check` / `status` / `preflight` print the identical string for one article; `status`'s `next:` splits into an argv that runs and still names the same string; paths outside the workspace stay absolute. The controlled mutation was to make `display_path` return `path.display()` unconditionally — unit, module, and process layers all went red together (12 tests), which is how we know the convention is not propped up by a single test.

**Evidence:** `src/human_path.rs` (`display_path` plus six boundary tests), `src/draft.rs` / `src/render.rs` / `src/bundle.rs` (`report(root)`, `next_step(root)`) / `src/preview.rs` (`next_step_after_preview(root, …)`) / `src/push.rs` / `src/app_article_commands.rs` / `src/protocol/check.rs` (`check_json` / `preflight_text` / `preflight_json` / `preview_json` gained `root`), plus four real-binary contracts in `tests/workspace_path_contract.rs`. Real binary: `new` → `status` → `check` all print the same `Articles/drafts/测试文章.md`, and the paths and `next_command` in `check --json` / `preview --json` / `status --json` agree. Both binaries rendered 512 real documents (every `.md` in the repo); the HTML is byte-identical — this change touches printing, not rendering. Suite 920 → 930 passed.

**Article angle:** "Printing a path" looks like formatting but is an interface contract: the string a user copies is the handshake between the tool and itself. Give one thing two spellings and the user has to do mental arithmetic — and a beginner's budget for that is zero.

### "How far has this gone?" is answered by the `.media_id` on disk, never by the status log

**Symptom:** An article had been pushed to the WeChat draft box (`Articles/drafts/x.media_id` held the id), yet `moonpub status` listed it as `[rendered]` — indistinguishable from "rendered, never pushed". After reviewing the local preview, `moonpub preview` still ended with `next: moonpub push … --render`. A user who wanted to fix a typo needed `update-draft`, and that id is generated by the tool; nothing else on screen could supply it, so the only route was to `cat` a hidden file.

**Root cause:** Three layers stacked. (1) `status` never read `.media_id` for the `drafts` stage — only `ready` / `published` used it as a status fallback. (2) The media_id reader and the bundle's own check existed separately, the former wrapped in `Result`, which made it look like it was "only read when needed". (3) `preview`'s next step was a hardcoded string, and its `--json` sibling built a second `push` string independently. The real mistake was picking the wrong source of truth: `.moonpub/status.jsonl` is an **append** log — `push` appends `ready + media_id`, then the user fixes a typo and `render` appends `rendered + empty`. Reading only the last line actively forgets that the article is already pushed. `<slug>.media_id` is a **file**, and a file is the one record that cannot be invalidated by "some other command ran afterwards".

**Fix:** `StatusFileEntry` gained `media_id`, read from `<slug>.media_id` for every stage; the read collapsed into `crate::bundle::read_media_id()`, shared by `status`, `preview` and `bundle`. The text listing appends `media_id: <id>` after the status tag, still after the path, so "the first whitespace-delimited token is a valid argument" holds. `next_action()` got the missing fourth rung, aligned rung-by-rung with `ArticleBundle::next_step()`: no html → `render`; html but no media_id → `check`; media_id present → `preview`. `preview`'s text and `--json` surfaces now share `next_step_after_preview()`, which says `update-draft` instead of `push` once the article is pushed. Per-file JSON collapsed into one `file_json()` so `status` and `workspace` stop maintaining two copies.

**Prevention:** "How far has this bundle gone" is **file state**, not **operation history**; anything that answers it from an append-only log goes wrong the moment a different command runs after `push`. The rule now in `AGENTS.md`: `status` / `preview` / `check` read the file, never the log. Second layer — "is there an id" and "what is the id" must use one definition: `exists()` counts an empty file as "pushed", and a failed `fs::write` leaves exactly such a file; feeding that empty string to `delete-draft ""` produces an error with no relation to the real cause, so `read_media_id()` requires the trimmed content to be non-empty. Third layer — **one command's text and JSON must share the decision**: the time `preview --json` built its own string, the human saw `update-draft` while the script got `push`, and the script is the end nobody watches. Never suggest `push` for an already-pushed article: `push` is not an update — it creates a new draft, deletes the old one and changes the `media_id`, staling both the copied id and the draft link in the WeChat backend.

**Evidence:** `read_media_id_file()` / `read_media_id()` / `ArticleBundle::media_id()` in `src/bundle.rs`; `StatusFileEntry::media_id` and the completed `next_action()` ladder in `src/status.rs`; `PreviewNext` / `next_step_after_preview()` in `src/preview.rs`; `file_json()` / `stage_json()` in `src/protocol/workspace.rs`. Tests: 3 new unit tests (a pushed draft prints its id and points at `preview`; an empty `.media_id` does not count as pushed; `preview` points at `update-draft` once pushed) plus 5 process-level contracts in the new `tests/status_contract.rs` (text carries the id, JSON carries the id, empty file is not a push, `preview` text points at `update-draft`, and `preview` text and `--json` must produce a byte-identical `next_command`). Three controlled mutations each hit precisely: dropping the media_id read in `status` → 2 contracts + 1 unit test red; ignoring media_id in `next_action` → 2 contracts + 1 unit test red (the empty-id test stayed green, proving it does not depend on that branch); forcing `preview` to always suggest `push` → 1 contract + 1 unit test red; restoring the hand-built `push` in `preview --json` → only the text/JSON agreement test went red. All shasums verified with `shasum -a 256 -c` after restoring.

### When "one line = one block", a single paragraph prints as three

**Symptom:** A paragraph was written across three lines — the most natural way Chinese writing looks, one clause per line. In the rendered article those three lines became three separate paragraphs with a 1.5em gap between each: the author wrote one paragraph, the reader saw three short sentences. The subtler consequence: inline code, bold and links that wrapped onto the next line were all cut in half, each half seeing an odd number of markers, so a span of inline code broken by a newline printed its backticks literally.

**Root cause:** The block parser treated *a line* as the only unit of *a block*. The paragraph branch received a line and immediately called `render_p(line)`; there was no notion of "does the previous line still belong to this paragraph". A blank line does split paragraphs, but between non-blank lines there is nothing to split on — in CommonMark that is a soft break, still the same paragraph. Same disease as the list-continuation bug: the line had no owner, so the downstream stage claimed it as a new block.

**Fix:** Body lines accumulate in `paragraph_buf` and are emitted once by `flush_paragraph()`; lines are joined with `<br>`, not a space (in Chinese writing a line break carries meaning — "symptom / cause / fix" written on three lines becomes mush when flattened). The critical detail is **join first, render second**: `render_paragraph_lines()` joins with `\n`, runs inline rendering, then replaces `\n` with `<br>`. Rendering line by line makes cross-line inline syntax see an odd number of markers.

**Prevention:** The paragraph buffer is the easiest buffer to forget to flush, and a missed flush shows up as wrong content order — visible to the reader, almost impossible to attribute. Write the rule as "every block branch must flush before emitting on the line it recognised", and distinguish two flushes: fences/dividers/headings/images/blank lines/end-of-document use the full `flush_blocks!()`; **table continuations, list items and checklist items may only use `flush_prose!()` + `end_blockquote!()`** — a full flush prints the header or item that was just pushed, splitting one table or one list into multiple `<section>` elements. (That exact mistake happened in the first cut of this fix and was caught by a mixed-content test.) Test the boundaries: a three-line paragraph must yield one `<p>` with two `<br>`, a blank line must yield two, cross-line inline syntax must close, and paragraph/heading/list/table order must hold.

**Evidence:** `paragraph_buf` / `flush_paragraph()` / `render_paragraph_lines()` in `src/markdown/plain.rs` with 4 new unit tests; 2 new process-level contracts in `tests/render_syntax_contract.rs`; a controlled mutation (flush immediately in the paragraph branch, restoring "one `<p>` per line") turned exactly 3 tests red (2 unit + 1 contract), and `shasum -a 256 -c` matched after restoring; 80 real documents rendered before and after showed 13 changed files, paragraph counts dropped sharply (one file 71 → 18), list and checklist counts identical, visible text byte-identical in 11 of them — the other 2 each lost one backtick, which is the cross-line inline code closing correctly.

**Article angle:** "A line" in Markdown can be a whole paragraph or one line of a paragraph. A renderer that conflates the two turns the most natural way of writing into the least natural output.

### A line with no owner gets claimed as somebody else's block

**Symptom:** An article wrote `- output ends with next:` and then two indented lines of explanation underneath — the most common way Chinese list writing looks, because a long item simply wraps. In the rendered article those two lines became top-level paragraphs with an extra paragraph gap: the list held a single line and the rest read like a separate statement. A subtler consequence: a span of inline code broken across the line ending (`intake photos --draft` then `photos/day1`) was split into two paragraphs, each half seeing an odd number of backticks, so the backticks printed literally in the reader-visible text.

**Root cause:** `parse_list_item()` only matched lines starting with a dash, an asterisk or an ordinal, each followed by one space. An indented continuation line matches none of those prefixes, so it fell into the `else if !list_buf.is_empty()` branch: the list was flushed and closed, the line kept walking down the parser and landed in the paragraph branch. It was never "unsupported" — it was **classified as a different block**. The blast radius in real material is large: the repository's own `docs/` contains 116 instances of this shape.

**Fix:** Added `is_list_continuation(line, prev_blank)`, which appends an indented line to the previous item's text; bullets, ordered lists and checklists share the one rule. Three conditions, all required: at least 2 columns of indent (content after a dash starts at column 2, so aligning there is what makes the line "part of this item"), the previous line is not blank (a blank line is the author's explicit paragraph break — joining across it means rewriting their structure), and the line is not itself another block marker (`#` / `>` / `|` / a divider degrades to body text instead of being swallowed). The check must run **after** `parse_list_item()`, otherwise a nested item indented by two columns is eaten by its parent.

**Prevention:** For every block syntax, ask "who catches its *other* legal spelling?" List items have continuation lines, headings have setext, fences have tildes, tables have unpiped borders — these are secondary spellings of the same block, and accepting only the primary spelling leaves the parser an opening to classify them as something else. Test the boundaries, not the output: two-space indent stays inside the list container, an indent after a blank line stays a separate paragraph, one space does not qualify, an indented `#`/`>`/`|`/divider is not swallowed, and a nested item after a continuation keeps its depth.

**Evidence:** `is_list_continuation()` in `src/markdown/plain.rs` with 7 new unit tests; 2 new process-level contracts in `tests/render_syntax_contract.rs` (the real binary must place the continuation text inside the list `<section>`, and an indented line after a blank line must land outside it); a controlled mutation (force `is_list_continuation` to return false) turned exactly 5 tests red (4 unit + 1 contract), and `shasum -a 256 -c` matched after restoring; 80 real documents rendered before and after showed 10 changed files — paragraph counts all dropped, list-item counts identical, visible text byte-identical except one file, and that one is the case where inline code spanning a line break now closes correctly instead of leaking two backticks.

**Article angle:** "Unsupported" and "classified as something else" are different failures. The first is visible to the author; the second only shows up as a wrong-looking result — and it tends to be more widespread, because it routes one block through another block's parser.

### When one syntax has two recognisers, the missing one turns code into prose

**Symptom:** An article wrote a code sample inside a tilde fence (`~~~bash`). In the rendered article the fence line itself printed as a paragraph, and the code inside was re-laid-out as ordinary Markdown: `# this is a comment` became a sub-heading, `echo "**do not bold**"` became bold, `if [ a == b ]` was eaten by `==highlight==` into a `<mark>` with the quotes shifted, `- not a list` became a bullet, `| a | b |` went into the table pipeline. The worse variant: a `~~~markdown` block teaching link reference definitions lost its `[docs]: https://example.com/x` line entirely — it was lifted out as a real definition, and a reference outside the fence then pointed at the address that was supposed to be *inside* the sample.

**Root cause:** Three recognisers for "what is a code fence" lived in the same repo: `article.rs::fence_marker` accepted both backticks and tildes, `quote_body_needs_blocks` accepted both, but the two entry points that actually render code (the block parser and `collect_link_refs`) accepted backticks only. So `~~~` counted as a fence on the "skip code while extracting the title" path and as prose on the "render" and "extract link definitions" paths. **Scattered rules guarantee one of them lags**, and the lagging one was the one that produces the output file.

**Fix:** Fence recognition collapsed into a pair of functions — `fence_open()` returns (fence char, run length, info string), `fence_closes()` requires the same char, a line of nothing but that char, and a run at least as long as the opening one. The block parser, `collect_link_refs` and `quote_body_needs_blocks` all share them. The length rule is not pedantry: a longer tilde run inside a `~~~` fence, or a three-backtick line inside a four-backtick fence, is *content*; treating it as a closer ends the outer block early and swallows the rest of the article into code.

**Prevention:** Once a syntax has two or more recognisers, merge them into one function before fixing anything; new fence characters change only that place. Tests must assert "what is inside the fence is not re-interpreted by another syntax", not "the fence renders as `<pre>`" — the former catches mutation, the latter does not. Unit tests cover four mismatches (shorter than opening, different character, longer tilde run inside, backtick fence inside), and the process-level contract asserts no `~~~` reaches visible text and the code sample is verbatim.

**Evidence:** `fence_open` / `fence_closes` in `src/markdown/plain.rs` plus four new unit tests; two new contracts in `tests/render_syntax_contract.rs`; a controlled mutation (backticks only) turned exactly four tests red and `shasum -a 256 -c` matched after restoring; 60 real documents rendered with both binaries changed in only two files, both of which nest a three-backtick example inside a four-backtick fence — the old code closed the outer block at the inner fence and swallowed 41 paragraphs of prose into code, the new code nests them per CommonMark.

**Article angle:** Writing the same rule twice buys you a bug that only fires on one path — and that path is usually the one users see.

### An unrecognised block syntax does not "fall back to text" — it mutates

**Symptom:** An article used a setext heading (`社区营造` followed by a line of `====`). In preview and in the pushed draft the heading degraded to a paragraph, followed by `<mark style="…"></mark><mark style="…"></mark>==` debris. Readers saw leftover tags; the author saw their heading disappear.

**Root cause:** The block parser knew only the ATX half of the heading spellings. The `====` line matched no block branch and fell through to the inline renderer, which supports `==highlight==`, so it turned the whole run of `=` into a pair of empty highlight tags plus a stray `==`. That is the real cost: unsupported syntax kept literally is merely "not rendered", but unsupported syntax handed to a different layer gets **re-interpreted**, and the output ends up containing markup the author never wrote. The same root cause has another exit: `~~~` fences (only ``` is recognised), where the code inside is parsed as prose.

**Fix:** One line of lookahead before the paragraph branch: if the next line is an all-`=` line (at column 0 or indented by at most three spaces), render the current line as a setext heading at the same level as `##` and consume the underline (`is_setext_underline()` / `render_setext_heading()`). `---` is deliberately **not** accepted: in real articles that line is far more often a divider or a frontmatter example boundary, so accepting it would promote "one `key: value` line plus a rule" into a sub-heading; leaving it as "paragraph + divider" misleads nobody. `=` has no second identity, so it is the only marker accepted.

**Prevention:** Every new block-syntax recogniser must answer "which layer does this line fall into when it is *not* recognised?". Any block marker that can fall into the inline renderer (`=`, `*`, `_`, `~`, `|`) needs a "stays literal" test. Block-parser unit tests must cover four boundaries — matches when adjacent, does not match across a blank line, does not match past the indent limit, does not match mixed symbols — not just the happy path. The acceptance surface for rendering fixes is the **produced file**: `tests/render_syntax_contract.rs` renders with the real binary and asserts a heading tag exists while no `=` survives in visible text.

**Evidence:** `is_setext_underline` / `render_setext_heading` in `src/markdown/plain.rs` plus six unit tests; two process-level contracts in `tests/render_syntax_contract.rs`; 60 real documents (`docs/`, `output/`, `ready/`, `writing/`) rendered with the pre-fix and post-fix binaries — `diff -rq` showed only the status log differing and zero HTML differences. Scoping the fix to `=` is what kept that zero-regression line.

**Article angle:** "Unsupported" is safe; "mutated" is dangerous. Every syntax a parser learns must also state where the line goes when it is not recognised.

### Backend configuration must verify persisted state, not click logs

**Symptom:** `moonpub ship` logs showed original / reward / comments as successful, but the actual WeChat backend draft remained "not declared / disabled / comments disabled". All settings were not saved, and multiple runs falsely reported success.

**Root cause:** Four layers stacked together: the steps only verified that clicks were issued, not that settings took effect; WeChat editor setting rows contain both inactive and active blocks, where the inactive block is `display:none` but still present in the DOM, so reading `textContent` treated hidden template text as real state; global text-based confirmation clicks such as `cdp_click_exact_last("OK")` could hit another unclosed dialog; the browser closed without explicitly saving the draft, so settings only lived in page memory.

**Fix:** All steps now follow "state first + scoped container + explicit save + reload verification". Original statement uses `.claim__original-dialog`; author input readiness is verified before confirming, and validation failures retry. Reward is enabled from the visible `.js_reward_open` setting row after original statement. Comments use `input.js_interaction_setting.checked` as real state. `step_baocun` explicitly clicks "save as draft"; `step_fucha` reloads the editor and verifies persisted state by visibility and selected values.

**Prevention:** Any WeChat editor state read must check visibility (`offsetParent`, and `getClientRects().length` for fixed elements). Confirmation clicks must be scoped to a concrete dialog. Picker dialogs must use coordinates gathered by script and trusted CDP mouse events, because synthetic JS clicks (`isTrusted=false`) are ignored. Close residual dialogs before saving. Reloaded page state is the only success standard. New editor automation steps need script tests with visibility assertions.

**Evidence:** `src/publish_steps.rs` steps and `ORIGINAL_*` / `ZANSHANG_*` / `COMMENT_*` / `FUCHA_SCRIPT` tests; 2026-07-26 real Walden `ship` run for draft `100011146` verified original, reward, creation source, comments, and mobile preview `ret=0`.

**Article angle:** Between "clicked" and "done" sit three verification layers. In automation, the expensive failure is not missing a click; it is clicking the wrong thing and believing it worked.

### Cookie-mode cover must be uploaded explicitly and passed to draft creation

**Symptom:** In cookie mode, `moonpub ship` created WeChat drafts with an empty cover. A book-review article with remote `cover:` frontmatter failed cover upload with `ret=200002`.

**Root cause:** `download_cover` saved JPEG bytes as `.cover.png`, so the declared MIME did not match the actual bytes and WeChat rejected it. `create_draft_cookie` ignored the upload result and always picked the first `https://` image in the body. After the footer QR code moved to a data URI, the body no longer had any `https://` images, so cover fallback disappeared.

**Fix:** Downloaded covers now choose the real extension from Content-Type / magic bytes (`.jpg` / `.png`) and clear stale `<slug>.cover.*` files before writing. `push_article_cookie` uploads the cover and passes the CDN URL and file id explicitly into `create_draft_cookie` as `cdn_url0` / `fileid0`; only when empty does it fall back to the first body `https://` image. Cookie mode also decodes embedded data URI images, uploads them to CDN, and replaces them so the WeChat editor does not strip them.

**Additional verified detail (2026-07-25):** `filetransfer?action=upload_material&type=image` response shape depends on query parameters. Without `writetype=doublewrite&groupid=1`, it only returns numeric file id; with them, it returns a real `cdn_url`. Synthesizing CDN URLs from file ids is rejected by `operate_appmsg`. After successful cover upload, use both channels: `cdn_url` for `cdn_url0`, numeric file id for `fileid0`.

**Prevention:** Image file extensions must reflect real bytes. Draft cover source must be passed explicitly, not depend on the body accidentally having an `https://` image. New image embedding methods (data URI, local paths) need CDN upload regression tests. Do not guess WeChat API return values from field names; inspect real responses first.

**Evidence:** `src/ship.rs` `download_cover` / `cover_extension`, `src/push_browser.rs` `UploadOutcome` / `decode_data_uri` regression tests, and a 2026-07-25 real Walden `ship` run with cover upload, QR code CDN replacement, draft `100011016`, and mobile preview `ret=0`.

**Article angle:** "Works" and "works correctly" are separated by implicit assumptions. "Cover comes from the first body image" silently fails once body structure changes.

### Browser handle must stay alive until QR scan and session save finish

**Symptom:** `moonpub login` opened the browser and quickly failed with `oneshot canceled`, so QR scan could not complete.

**Root cause:** The login flow released the underlying `Browser` handle too early, canceling the CDP session.

**Fix:** The login path now keeps an active `Browser` alive until QR scan completes and session is saved.

**Prevention:** Login and QR recovery paths must not pass only `Page`. Browser lifecycle changes must be covered by resource-lifetime regression.

**Evidence:** `src/cdp.rs` and `PROGRESS.md` 2026-06-30 record.

**Article angle:** Browser automation failures are not always about QR scan. Sometimes they are client-side resource lifetime bugs.

### Headless mode must not wait for invisible QR codes

**Symptom:** After login state expired, background automation waited for a QR code in headless mode and timed out; the user had no visible window to operate.

**Root cause:** Reusable-session background flows and interactive QR login flows were mixed into one path.

**Fix:** In headless mode, failed session restore now fails fast and asks the user to run `moonpub login` or `moonpub configure --headed`. Temporary profile mode explicitly states that it cannot reuse persistent session.

**Prevention:** Invisible flows must not wait for human input. New browser entries must clearly define `headed`, persistent profile, and temporary profile login semantics.

**Evidence:** `src/cdp.rs` `headless_login_required_message` and `docs/USER_GUIDE.md`.

### A locked persistent Chrome profile is not token expiration

**Symptom:** Chrome launch errors containing `SingletonLock` or `ProcessSingleton` were easy to misread as expired WeChat login state.

**Root cause:** Another MoonPub automation Chrome window was using the persistent profile.

**Fix:** Low-level errors are now collapsed into a readable message: close the existing automation window, or use `--temporary-profile` for an isolated one-off check.

**Prevention:** Browser launch failures must first distinguish profile locks, login-state expiration, and backend page changes. Do not ask users to re-login for every browser failure.

**Evidence:** `src/cdp.rs` `browser_launch_error_message` regression test and `PROGRESS.md` 2026-07-04 record.

**Additional detail:** A 2026-07-14 real regression confirmed that when another `moonpub test-yulan --headed` process occupies the same profile, Chrome may first appear as a WebSocket address timeout instead of returning `SingletonLock`. Check for existing MoonPub automation processes and Chrome using the same profile before blaming WeChat login state.

### WeChat editor selectors should prefer DOM structure, not visible text

**Symptom:** Creation-source text could be split by icons and whitespace, so `textContent` option matching failed or misread state intermittently.

**Root cause:** The WeChat editor is a dynamic page; visible copy is not a stable DOM interface.

**Fix:** Open the entry with `.js_claim_source_desc`, select the stable source with `input[type="radio"][value="4"]`, and verify through `.js_claim_source_selected`.

**Prevention:** Automation should prefer DOM structure, class names, and input values. Setting steps should soft-fail without blocking WeChat API draft creation.

**Evidence:** `src/publish_steps.rs`, `docs/BROWSER_AUTOMATION.md`, and `PROGRESS.md` 2026-07-03 real regression.

**Article angle:** Why MoonPub does not promise browser automation will be fully automatic forever.

### Config assets and article-relative assets need different resolution bases

**Symptom:** Footer QR code existed in local HTML, but failed to upload or display after pushing to WeChat.

**Root cause:** Article-relative images should resolve from the article directory, while config assets such as `qrcode` / `cover` should resolve from the articles root. These rules were mixed.

**Fix:** Rendering now resolves config assets to absolute paths using articles root before passing them to upload logic.

**Prevention:** Config paths should be normalized at the config boundary. New config assets need articles-root resolution tests and must not reuse article-directory rules.

**Evidence:** `AGENTS.md` config-asset constraint and historical notes in `CLAUDE.md`.

### WeChat cover media_id is not permanent cache

**Symptom:** A once-valid permanent material was deleted, and the stale `thumb_media_id` caused `ship` push failure.

**Root cause:** WeChat material lifecycle is not controlled by local config; cached media ids can expire.

**Fix:** `ship` now generates and uploads a cover PNG each run and uses the returned media id. Config value is only a final fallback.

**Prevention:** External platform resource identifiers are invalidatable cache. Write down refresh strategy and failure recovery.

**Evidence:** `src/ship.rs` cover upload behavior, regression tests, and historical notes in `CLAUDE.md`.

### WeChat IP whitelist failures can come from public egress drift

**Symptom:** The user added the previous `40164 invalid ip` address to the whitelist, but a later retry still failed and the error's `current IP` had changed.

**Root cause:** WeChat validates the actual public egress IP of each API request. Home broadband, mobile networks, and rotating proxies can change egress. A historical IP being whitelisted does not mean the current request still uses it.

**Fix:** `40164` errors now print the current IP and clearly suggest disabling rotating proxies or using stable egress before updating the whitelist. User docs explain that accumulating historical IPs does not fix drift.

**Prevention:** During debugging, verify both default environment and no-proxy requests. If both have the same IP but it differs from history, treat it as public egress change. Do not conflate browser login state, token expiration, and API IP whitelist.

**Evidence:** On 2026-07-15 the same `update-draft` returned `1.80.191.120`, `1.80.191.168`, and `117.35.173.2`; current default and direct egress were both `117.35.173.2`. `src/wechat.rs` `errcode_detected` regression covers stable-egress hint.

### Cover screenshot success cannot be inferred from stale PNG existence

**Symptom:** After changing the cover template and rerunning `cover --screenshot`, the command reported success but the visible cover was still the previous PNG.

**Root cause:** Chrome headless did not reliably overwrite an existing screenshot file. The screenshot flow did not remove the old PNG first and only checked whether the path existed after running, so stale output was misread as the current result.

**Fix:** Chrome now writes to a temporary screenshot in the same directory first; after confirming the temp file was generated, it replaces the official PNG. When Chrome fails, the previous usable cover is preserved.

**Prevention:** Fixed-path generated artifacts must not prove current success only by "file exists after command". Overwrite generation should write an independent temporary artifact first, then replace the final file.

**Evidence:** `src/cover.rs` `completed_capture_replaces_stale_png_and_removes_temp_file` regression test and 2026-07-15 real `workflow` cover PNG verification.

### A complete cover source does not mean WeChat crop is usable

**Symptom:** The 900x500 original image had complete flow cards and phone mockup, but center-cropping to WeChat landscape ratio cut off bottom material entries and the phone confirmation button. Square thumbnails also cut off the left-aligned title.

**Root cause:** Cover design was laid out only for the generated canvas, without a shared safe area for WeChat list landscape image, share square, and small thumbnails.

**Fix:** The `workflow` cover title became centered, flow component height was compressed, and the three input types, MoonPub core, and phone confirmation were moved into the central landscape safe area.

**Prevention:** After changing covers, check at least the 900x500 original, central 900x383 crop, 500x500 square crop, and around 360px thumbnail. Do not use original screenshot as the only visual acceptance evidence.

**Evidence:** 2026-07-15 crop checks for `docs/MOONPUB_INTRO_ARTICLE_ZH.cover.png` and the cover visual QA constraint in `AGENTS.md`.

### Mobile WeChat typography must not depend on multi-column tables

**Symptom:** The project-introduction article looked correct in local preview, but mobile WeChat preview compressed two-column blocks such as `meta-strip` into narrow table columns. Dates, mood, and descriptive text squeezed together; reference links looked like ordinary body text.

**Root cause:** WeChat mobile readable width is small. Horizontal `<table>` works for light alignment, not body-level information blocks. Bare URLs were previously rendered as normal text with little visual weight.

**Fix:** `summary`, `callout`, `steps`, `checklist`, `key-points`, `photo-grid`, `meta-strip`, normal Markdown tables, and illustration blocks such as `comparison` / `concept-card` / `timeline` now render as vertical cards or information blocks. Explicit Markdown links and bare URLs render as bold highlighted links.

**Prevention:** Markdown / block / illustration regressions assert that mobile-sensitive blocks no longer output `<table>`. Link highlighting has regression coverage. New typography blocks should be checked with real mobile preview or at least narrow screenshots, not just desktop local HTML.

**Evidence:** Regression tests in `src/markdown/blocks.rs`, `src/markdown/plain.rs`, `src/markdown/inline.rs`, and `src/illustrate.rs`; 2026-07-15 real article rerender passed `layout-audit`, updated the same WeChat draft, and completed original / reward / comments / creation source / mobile preview.

### Input material is conservative by default; visual analysis needs explicit confirmation

**Symptom:** If photo organization uploaded pixels by default or treated model descriptions as facts, it would risk privacy leakage and content distortion.

**Root cause:** File metadata organization and visible image information recognition are two different data processing boundaries.

**Fix:** The default photo path only processes path, file name, size, and modification time. Only explicit `--analyze-images` sends a limited number of images to OpenAI. Results are written back to Inbox and marked as requiring human verification.

**Prevention:** Any capability expanding material data scope needs an independent confirmation step. AI photo descriptions must not automatically advance to WeChat draft as verified facts.

**Evidence:** `src/ai_workflow.rs`, `src/intake/photos.rs`, and `docs/FIRST_RUN_WALKTHROUGH_ZH.md`.

**Article angle:** How to turn photos into life records without letting tools invent the life.

### Release smoke must run in case-sensitive paths

**Symptom:** The first `v0.4.2` tag workflow's Linux ARM64 archive smoke failed because `Archive-Smoke.md` and `archive-smoke.md` did not match.

**Root cause:** The smoke title and later command path had different casing, relying on a case-insensitive environment.

**Fix:** The workflow now consistently uses lowercase kebab-case titles and runs credential-free smoke on the officially downloaded macOS ARM64 asset.

**Prevention:** After generating files, reuse actual paths or normalize names consistently. Treat case-sensitive environments as the release-smoke baseline.

**Evidence:** `.github/workflows/release.yml`, `docs/RELEASE_GATE_v0.4.2_ZH.md`, and `PROGRESS.md` 2026-07-13 record.

### Draft titles with spaces need normalized matching in automation

**Symptom:** In cookie mode, `ship` pushed successfully and media id was generated, but `auto_configure` reported `draft title not found`. Debug output showed the target draft first in the list.

**Root cause:** The ASCII space in the draft title was rendered by WeChat as non-breaking or multiple whitespace characters. Raw `card.innerText.includes(targetTitle)` failed. Debug output normalized `\s+` to a single space, so it looked like it should have matched.

**Fix:** `setup_editor_for_title` normalizes whitespace on both card text and target title using `replace(/\s+/g, ' ').trim()` before `includes` comparison (`src/cdp.rs`).

**Prevention:** Normalize DOM text whitespace before matching. Debug-output normalization must match decision logic, otherwise logs hide real mismatches.

**Evidence:** 2026-07-26 real `ship` full path: the title with a space normalized, selected correctly, and original / reward / comments / creation source / preview all configured successfully with media id `100011077`; stale draft was deleted automatically.

## New Record Template

```markdown
### <Symptom or failure message>

**Symptom:** <What the user saw and which path was affected.>

**Root cause:** <Verified cause.>

**Fix:** <Final code or process fix.>

**Prevention:** <Test, constraint, monitoring, or documentation entry.>

**Evidence:** <Source / test / PR / docs, without sensitive data.>

**Article angle:** <Optional, suitable public article angle.>
```

## 2026-11-18 Pipe-less Markdown tables fell through as body text, leaving a row of `|` in WeChat

**Symptom:** Hand-written and pasted Markdown tables routinely omit the outer pipes (`日期 | 事件 | 备注` / `--- | --- | ---` / `11-01 | 报名 | 20 人`). That is valid GFM, but `moonpub render` treated the whole block as prose, so the WeChat article showed three paragraphs full of `|`. One class of the long-standing "tables look broken in WeChat" report is exactly this.

**Root cause:** The `is_table_row()` contract was "trimmed length ≥ 2, starts and ends with `|`". That shape was narrowed in while fixing a `[1..0]` panic on a lone `|`, and the narrowing silently excluded every pipe-less table — it looked at the *ends of a line* instead of GFM's real table signal (header row + delimiter row). Detection and rendering also each carried their own "strip the outer pipes" assumption (`&trimmed[1..trimmed.len() - 1]`), so a pipe-less row that did get in would lose one character at each end.

**Fix:** Table detection now has three entry points: (1) the classic outer-pipe row; (2) a line containing `|` whose next line is a delimiter row (`is_delimiter_row()`, every cell shaped like `:?-+:?`); (3) a line already inside a table block that contains `|` and matches the header's column count (`is_table_continuation()`). A new `strip_edge_pipes()` replaces the `[1..len-1]` slice everywhere; a trailing escaped pipe (`\|`) is cell content and is not stripped. A lone `---` is still a divider, never a one-column table. Rows with a mismatched column count fall back to prose instead of being swallowed — show it as written rather than guess.

**Prevention:** Three TDD cases: a pipe-less table must render as exactly one card with no `|` left in the body; mixed edge pipes (header with, body without) must stay one table; genuine prose containing `|` but *not* followed by a delimiter row (for example `用 A | B 表示两个变量。` followed by `---`) must stay prose with no invented card. Controlled mutation: reverting `starts_table` to `is_table_row(trimmed)` turned the first two red immediately and green again on restore. A 400-line adversarial input of random pipes, backslashes and CJK text rendered with exit code 0 and no panic.

**Evidence:** `src/markdown/plain.rs` (`strip_edge_pipes` / `is_delimiter_row` / `is_table_continuation` / `render_table`), three new cases in `src/markdown.rs`; the real binary left zero `|` in the body and turned both sample tables into cards.

**Article angle:** Narrowing a table check to stop a panic is fine; narrowing past that, until a legal syntax is excluded wholesale, is not. When the condition describes what a line *looks like* rather than what a block *is*, whole classes of input leak through silently — GFM's signal is a header plus a delimiter row, not the two characters at the ends.

## 2026-11-18 Parenthesised links truncated into dead links, mid-paragraph images emitting nested `<p>`

**Symptom:** Two visible-in-WeChat defects from the same class of input: (1) `[维基](https://zh.wikipedia.org/wiki/Rust_(编程语言))` rendered as `href="https://zh.wikipedia.org/wiki/Rust_(编程语言"` with a stray `)` leaking into the body — a dead link; Wikipedia article URLs (`Rust_(programming_language)`, `Mercury_(planet)`) carry paired parentheses all the time. (2) `文字 ![图](/x.png) 文字` rendered as `<p>…文字 <p style="margin:1.5em 0"><img …></p> 文字</p>` — a paragraph inside a paragraph, which the WeChat editor breaks into fragments.

**Root cause:** `parse_link()` / `parse_image()` both located the destination with `position(|&c| c == ')')`, i.e. "the first `)`". That is fine for plain URLs, but a URL may legitimately contain balanced parentheses, so the closing position is wrong as soon as a `(` appears first. The inline image case is more direct: `inline_md()` reused the block-level figure styling (which wraps itself in `<p>`), even though by the time it runs the outer `<p>` already exists — the function only considered what it emits, never where it emits it.

**Fix:** New `destination_end()` finds the real closing `)` by tracking paren depth, treating `\(` / `\)` as literals (`unescape_parens()` restores them); if depth never returns to zero (e.g. `/a(b`) it falls back to the old first-`)` rule so a link that used to work does not degrade to plain text. Inline images now emit an inline `<img>` (`display:block` keeps the spacing, no `<p>`), and both its `src` / `alt` and the block-level `render_image_figure()` are `html_escape()`d — URLs containing `&` or `"` used to break the attribute.

**Prevention:** Five TDD cases: balanced parentheses must survive in `href` and `</a>)` must never appear; `\(` / `\)` must be unescaped; an unbalanced `(` keeps the legacy behaviour; `inline_md` output must contain **no** `<p`; `src` / `alt` must be escaped. Plus a block-image attribute-escaping case. Controlled mutation: making `if depth == 0` always true, and restoring the `<p>` wrapper without escaping, turned three cases red at once; on restore (sha256 verified) all 21 inline cases are green. The real binary on the same probe article shows a full `href` and no nested `<p>`.

**Evidence:** `src/markdown/inline.rs` (`destination_end` / `unescape_parens` / `parse_link` / `parse_image` / inline image branch), `src/markdown/plain.rs` (`render_image_figure` escaping), block-image case in `src/markdown.rs`.

**Article angle:** Both bugs come from one habit: handling only the layer you can see. The URL parser saw "the first `)`" but not "parentheses pair up"; the inline image renderer saw "an image wants spacing" but not "what is wrapping me". Asking "can this delimiter appear inside the content?" while writing a parser, and "what does my caller's context look like?" while writing a fragment renderer, blocks this whole class.

## Related Records

- `AGENTS.md`: current constraints and module boundaries to follow during development.
- `CLAUDE.md`: older detailed debugging notes kept as background; this document wins for new conclusions.
- `PROGRESS.md`: chronological record of completed features, verification, and release facts.
- `docs/WECHAT_REGRESSION_CHECKLIST_ZH.md`: real WeChat regression checklist; does not replace root-cause records.

### Standard template footer visual structure must be stable and replaceable

**Symptom:** The user reported an unwanted gray background frame, repeated personal-brand name, and the word "Official Account" in the standard template footer. The community QR code area also rendered unstably when text existed but no image was configured.

**Root cause:** The brand card used `<table>` + gray background + `border-radius`, which appeared as a visible frame in the WeChat editor. The brand description repeated the name. The `follow_image` alt text contained "Official Account". The community section display condition included only `description` / `rules` / `qrcode`, not `title` and `qrcode_note`, so text-only configuration could make the whole section disappear.

**Fix:** The brand card keeps table layout because WeChat strips flex, but removes gray background and rounded corners. The description no longer repeats the name. `follow_image` alt changed to a neutral follow label. The community section displays when any of `title` / `description` / `rules` / `qrcode_note` / `qrcode` is non-empty. `render` now warns in the terminal when the local qrcode path is unreadable.

**Prevention:** Footer visual changes must check whether WeChat strips critical style, whether brand name repeats, whether any alt/copy contains unwanted product naming, whether text-only config renders stably, and whether local QR code path readability is reported during render rather than discovered only after pushing to WeChat.

**Evidence:** `src/footer.rs` brand card and community section rendering, `src/render.rs` qrcode readability check, footer tests, and user confirmation on 2026-07-26 that the structure can be fixed and only the group QR image needs replacement later.

**Article angle:** A template footer is not better when it is richer. Stable structure, replaceable assets, and no redundant copy let users care only about the one image they need to swap.

### Obsidian plugin homepage must evolve from status string to card workspace

**Symptom:** After opening the plugin homepage, users saw a chain of `ul` / `li` and multiple `h3` sections and did not know what to click first. First-time users were likely to get lost between workspace status, current context, recommended next step, and risk boundaries.

**Root cause:** The early homepage flattened fields from `workspace --json` without information hierarchy or primary buttons. Current file and workspace overview were mixed together, and action buttons were scattered.

**Fix:** The homepage is now split into 8 card layers: current file with context kind / path / recommendation / primary action; workspace overview with CLI status and stage counts; recommended next step and first-run suggestions; available workflows; v0.4.2 evidence / gates; action entries; WeChat reach reminder; permanent help prompt. Current article, Feishu/photo results, preflight, and layout-audit workspaces now share the same `moonpub-card` + `moonpub-action-row` styling. Auxiliary dialogs also share the `moonpub-homepage` class.

**Prevention:** New plugin workspaces should reuse `moonpub-card` / `moonpub-card-title` / `moonpub-action-row`. Homepage information order should follow current file -> workspace -> next step -> workflow -> gates -> actions -> reminder -> help. Any action button must close the current modal before triggering the next step so notices are not obscured.

**Evidence:** `obsidian-plugin/main.ts` `MoonPubWorkspaceModal` / `MoonPubArticleModal` / `MoonPubIntakeResultModal` / `MoonPubPreflightModal` / `MoonPubLayoutAuditModal`, `obsidian-plugin/styles.css` card styles, and plugin docs in `obsidian-plugin/README.md` and `docs/USER_GUIDE.md`.

**Article angle:** The distance from CLI tool to regular user is often a layer of "what should I click now?"

### A Chinese title panics ai-lab cover generation: never slice strings at fixed byte offsets

**Symptom:** With a normal Chinese title such as `title: "A一二三四五六七八"` and `ai-lab` cover style, `moonpub cover <article.md> --style ai-lab` crashed with exit code 101 and `thread 'main' panicked at src/cover.rs:931:16: end byte index 15 is not a char boundary; it is inside '五' (bytes 13..16 of string)`, producing no cover HTML. The same title rendered fine with `dark` and other styles, and short ASCII titles were fine too, so the failure only surfaced on the real "long Chinese title + ai-lab" publishing path.

**Root cause:** `render_ai_lab_cover` wrapped the escaped title using **byte** offsets: `else if title.len() > 18 { (&title[..15], Some(&title[15..])) }`. In UTF-8 a CJK glyph is 3 bytes and an emoji is 4 bytes, so a fixed byte offset 15 is not guaranteed to be a char boundary and slicing panics. `title.len() > 18` also used a byte count as "is the title too long", so six Chinese characters (18 bytes) were already treated as long. One layer deeper: `title` had already been HTML-escaped before reaching this function, so it may contain ASCII entities such as `&amp;`; even without a panic the cut could split `&amp;` into `&am` + `p;` and emit broken HTML. The same root cause existed in the `&cur[..80]` URL log in `src/publish.rs`, and in a test assertion's `&html[..80]` / `&html[len-80..]` diagnostic slices (the latter would panic a second time while formatting the failure message, masking the real reason).

**Fix:** `src/cover.rs` now extracts a pure `split_ai_lab_title` helper: length is measured with `chars().count()`, the cut point comes from `char_indices().nth(15)` (always a char boundary), and the cut retreats to a preceding unclosed `&` so an HTML entity is never split in half. The two separator branches use `'：'.len_utf8()` / `": ".len()` instead of magic numbers 3 / 2. `src/publish.rs` extracts `log_url_preview`, truncating log URLs with `char_indices().nth(80)`. The test assertion's diagnostic slices now use `char_indices()` from both ends. The crate MSRV is 1.88 while `str::floor_char_boundary` only stabilized in 1.91, so the newer API is not available; boundaries must be located explicitly with iterators.

**Prevention:** (1) Any truncation or split of user-visible strings (titles, digests, URLs, HTML) must use `chars()` / `char_indices()`; fixed byte offsets like `&s[..N]` or `&s[len-N..]` are forbidden. `grep -rnE '&[a-z_]+\[\.\.[0-9]+\]' src/` is a quick self-check. (2) `cover.rs` adds regression tests for Chinese / emoji / `&` entities / full-width and ASCII separators, plus `all_cover_styles_handle_multibyte_titles` which runs multibyte titles across all 20 styles so a new style cannot reintroduce a byte slice. (3) `publish.rs` adds three `log_url_preview` boundary tests (long ASCII, 81-byte Chinese, short Chinese URL). (4) Semantic length checks such as "is the title too long" must use character counts, never byte counts.

**Evidence:** Before the fix, the real binary returned `rc=101` with the panic above for `moonpub cover Articles/drafts/demo.md --style ai-lab`, while `--style dark` on the same title returned `rc=0`. After the fix the same command returns `rc=0` with empty stderr; six multibyte titles (including `R&D ...` and emoji) all produce HTML with `R&amp;D` intact. `cargo test --lib cover::` 53 passed; full `cargo nextest run --lib` (excluding another session's in-progress `src/illustrate.rs`) 690 passed / 11 skipped.

**Article angle:** "A Chinese title crashed the program" — in UTF-8, "byte 15" and "character 15" are not the same thing, a boundary lesson any Chinese-language software has to learn.

### Value-taking flags still pinned the path to argv[2]: the same bug fixed three times

**Symptom:** The Obsidian plugin's "AI polish and push to WeChat draft" action always failed. The plugin builds `ship --ai <path>` (`buildArgs` in `main.ts` always appends the file path after the subcommand), and the CLI parsed `--ai` as the article path, failing with `article path must point to a .md file: --ai`. Manual invocations failed the same way: `moonpub cover --style ai-lab a.md` reported `file not found: --style` and ignored the real article; `moonpub cover a.md bogus` exited 0 and silently dropped `bogus`; `moonpub cover a.md --style --screenshot` treated `--screenshot` as the style value and silently disabled the screenshot switch.

**Root cause:** Six "single path + value-taking flag" commands (`render`, `cover`, `publish`, `ship`, `update-draft`, `export`) each hard-coded the path at `path_arg(rest, 1)` and scanned flags from `rest[2..]`. `cover` and `ship` ended their match with `_ => {}`, silently swallowing extra positionals, and `flag_value` consumed the next token without checking whether it was itself a flag. More fundamentally, the repair pattern was wrong: PR #168 built a shared entry point only for boolean-flag commands and PR #169 added tail validation only for zero-flag commands, so one class of defect was patched one syntax shape at a time, leaving the next shape broken.

**Fix:** Added `path_and_flags_with_values(rest, usage, value_flags)` as the single scanner. It pairs each known value-taking flag with its value, returns `MissingValue(flag)` when the value is absent or starts with `--`, keeps other dash-prefixed tokens, treats the first remaining non-flag token as the path, and returns stray tokens in their original order so the existing `match` still decides between `UnknownOption` and `UnknownCommand` (preserving "unknown flag wins over extra positional"). `path_and_flags` became a thin `value_flags = &[]` wrapper; `flag_value` gained the same missing-value guard; all six arms moved to the new entry point, and `cover` / `ship` changed `_ => {}` to `UnknownCommand`.

**Prevention:** (1) `AGENTS.md` now requires any single-path command to use `path_and_flags*` and to register value-taking flags in `value_flags`; hand-written `path_arg(rest, 1)` plus `rest[2..]` is forbidden. (2) `grep -nE 'path_arg\(rest, 1|rest\[2\.\.\]' src/cli.rs` is the self-check (structural exceptions such as `intake` / `configure` need an explicit comment). (3) A `VALUE_FLAG_POSITIONAL_COMMANDS` table-driven test covers all six commands for flags-before-path, flags-after-path, extra positionals, unknown flags, flag-as-value, and missing path. (4) Known residual cases are recorded here — `intake photos` multi-positional syntax, the hand-written `configure` / `test-yulan` flag loops, and `resolve_config` using the first non-flag token as its config anchor — so the next round does not rediscover them from scratch.

**Evidence:** Before the fix the real binary reported `file not found: --style` for `cover --style dark /tmp/x.md` and exited 0 for `cover /tmp/x.md bogus`; after the fix the first reaches the normal business layer and the second reports `unknown command: bogus` with a non-zero exit. `cover /tmp/x.md --style --screenshot` now reports `missing value for --style`. `cargo nextest run --lib cli::` went from 91 to 99 passed, and the plugin argv `ship --ai drafts/post.md` changed from `InvalidArticlePath("--ai")` to a normal `ShipAi` parse.

**Article angle:** "The same bug fixed three times" — when every fix is a patch for one syntax shape, the defect returns in the next shape; the thing that needs converging is the entry point, not the branches.

### `radar scrape` panics on a Chinese page with no closing tag: byte windows need char boundaries too

**Symptom:** `moonpub radar scrape --platform wechat --keyword 测试 --url file:///tmp/radar-bad.html` exited 101 on a 220-byte page (`<h2>` followed by twelve Chinese phrases, no `</h2>`), with `thread 'main' panicked at src/radar/scrape.rs:228:34: end byte index 204 is not a char boundary; it is inside '中' (bytes 202..205 of string)`, and stored zero samples. `--url` is a documented flag (`docs/COMMANDS.md`), so this parser is expected to survive arbitrary third-party pages: truncated responses, uppercase tags, sloppy markup with unclosed tags.

**Root cause:** When no closing tag is found after an opening one, `extract_from_html` falls back to a fixed byte window: `unwrap_or(content_start + 200.min(text.len() - content_start))`. `content_start` is always a char boundary (it follows an ASCII `>`), but `content_start + 200` is not: CJK glyphs are 3 bytes and emoji are 4, so a 200-byte window routinely ends inside a glyph, and the following `&text[content_start..content_end]` panics. The trigger is a page containing `<h2` / `<h3` / `<a` opening tags with no matching closing tag anywhere in the remainder (truncated page, mismatched `</H2>` case, unclosed tag) and more than 200 bytes left. It slipped past the previous UTF-8 audit because `src/radar/scrape.rs` had no `#[cfg(test)]` module at all, and the only nearby test (`scrape_stores_samples_in_jsonl`) feeds well-formed, closed HTML.

**Fix:** Added `floor_char_boundary(text, start, end)`, which walks back with `str::is_char_boundary` to the nearest boundary (at most three steps) and is used only by the fallback branch; when a closing tag is found the output is byte-for-byte unchanged. The crate MSRV is 1.88 while `str::floor_char_boundary` stabilized in 1.91, so the standard helper is unavailable.

**Prevention:** (1) `src/radar/scrape.rs` now has its first `#[cfg(test)] mod tests`, covering closed Chinese headings, a missing closing tag (3-byte CJK), an uppercase closing tag, a 4-byte emoji fallback, `<script>` stripping, duplicate-title dedupe, and plain text with no tags. (2) `AGENTS.md` widened the rule from "no fixed-byte truncation" to "no fixed-byte windows or substrings", and now requires any new parsing / rendering / scraping module to ship its own tests. (3) Besides `grep -rnE '&[a-z_]+\[\.\.[0-9]+\]' src/`, the self-check must also look for window arithmetic such as `+ 200` or `+ N`.

**Evidence:** Before the fix the same command returned `rc=101` with the panic above and `panicked` count 1; after the fix it returns `rc=0`, no panic, and prints `scraped 1 samples → /tmp/radar-probe/.moonpub/trends.jsonl`. `cargo nextest run --lib radar::` went from 68 to 75 passed (7 new tests).

**Article angle:** "Not every 200 is safe" — the most dangerous offset in bug hunting is rarely a bare constant but a constant addition: it looks like a bounded window and is actually a bet on character boundaries.

### A destructive command silently dropped extra arguments: `delete-draft <id> --dry-run` really deleted the draft

**Symptom:** `moonpub delete-draft <media_id>` calls the WeChat API directly, has no dry-run mode, and asks for no confirmation — yet the parser only read `rest.get(1)` and discarded everything after it. `moonpub delete-draft 999999 --dry-run` returned `rc=0` and printed `已删除草稿(浏览器会话): 999999`: the caller believed it was previewing, the program had already hit the delete endpoint. The same silent drop affected `status junk`, `version junk`, `list-drafts --bogus`, `fetch <url> extra` and `workspace extra`, all returning `rc=0`.

**Root cause:** A group of fixed-arity commands never validated `rest[n..]`. Five arms returned `Command::*` immediately (`status`, `version`, `workspace`, `layout-recipes`, `list-drafts`) and two only read a single positional (`delete-draft`, `fetch`). PR #169's `reject_stray_args_after_path` covered only zero-flag commands with one path; PR #171's `path_and_flags_with_values` covered only single-path commands with value-taking flags. Fixed-arity commands with no path concept were never converged. The risk is not uniform: for read-only commands a swallowed argument merely misleads, while for `delete-draft` the swallowed token may be exactly the `--dry-run` the caller assumed made it safe.

**Fix:** Added `reject_extra_args(rest, consumed)`, which takes `rest.get(consumed..)` and hands it to the existing `reject_unexpected` (leading dash → `UnknownOption`, otherwise `UnknownCommand`), keeping the trailing `--json` allow-list for compatibility. Seven commands now call it: `status`, `version`, `workspace`, `layout-recipes`, `list-drafts` with `consumed = 1`; `delete-draft`, `fetch` with `consumed = 2`. `help.rs` now states for `delete-draft` that there is no dry-run and that extra arguments are rejected.

**Prevention:** (1) `AGENTS.md` extends the CLI rule: fixed-arity commands must call `reject_extra_args` and never let `rest[n..]` vanish. (2) A `ZERO_ARG_COMMANDS` table test covers stray flags, stray positionals and trailing `--json` for all five zero-argument commands. (3) A dedicated `delete_draft_rejects_dry_run_instead_of_deleting` test pins the real argv of the destructive command: an unknown argument on a destructive command must fail rather than degrade into the default action. (4) Known gaps are recorded: `new <title>` / `write <idea>` take free text (`moonpub new 我的 文章` silently keeps only `我的`), `intake photos` / `intake feishu` emit misleading errors when flags come first, and `test-yulan --title --headed` still consumes a flag as a value.

**Evidence:** Before the fix the real binary returned `rc=0` for `delete-draft 999999 --dry-run` with `已删除草稿(浏览器会话): 999999`, and `rc=0` for `status junk` / `version junk`. After the fix the same commands return `rc=1` with `unknown option: --dry-run` and `unknown command: junk`, while `status --json` still prints its `status/v1` payload. `cargo nextest run --lib cli::` went from 99 to 106 passed (7 new tests).

**Article angle:** "Dry-run is sometimes a lie" — the dangerous failure is not an error but a success that is too quiet: the user thinks they pressed preview, the program receives a ticket to execute. Destructive commands must be intolerant of arguments they do not understand.

### A title quietly cut down to its first word: the CLI must not be dumber than the layer below it

**Phenomenon:** `moonpub new 我的 第一篇 文章` exits 0 and prints `created`, yet the only file produced is `Articles/drafts/我的.md` with `title: 我的` in its frontmatter — the last two words vanish and nothing tells the caller. `write <idea>` likewise hands just one word to the AI. `moonpub help preflight extra` silently drops `extra`. `test-yulan --title --headed` is worse: the title becomes `--headed` and the switch itself is lost.

**Root cause:** Three layers. (1) `src/cli.rs` took only `rest.get(1)` for `new` / `write`, while `src/draft.rs` already had a unit test using `"我的 第一篇 文章"` as the title and producing `我的-第一篇-文章.md` — the storage layer fully supports spaced titles; the CLI was the component throwing user input away. Typing a Chinese title without quotes is the normal case, not an edge case. (2) These were the last leftovers of a fix-by-syntax-shape sequence: #169 covered zero-flag single-path commands, #171 single-path plus value flags, #173 fixed-arity commands — free text, single-topic commands and hand-rolled index loops were never touched. (3) `test-yulan` was the only value-flag command still using `index += 1` with `rest.get(index)`, bypassing the guard #171 added inside `flag_value`. A guard that is not reused is a guard that was never installed.

**Fix:** Added `join_free_text(rest, usage)`: joins every non-flag token in `rest[1..]` with a space, tolerates a trailing `--json` (`new` / `write` are not in `supports_subcommand_json`, so that spelling reaches the arm), rejects any other leading-dash token as `UnknownOption`, and reports `MissingValue` for empty text. `new` / `write` now use it; `help` calls `reject_extra_args(rest, 2)`; `test-yulan` was rewritten to `rest[1..].iter()` with `flag_value`, matching `configure`. No error variants, exit codes or documented forms changed.

**Prevention:** (1) `AGENTS.md` now states the rule in one sentence — **no command may silently discard a token the user supplied** — and names the required entry points (`join_free_text` for free text, `flag_value` for every value flag, no `rest.get(1)` or hand-rolled index loops). (2) A `FREE_TEXT_COMMANDS` table test covers multi-word Chinese and English titles, single-word titles, trailing `--json`, unknown flags and missing text. (3) `test_yulan_flag_values_reject_a_following_flag` pins "the guard must actually be reused" so a future hand-rolled loop cannot quietly bypass it. (4) The remaining `intake photos` / `intake feishu` multi-positional flag-order issues are recorded as a follow-up: they fail loudly with an error, so they rank below silent truncation.

**Evidence:** Before the fix, the real binary turned `new 我的 第一篇 文章` into `我的.md` with `title: 我的`, and `help preflight extra` returned rc=0. After the fix, the same argv produces `我的-第一篇-文章.md` with `title: 我的 第一篇 文章`, `help preflight extra` returns rc=1 with `unknown command: extra`, and `help preflight --json` still emits `help/v1`. `cargo nextest run --lib cli::` went from 106 to 116 passed (10 new tests).

**Article angle:** "The most dangerous bug is the one that succeeded" — the program returns 0, prints `created`, and the user still gets half a sentence. Verification has to inspect the artifact, not just the exit code. Also: "the guard exists" is not the same as "the guard is used"; relying on humans to remember a call site guarantees one branch will eventually miss it.

### `intake` read a flag as a file and `--draft` as a keyword: the last un-converged parser shape

**Phenomenon:** Six real-binary probes: `intake photos --draft photos/day1` and `intake photos --analyze-images a.jpg b.jpg` both reported `missing value for intake photos <file-or-dir> [more files or dirs]` even though a path was supplied; `intake photos a.jpg --draft b.jpg` reported `unknown command: b.jpg`; `intake feishu --draft minutes.txt` reported `unknown option: --draft`. The quiet one was `intake feishu --query --draft --push`: `--draft` became the query keyword, then the command reported `missing value for intake feishu --push requires --draft` — the switch the caller did pass had vanished. `--minute-token --draft --push` behaved the same. The previous round had logged this as "multi-positional + flag-first, fails loudly, lower priority"; probing proved it also hid a silently swallowed token.

**Root cause:** `intake` in `src/cli.rs` was the last leftover of the fix-by-syntax-shape sequence. `feishu` fixed the source at `rest.get(2)`, its value at `rest.get(3)`, then hard-coded `flag_start = 3 | 4` and handed every remaining token to the flag validator. `photos` scanned `rest[2..]` until the first leading-dash token and `break`, treating everything after it — including valid paths — as flags. Both assumed positionals first and all flags after, and neither reused the guard #171 added to `flag_value` ("a following switch means the value is missing"). A guard that is not reused is a guard that was never installed.

**Fix:** Added one scanner, `split_intake_tokens(tokens, value_flags)`, which separates positionals, boolean switches and value-taking switches in a single pass and applies the same guard as `flag_value` (a following `-` token is `MissingValue(usage)`). Unknown switches stay in `flags` so the existing validator still reports `UnknownOption`. `parse_intake_feishu` / `parse_intake_photos` then own their shapes: feishu's four source selectors are mutually exclusive and any conflict or absence reports the documented usage; photos collects any number of paths. Command enums, error variants, JSON contracts and every documented spelling are unchanged.

**Prevention:** (1) `AGENTS.md` names the required entry points for `intake` and forbids `rest.get(2)` / `rest[flag_start..]`. (2) Eight table-driven / equivalence tests cover flags before, between and after paths, all four source shapes with flags first, value selectors rejecting a following switch, conflicting sources, unknown flags at either position, missing paths, and "reordering the same tokens must produce the same `Command`". (3) `help.rs` and both COMMANDS documents now state that flag order is irrelevant and value selectors require a value. (4) The `intake` follow-up logged in PROGRESS and in both lessons files is closed by this round.

**Evidence:** Before/after real-binary comparison as above; after the fix `intake photos --draft <missing-file>`, `intake feishu --draft <missing-file>` and `intake photos a.jpg --draft b.jpg` reach the business layer and report `No such file or directory` (parsing accepted them; nothing was written), while `--query --draft --push` now reports `missing value for intake feishu --query <keyword>`. The documented `intake photos <path> --draft --preview --no-open` spelling is unchanged. `cargo nextest run --all-features --lib cli::`: 116 → 124 passed.

**Article angle:** A second confirmation that "the guard exists" is not "the guard is used". Adding one more special case would not have helped; removing the second parsing path and pinning order-independence with an equivalence test does.

### Reordering the arguments changed the theme: config discovery did not follow the new flag-first spelling

**Phenomenon:** A real-binary A/B probe in an isolated tree (`work/moonpub.toml` sets `[wechat] theme = "blueprint"`; `--articles` points at an empty directory): `moonpub --json render <article> --author FLAG_AUTHOR` produced HTML with `background: #f5f8ff` (blueprint), while moving only `--author` in front of the article — `render --author FLAG_AUTHOR <article>` — produced the default white theme (`color: #333; background: #fff`). Both returned rc=0 with byte-identical stdout; only the artifact differed. A second probe pointed `--articles` at a subdirectory whose parent held an unrelated `theme = "ai-lab"` config: the flag-first invocation loaded that config and produced `background: #10111f`. So the failure is not merely "config missed" but "config from the wrong project adopted".

**Root cause:** `resolve_config` took the first non-flag token in `rest[1..]` as the path anchor and walked upwards looking for `moonpub.toml`. That was correct while the path had to follow the command name. After #168/#171 allowed flags before the positionals, #174 allowed free text and #175 allowed multiple positionals, that first token is usually a flag value, a subcommand name or a title word. The upward walk therefore started in the wrong directory: no hit silently fell back to the default config, and a hit on an ancestor adopted an unrelated project's config. Old code reading the same argv was never updated for the new spelling — a regression introduced by the flag-order fixes.

**Fix:** Added `config_anchor_candidates(rest, articles_dir)`: every non-flag token becomes a path candidate, and a stable `sort_by_key(|p| !p.exists())` puts existing paths first while preserving argv order for the rest. `resolve_config` walks upward for each candidate and returns on the first `moonpub.toml` hit. Deliberately no global "value-taking flag" allow-list: such a list drifts with every new command, whereas "an existing file wins" holds for both old and new spellings. Explicit `--config`, articles-root lookup and path-less commands (`status`, etc.) are unchanged.

**Prevention:** (1) Four tests cover eight argv shapes (render / cover / intake photos / preview / check, path first and flag first), the decoy-config case, explicit `--config` plus path-less commands, and the candidate ordering itself. (2) `AGENTS.md` now forbids assuming the first non-flag token is the article and requires a real-binary "same arguments, different order, same artifact" comparison whenever CLI parsing shapes change. (3) Both COMMANDS documents state the order-independence as a user-visible contract. (4) The `resolve_config` residue logged last round is closed here.

**Evidence:** Before/after artifact comparison as above; after the fix all four combinations (empty / decoy articles root × article first / flag first) produce `background: #f5f8ff`. `cargo nextest run --all-features --lib cli::` 124 → 128 passed; full suite 749 → 753 passed / 0 skipped.

**Article angle:** "When you accept a new spelling, audit every old reader of the same argv." The parser relaxed the order but config discovery kept guessing the old one. Verification that only checks the exit code and stdout will never notice a theme change — inspect the artifact.

### `theme: blu-print` rendered successfully in the wrong theme — and `check --json` claimed `publishable: true`

**Phenomenon:** Real-binary probes: a frontmatter `theme: blu-print` typo made `moonpub render` return rc=0 and print `rendered`, but the HTML background was the default `#fff` (the correct `blueprint` is `#f5f8ff`; omitting `theme` also gives `#fff`). The config path is worse: with `[wechat].theme = "blu-print"`, `check --json` explicitly reported `effective_theme: blu-print`, `theme_source: wechat_config` and `publishable: true` while rendering still used the default theme. `check` is what agents use to verify a draft before publishing, so it actively confirmed a theme that never took effect.

**Root cause:** `Theme::from_name` ends in `_ => Self::default()`, silently degrading every unknown name. That fallback is harmless for built-in themes but both frontmatter and config are user input — a typo hits it. Worse, `protocol/check.rs::check_json` is a completely separate path that echoes the configured string as `effective_theme` without rendering or validating, so the report and the artifact could diverge indefinitely. Earlier rounds hardened CLI *argument* parsing thoroughly, but the *content* of a value (a finite enum) was only validated for `configure` steps; theme names were missed.

**Fix:** Extracted a private `resolve(name) -> Option<Self>` in `theme.rs` as the single source of truth (including the `geek_black` / `ai_lab` / `one_dark` aliases); `from_name` is now `resolve(...).unwrap_or_else(Self::default)` and a new `is_known_name()` serves input boundaries. `render_article` validates `effective_theme` immediately after computing it and returns the new `AppError::InvalidThemeName` for unknown values; `check_json` validates too and now returns `Result` instead of emitting a `check/v1` success payload for an invalid theme. The error-code chain is complete: `ErrorCode::InvalidThemeName` + `ALL` (23→24) + `as_str` / `summary` / `example_next_step` / `classify` / `all_failures`, keeping the registry and classifier in lockstep. Command enums and existing JSON payloads are unchanged.

**Prevention:** `AGENTS.md` now states that theme names, like `configure` steps, are finite enums requiring a shared validator, that both the render and the check entry points must reject unknown values, and that the validator and `from_name` share one `resolve()`. A test walks `for name in Theme::names()` through `is_known_name`, so adding a theme without updating `resolve()` fails immediately; a typo table (`blu-print` / `BLUEPRINT` / `blue print` / empty) pins the reject behavior.

**Evidence:** Before the fix, the real binary rendered rc=0 with `#fff` and `check --json` claimed `effective_theme: blu-print` + `publishable: true`. After the fix both `render` and `check --json` return `error/v1` `invalid_theme_name` (`command` = `render` / `check` respectively); after clearing stale artifacts, a typo'd theme writes zero HTML / draft.json; the valid `blueprint` still renders `#f5f8ff` and the `ai_lab` alias still works. Full `cargo nextest run --all-features` 766 → 772 passed / 0 skipped (+6); clippy `-D warnings`, rustfmt, doc lint all pass.

### `configure zanshangg` was silently accepted: a misspelled step never ran but the draft was still saved

**Phenomenon:** Real-binary probes: `moonpub --json configure --boguss` is rejected at parse time with `unknown_option`; but `moonpub --json configure zanshangg` (a misspelled step name) passes parsing with no error and enters browser automation. By the code's semantics, `run_step("zanshang")` compares strings exactly and will never match `steps = ["zanshangg"]`, so the "reward" step is silently skipped while `step_baocun` / `step_fucha` run unconditionally and the draft is saved normally — the user believes it was configured, it was not, and the command reports success.

**Root cause:** The `configure` arm in `src/cli.rs` was the last hand-rolled loop left in the parser; it accepted every other token with `s => steps.push(s.to_owned())`. Step names are a fixed six-value enum (`yuanzhuang` / `zanshang` / `liuyan` / `chuangzuo` / `moban` / `yulan`, defined in `src/publish.rs`), but the parser had no whitelist. Earlier rounds converged single-path, value flags, fixed arity, free text, intake and radar by syntax shape; this "finite-enum positional" shape was the one not touched.

**Fix:** Added `is_valid_configure_step(name)` in `src/publish.rs`, matching on the same `STEP_*` constants that `run_step` uses (zero list drift), and the `configure` arm now validates every positional through it, reporting `UnknownCommand(format!("configure {s}"))` for unknown names. Command enums, error variants and JSON contracts are unchanged. TDD: wrote `configure_rejects_unknown_step` (covering `zanshangg` / `YUANZHUANG` / `clips` / a Chinese token) and watched it FAIL with `Ok(steps:["zanshangg"])`, then implemented the fix to PASS; `configure_accepts_every_known_step` pins all six valid names.

**Prevention:** `AGENTS.md` now states that finite-enum positionals must be validated through a shared validator (no accepting every token), and that the validator and executor share one set of constants. This extends "no command may silently discard a token" from flags/surplus arguments to "enum values may not be silently ineffective".

**Evidence:** The failing test before the fix (`unwrap_err()` on `Ok(steps:["zanshangg"])`); after the fix, real-binary `configure zanshangg` / `configure YUANZHUANG` → `error/v1` `unknown_command`, `configure --boguss` still `unknown_option`, and all six valid steps confirmed via unit test to reach `Configure { steps }`. Full `cargo nextest run --all-features` 766 passed / 0 skipped (+2); clippy `-D warnings`, rustfmt, doc lint all pass.

### `init a.toml b.toml` silently wrote `b.toml` and `moonpub init --yes --help` reported an error

**Phenomenon:** Real-binary probes: `moonpub init a.toml b.toml` exits 0 and only creates `b.toml` — `a.toml` is silently discarded (a different filename, the wrong target). `moonpub init --yes --help` does not show help; it errors with `unknown option: --help`. Both are "success/error disagrees with intent"; the first is silent, the second turns a help request into an unknown-option error.

**Root cause:** The `init` parse loop assigned `path_arg = Some(v)` for every non-`-` token, so a second positional overwrote the first with no error. `Options::parse` only recognized subcommand help when `rest.get(1) == "--help"` — i.e. only the token immediately after the command name. In `init --yes --help`, `--help` is at index 2, so the help check was skipped and the token fell through to the subcommand parser as an unknown option. The legacy code assumed positionals live at a fixed index; after the earlier rounds relaxed flag order, that assumption also broke the help check.

**Fix:** A second positional to `init` now reports `UnknownCommand` (matching `browser_flags` / `draft-from-inbox`) instead of overwriting. `Options::parse` now scans every trailing token: `rest.iter().skip(1).any(|s| s == "--help" || s == "-h")`. Three new cli parse tests: `init` rejects a second positional, still parses a single positional, and `--help` after a flag still resolves to help (the existing "help right after the command" and "single-topic help" tests are unchanged).

**Prevention:** `AGENTS.md` now states that subcommand `--help`/`-h` must scan every token in `rest[1..]` (not only `rest.get(1)`), and that `init` accepts at most one positional, with a second reported as `UnknownCommand`. This extends the standing "no command may silently discard a token" rule — a silently overwritten positional is just a discarded token (the first one).

**Evidence:** Before: `init a.toml b.toml` → rc=0, only `b.toml`. After: rc=1 `unknown command: b.toml`, no file written. `init --yes --help` went from an error to the `moonpub init` help page (rc=0). `cargo nextest run --all-features` 764 passed / 0 skipped (+3 new tests).

### `radar list --platform --keyword` returned rc=0 with an empty result: the last hand-rolled parser

**Phenomenon:** Real-binary probes across the whole `radar` family: `radar list --platform --keyword` exits 0 and prints `trend samples (empty)` — the caller thinks they filtered by platform, but `platform` was set to the literal string `"--keyword"`, so the filter necessarily matched nothing while the program reported success. `radar list --platform --keyword 读书` reports `unknown command: 读书` and never mentions that `--platform` has no value. `radar suggest FIRST.md SECOND.md --platform wechat` complains only about `SECOND.md`; `FIRST.md` was silently discarded. `radar suggest A.md --platform wechat --typo-flag` swallows the misspelled switch without a word. `radar import --platform wechat x.csv` reports `unknown command: wechat` because `--platform` became the CSV path. The same value-flag bug reproduced in `add`, `scrape` and `analyze`.

**Root cause:** `src/radar/cli.rs` was the last module never connected to the shared parser. Each subcommand ran its own `while let Some(arg) = args.next()` and read values with a bare `next_arg = args.next().cloned()`, missing the guard #171 added to `crate::cli::flag_value` ("a following switch means the value is missing"). `parse_radar_suggest` even had a `_ => {}` arm that made unknown switches and extra paths vanish, and `import` / `analyze` took the path from `args.first()` on the assumption that it always comes first. Earlier rounds converged the parser by syntax shape inside `src/cli.rs` (zero-flag, value flags, fixed arity, free text, intake, config discovery) and never touched this **second parser in a separate file** — grepping `src/cli.rs` looked clean, so it escaped every self-audit.

**Fix:** Rewrote `src/radar/cli.rs` around one scanner, `split_radar_tokens(args, value_flags)`, which separates positionals from registered value flags in a single pass and reads every value through `crate::cli::flag_value` (one guard, shared with the main CLI). `RadarTokens` supplies `require` / `parsed<T>` / `no_positionals` / `single_positional`; each of the six subcommands now only declares its `*_VALUE_FLAGS` constant and its required fields. Repeated flags, unknown switches and extra positionals are errors; `import` / `analyze` / `suggest` are order-independent. The `_ => {}`, `next_arg`, `parse_u64` and `parse_radar_article_arg` hand-rolled paths are gone. Command enums, error variants, JSON contracts and every documented spelling are unchanged.

**Prevention:** (1) `AGENTS.md` folds `src/radar/cli.rs` into the same "no command may silently discard a token" rule and explicitly bans bare `args.next()`, `args.first()` and `_ => {}`. (2) `radar/cli.rs` now has 11 unit tests (3 existing + 8 new): table-driven "switch used as a value" and "trailing switch" cases across all six subcommands, `suggest` extra paths / unknown switches, stray paths on no-positional commands, flag-first equivalence, repeated flags, invalid numbers, and a regression test for the documented spellings. (3) `help.rs` and both COMMANDS documents state order-independence and the missing-value behaviour. (4) The audit entry changes from "scan `src/cli.rs`" to a repo-wide grep for hand-written `args.next()` / `starts_with('-')`, so a second parser in a separate file cannot hide again.

**Evidence:** The 11 before/after real-binary probes above; after the fix `radar list --platform --keyword` → rc=1 `missing value for --platform`, `radar suggest FIRST.md SECOND.md` → `unknown command: SECOND.md`, `--typo-flag` → `unknown option: --typo-flag`, `radar import --platform wechat x.csv` correctly resolves `x.csv` (the business layer then reports that file missing), and `radar list --platform wechat` still exits 0. All probes pointed at an isolated directory, triggered no writes or network calls, and left the probe directory empty. `cargo nextest run --all-features --lib radar:: cli::` 208 passed.

**Article angle:** "It succeeded, but the answer is wrong" is harder to notice than a crash — rc=0, normal stdout, and the only clue is an implausibly empty result. Convergence has to be counted in **parsers**, not command shapes: one hand-rolled loop in a separate file is enough to make the main parser's cleanliness irrelevant.

### `radar import` read `likes=abc` as an empty value: rc=0 reported success and silently lost engagement data

**Symptom:** Real user path `moonpub radar import trends.csv`: a `likes` cell contains `abc` (or `N/A`, a negative number, `1.5`, or a value beyond u64), and the command exits 0 printing `imported 2 samples`. The stored sample has `likes: None`, so `radar analyze` ranks those articles at the bottom. The user sees a successful import while data they actually typed was erased — with no warning and no hint in stdout or stderr pointing at the column.

**Root cause:** `src/radar/import.rs::import_csv` read numeric columns with `get_u64 = |idx| get(idx).and_then(|v| v.parse().ok())`. The outer `get()` already used `.filter(|s| !s.is_empty())` to distinguish "not filled" from "filled", so every `v` reaching `parse().ok()` was a string the user had explicitly entered; `.ok()` collapsed every parse failure into `None`, merging it with "column absent". On top of that, rows were written one at a time via `add_trend_sample`, so even after adding validation, earlier rows would already be on disk and a retry would duplicate them. Earlier rounds tightened CLI tokens and finite-enum values, but the **content of the parsed value** was never checked.

**Fix:** Replaced `get_u64` with a `parse_metric` closure: a blank cell (column missing or unfilled) still returns `Ok(None)`, while a non-blank value whose `parse::<u64>()` fails returns `AppError::InvalidCsv("line {n}: invalid unsigned integer in '{column}' column: {value:?}")` carrying the physical line number, actual header name and offending value (reusing the existing error code, no new enum). The loop gained `enumerate()` for accurate line numbers (header is line 1), and rows are now all parsed into `Vec<TrendSample>` before anything is written, so a failure leaves zero samples behind. Command enum, error variants, JSON contract and valid-CSV behaviour are unchanged.

**Prevention:** `AGENTS.md` now states that optional numeric columns mean "blank is fine, filled must be valid", bans `.parse().ok()` for collapsing invalid values into `None`, and requires bulk imports to validate everything before writing so no half-import survives. Three regression tests lock it down: malformed value rejected with line / column / value in the message, zero samples persisted after failure, and a blank cell still staying `None` with a successful import. The audit entry point now covers parsed value content, not just argument parsing.

**Evidence:** TDD red before the fix: `import_csv` returned `Ok("imported 2 samples")` for both `likes=abc` and `comments=N/A` and the two new tests failed. After the fix, real binary: `radar import trends.csv` (bad likes) → rc=1 `Error: invalid csv: line 3: invalid unsigned integer in 'likes' column: "abc"`, with only the input CSV in the isolated directory and zero sample files; a valid CSV still exits 0 with `imported 1 samples`. `cargo nextest run --all-features --lib radar::` 86 passed (+3).

**Article angle:** "Not filled" and "filled incorrectly" are completely different user intents. Merge them into the same `None` and the program loses its ability to complain — this kind of silent downgrade is harder to spot than a crash, because stdout looks perfectly normal.

### `moonpub humanize` panicked on mixed Chinese/English prose: a 30-byte fallback window was not aligned to a character boundary

**Symptom:** Real-binary probe: running `moonpub humanize <article.md>` on text containing `——确保abcdefghij中文测试内容继续写下去` (a dash explanation with no `。`, `；` or newline after it) exits 101 with `panicked at src/humanize.rs:133:13: end of range should be a character boundary`, leaving the article untouched; the same article with an ordinary Chinese sentence exits 0. `humanize` is a documented local command also reached via `render --humanize`, and technical writing full of "——确保 API and Webhook ..." easily hits this shape.

**Root cause:** `src/humanize.rs::break_parallelism` first looks for `。；\n` to end a `——确保` explanation. When none is found it uses `start + 30.min(out.len() - start)` as the deletion end. `start` comes from `find("——确保")` and is always a boundary, but `+30` is a **byte** count: ten leading ASCII bytes plus three-byte Chinese characters place byte 30 inside the third Han character, so `out.replace_range(start..segment_end, "")` panics. This is another shape of the fixed-byte-offset bug (a byte window) that was already banned in `AGENTS.md` when `radar/scrape.rs` was fixed, but the earlier audit only scanned markdown / theme / scrape modules and missed `humanize.rs` — it had just six tests, none covering the no-terminator fallback.

**Fix:** The fallback now walks `out[start..].char_indices()`, accumulates `ch.len_utf8()`, and takes the last character boundary within 30 bytes as `segment_end`; the path with a terminator is byte-for-byte unchanged. MSRV 1.88 has no `floor_char_boundary`, so no new API shortcut. The file's other byte slice, `start + end + 6` (6 is exactly the UTF-8 length of 「而且」), was verified to always land on a boundary and was left alone.

**Prevention:** The `AGENTS.md` string-boundary rule now names `break_parallelism` and states that any "take an N-byte window then slice / replace" logic must first align via `char_indices()` / `is_char_boundary`. Two new tests lock it down: the exact ASCII + Han trigger, and a table covering emoji, pure Chinese, pure ASCII and short text through the `——确保` fallback. Both call `humanize()` directly, so any later phase change reintroducing the panic fails immediately.

**Evidence:** Before: real binary rc=101 with `end of range should be a character boundary`, ordinary control article rc=0; the two TDD tests failed on the same line. After: the same input exits 0 and produces `测试内容继续写下去`, and a realistic article with API / Webhook mixed text and no terminating punctuation exits 0 with the explanation correctly removed; `humanize::` 8 passed (+2).

**Article angle:** "If there is no punctuation, take 30 bytes" sounds perfectly reasonable — until byte 30 lands inside a Han character. The UTF-8 trap is not only `&s[..N]`; any "offset plus a byte constant" window needs alignment before slicing.

### The WeChat digest's 120-character limit was treated as 120 bytes: preflight said pass, draft.json held only 40 characters

**Symptom:** A frontmatter `digest` of 100 Han characters (100 chars / 300 bytes) still made `moonpub render` exit 0 and print `rendered`, but the generated `<slug>.draft.json` contained only the first 40 characters. Running `moonpub preflight` on the same article printed `digest_limit [pass]: digest within 120-char limit (100 chars)`. `render`, `push` and `ship` all go through `build_draft_json`, so the truncated digest reached the WeChat draft while the quality gate kept reporting success — with no hint in stdout or stderr about a unit mismatch.

**Root cause:** `src/render.rs::build_draft_json` used `120usize.min(digest.len())` as the cut point, and `digest.len()` counts **bytes**. The following `while !digest.is_char_boundary(end) { end -= 1; }` only walked back to a legal boundary to avoid a panic; it did not change the fact that 120 bytes is roughly 40 Han characters. Meanwhile `src/preflight.rs` validated the same limit with `d.chars().count()` and even documented that the limit is character-based (not UTF-8 bytes). The checker and the writer disagreed on units, so the artifact could silently diverge from the gate's verdict.

**Fix:** Switched to character semantics: `match digest.char_indices().nth(120)` takes the byte offset of the 121st character (the boundary after 120 characters) as `&digest[..byte_idx]`; with 120 characters or fewer it returns the original string byte-for-byte. `char_indices()` guarantees a valid boundary, so MSRV 1.88 (no `floor_char_boundary`) is fine. Strings of ≤120 characters — empty, short, or exactly 120 — are unchanged, and `escape_json` plus the JSON shape are untouched.

**Prevention:** `AGENTS.md` now carries an "N characters means characters" rule: `str::len()` returns bytes, the validator and the writer must use the same character-based API (`chars().count()` and `char_indices().nth(N)`), and `N.min(s.len())` plus `is_char_boundary` is banned because it only prevents a panic, not silent truncation. Five tests lock it down: 100 Han characters intact, 115 ASCII + 5 Han at exactly 120 chars intact, 121 Han truncated to 120, a short-digest control, and a pure-ASCII 121 control. Real-binary before/after: 40 → 100 characters, control 2 → 2.

**Evidence:** TDD red first: the three new assertions reported `left: 40 / right: 100`, `left: 116 / right: 120` and `left: 40 / right: 120`; the two controls stayed green. After the fix all seven `build_draft_json` tests pass. Isolated-directory binary probe: before, the `cjk` draft held 40 characters while `control` held 2 and preflight reported pass for the 100-char digest; after, `cjk` holds 100 characters, `control` still 2, and preflight's verdict is unchanged.

**Article angle:** A limit that is checked in one place and written in another is a unit-mismatch waiting to happen. Counting bytes prevents crashes but not silent Han-character truncation; only a shared `chars()` / `char_indices()` vocabulary makes a quality gate's "pass" mean the artifact is actually correct.

### No-space hash headings and table pipes: two layout leaks into WeChat's body

**Symptom:** Real-binary `moonpub render` probes exposed two defects. (1) When a Chinese author writes `#小标题` (no space after the hash), the heading is not recognized and the hash leaks into the article body as a literal `#小标题`; only `# 小标题` renders as a heading. (2) A table cell written as `| a \| b |` (escaped pipe) or `` | 正则 | `a|b` 匹配 | `` (pipe inside inline code) gets split at the pipe: `a \| b` renders as label `a \` plus value `b`, and `` `a|b` `` turns into a bare `` `a `` with the remaining column lost. A header-only table also emits an empty `<section>`.

**Root cause:** Heading detection in `src/markdown/plain.rs` was four `line.strip_prefix("# ")` branches, hard-requiring a space after the hash — CommonMark convention, but not how Chinese authors write. (`C#` / `#tag` in mid-line prose were correctly left alone.) Table cells used a naive `inner.split('|')`, unaware that `\|` is an escape and that a `|` inside backticks is code content, so every pipe became a separator.

**Fix:** Extracted `heading_parts()` as the single judge: count leading hashes (1–4 only; `#####` is not a heading), require non-empty trimmed text, map levels 1/2→h2, 3→h3, 4→h4. Only line-start hashes count, so mid-line hashes are untouched. Tables now use `split_table_cells()`, a char scanner that toggles `in_code` on backticks, folds `\` + `|` into a literal pipe, and only splits on `|` outside code. Headers/rows changed from `&str` to owned `String` to hold unescaped content; header-only tables return an empty string instead of an empty section.

**Prevention:** `AGENTS.md` names `heading_parts()` / `split_table_cells()` as the only entry points and bans falling back to scattered `strip_prefix` branches or naive splits. Three regression tests cover no-space headings (plus mid-line hashes not misread), escaped/inline-code pipes not splitting cells, and header-only tables emitting no section. Post-fix binary probe: `#小标题` → `<h2>无空格标题</h2>` while `C#` / `#话题` stay in prose, `a \| b` displays as `a | b`, `` `a|b` `` renders as a complete code span, and the empty table yields zero sections.

**Evidence:** TDD red first (literal hash body, split `a \` label, empty `<section>`), green after. Probes ran in isolated `/tmp` directories, never touching the user's Obsidian vault; WIP `src/cover.rs` / `src/illustrate.rs` were backed up, temporarily restored to HEAD for probing, then byte-exactly restored with matching sha256 and never committed.

**Article angle:** "A separator is a separator" is an ASCII-era intuition. Once content can contain escapes and code, a naive split tears one user cell into two columns. Headings are the same story — "a hash needs a following space" is the spec, but users won't type fewer typos because you follow it.

### A lone pipe crashes the whole render: endpoint checks are not valid ranges

**Symptom:** Real-binary probe: a single `|` line in an article (a copy-paste remnant, a half-finished table) makes `moonpub render` exit 101 with `panicked at src/markdown/plain.rs:248: byte range starts at 1 but ends at 0`. The article is left completely unchanged; removing that one line makes the same article render fine.

**Root cause:** Both the main loop and `render_table()` identified a table row with `trimmed.starts_with('|') && trimmed.ends_with('|')`. A lone `|` satisfies both ends at once, so it was pushed into `table_buf`; `render_table()` then evaluated `&trimmed[1..trimmed.len() - 1]` with `len == 1` — the range `[1..0]` panics. The previous round made cell splitting escape- and code-aware but never revisited the precondition for entering that splitter. An endpoint check says what the edges look like; it does not guarantee a valid range exists between them.

**Fix:** Extracted `is_table_row(trimmed) -> bool` requiring `trimmed.len() >= 2 && starts_with('|') && ends_with('|')`, shared by the main loop and `render_table()`. A lone `|` now falls through to the ordinary paragraph path and is emitted as `<p>|</p>` — no panic, no silent loss. Valid tables parse and render byte-for-byte identically.

**Prevention:** `AGENTS.md` names `is_table_row()` as the single table-row entry point and bans slicing on an endpoint check alone. Two TDD tests went red with the same panic first (lone pipe in prose; lone pipe between two complete tables) and green after. Post-fix binary probe: the triggering article went from rc=101 to rc=0 with `|` preserved as a paragraph and surrounding prose intact; a normal-table control rendered both rows unchanged.

**Evidence:** 785 → 787 passed (+2), 6/6 quality gates; probes isolated in `/tmp`, never touching the user's Obsidian vault; WIP `src/cover.rs` / `src/illustrate.rs` backed up, temporarily restored to HEAD, then byte-exactly restored with matching sha256 and never committed.

**Article angle:** A predicate that checks only "what the first and last characters are" will admit length-1 input into logic written for length >= 2. The crash wasn't in the table parser — it was in the decision that this line was a table at all. Put the guard on the door, not in every corridor behind it.

### BOM and reversed brackets: two metadata killers at the frontmatter door

**Symptom:** Real-binary probes exposed two defects. (1) An article saved by Windows Notepad starts with a UTF-8 BOM (U+FEFF); `moonpub render` still exits 0 and prints `rendered`, but the `draft.json` title and digest are empty strings, `theme: blueprint` silently falls back to default, and the YAML lines plus the BOM character are rendered into the body. `preflight` then reports `title_required [fail]: title missing` — the user did write a title. (2) Frontmatter with `tags: ]生活[` (brackets reversed) makes `moonpub render` exit 101 with `panicked at src/article.rs:204: byte range starts at 14 but ends at 0`, leaving the article unchanged. A normal article controls at rc=0.

**Root cause:** (1) `parse_frontmatter()` / `strip_frontmatter()` used `md.trim_start()` to find the leading `---`, but BOM is not `char::is_whitespace()`, so `trim_start()` leaves it; `starts_with("---")` fails and the whole frontmatter block is treated as body text. (2) `parse_yaml_string_array()` located the array with `find('[')` and `rfind(']')`; in `]生活[` the `]` comes before the `[`, so `[bracket_start+1..bracket_end]` became `[14..6]` and panicked. Both sit at the frontmatter entry and both assume well-formed input without validating its shape.

**Fix:** (1) Extracted `strip_bom(md)`, called before trimming in both `parse_frontmatter()` and `strip_frontmatter()`; the "no frontmatter / incomplete frontmatter" return paths also strip the BOM so a heading after it can still be found. (2) `parse_yaml_string_array()` checks `bracket_end > bracket_start` before slicing and treats reversed brackets as an empty array. Valid articles are byte-for-byte unchanged.

**Prevention:** `AGENTS.md` names `strip_bom()` as a required frontmatter prelude and requires bracket-order validation before slicing. Four tests added: BOM frontmatter parsed, BOM body does not leak YAML, reversed brackets do not panic, and H1 after a BOM without frontmatter is still extracted. Post-fix binary: title/digest restored, no YAML or BOM char in the body, `preflight`'s `title_required [fail]` gone (`digest_limit` turns pass), reversed-bracket article renders at rc=0, control unchanged.

**Evidence:** TDD four red first (title all None / YAML leak / same panic / H1 miss) then green; 787 → 791 passed (+4). Probes isolated in `/tmp`; WIP `src/cover.rs` / `src/illustrate.rs` backed up, temporarily restored to HEAD, then byte-exactly restored with matching sha256 and never committed.

**Article angle:** "A few invisible characters at the start of a file" and "a user typed the brackets backwards" are not exotic inputs — they are an editor default and an ordinary typo. A parser that assumes well-formed input fails by silently dropping data or crashing, not by reporting an error. Tolerance at the entrance is far cheaper than cleanup afterwards.

## 2026-11-17 Bidirectional bridge tests between the command registry and the parser

**Symptom:** The help registry (`help::COMMANDS`, 44 entries), the CLI parser's `parse_command` arms, and the structured-JSON lists (`supports_subcommand_json` string list plus the `has_structured_json_output` enum list) are four independently maintained inventories. Four separate PRs (#150 / #155 / #160 / #177) patched drift where one place gained a command and another forgot: registered in help but reported unknown by the parser, or documented as `--json`-capable while the flag was silently swallowed as a positional.

**Root cause:** Only one direction was tested (registry → the four docs). Nothing checked "registry ↔ parser" or "JSON list ↔ registry"; the sole related test, `trailing_json_is_stripped_for_every_machine_readable_command`, inlined its own hand-written array and then asserted `supports_subcommand_json(name)` against it — testing a hand-written copy against itself, permanently green even when both sides drift.

**Fix:** In the `src/cli.rs` test module, extracted a shared `JSON_CAPABLE` constant (the old self-referential test now reads it) and added two bridge tests: `every_registered_command_is_recognized_by_parser` walks `COMMANDS` and asserts `parse_command(spec.name, …)` never returns `UnknownCommand`; `every_json_capable_name_is_a_registered_command` walks `JSON_CAPABLE` and asserts every name resolves via `for_command(name)`. Deliberately left `version` out of `has_structured_json_output` — it uses the fallback wrapper by design, not by drift.

**Prevention:** `AGENTS.md` names both bridge tests and the shared constant and forbids self-referential tests. Controlled-mutation evidence: deleting the `version` spec from help turned the reverse bridge red; renaming `status` in help turned the forward bridge red; both restored green. Note that after `cp`-restoring a file, Cargo may reuse a stale object because mtime did not change — `touch` the file or verify the fingerprint before trusting a green run.

**Article angle:** Storing the same fact in three places is not the danger; checking only one direction is. A one-way test proves that when A appears B follows, but it cannot see a ghost entry that exists only in B. A real guard walks both directions, with each side derived from an independent source of truth — a test that checks its own hand-written copy against itself only locks the drift in tighter.

## 2026-11-18 Install links pinned to a version number, three first-run docs ten releases behind

**Symptom:** The install commands in `docs/USER_GUIDE.md`, `docs/USER_ONBOARDING_ZH.md`, and `docs/USER_ONBOARDING.md` all hardcoded `releases/download/v0.4.2/...`, while GitHub Latest and `Cargo.toml` were both already `v0.4.13` (published 2026-11-06). A newcomer following any of those three documents installs the 2026-06-23 binary, ten releases behind — and those exact documents call themselves "the user manual" and "the complete user guide". In the same pass `ROADMAP.md` reported `v0.4.12` as the current source version, and the `docs/PROGRESS.md` stub still listed `v0.4.2` / `v0.4.1`.

**Root cause:** The same class of bug was patched on 2026-11-06 (`docs/index.html`, `docs/slides.html`, `GETTING_STARTED.md` moved to `releases/latest/download/`), but that pass was a file-by-file edit with no machine-readable rule left behind, so `USER_GUIDE` and both `USER_ONBOARDING` files were missed; `ROADMAP.md` and `docs/PROGRESS.md` were outside its scope and kept drifting. `docs/PROGRESS.md` is the textbook case: it exists only to redirect old links, yet it also carried a copy of the "current summary" — a second source of truth for the same facts.

**Fix:** All three entry documents now use `releases/latest/download/<asset>` (GitHub's `latest` alias follows the newest release, so no version number needs editing) with wording that says so and mentions the matching `.sha256` per archive. `ROADMAP.md`'s current source version was corrected to `v0.4.13`. The `docs/PROGRESS.md` stub lost its entire "current summary" block and keeps only "look at the root `PROGRESS.md`" plus three self-check commands (`moonpub --version`, `--json doctor`, `--json workspace`) — the second source of truth was deleted, not synchronized.

**Prevention:** New `tests/doc_version_contract.rs` with three contracts: (1) recursively scan every `.md` / `.html` in the repo and fail on `releases/download/vX.Y.Z` in a live document (snapshots pass via the `RELEASE_NOTES_v*` / `RELEASE_GATE_v*` filename prefixes plus an explicit `LAUNCH_ARTICLE_ZH.md` allowlist, and each allowlist entry must state why it cannot follow the version); (2) `ROADMAP.md`'s "current source version" must equal the compile-time `env!("CARGO_PKG_VERSION")`, so a release bump that forgets this line goes red immediately; (3) `docs/PROGRESS.md` may not mention "当前源码版本" again, forbidding a summary copy-paste. Controlled mutation: reverting `USER_GUIDE` to `v0.4.2` and `ROADMAP` to `v0.4.12` turned both contracts red with the exact file, line, and fix in the message; restoring brought all three back green.

**Evidence:** `docs/USER_GUIDE.md`, `docs/USER_ONBOARDING.md`, `docs/USER_ONBOARDING_ZH.md`, `ROADMAP.md`, `docs/PROGRESS.md`, `tests/doc_version_contract.rs`; GitHub Latest verified as `v0.4.13` (2026-11-06), `Cargo.toml` and `.release-please-manifest.json` both `0.4.13`.

**Article angle:** Switching to the `latest` link is a fix; making the old pattern fail CI is the actual repair. When the same problem returns after being "fixed", the usual cause is that only the files changed, not the rule. And a document that calls itself a stub while still carrying a data copy becomes a second source of truth — the remedy is deleting it, not keeping it in sync.

## 2026-11-18 Nested lists flattened to one level, blockquote lists leaking a literal `-`

**Symptom:** Two defects in one probe article, both visible in WeChat:

1. A top-level `-` item followed by a bullet indented two spaces rendered as two paragraphs with the same `padding-left:1.2em` — the child looked exactly like its parent, so the hierarchy was unreadable. Under an ordered parent, `1.` / `2.` children came out as `1. 1. 2. 2.` at the same level, reading like a numbering mistake.
2. `> - 引用里的列表` inside a blockquote printed the literal text `- 引用里的列表`; the raw `-` marker leaked into the article body.

**Root cause:** Both come from who is responsible for reading indentation and markers. The list loop passed `line.trim()` into `parse_list_item()` / `parse_checklist_item()`, so the indentation was gone before parsing began — and indentation is the *only* hierarchy signal a list has; once `trim()` runs, the level is unrecoverable. The blockquote path was blunter: `render_blockquote()` fed every line straight to `inline_md()`, but `-` and `1.` are block-level markers. The inline parser was not wrong, it simply did not recognise them, so it echoed them verbatim.

**Fix:** Parsers now take the raw line; `indent_columns()` counts leading whitespace (tabs to the next 4-column stop) and `ListItem` / `ChecklistItem` carry `indent`. Depth is resolved at render time by an **indentation stack** (`push_indent()`): deeper than the stack top pushes a level, shallower pops back — so 2-space children of a `-` bullet, 3-space children of a `1.` item, and 4-space indentation from an editor expanding a tab all advance exactly one level, instead of "divide indentation by a fixed step" sending 4 spaces two levels down. Depth is capped at three (`MAX_LIST_DEPTH`) because the phone column runs out of room. `render_blockquote()` now recognises list markers per line, renders bullets / numbers, and expresses depth with `&nbsp;&nbsp;` (a blockquote card has no hanging indent, and empty spans get stripped by the WeChat editor).

**Prevention:** Six TDD cases: a child must use `padding-left:2.5em`; a 3-space ordered child must nest; levels four and five both clamp to 5.1em; 4 spaces must be one level, not two; blockquote items must not contain the literal `- 第一条` and must contain a bullet; a plain blockquote must not grow bullets. Controlled mutation, three groups: measuring indentation on the `trim()`-ed line turned 4 cases red; reverting the blockquote path to plain `inline_md` turned 2 red; dropping `.min(MAX_LIST_DEPTH)` turned the clamp case red; after restoring (sha256 verified) all 13 were green. The real article `ready/gzh-tools-roundup-2026.md` renders **byte-identically** before and after, confirming the change touches only nested and blockquote lists.

**Evidence:** `src/markdown/plain.rs` (`indent_columns` / `push_indent` / `resolve_depths` / `parse_list_item` / `parse_checklist_item` / `render_blockquote` / `LIST_PADDING` / `CHECKLIST_PADDING`). Post-fix binary on the same probe: indentation went from `1.2 / 1.2 / 1.2` to `1.2 / 2.5 / 2.5`, and the blockquote went from literal `- 引用里的列表` to `• 引用里的列表` with an indented `• 引用里的二级`.

**Article angle:** Indentation is not "extra spaces" — it is the only hierarchy signal a list carries, and the moment `trim()` is called the list is already flat; no downstream code can restore the level. The second defect is the more instructive one: `-` is a block-level marker, and when it reaches an inline parser nothing goes wrong — the parser simply does not recognise it and echoes it. Both ends of the pipe behaved "correctly", and the semantic gap in the middle became a literal hyphen in the reader's article.

## 2026-11-18 Link titles stuffed into `href`, autolinks mangled into `<<a href="…&gt;">`

**Symptom:** A probe article covering many Markdown constructs produced two defects that read as "the link does not work / something looks off":

1. `[示例](https://example.com "标题文字")` rendered `href="https://example.com &quot;标题文字&quot;"` — in GFM that is "URL plus title", but the title became part of the URL and the link is dead. Images had the same disease: `![替](/a.png "图片标题")` emitted `src="/a.png &quot;图片标题&quot;"`, so the image never loads.
2. `<https://example.com>` rendered `<<a href="https://example.com&gt;" …>https://example.com&gt;</a>` — a stray `<` in the body and `&gt;` hanging off the end of `href`.

**Root cause:** Both are "a syntax marker was never stripped from the data". (1) `parse_link()` / `parse_image()` treated everything between `(` and `)` as the destination, but the GFM grammar is destination **plus optional title**; when the parentheses contain whitespace followed by a quote, the implementation cannot tell that it is a title. (2) Autolinks had no branch in `inline_md()`: `<` was emitted as an ordinary character, then `starts_url()` recognised `https://` one position later, and `take_url()` did not treat `>` as a terminator, so `>` was swallowed into the URL.

**Fix:** New `split_destination()` — a `<...>` destination gets its angle brackets stripped while preserving inner spaces; otherwise everything from the first whitespace onward is dropped as a title (a `title` attribute is invisible in WeChat and the editor strips it anyway, so discarding it beats emitting a half-dead attribute). New `parse_autolink()` accepts only `<` + `http(s)://` + no whitespace + `>`, and emits a link without the angle brackets. `take_url()` gained `<` and `>` as terminators so bare URLs stop swallowing ASCII brackets.

**Prevention:** Seven TDD cases: a double-quoted title stays out of `href` and out of the body; same for a single-quoted title; an image title stays out of `src` and the whole construct is consumed; an autolink has no `&gt;` in either `href` or link text and leaves no bare `<`; spaces inside a `<…>` destination survive; a title-less link is unchanged; `a < b` in prose must not become an autolink. Controlled mutation, two groups: making `split_destination()` return its input turned 3 red; making `parse_autolink()` always return `None` turned the autolink case red; after restoring (sha256 verified) all 33 inline cases were green. The real article `ready/gzh-tools-roundup-2026.md` renders byte-identically before and after.

**Evidence:** `src/markdown/inline.rs` (`split_destination` / `parse_autolink` / `parse_link` / `parse_image` / `take_url`). Post-fix binary on the same probe: `href="https://example.com&gt;"` is gone, `<<` is gone, `src="/a.png &quot;` is gone, and both links carry a clean `https://example.com`.

**Article angle:** The assumption "everything inside the parentheses is the link" stops being true at the first quote character. A parser rarely fails by crashing; it fails by **treating a longer, legal grammar fragment as a single value** — the caller receives a usable-looking string that merely contains something it should not, and the error travels all the way to the moment the reader taps a dead link. The judgement has to move from "what did I capture" to "what is the complete shape of this syntax": a destination is a URL plus an optional title, not "whatever sits inside the parentheses".

## 2026-11-18 Triple emphasis leaks bare asterisks, h5/h6 print their hashes

**Symptom:** The same probe article had two more "raw marker leakage" defects:

1. `***粗斜***` rendered as `<strong style="…">*粗斜</strong>*` — two bare `*` in the body, one of them inside the bold run.
2. `##### 五级标题` / `###### 六级标题` were emitted as ordinary paragraphs, hashes and all.

**Root cause:** Both widen a narrow predicate and leave the excluded case with no fallback. (1) The `**` branch sat before `***` in `inline_md()` and closes with `windows(2)`: on `***粗斜***` the first `**` matches, the inner text becomes `*粗斜`, the closer stops at the second-to-last `*`, and one asterisk is left over. (2) `heading_parts()` accepted only 1–4 hashes while CommonMark defines six levels, so levels five and six fell through to the paragraph path.

**Fix:** A `***` branch placed before `**` (closing with `windows(3)`, emitting nested `<strong><em>`), and `heading_parts()` widened to `1..=6` with levels five and six styled as h4 — WeChat has no fifth subheading tier, and degrading to h4 beats printing hashes at the reader. Seven hashes remain body text (past CommonMark's six).

**Prevention:** Four TDD cases: `***粗斜***` contains no bare `*` and carries both `<em>` and `<strong>`; mid-sentence triple emphasis also leaks nothing; h5/h6 render as `<h4>` with no hashes; seven hashes are not a heading. Controlled mutation, two groups: disabling the `***` branch turned 2 red; reverting `1..=6` to `1..=4` turned the heading case red; after restoring (sha256 verified) everything was green. The real article `ready/gzh-tools-roundup-2026.md` renders byte-identically.

**Evidence:** `src/markdown/inline.rs` (the `***` branch), `src/markdown/plain.rs` (`heading_parts`). Post-fix binary on the same probe: bare `*` count 0, `#####` count 0, output `<strong style="color: #1a1a1a;"><em>粗斜</em></strong>`.

**Article angle:** The most dangerous parser failure is not an error, it is **matching a shorter prefix**. Both `**` and `***` are legal; matching the short one does not fail, it just leaves the extra character behind as noise. Branch order *is* grammar precedence, so order checks longest-first. The second defect is the same problem from outside: a predicate that enumerates too few cases (only four heading levels) does not error on the rest, it silently downgrades them to body text — any finite enumeration needs a defined home for values outside it.

## 2026-11-18 `init` writes a config it cannot read back — first `render` fails with invalid theme name (v0.4.14 was withdrawn)

**Symptom:** The v0.4.14 release build failed across the macOS / Linux packaging smoke tests (`init` → `new` → `render`):

```text
Error: invalid theme name: "default" # default | warm | dark | ...
```

The error message itself lists `default` as the first legal value. Reproduced locally: `moonpub init --yes` followed by `moonpub render` exits 1.

**Root cause:** The config reader is a hand-written minimal TOML parser. `split_toml_pair()` only stripped quotes when the whole line ended with a quote:

```rust
let value = rest.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(rest);
```

But the config `init` writes carries trailing documentation: `theme = "default" # default | warm | ...`. The line ends with `gruvbox`, so `strip_suffix('"')` fails and `unwrap_or(rest)` keeps `"default" # default | ...` as the value. `auth_method = "appsecret" # ...` and `variant = "community" # ...` carry the same defect.

The polluted value had existed for a long time, hidden by "unknown theme silently falls back to default". This round turned theme names into a closed enum and made unknown names an error, so the dirty value became a hard failure. Meanwhile the `init` test only asserted that the *file content contains* `theme = "default"` — it never read the generated config back.

**Fix:** New `parse_toml_value()`: a quoted string is read up to its matching closing quote (`\"` does not close it, and a `#` inside quotes is not a comment); an unquoted value is cut at the first whitespace followed by `#`; an unterminated quote falls back to the raw text.

**Prevention:** Five unit cases (trailing comment, unquoted boolean with comment, `#` inside quotes, escaped quote, sample config round-trip) plus a new `tests/first_run_contract.rs` with two process-level contracts: `init --yes` then `render` must exit 0 and produce HTML; `check --json` must report `effective_theme == "default"`. Controlled mutation, two groups: reverting the parser turned all 5 unit + 2 contract cases red; cutting at the first `#` unconditionally turned 4 red; after restoring (sha256 verified) everything was green. This path previously ran only in `release.yml`'s packaging smoke — i.e. "discovered halfway through a release build"; it now runs in every PR's `test` job.

**Evidence:** `src/config.rs` (`parse_toml_value` / `split_toml_pair`), `tests/first_run_contract.rs`; the linux-arm64 smoke failure in release run 44234295565; local `init --yes` + `render` reproduction and exit code 0 after the fix.

**Article angle:** A textbook case of silent fallback: the dirty value existed for ages because a downstream "unknown means default" path absorbed it, so nobody noticed the parser had never actually worked. Replacing the fallback with an explicit error made the problem visible at once — **the error is not the bug; the error is the step that exposes it**. The deeper lesson is the second layer: any file a program writes itself must be read back in a test. Asserting that the written content contains some string only verifies `format!`, not the parser.

## 2026-11-18 Block-level constructs inside blockquotes all degrade: nested quote leaks `>`, code block eats the language tag, heading leaks `##`

**Symptom:** A probe article dedicated to blockquotes failed in four places at once:

1. `> > 内层一句话` rendered as "外层一句话。> 内层一句话。" — the inner `>` printed as body text.
2. A fenced block under `> ```js` became an inline code span with the language tag welded into the code: `` `jsconst a = 1;` ``.
3. `> ## 小标题` printed `## 小标题` as a paragraph.
4. `>无空格引用` (legal CommonMark) was not recognised at all and kept its literal `>`; a bare `>` line inside a quote was dropped entirely, merging two paragraphs into one.

**Root cause:** `render_blockquote()` treated the quote body as "a run of inline text" — line by line through `inline_md()`, joined with `<br>`. So no *block-level* construct inside a quote had a parser: `>` is not an inline marker, so it passed through silently; a fence is not an inline marker, so only the backtick branch matched and the language tag plus code became one inline span; `##` likewise. The previous round patched only lists (hand-writing bullets and numbers inside the quote), producing "half a Markdown implementation" living inside blockquotes — lists rendered, everything else still leaked. On top of that, `strip_prefix("> ")` required a space, so `>引用` was not a quote at all.

**Fix:** When the quote body contains block structure, the quote no longer renders itself: it **strips one level of `>` and hands the body back to `render_markdown_segment()`**, so nested quotes, code blocks, headings, lists and tables all come from the single block parser. A plain-text quote (no block markers, no blank lines) keeps the "per-line `inline_md` + `<br>`" fast path and its existing compact layout. New `strip_quote_marker()`: the space after `>` is optional and exactly one level is stripped; a bare `>` line is preserved as a blank line so the quote renderer sees a paragraph break.

**Prevention:** Five TDD cases: a nested quote produces at least two quote cards and its visible text contains no literal `> 内层`; a code block inside a quote must emit `font-family:Menlo` and must not contain `jsconst`; a heading inside a quote must render as `<h2>`/`<h3>`/`<h4>` with no `## 小标题`; `>无空格引用` must be recognised; two paragraphs inside a quote must produce two `<p>`. The tests gained a `visible_text()` tag stripper — the first version also swallowed a literal `>` sitting right after a tag, so the "leaked `>`" assertions could never fail, which the controlled mutation exposed. Three mutation groups: reverting the body to per-line `inline_md` turned 6 red (including both list cases); requiring a space after `>` turned 2 red; dropping bare `>` lines turned 1 red; after restoring (sha256 verified) all 10 were green.

**Evidence:** `src/markdown/plain.rs` (`strip_quote_marker` / `quote_body_needs_blocks` / `render_blockquote` / the quote accumulation branch). Post-fix binary on the same probe: literal `>` count 0, `## 引用里的标题` count 0, `font-family:Menlo` present, `jsconst` gone, five quote cards (the nested one drawn as two layers). Corpus regression across all 82 markdown files in the repo: only `docs/USER_ONBOARDING.md` and its `_ZH` twin changed — their source separates two paragraphs with a bare `>`, which `<br>` used to fuse and now correctly becomes two `<p>`.

**Article angle:** A textbook case of growing a parallel implementation to fix one symptom: when a list leaked its `-`, the cheapest fix was to hand-write bullets and numbers inside the quote — and that gave blockquotes their own half-finished Markdown, with every remaining construct still leaking silently. The right question is "which blocks is this container allowed to hold?", and the answer is "all of them", which means handing the body back to the one parser instead of writing a second one inside the container. Item 4 is the same lesson from the other side: a predicate stricter than the spec ("a space must follow `>`") shuts a whole class of legal input out, and excluded input turns into body text quietly.

## 2026-11-18 Reference-style links: definition lines printed as body text, `[text][label]` never resolved

**Symptom:** Rendering the repo's own `CODE_OF_CONDUCT.md` produced two visible defects in one sentence: `This Code of Conduct is adapted from the [Contributor Covenant][homepage],` left the raw marker in the paragraph (no link at all), and the definition line at the bottom became a standalone paragraph reading `[homepage]: https://www.contributor-covenant.org` with the URL auto-linked as if it were prose.

**Root cause:** The inline layer only knew one link form: `[text](url)`. Reference-style links are a two-part construct — a marker in the text plus a definition line anywhere in the document — and nothing in the pipeline collected the second part. So the marker fell through to the "unrecognised text" branch and kept its brackets, while the definition line was just another line of Markdown to the block parser: it became a paragraph, and the bare URL inside it tripped the autolink rule. Both halves of the construct were being rendered as content.

**Fix:** A pre-pass in `render_markdown_segment()`. `collect_link_refs()` walks the segment, lifts every `[label]: url` line out of the body, and builds a `LinkRefs` map; the block parser then only ever sees body text. `parse_link()` gained the other two forms — `[text][label]`, `[text][]` (collapsed: the text is its own label) and `[label]` (shortcut) — and resolves them against the map. Resolution is **conditional**: an undefined label returns `None` and the brackets stay literal, so `[0]` or `[x]` in prose is never eaten by a syntax that merely looks similar. Labels compare through `normalize_label()`: case-insensitive with internal whitespace collapsed, per CommonMark, otherwise `[Homepage]` and `[homepage]` silently become a dead pair. The map is threaded down recursively so a quote (`> 见[文档]`) can use a definition written outside it, with the segment's own definitions winning.

**Prevention:** Ten TDD cases written before the fix: definition line produces no body text; full / collapsed / shortcut forms each resolve; an undefined label stays literal; labels ignore case and extra spaces; a definition inside a code fence is left alone; a blockquote inherits the outer map; a reference image resolves; and the real two-line `CODE_OF_CONDUCT` excerpt. One assertion was proven weak by mutation and had to be replaced: `!visible_text.contains("[homepage]")` stayed green with the leak reintroduced, because once the definition is in the map a bare `[homepage]` resolves into a link and **the brackets themselves disappear** — the test could no longer see them. Counting paragraph elements (`<p` occurrences == 1) is what actually detects the leaked line. Three mutation groups: keeping the definition line in the body turned 2 red; degrading `normalize_label` to an exact match turned exactly 1 red; disabling the shortcut form turned 2 red. After restoring (sha256 verified, `touch` to defeat Cargo's stale-target reuse) all 12 were green.

**Evidence:** `src/markdown/plain.rs` (`collect_link_refs` / `parse_link_definition` / `render_segment_with_refs` / threaded `LinkRefs`), `src/markdown/inline.rs` (`parse_link` / `parse_image` / `normalize_label`). Real binary on `CODE_OF_CONDUCT.md`: the attribution sentence now yields `<a href="https://www.contributor-covenant.org">Contributor Covenant</a>` and the definition line is gone entirely. Corpus regression across 536 markdown files (repo + a `node_modules` tree left as extra input): 2 files changed, both correct.

**Article angle:** A link definition is *data about the document*, not text in it — and the parser had no place to put data, so it treated it as text. The interesting part is the second-order effect: once the definition became resolvable, every bracket pair in the document gained a new possible meaning, so the fix had to be guarded by "only when the label actually exists". Any time a syntax becomes stateful, the blast radius is every place where the same characters were previously inert.

## 2026-11-18 Link text truncated at the first `]`, so `[![badge][img]][url]` rendered as garbage

**Symptom:** Found by the corpus diff run for the reference-link fix, not by a user report. `obsidian-plugin/node_modules/moment/README.md` changed from literal `[![NPM version][npm-version-image]][npm-url]` to `<a href="…v.svg">![NPM version</a>]<a>npm-url</a>` — worse than before: the anchor pointed at the *badge image* URL, and the remnants `]`, `![` and a second anchor spilled into the paragraph. The same file shape appears in four other READMEs in the corpus (typescript, two codemirror packages), and the inline variant `[![a](x.png)](https://x)` failed identically **before** this round started.

**Root cause:** `parse_link()` located the end of the link text with `position(|&c| c == ']')` — the *first* closing bracket. Link text is allowed to contain a nested bracket pair, and the clickable-badge idiom `[![alt][img]][url]` nests an image *inside* a link, so the first `]` belongs to the image. The text was cut to `![NPM version`, and the rest of the construct was re-parsed as unrelated inline syntax. The bug was pre-existing but invisible in practice: before reference links worked, the whole construct rendered as inert literal text, which looked "not yet supported" instead of "wrong".

**Fix:** A shared `matching_bracket(chars, open)` helper that walks from a `[` counting nesting depth and honouring `\]` as a literal, returning the paired `]`. Both `parse_link()` and `parse_image()` now use it, so the text is the complete nested construct and is then handed back to `inline_md` — which renders the inner image through the same reference map. Output is now the correct `<a href="link"><img src="badge" alt="alt"></a>`.

**Prevention:** Two TDD cases, both red first: the reference form `[![NPM version][img]][url]` must yield an anchor whose `href` is the *link* target and an `img` whose `src` is the *badge* URL with no `][` left in the visible text; the inline form must keep both the outer `href` and the inner `src`. The real corpus is the third check — 536 files diffed byte-for-byte before and after, and every one of the four badge READMEs now renders as a proper nested anchor.

**Article angle:** The cheapest fuzz test this project has is rendering its own corpus twice and diffing. This defect was invisible to the feature's own test suite and to the file the round was aimed at; it only appeared because "did anything else change?" was asked. The deeper point: adding a syntax feature **re-enables code paths that were previously inert**. `[![a][b]][c]` always had this bug, but nobody could see it while the construct was just text. When a syntax starts working, re-check the neighbouring parsing code — the bug was sleeping there, waiting for the feature to make it reachable.

## 2026-11-18 `status` printed a filename its own `render` rejected

**Symptom:** A fresh workspace, one article, three commands — and the third one failed. `moonpub new "我的第一篇文章"` created `Articles/drafts/我的第一篇文章.md`; `moonpub status` listed it as `我的第一篇文章.md`; pasting that back as `moonpub render 我的第一篇文章.md` returned `Error: /…/我的第一篇文章.md: No such file or directory (os error 2)`. Dropping the extension did not help either — `article path must point to a .md file`. The tool was printing a name that was not a valid argument to the tool. An empty workspace was the other dead end: three lines of `(empty)` and exit code 0, no hint of what to do next.

**Root cause:** Two lists had grown apart. `status --json` resolves the next action through a branch that formats `Articles/drafts/{file}` — a usable path — while `format_status()`, the human rendering, printed `file.file` alone. The same four-branch "what next" logic existed in two places (`status_json()` and `next_workspace_action()`), byte-identical, so nobody noticed that the third consumer — the text output — had been written before either of them and never gained the path prefix or the `next:` line. The text view was treated as decoration rather than as an interface.

**Fix:** `format_status()` prints `Articles/<stage>/<file>` for every row, pushing the status tag (`[rendered] ok`) after the path so "first whitespace-delimited token" is always a valid argument. It then appends the same `next:` / `step:` pair that `workspace` already showed. The duplicated logic moved into `crate::status::next_action()`, which `status` text, `workspace` text, and `status --json` all call — one opinion, three surfaces. An empty workspace now ends with `next: moonpub new "你的第一篇文章"` instead of three `(empty)` lines.

**Prevention:** Two TDD cases. The important one is a **round-trip**: parse the printed list, take the first token of every article line, and feed it back into `check_article()` — if the tool cannot read its own output, the test fails. The second asserts the empty workspace carries `next: moonpub new`. Controlled mutation: reverting the path prefix turned exactly the round-trip test red; removing the `next:` lines turned exactly the empty-workspace test red. Both restorations verified by sha256. The README's 30-second block was then executed verbatim in a live temp directory (English and Chinese slugs) rather than being read for plausibility.

**Article angle:** A CLI's output *is* its API — the moment a human (or an agent) copies a line and runs it, decoration became contract. `status --json` was already correct and useless to the person looking at the terminal; the machine was better informed than its user. The generalisable check is cheap: **take whatever you print, parse it back, feed it to your own entry point.** A command that cannot consume its own output is not finished, no matter how nice the table looks.

**Evidence:** `src/status.rs` (`next_action`, `format_status`, `status_text_prints_paths_that_are_valid_arguments`, `status_text_gives_next_step_when_workspace_is_empty`), `src/protocol/workspace.rs` (both call sites collapsed into `next_action`). Real binary: empty workspace now ends with `next: moonpub new "你的第一篇文章"`; after `new`, the printed `Articles/drafts/我的第一篇文章.md` renders successfully when pasted verbatim. 852 → 854 tests, coverage 79.08%.

## 2026-11-18 The recommended next step was a read-only dead end

**Symptom:** After `moonpub new` the status list ended with `next: moonpub check Articles/drafts/x.md`. Following it printed four lines and `publishable: no`, then stopped — exit code 0, no suggestion. Two hops, zero progress. Meanwhile `check --json` for the very same bundle returned `next_command: "moonpub render …"`, so the machine knew what to do and the person at the terminal did not. The chain broke in two more places: `moonpub new` printed only `created <path>`, and `moonpub render` printed only `rendered` plus two file paths — the two commands a beginner runs first were the two that said nothing about what came next.

**Root cause:** "Check" was mistaken for "advance". `status` picked the first draft and recommended `check` because that is the safe read-only command — but a read-only command cannot move a workspace, so recommending it just relocates the dead end one hop later. The recommendation also had no idea whether the article had ever been rendered: `status` only sees `.moonpub/status.jsonl`, which may be absent (no `new` writes there) or stale, while the ground truth is simply whether `<slug>.html` exists. And on the `check` side the bundle report printed `publishable: no` as the last line, dropping the four-branch next-step logic that `check --json` already ran — the same text/JSON divergence the `status` fix had just removed elsewhere.

**Fix:** `StatusFileEntry` gained `has_html`, filled from the filesystem rather than the status log; an unrendered draft now recommends `moonpub render`, a rendered one still recommends `check`. The bundle's four-branch judgement moved into `ArticleBundle::next_step()` and both the text report and `check --json` call it. `new` now ends with `next: moonpub render …`, `render` with `next: moonpub preview …`, and `preview` — which already had a `next:` but no `step:` — was brought in line. The onboarding chain now reads `new` → `render` → `preview` → `push`, each hop naming the next.

**Prevention:** Three TDD cases: an unrendered draft must recommend `render` and a rendered one must recommend `check` (with a real `.html` created between the two assertions); the bundle report must say `render` when `draft.json` is missing and `push` once it is complete; `new` and `render` outputs must contain their successor command. Controlled mutation: dropping the next lines from the bundle report turned exactly the two report tests red; ignoring `has_html` turned exactly the draft-recommendation test red; removing both `new`/`render` next lines turned exactly those two tests red. Restorations verified by sha256. The whole chain was then walked with a release binary in a temp workspace, each command copied from the previous command's `next:` line.

**Article angle:** A recommendation is a promise. The question to ask of any "next step" is not "is this command safe?" but **"after running it, has the world moved?"** — a read-only check is a fine inspection tool and a terrible next step. The second, cheaper question: does the surface that tells humans the next step read it from the same place as the surface that tells machines? Here the JSON had been right for months while three text commands withheld it.

**Evidence:** `src/status.rs` (`StatusFileEntry::has_html`, `next_action`), `src/bundle.rs` (`BundleNextStep`, `ArticleBundle::next_step`, `report`), `src/protocol/check.rs` (delegates to `next_step`), `src/draft.rs`, `src/render.rs`, `src/preview.rs`. Real binary: `new` → `render` → `check` → `status` each printed a `next:` that the next command accepted verbatim. 854 → 859 tests, coverage 79.26%.

## 2026-11-18 Missing WeChat credentials: the error named the gap, `doctor` never looked

**Symptom:** A newcomer walks `init → new → render → preview`, then the first `push` answers with one line: `Error: missing env var: WECHAT_APPID`. Nothing about where that value lives, nothing about the cookie channel that needs no AppSecret at all. `push --json` points `next_step` at `moonpub doctor`, and `doctor` never inspected credentials — even though its own `first_check` promises "environment variable sources (env > .env > toml)". Worse, on the same workspace with the same `WECHAT_AUTH_METHOD=COOKIE`, `ship` took the cookie path while `push` fell back to appsecret and demanded an AppSecret; and once the toml carried `auth_method = "appsecret"` (what `init` writes by default), `WECHAT_AUTH_METHOD=cookie` was ignored outright.

**Root cause:** Three paths decided independently. (1) The error carried the fact but not a way out, on the one step where the user must fetch something from outside the machine. (2) `doctor` defined itself as a local-readiness check and left credentials out of "ready". (3) The auth_method decision existed twice: `push` had `cfg.wechat_auth_method.or_else(env)` plus `== "cookie"` (toml first, case-sensitive), `ship` had env-first plus `eq_ignore_ascii_case`. Same environment, opposite conclusions — the user could only find out by trying.

**Fix:** The decision lives in `Config::is_cookie_auth(env)` (non-empty env overrides toml, case-insensitive, surrounding whitespace ignored) and both `push` and `ship` call it. `MissingEnvVar` now prints the fact line and then two copyable paths: cookie mode (`moonpub login`, then `WECHAT_AUTH_METHOD=cookie moonpub push <article.md>`) and AppSecret (WeChat backend → Development → Basic Configuration, two lines into `.env`); non-WeChat variables get one generic line instead of the WeChat tutorial. `doctor` gained a `wechat_auth:` line (text plus a `wechat_auth` field in `doctor/v1`) that states which channel `push` will use, which variable is missing, and both ways out.

**Prevention:** Unit tests lock four things: env cookie beats a toml `appsecret`; `COOKIE` and a whitespace-padded `Cookie` mean the same as `cookie`; empty env falls back to toml and no setting at all means appsecret; `wechat_auth_status_with` covers all four branches (both missing, secret missing, appid missing, ready — including "an appid in the toml counts, whitespace does not"). The `doctor` line states facts and adds no warning on purpose: warning counts depend on the machine's environment, so the same workspace would print a different structure per host and the tests would go red on other people's laptops. Release-binary re-check: credential-less `push` prints both paths; `WECHAT_AUTH_METHOD=COOKIE` with toml `appsecret` now takes the cookie path and asks for `moonpub login`; `doctor` prints missing / ready / cookie for the three states.

**Article angle:** "What is missing" is a fact; "how to fix it" is the help. An error that only states facts at the one point the user cannot resolve alone is a locked door with the key hidden. The second layer is the more interesting one: the `next_step` handed to machines pointed at a command that did not know the answer either — whoever recommends a place is responsible for that place actually answering.

**Evidence:** `src/config.rs` (`Config::is_cookie_auth` plus four tests), `src/push.rs` (delegates the decision), `src/ship.rs` (same decision, duplicate removed), `src/error.rs` (`missing_env_var_hint`, two tests), `src/app.rs` (`wechat_auth_status` / `wechat_auth_status_with`, two tests), `src/protocol/doctor.rs` (`wechat_auth` in text and JSON, two tests). Real binary on `/tmp/mp_cfg`: `push` without credentials prints both paths; `WECHAT_AUTH_METHOD=COOKIE` + toml `appsecret` reaches the cookie session error; `doctor` prints `missing WECHAT_APPID + WECHAT_SECRET` / `ready` / `cookie`. 859 → 868 tests.

## 2026-11-18 Cookie mode was offered as the way out of commands it never covered

**Symptom:** After the previous round added "two ways forward" to `AppError::MissingEnvVar`, the line `cookie mode (no AppSecret)` started appearing in *every* missing-credentials failure. But the cookie channel only implements three things — `push`, `ship`, `delete-draft` (`push_browser` wraps draft creation and draft deletion; there is no list or update wrapper). So a cookie user running `list-drafts` got the same "missing WECHAT_APPID", followed the hint (`moonpub login` plus `WECHAT_AUTH_METHOD=cookie`), ran it again, and got the same line back — the hint returned the user to the starting point while the command never ran. `update-draft` was worse: its correct cookie-mode answer is `moonpub push --render` (also cookie, and it rebuilds the draft), yet the error said nothing about it.

**Root cause:** The hint treated "there is a path that needs no AppSecret" as a universal fact, when it is a fact about *some commands only*. The scope had no single source: the text hard-coded one `moonpub push` example, so a reader naturally generalises "cookie works" to whichever command they just ran, and no code recorded which commands a cookie session covers — that knowledge lived only in the names of functions inside `push_browser`. The project has seen this class before: `workflow-registry`'s `safe_commands` earned its bridge test precisely because hand-maintained lists drift.

**Fix:** `config::COOKIE_AUTH_COMMANDS = ["push", "ship", "delete-draft"]` is now the single source; the missing-credentials hint, the error text and the docs all read it. A new `cookie_auth_unsupported` code covers the gap: in cookie mode, commands with no cookie implementation (`list-drafts`, `update-draft`) now say which command is uncovered, list what cookie does cover, and offer a step that actually advances (`update-draft` → `moonpub push <md> --render`, noting the media_id changes; `list-drafts` → `moonpub status` for the local view). The decision lives in `cookie_mode_block(appsecret_ready, is_cookie_auth)`: block only when cookie is chosen *and* appsecret credentials are absent; with credentials present the command still uses appsecret (cookie is unimplemented, not disabled); with neither and no cookie, `MissingEnvVar` still reports which variable is missing. While there, the third copy of the appid/secret read inside `update_draft` was deleted in favour of `wechat_client(cfg)`.

**Prevention:** Five assertions, one per face: (1) the hint's coverage text must equal `COOKIE_AUTH_COMMANDS.join(" / ")` (hard-coding it turns the test red); (2) every command in the constant must exist in the `help` registry; (3) `list-drafts` and `update-draft` must *not* appear in the constant (no implementation means no promise); (4) all four branches of `cookie_mode_block`; (5) `tests/json_error_contract.rs` drives the real binary through both commands and asserts `error.code == cookie_auth_unsupported`, an executable `next_step`, and the same sentence on stderr. Controlled mutation: adding `list-drafts` to the constant turns (3) red; reverting `cookie_mode_block` to a bare `is_cookie_auth` check turns (4) and (5) red (credentials-present users get blocked too).

**Article angle:** A hint is more dangerous than an error, because people follow hints. Whenever you say "there is a path", say where it leads — scope is not a wording detail, it is the boundary of a promise.

**Evidence:** `src/config.rs` (`COOKIE_AUTH_COMMANDS`, `appsecret_parts_from`, `appsecret_ready`, `cookie_auth_from_env`, four tests), `src/push.rs` (`ensure_cookie_mode_can_run`, `cookie_mode_block`, `shell_quote`, both command entry points), `src/error.rs` (`CookieAuthUnsupported`, coverage text in the hint), `src/protocol/error.rs` (25th registered code plus its `classify` arm), `tests/json_error_contract.rs`. Real binary in `/tmp/mp_cfg2`: cookie-mode `list-drafts` prints "cookie mode does not cover `list-drafts`", `update-draft` appends `moonpub push /private/tmp/.../Auth-precedence-probe.md --render`, while appsecret mode without credentials still prints "missing WECHAT_SECRET" plus both paths.

## 2026-11-18 Falling back to a channel that cannot run deletes the real reason

**Symptom:** `delete-draft 2247483648` in cookie mode printed two mangled lines and then an unrelated error:

```
  ⚠ cookie 会话删除失败 (WeChat cookie session not ready: session.json 不存在，cookie 模式无法删除草稿
  run 'moonpub login' to scan the QR code, then retry push (cookie mode bypasses the IP whitelist))，回退 appsecret 通道
Error: missing env var: WECHAT_SECRET
  two ways forward:
    cookie mode (no AppSecret, covers push / ship / delete-draft): run `moonpub login`, then `WECHAT_AUTH_METHOD=cookie moonpub push <article.md>`
```

Three faults stacked: (1) the real cause (no login session) was overwritten by "missing WECHAT_SECRET", and `error.code` lied with it — the JSON payload said `missing_env_var`; (2) the advice in that message *was* the cookie channel that had just failed — a closed loop; (3) the advice hard-coded `moonpub push <article.md>` while the user was deleting a draft, so following it created another draft instead. In appsecret mode the same function printed a warning about a channel the user never selected on *every* delete — and because a multi-line error was stuffed into parentheses, the sentence was cut into half-English/half-Chinese with a stray `retry push` in the middle.

**Root cause:** Fallback was written as an unconditional action. `delete_draft` tried cookie, warned on failure, then ran appsecret — but appsecret also needs credentials, so with none present its only possible outcome was to replace cookie's failure with "missing variable". The fallback never asked whether the path it was falling back to could actually run, so it fell into a strictly less informative error. Two message templates also hard-coded a command name: `CookieSessionRequired` always ended with "retry push", and `MissingEnvVar` always pointed at `moonpub push <article.md>`. Both templates are shared by `push`, `ship` and `delete-draft`; naming any one of them misleads the other two.

**Fix:** (1) The decision became a three-way pure function, `cookie_delete_outcome(appsecret_ready, cookie_selected)`: fall back when appsecret can run (warning only if the user chose cookie — otherwise it is a channel they never opted into); re-raise cookie's own error when appsecret cannot run and cookie was chosen (`cookie_session_required`, retry `moonpub delete-draft <id>`); otherwise continue to `MissingEnvVar`, which is the truth in that mode. (2) The warning keeps only the first line (`first_line`): a multi-line error inside parentheses truncates the sentence. (3) `CookieSessionRequired` gained a `retry` field filled at the call site — `moonpub push <path>` for `push`/`ship`, `moonpub delete-draft <id>` for delete. (4) The missing-credentials hint stopped naming any command: "run `moonpub login` once, add `WECHAT_AUTH_METHOD=cookie` to `.env`, then run this command again" — `.env` is loaded into the environment at startup, so the sentence is true for every command. (5) Non-numeric ids can only go through AppSecret; when those credentials are missing too, the error states both ways out instead of looping back to the variable hint.

**Prevention:** (1) Four assertions on `cookie_delete_outcome`, one per branch (credentials present/absent × cookie selected/not). (2) `first_line` keeps exactly the first line. (3) The error text must contain `then: moonpub delete-draft 2247483648` and must *not* contain `moonpub push` or `retry push`. (4) The hint test iterates `push` / `ship` / `delete-draft` and asserts none appear, plus asserts "run this command again" is present. (5) `tests/json_error_contract.rs` drives the real binary with `--json delete-draft`, `HOME` pointed at a temp dir so it can neither find a real session nor touch the test machine's `~/.config/moonpub/session.json`, and asserts `error.code == cookie_session_required`, `next_step == moonpub login`, the correct retry command in the message, and a stderr free of both `missing env var` and the fallback warning. Controlled mutations: reverting to an unconditional `Fallback { warn: true }` turns (1) and (5) red; replacing `first_line` with `&err.to_string()` turns (2) red; hard-coding the retry back to `moonpub push` turns (3) red.

**Article angle:** A fallback is the only kind of error handling that *destroys evidence*: it rewrites A's failure into B's failure, and B usually fails for reasons unrelated to the problem. Before falling back, ask whether the path you are falling back to can run right now. If it cannot, do not fall back — hand the user A's own words.

**Evidence:** `src/push.rs` (`delete_draft`, `CookieDeleteOutcome` / `cookie_delete_outcome` / `first_line`, three tests), `src/error.rs` (`CookieSessionRequired { retry }`, command-agnostic `missing_env_var_hint`, two tests), `src/push_browser.rs` (seven construction sites now pass `retry`; `build_cookie_header` takes it), `src/protocol/error.rs`, `tests/json_error_contract.rs`, `src/help.rs`. Real binary in `/tmp/mp_del`: cookie mode without credentials → `WeChat cookie session not ready … then: moonpub delete-draft 2247483648`; appsecret mode with credentials → no warning at all; cookie mode with credentials → one single-line warning then AppSecret; `WECHAT_AUTH_METHOD=cookie` written into `.env` confirmed to be picked up. 875 → 880 tests.

## 2026-11-18 One wrong character in an argument, and the diagnosis points at the whole channel

**Symptom:** `moonpub delete-draft R1LkThumb` (a material id passed where a draft id belongs) failed in cookie mode with:

```
Error: push failed: cookie 会话只能删数字 AppMsgId，`R1LkThumb` 不是；appsecret 凭据又缺失 —— …
```

Every diagnostic field in `--json` was wrong for the situation: `error.code` was `push_failed`; `error.first_check` said "run `moonpub wechat-health` first: separate credentials / IP whitelist / login state"; `next_step` was `moonpub wechat-health`; and `error.lesson` even offered "微信 IP 白名单失败可能来自公网出口漂移". The user had simply copied the wrong id and was sent to diagnose a perfectly healthy channel — where, of course, nothing would be found.

**Root cause:** `PushFailed` had become a bucket for "WeChat side could not do it". It legitimately covers "the API rejected the request", but here no request was ever sent — the argument was invalid. An error code is not a label; it carries a whole diagnostic: `first_check` (what to inspect), `lesson` (has this been hit before), `next_step` (what to do). The bigger the bucket, the more likely it points somewhere wrong: `push_failed`'s three fields are all channel-shaped, so anything argument-shaped stuffed into it must point wrong. The same spot had a second waste: `delete_draft_cookie_session`'s numeric guard sat *after* the browser launch (the only caller already pre-checks, so it was unreachable), meaning any real entry through it would first wait for Chrome to come up.

**Fix:** Added `AppError::InvalidMediaId { media_id, reason }` and the `invalid_media_id` code (26th in the registry). The message names only the literal on the first line (`invalid media_id: R1LkThumb`), says where a real id comes from on the second (`moonpub push` prints `media_id:`; `moonpub list-drafts` lists them), and states this instance's specific limit on the third (cookie sessions take only numeric AppMsgIds / with no AppSecret credentials, delete it in the backend). `first_check` became "confirm where the id came from"; `next_step` is `moonpub help delete-draft` — the only action that advances an argument error is reading the usage; no `lesson` is registered, because the IP-whitelist lesson has nothing to do with a typo and would mislead. `push_browser::delete_draft_cookie_session`'s entry guard now returns the same code, before any browser starts.

**Prevention:** (1) A message test: first line equals ``invalid media_id: `R1LkThumb` ``, does not start with `push failed`, mentions `moonpub list-drafts`, and stays at three lines or fewer — an argument error must be short. (2) `tests/json_error_contract.rs` drives the real binary with `--json delete-draft R1LkThumb` and asserts `error.code == invalid_media_id`, `next_step == moonpub help delete-draft`, `first_check` free of `wechat-health`, and an absent `lesson` — that last one is the important one, it locks "never attach a channel lesson to an argument error". (3) A `push_browser` unit test calls `delete_draft_cookie_session("R1LkThumb")` and asserts `InvalidMediaId` rather than `PushFailed`; it needs no browser, which also documents that the check is an early return. Controlled mutations: changing `reason` to start with `push failed` turns (1) red; mapping the variant back to `PushFailed` turns (2) red on both `first_check` and `next_step`; moving the guard inside `block_on` leaves (3) green (it only observes the return value), which is exactly why "no browser launched" has to be guaranteed by code position — the two together are what close the hole.

**Article angle:** An error code is a signpost, and the bigger the bucket, the more likely it points the wrong way. The test is simple: do this code's `first_check` and `next_step` have anything to do with what the user actually got wrong? If not, the code is wrong — not the wording.

**Evidence:** `src/error.rs` (`InvalidMediaId` plus its message test), `src/protocol/error.rs` (`ErrorCode::InvalidMediaId`, `ALL` at 26, `as_str` / `summary` / `example_next_step` / `first_check` / `classify` / `all_failures`), `src/push.rs` (non-numeric without credentials), `src/push_browser.rs` (entry guard plus unit test), `tests/json_error_contract.rs`. Real binary in `/tmp/mp_chain`: the text form is three lines; `--json` reports `invalid_media_id`, `next_step` `moonpub help delete-draft`, and omits `lesson`.

## 2026-11-19 Escaping is not one operation: HTML, CSS, TOML and HTTP headers each have their own alphabet

**Symptom:** Three sinks that had never been audited, all reachable from ordinary user input. (1) `moonpub export` on an article whose frontmatter was `title: "x\n[extra]\ninjected = true\n#"` wrote a Zola file in which the title's value closed at the first real newline and `[extra]` / `injected = true` became genuine TOML fields, rewriting the downstream site config. (2) AI cover HTML is built as `background-image:url('{png_name}')` where `png_name` comes from the article file stem, so a file named `a';background:url(//evil)#.md` closed the CSS string and injected a declaration. (3) `moonpub init` escaped only `"`, so typing the Windows path `C:\Users\me\articles` produced `root = "C:\Users\me\articles"`, whose `\U` is an invalid TOML escape — the config it wrote could not be read back.

**Root cause:** Rounds #216–#223 fixed seven classes of "user text reaching HTML", which trained us to read "escape" as one operation applied to one alphabet. It is not: a value is safe only relative to a sink. HTML text needs `&<>`; HTML attributes need quotes on top of that; URLs need a scheme allowlist; TOML basic strings need `\` `"` **and** `\n` `\r` plus control characters, because a real newline closes the string and starts a new key; CSS `url('...')` needs `\'`; multipart `filename="..."` needs quote escaping and CRLF stripping, because it is an HTTP header, not a body. The three sinks above were skipped precisely because none of them looks like HTML — two of them are not even markup. A second, quieter defect: `escape_toml_string` existed twice, in `export.rs` and `config.rs`, with identical bodies and the identical omission. Duplicated escape logic does not drift apart over time; it drifts *together* and stays wrong in unison, so a reviewer reading either copy sees a plausible function.

**Fix:** One shared `html::escape_toml_string` handling `\` `"` `\n` `\r` `\t` and any other control character as `\uXXXX`; `export`, `config` and `init` all delegate to it. `init`'s manual `.replace('\n', "\\n")` at the footer call sites had to go in the same change — leaving them would have double-escaped a real newline into a literal backslash-n. `cover::escape_css_url_string` covers the CSS string context (`\\`, `\'`, `\a` and `\d` for newline and carriage return — CSS hex escapes need a trailing space to terminate them — plus other control characters as hex escapes). `wechat::escape_multipart_filename` is shared by both upload paths and applied *after* `mime_for(filename)`, so extension-based MIME guessing still sees the original name. `config::unescape_toml_string` gained `\r` and `\t` so what escape writes can be read back.

**Prevention:** (1) A unit test per alphabet, each asserting the payload loses the ability to close its container and that a plain value passes through untouched. (2) An end-to-end `export` test that runs the real export, slices the generated frontmatter, and asserts no line starts with `injected` and that the newline survives as a literal `\n` inside the title — the parse-back is what proves the file is still valid TOML, not the escaping function's own output. (3) Assertions phrased as an invariant, not an implementation detail: "every quote is preceded by a backslash", never "the output contains no quote". The first draft used the latter and failed, because escaping does not delete the quote — it neutralises it. That failure is the lesson: assert that the string cannot escape its container, not that a character is absent.

**Article angle:** "Escaping" is a word people use as if it were one thing. It is four things — text, attribute, URL, structured value — plus two non-HTML sinks that are easy to forget because they never appear in a template: CSS strings and HTTP headers. The test is not "did I call escape?" but "which alphabet does this sink parse, and did I speak it?"

**Evidence:** `src/html.rs` (`escape_toml_string`, two unit tests), `src/export.rs` (delegation plus `export_cannot_inject_toml_fields_via_title`), `src/config.rs` (delegation, `unescape_toml_string` accepts `\r`/`\t`), `src/init.rs` (`toml_prompt_value` delegates; three manual newline replacements removed; test renamed to cover quotes, backslashes and newlines), `src/cover.rs` (`escape_css_url_string` plus two tests), `src/wechat.rs` (`escape_multipart_filename` plus one test), `src/push_browser.rs` (same helper in the cookie upload path). 956 → 962 tests, clippy clean, `fmt` clean.
