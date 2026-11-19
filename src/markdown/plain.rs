use crate::theme;

use super::inline::{LinkRefs, inline_md_with_refs, normalize_label, parse_image};

pub(super) fn render_markdown_segment(md: &str, theme: &theme::Theme) -> String {
    render_segment_with_refs(md, theme, &LinkRefs::new())
}

/// 收集本段里的引用式链接定义，再交给块解析器。
///
/// 引用式链接（`[文字][标签]` + 文末 `[标签]: url`）是 GFM 的常规写法，
/// 好处是长 URL 不打断正文。老实现完全不认这套语法：正文露出
/// `[Contributor Covenant][homepage]` 这样的字面标记，文末还多出一整行
/// `[homepage]: https://…`——那是定义，不是给人读的段落。
///
/// 定义要在**块解析之前**就抽走并建表，因为行内渲染器是被段、标题、列表、
/// 表格、引用逐层调用的，让每一层都自己去全文找定义既重复又会漏。
///
/// `inherited` 是外层已经建好的表：引用块里的 `> 见[文档]` 要用到引用块外面
/// 的定义，递归时把表带下去；本段自己的定义优先于外层同名定义。
fn render_segment_with_refs(md: &str, theme: &theme::Theme, inherited: &LinkRefs) -> String {
    let (own, body) = collect_link_refs(md);
    if own.is_empty() {
        return render_segment_body(&body, theme, inherited);
    }
    let mut refs = inherited.clone();
    for (label, url) in own {
        refs.entry(label).or_insert(url);
    }
    render_segment_body(&body, theme, &refs)
}

/// 代码围栏的开头行，返回 (围墙字符, 连续长度, 语言标记)。
///
/// CommonMark 有**两种**围栏：反引号 ``` 和波浪线 `~~~`。此前只认反引号，
/// `~~~rust` 那一行被当正文，围栏里的代码于是被行内语法重新解析——`# 注释`
/// 变成小标题、`**x**` 变成粗体、`==x==` 吃成空高亮，连 `[a]: url` 都会被
/// 当成链接定义从正文里抽走。同一仓里 `article.rs::fence_marker` 与
/// `body_needs_blocks` 早就把 `~~~` 当围栏，只有真正渲染代码的这两处
/// 没跟上；判定收成这一个函数，新增围栏字符只改这里。
///
/// 标记字符都是 ASCII，字符数与字节数一致，`&trimmed[run..]` 不会切进汉字。
pub(super) fn fence_open(trimmed: &str) -> Option<(char, usize, &str)> {
    let marker = trimmed.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let run = trimmed.chars().take_while(|&ch| ch == marker).count();
    if run < 3 {
        return None;
    }
    Some((marker, run, trimmed[run..].trim()))
}

/// 收尾围栏：同一字符、整行只有围墙字符、长度不短于开头。
///
/// 长度这条不是洁癖：`~~~` 围栏里的 `~~~~` 是代码内容（画波浪线、嵌套示例），
/// 认成收尾会把后半段代码漏成正文。
pub(super) fn fence_closes(trimmed: &str, marker: char, open_len: usize) -> bool {
    let run = trimmed.chars().take_while(|&ch| ch == marker).count();
    run >= open_len && trimmed.chars().all(|ch| ch == marker)
}

/// 抽出 `[标签]: url` 定义行，返回 (定义表, 去掉定义行后的正文)。
///
/// 只认**整行**就是定义的情况（允许最多 3 个前导空格，CommonMark 的口径）。
/// 代码块里的同名行是教学示例，必须原样留着，所以要跟踪围栏——
/// 这里与 [`render_markdown_segment`] 共用 [`fence_open`] / [`fence_closes`]，
/// 两种围栏（``` 与 `~~~`）口径一致，不再各认一半。
///
/// 定义后面允许跟标题（`[a]: url "标题"`），一律丢掉：微信编辑器会剥掉
/// `title` 属性，留着只会让 `href` 变成死链，与行内链接的处理口径一致。
fn collect_link_refs(md: &str) -> (LinkRefs, String) {
    let mut refs = LinkRefs::new();
    let mut body = String::with_capacity(md.len());
    let mut fence: Option<(char, usize)> = None;
    for line in md.lines() {
        let trimmed = line.trim();
        if let Some((marker, len, _)) = fence_open(trimmed) {
            match fence {
                None => fence = Some((marker, len)),
                Some((open_marker, open_len)) if fence_closes(trimmed, open_marker, open_len) => {
                    fence = None;
                }
                // 字符不同、或比开头更短的围墙行，是代码内容而不是收尾。
                Some(_) => {}
            }
            body.push_str(line);
            body.push('\n');
            continue;
        }
        if fence.is_none()
            && let Some((label, url)) = parse_link_definition(line)
        {
            // 同名定义以第一条为准（CommonMark 也是先到先得）。
            refs.entry(normalize_label(label)).or_insert(url);
            continue;
        }
        body.push_str(line);
        body.push('\n');
    }
    (refs, body)
}

/// 判断整行是不是链接定义，返回 (标签, URL)。
///
/// 形态固定：`[标签]:` 之后是**可选的**空白，再跟目标——CommonMark 允许零空白
/// （`[a]:url`），判定不能比规范严，否则合法写法会安静地变成正文。目标可以是
/// `<带空格的 url>`，也可以是不含空白的一段；后面跟的标题按行内链接的口径丢掉。
/// `[标签]:` 后面什么都没写说明作者只是打了个方括号冒号，按正文处理，不吞他的字。
fn parse_link_definition(line: &str) -> Option<(&str, String)> {
    let trimmed = line.trim();
    let indent = line.len() - line.trim_start().len();
    if indent > 3 {
        return None;
    }
    let rest = trimmed.strip_prefix('[')?;
    let label_end = rest.find("]:")?;
    let label = &rest[..label_end];
    if label.trim().is_empty() || label.contains('[') {
        return None;
    }
    let raw = rest[label_end + 2..].trim_start();
    if raw.is_empty() {
        return None;
    }
    if let Some(inner) = raw.strip_prefix('<')
        && let Some(close) = inner.find('>')
    {
        let url = inner[..close].trim();
        if url.is_empty() {
            return None;
        }
        // 尖括号只用来框住带空格的 URL，它自己不是地址的一部分。
        return Some((label, url.to_owned()));
    }
    let url = raw.split_whitespace().next().unwrap_or_default();
    if url.is_empty() {
        return None;
    }
    Some((label, url.to_owned()))
}

fn render_segment_body(md: &str, theme: &theme::Theme, refs: &LinkRefs) -> String {
    let mut out = String::new();
    let mut in_blockquote = false;
    let mut is_callout = false;
    let mut blockquote_buf = String::new();
    let mut table_buf: Vec<&str> = Vec::new();
    let mut checklist_buf: Vec<ChecklistItem> = Vec::new();
    let mut list_buf: Vec<ListItem> = Vec::new();
    // 段落是**连续多行**（CommonMark 的软换行）：`第一行` 换行接 `第二行` 仍是同一段，
    // 空行才分段。此前每行各印一个 `<p>`，一段三行变成三段、每段之间 1.5em 段距——
    // 用户写的是一段话，读者看到的是三个短句。这里把连续的正文行攒起来，遇到空行或
    // 任何块级语法再一次性输出。
    let mut paragraph_buf: Vec<&str> = Vec::new();
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    // 打开中的围栏：(围墙字符, 开头长度)。`None` 表示当前不在代码块里。
    let mut fence: Option<(char, usize)> = None;

    let lines: Vec<&str> = md.lines().collect();
    let mut i = 0;

    // 每个块级分支在自己的行被认出来时先调它：把上面积攒的块输出掉，再输出自己。
    // 收成一个宏是因为字段有八个，逐个内联写会让「哪个分支漏了 flush」看不出来——
    // 而漏一处就是内容顺序错乱，比漏一个块更难发现。
    macro_rules! flush_blocks {
        () => {
            flush_paragraph_buffers(
                &mut out,
                &mut ParagraphBuffers {
                    in_blockquote: &mut in_blockquote,
                    is_callout: &mut is_callout,
                    blockquote_buf: &mut blockquote_buf,
                    table_buf: &mut table_buf,
                    checklist_buf: &mut checklist_buf,
                    list_buf: &mut list_buf,
                    paragraph_buf: &mut paragraph_buf,
                },
                theme,
                refs,
            )
        };
    }

    // 引用块结束。与 `flush_blocks!` 分开：列表项/勾选项要继续自己那一块时，
    // 只该收尾前面的段落和引用，不能连自己一起收尾。
    macro_rules! end_blockquote {
        () => {
            if in_blockquote {
                if !is_callout {
                    out.push_str(&render_blockquote(&blockquote_buf, theme, refs));
                }
                blockquote_buf.clear();
                in_blockquote = false;
                is_callout = false;
            }
        };
    }

    macro_rules! flush_prose {
        () => {
            flush_paragraph(&mut out, &mut paragraph_buf, theme, refs)
        };
    }

    while i < lines.len() {
        let line = lines[i];
        i += 1;

        // 上一行是不是空行。`i` 已自增，上一行的下标是 `i - 2`。
        // 空行是用户明确的分段信号，跨过它继续拼就是替用户改结构。
        let prev_blank = i < 2 || lines[i - 2].trim().is_empty();

        let trimmed = line.trim();
        // 两种围栏（``` 与 `~~~`）走同一段判定，见 [`fence_open`]。
        if let Some((marker, len, info)) = fence_open(trimmed) {
            match fence {
                None => {
                    flush_blocks!();
                    code_lang = info.to_owned();
                    fence = Some((marker, len));
                    continue;
                }
                Some((open_marker, open_len)) if fence_closes(trimmed, open_marker, open_len) => {
                    out.push_str(&crate::illustrate::render_code_block(
                        &code_lang,
                        code_buf.trim_end(),
                        theme,
                    ));
                    code_buf.clear();
                    code_lang.clear();
                    fence = None;
                    continue;
                }
                // 字符不同、或比开头更短的围墙行，落进下面的代码缓冲。
                Some(_) => {}
            }
        }
        if fence.is_some() {
            code_buf.push_str(line);
            code_buf.push('\n');
            continue;
        }

        // 表格起始：带首尾竖线（传统写法），或「本行带竖线 + 下一行是分隔行」
        // （GFM 允许省略首尾竖线的手写表格）。块内续行按列数匹配，
        // 避免把含 `|` 的正文吞进表格。
        let starts_table = is_table_row(trimmed)
            || (trimmed.contains('|')
                && lines
                    .get(i)
                    .is_some_and(|next| is_delimiter_row(next.trim())))
            || (!table_buf.is_empty() && is_table_continuation(trimmed, table_buf[0]));
        if starts_table {
            // 表格续行也会走到这里，所以**不能**全量 flush——那会把已经攒下的
            // 表头/分隔行打印成空表格，真正的表格内容反而丢掉。
            flush_prose!();
            end_blockquote!();
            if !list_buf.is_empty() {
                out.push_str(&render_list(&list_buf, theme, refs));
                list_buf.clear();
            }
            if !checklist_buf.is_empty() {
                out.push_str(&render_checklist(&checklist_buf, theme, refs));
                checklist_buf.clear();
            }
            table_buf.push(line);
            continue;
        } else if !table_buf.is_empty() {
            out.push_str(&render_table(&table_buf, theme, refs));
            table_buf.clear();
        }

        if let Some(item) = parse_checklist_item(line) {
            // 勾选项继续同一个勾选块，只冲段落、引用和前面的列表，别把自己冲掉。
            flush_prose!();
            end_blockquote!();
            if !list_buf.is_empty() {
                out.push_str(&render_list(&list_buf, theme, refs));
                list_buf.clear();
            }
            checklist_buf.push(item);
            continue;
        } else if !checklist_buf.is_empty() {
            // 勾选项的缩进续行和 bullet 列表同一条判定，见 [`is_list_continuation`]。
            if is_list_continuation(line, prev_blank) {
                if let Some(last) = checklist_buf.last_mut() {
                    last.text.push(' ');
                    last.text.push_str(line.trim());
                }
                continue;
            }
            out.push_str(&render_checklist(&checklist_buf, theme, refs));
            checklist_buf.clear();
        }

        if let Some(item) = parse_list_item(line) {
            // 列表项继续同一个列表，只冲段落和引用，别把刚推进去的那项冲掉。
            flush_prose!();
            end_blockquote!();
            list_buf.push(item);
            continue;
        }

        if !list_buf.is_empty() {
            // 缩进续行必须放在 `parse_list_item` **之后**判：嵌套列表项（`  - 子项`）
            // 要先进 list_buf 当独立项，不能拼进上一项的文本。
            if is_list_continuation(line, prev_blank) {
                if let Some(last) = list_buf.last_mut() {
                    last.text.push(' ');
                    last.text.push_str(line.trim());
                }
                continue;
            }
            out.push_str(&render_list(&list_buf, theme, refs));
            list_buf.clear();
        }

        if let Some(rest) = strip_quote_marker(line) {
            // 引用块开始前要把前面攒的段落收掉，否则段落会跑到引用后面去。
            flush_prose!();
            if !in_blockquote {
                is_callout = rest.starts_with("[!");
                in_blockquote = true;
            }
            // 空引用行（单独一个 `>`）是段落分隔，此前被整行丢掉，
            // 引用里的两段会挤成一段。这里保留成空行交给引用渲染器。
            if !is_callout {
                if !blockquote_buf.is_empty() {
                    blockquote_buf.push('\n');
                }
                blockquote_buf.push_str(rest);
            }
            continue;
        }
        if in_blockquote {
            flush_blocks!();
        }

        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            flush_blocks!();
            out.push_str(&render_hr(theme));
            continue;
        }

        if let Some((level, rest)) = heading_parts(line) {
            flush_blocks!();
            match level {
                1 | 2 => out.push_str(&render_h2(rest, theme, refs)),
                3 => out.push_str(&render_h3(rest, theme, refs)),
                _ => out.push_str(&render_h4(rest, theme, refs)),
            }
            continue;
        }

        if trimmed.is_empty() {
            // 空行是唯一的分段信号：软换行在同一段里，空行才把段落切开。
            flush_blocks!();
            continue;
        }

        if trimmed.starts_with("![") {
            let chars: Vec<char> = trimmed.chars().collect();
            if let Some((alt, url, _)) = parse_image(&chars, refs) {
                flush_blocks!();
                out.push_str(&render_image_figure(&alt, &url, theme, refs));
                continue;
            }
        }
        // Setext 标题：正文的下一行整行是 `===`。
        // 认不出来的代价不是「退化成正文」，而是**变形**：`====` 会撞上行内的
        // `==高亮==` 语法，成品里冒出两个空 `<mark>` 和半个 `==`——用户写的是标题，
        // 读者看到的是噪声。下划线必须紧贴正文行，空行之后的下划线照旧走正文。
        if lines.get(i).is_some_and(|next| is_setext_underline(next)) {
            flush_blocks!();
            out.push_str(&render_setext_heading(line.trim(), theme, refs));
            // 下划线行本身不是正文，吃掉它。
            i += 1;
            continue;
        }

        // 走到这里说明这一行不属于任何块，就是正文的一行——攒进当前段落。
        paragraph_buf.push(line);
    }

    // 文末的尾巴：段落必须先于其它缓冲输出，它出现在文档流的最后。
    flush_blocks!();
    // 没写收尾围栏的代码块：到文末为止都算代码，不能悄悄漏成正文。
    if fence.is_some() {
        out.push_str(&crate::illustrate::render_code_block(
            &code_lang,
            code_buf.trim_end(),
            theme,
        ));
    }
    if in_blockquote && !is_callout {
        out.push_str(&render_blockquote(&blockquote_buf, theme, refs));
    }

    out
}

/// 表格行判定：首尾必须是 `|` 且总长至少 2 字节。
/// 单独一个 `|` 同时满足 starts_with / ends_with，直接做 `[1..len-1]` 会变成
/// `[1..0]` 并 panic；这类残行（复制残留、没写完的表格）应走正文而不是表格。
fn is_table_row(trimmed: &str) -> bool {
    trimmed.len() >= 2 && trimmed.starts_with('|') && trimmed.ends_with('|')
}

/// 去掉表格行首尾**可选**的竖线。GFM 里首尾竖线可以各自省略，`a | b` 同样是表格行。
/// 结尾若跟着转义管道（`\|`）那是单元格内容，不能当边框剥掉。
fn strip_edge_pipes(line: &str) -> &str {
    let line = line.strip_prefix('|').unwrap_or(line);
    match line.strip_suffix('|') {
        Some(body) if !ends_with_escaped_backslash(body) => body,
        _ => line,
    }
}

/// `body` 是否以奇数个反斜杠结尾——即末尾那个字符被转义了。
fn ends_with_escaped_backslash(body: &str) -> bool {
    body.chars().rev().take_while(|&c| c == '\\').count() % 2 == 1
}

fn table_cell_count(line: &str) -> usize {
    split_table_cells(strip_edge_pipes(line.trim())).len()
}

/// GFM 分隔行（`--- | :---: | ---`），用来确认「上一行是表头」。
/// 必须含至少一个 `|`：单独一行 `---` 是分隔线，不能当成单列表格。
fn is_delimiter_row(trimmed: &str) -> bool {
    trimmed.contains('|')
        && split_table_cells(strip_edge_pipes(trimmed))
            .iter()
            .all(|cell| is_dash_cell(cell.trim()))
}

/// `:---:` / `---` / `:` 之外一律不算；`-:-` 这种残行也不算。
fn is_dash_cell(cell: &str) -> bool {
    let s = cell.strip_prefix(':').unwrap_or(cell);
    let s = s.strip_suffix(':').unwrap_or(s);
    !s.is_empty() && s.bytes().all(|b| b == b'-')
}

/// 表格块内的续行：带竖线且列数与表头一致，防止把带 `|` 的正文吞进表格。
fn is_table_continuation(trimmed: &str, header: &str) -> bool {
    trimmed.contains('|') && table_cell_count(trimmed) == table_cell_count(header)
}

/// 识别顶格 ATX 标题，兼容中文写作里常见的 `#标题`（井号后无空格）写法。
/// 返回 (级别, 正文)：级别 1/2 映射 h2（文章主标题由正文 H1 承担）、3 映射 h3、4 映射 h4。
/// 只认顶格、连续 1~4 个 `#`；五个以上不识别，避免把 `#####` 之类的噪声吞成标题。
/// `#` 后必须有内容，单独一行 `#` 仍按正文处理。
fn heading_parts(line: &str) -> Option<(u8, &str)> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    // CommonMark 支持六级；微信正文只有一套小标题层级，五六级降级到 h4 样式，
    // 但绝不能落到「认不出来 → 把井号原样印进正文」。
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let text = line[hashes..].trim();
    if text.is_empty() {
        return None;
    }
    Some((hashes as u8, text))
}

/// 按单元格边界切分表格行，尊重 `\|` 转义和反引号行内代码里的 `|`。
/// 返回拥有所有权的 String，因为转义的反斜杠要被去掉。
fn split_table_cells(inner: &str) -> Vec<String> {
    let mut cells: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_code = false;
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '`' => {
                in_code = !in_code;
                current.push(ch);
            }
            '\\' if chars.peek() == Some(&'|') => {
                chars.next();
                current.push('|');
            }
            '|' if !in_code => cells.push(std::mem::take(&mut current)),
            _ => current.push(ch),
        }
    }
    cells.push(current);
    cells
}

fn render_table(lines: &[&str], theme: &theme::Theme, refs: &LinkRefs) -> String {
    let mut headers: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if is_table_row(trimmed) || trimmed.contains('|') {
            // 首尾竖线是可选的，切分前统一剥掉；不能直接 `[1..len-1]`，
            // 那会把无边框行砍掉首尾各一个字符。
            let cells = split_table_cells(strip_edge_pipes(trimmed));
            if cells
                .iter()
                .all(|c| c.trim().chars().all(|x| x == '-' || x == ':' || x == ' '))
            {
                continue;
            }
            let cells: Vec<String> = cells.into_iter().map(|c| c.trim().to_owned()).collect();
            if headers.is_empty() {
                headers = cells;
            } else {
                rows.push(cells);
            }
        }
    }

    // 只有表头没有数据行（或整块被误判）时不要吐空 section：微信里是一块空白。
    if headers.is_empty() || rows.is_empty() {
        return String::new();
    }

    // 微信正文对 <table> 支持极差：多列表格被拆成「标签一行 + 每个单元格各一行」，
    // 3 列表格在手机上变成 3 行竖排，一屏读不到两件事。改成每行压成一句正文：
    // 「标签：值1；值2」，标签用 accent 小字内嵌，行间不加卡片不画框。
    let mut html = String::from("<section style=\"margin:22px 0;\">\n");
    for row in rows {
        let label = row.first().map(String::as_str).unwrap_or("");
        let mut parts: Vec<String> = Vec::new();
        for (idx, header) in headers.iter().enumerate().skip(1) {
            let cell = row.get(idx).map(String::as_str).unwrap_or("");
            if cell.is_empty() {
                continue;
            }
            // 只有两列时（标签 + 一个值）值本身已经说清含义，不再重复表头；
            // 三列以上时把表头塞进值前面，否则读者不知道这个值属于谁。
            if headers.len() > 2 {
                // 表头与值都是渲染过的行内 HTML：中间用裸空格会变成只含空白的
                // 文本节点，公众号后台删掉之后「表头值」粘在一起（同
                // `inline.rs` 里跨元素空白的处理）。
                parts.push(format!(
                    "{}&nbsp;{}",
                    inline_md_with_refs(header, theme, refs),
                    inline_md_with_refs(cell, theme, refs)
                ));
            } else {
                parts.push(inline_md_with_refs(cell, theme, refs));
            }
        }
        if label.is_empty() && parts.is_empty() {
            continue;
        }
        let mut line = String::new();
        if !label.is_empty() {
            line.push_str(&format!(
                "<span style=\"font-size:12px;font-weight:700;letter-spacing:0.1em;color:{};\">{}</span>：",
                theme.accent,
                inline_md_with_refs(label, theme, refs)
            ));
        }
        line.push_str(&parts.join("；"));
        html.push_str(&format!(
            "<p style=\"margin:0 0 12px;font-size:15px;line-height:1.85;color:{};letter-spacing:0.02em;\">{}</p>\n",
            theme.text_color, line
        ));
    }
    html.push_str("</section>\n\n");
    html
}

struct ParagraphBuffers<'a, 'md> {
    in_blockquote: &'a mut bool,
    is_callout: &'a mut bool,
    blockquote_buf: &'a mut String,
    table_buf: &'a mut Vec<&'md str>,
    checklist_buf: &'a mut Vec<ChecklistItem>,
    list_buf: &'a mut Vec<ListItem>,
    paragraph_buf: &'a mut Vec<&'md str>,
}

/// 把攒起来的正文行输出成一个 `<p>`。
///
/// 单独抽出来是因为「列表项继续」时只该冲段落、不能冲列表自己——用全量 flush 会把
/// 刚推进去的那一项提前打印，一个列表被劈成多个 `<section>`。
fn flush_paragraph(
    out: &mut String,
    paragraph_buf: &mut Vec<&str>,
    theme: &theme::Theme,
    refs: &LinkRefs,
) {
    if paragraph_buf.is_empty() {
        return;
    }
    let html = render_paragraph_lines(paragraph_buf, theme, refs);
    out.push_str(&html);
    paragraph_buf.clear();
}

fn flush_paragraph_buffers(
    out: &mut String,
    buffers: &mut ParagraphBuffers<'_, '_>,
    theme: &theme::Theme,
    refs: &LinkRefs,
) {
    // 段落缓冲与其余缓冲互斥（一行要么是正文、要么属于某个块），这里只决定同一次
    // flush 内谁先打印。段落放最前，与「它在文档里出现在块之前」的直觉一致。
    flush_paragraph(out, buffers.paragraph_buf, theme, refs);
    if !buffers.table_buf.is_empty() {
        out.push_str(&render_table(buffers.table_buf, theme, refs));
        buffers.table_buf.clear();
    }
    if !buffers.list_buf.is_empty() {
        out.push_str(&render_list(buffers.list_buf, theme, refs));
        buffers.list_buf.clear();
    }
    if !buffers.checklist_buf.is_empty() {
        out.push_str(&render_checklist(buffers.checklist_buf, theme, refs));
        buffers.checklist_buf.clear();
    }
    if *buffers.in_blockquote {
        if !*buffers.is_callout {
            out.push_str(&render_blockquote(buffers.blockquote_buf, theme, refs));
        }
        buffers.blockquote_buf.clear();
        *buffers.in_blockquote = false;
        *buffers.is_callout = false;
    }
}

#[derive(Debug)]
struct ChecklistItem {
    checked: bool,
    indent: usize,
    text: String,
}

#[derive(Debug)]
struct ListItem {
    marker: ListMarker,
    indent: usize,
    text: String,
}

#[derive(Debug)]
enum ListMarker {
    Bullet,
    Ordered(usize),
}

/// 行首缩进的列数（tab 按 4 列对齐），只数前导空白。
fn indent_columns(line: &str) -> usize {
    let mut cols = 0;
    for c in line.chars() {
        match c {
            ' ' => cols += 1,
            '\t' => cols += 4 - cols % 4,
            _ => break,
        }
    }
    cols
}

/// 手机端正文窄，四级以上缩进会把文字挤出可视区，层级封顶。
const MAX_LIST_DEPTH: usize = 3;

/// 缩进列数 → 层级。用缩进栈而不是「除以固定步长」：`- ` 的子项缩进 2 格、
/// `1. ` 的子项缩进 3 格、编辑器把 Tab 展开成 4 格，三种习惯都只该进一级。
fn push_indent(stack: &mut Vec<usize>, indent: usize) -> usize {
    while stack.last().is_some_and(|&top| indent < top) {
        stack.pop();
    }
    if stack.last().is_none_or(|&top| indent > top) {
        stack.push(indent);
    }
    (stack.len() - 1).min(MAX_LIST_DEPTH)
}

fn resolve_depths(items: &[ListItem]) -> Vec<usize> {
    let mut stack = Vec::new();
    items
        .iter()
        .map(|item| push_indent(&mut stack, item.indent))
        .collect()
}

fn parse_checklist_item(line: &str) -> Option<ChecklistItem> {
    let rest = line.trim_start().strip_prefix("- [")?;
    let (marker, content) = rest.split_once("] ")?;
    let marker = marker.trim();
    let checked = if marker.eq_ignore_ascii_case("x") {
        true
    } else if marker.is_empty() {
        false
    } else {
        return None;
    };
    Some(ChecklistItem {
        checked,
        indent: indent_columns(line),
        text: content.trim().to_owned(),
    })
}

/// 这一行是不是上一个列表项的缩进续行。
///
/// `- 第一项` 下面缩进写的那一行是这一项的继续，不是新段落。中文写列表时这是最
/// 常见的写法——一项写长了就换行。此前这一行没被任何分支接住：列表先被 flush 掉，
/// 续行带着自己的缩进落进段落分支，成品里本该挂在项目符号下面的说明变成顶格段落，
/// 还多一个段间距，读者看到的是「列表只有一行，剩下是另一段话」。
///
/// 三个条件都必要：
/// - 缩进至少 2 列：`- ` 的内容从第 2 列开始，对齐到它才算「属于这一项」。
/// - 上一行不是空行：空行是用户明确的分段信号，跨过它继续拼就是替用户改结构。
/// - 自己不是别的块级语法：`#` / `>` / `|` / 分隔线宁可退化成正文，也不许被吞进
///   列表项——这正是 setext 下划线和波浪线围栏踩过的坑。
fn is_list_continuation(line: &str, prev_blank: bool) -> bool {
    if prev_blank || indent_columns(line) < 2 {
        return false;
    }
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    if matches!(trimmed.chars().next(), Some('#' | '>' | '|')) {
        return false;
    }
    !matches!(trimmed, "---" | "***" | "___")
}

fn parse_list_item(line: &str) -> Option<ListItem> {
    let trimmed = line.trim_start();
    let indent = indent_columns(line);
    if let Some(rest) = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .or_else(|| trimmed.strip_prefix("+ "))
    {
        return Some(ListItem {
            marker: ListMarker::Bullet,
            indent,
            text: rest.trim().to_owned(),
        });
    }

    let (num, rest) = trimmed.split_once(". ")?;
    let value = num.parse().ok()?;
    Some(ListItem {
        marker: ListMarker::Ordered(value),
        indent,
        text: rest.trim().to_owned(),
    })
}

/// 列表项左侧悬挂缩进：`padding-left` 撑开层级，`text-indent` 把标记拉回行首。
/// 逐档写死而不是算浮点数，避免 `1.2 + 1.3` 拼出 `2.5000000000000004em`。
const LIST_PADDING: [&str; 4] = ["1.2em", "2.5em", "3.8em", "5.1em"];
const CHECKLIST_PADDING: [&str; 4] = ["1.3em", "2.6em", "3.9em", "5.2em"];

fn list_marker(marker: &ListMarker, theme: &theme::Theme) -> String {
    match marker {
        ListMarker::Bullet => format!(
            "<span style=\"color:{};font-weight:700;margin-right:8px;\">•</span>",
            theme.accent
        ),
        ListMarker::Ordered(n) => format!(
            "<span style=\"color:{};font-weight:700;margin-right:6px;\">{n}.</span>",
            theme.accent
        ),
    }
}

fn render_checklist(items: &[ChecklistItem], theme: &theme::Theme, refs: &LinkRefs) -> String {
    // 勾选清单也走段落，不用两列表格——微信里表格单元格宽度不可控，
    // 图标列经常被压成 0 宽，只剩文字缩进一截。
    let mut html = String::from("<section style=\"margin:18px 0 22px;\">\n");
    let mut stack = Vec::new();
    for item in items {
        let depth = push_indent(&mut stack, item.indent);
        let (mark, color) = if item.checked {
            ("✔", theme.accent)
        } else {
            ("○", theme.text_muted)
        };
        let pad = CHECKLIST_PADDING[depth];
        html.push_str(&format!(
            "<p style=\"margin:0 0 10px;padding-left:{pad};text-indent:-1.3em;font-size:15px;line-height:1.85;color:{};letter-spacing:0.02em;\"><span style=\"color:{color};font-weight:700;margin-right:8px;\">{mark}</span>{}</p>\n",
            theme.text_color,
            inline_md_with_refs(&item.text, theme, refs)
        ));
    }
    html.push_str("</section>\n\n");
    html
}

fn render_list(items: &[ListItem], theme: &theme::Theme, refs: &LinkRefs) -> String {
    // 列表同样不进 <table>：微信把单列表格的一些单元格样式吃掉后，行间距会乱掉，
    // 长列表还容易被截断。用带悬挂缩进的段落，圆点/编号内嵌在行首。
    let mut html = String::from("<section style=\"margin:18px 0 22px;\">\n");
    for (item, depth) in items.iter().zip(resolve_depths(items)) {
        let marker = list_marker(&item.marker, theme);
        let pad = LIST_PADDING[depth];
        html.push_str(&format!(
            "<p style=\"margin:0 0 10px;padding-left:{pad};text-indent:-1.2em;font-size:15px;line-height:1.85;color:{};letter-spacing:0.02em;\">{marker}{}</p>\n",
            theme.text_color,
            inline_md_with_refs(&item.text, theme, refs)
        ));
    }
    html.push_str("</section>\n\n");
    html
}

fn render_hr(theme: &theme::Theme) -> String {
    format!(
        "<section style=\"margin: 2.2em 0; text-align:center;\"><section style=\"display:inline-block;width:42px;height:2px;background:{};vertical-align:middle;\"></section><span style=\"display:inline-block;width:6px;height:6px;border:1px solid {};border-radius:50%;margin:0 10px;vertical-align:middle;background:{};\"></span><section style=\"display:inline-block;width:42px;height:2px;background:{};vertical-align:middle;\"></section></section>\n\n",
        theme.border, theme.accent, theme.section_bg, theme.border
    )
}

/// 这一行是不是 setext 标题的下划线（整行 `===`）。
///
/// **只收 `=`**：CommonMark 也允许 `---`，但 `---` 在真实语料里更常是分隔线、
/// 或 frontmatter 示例的边界线（`key: value` 紧跟一行 `---`）。把它一并收成标题，
/// 会把「一行配置 + 一条分隔线」误升级成小标题；而 `---` 退化成「正文 + 分隔线」
/// 时读起来并不误导。`=` 没有第二重身份：认不出来就只剩变形——
/// 行内 `==高亮==` 会吃掉它，成品里冒出空 `<mark>` 和半个 `==`。
///
/// 缩进按 CommonMark 口径最多 3 个空格——4 个空格在 Markdown 里是代码。
fn is_setext_underline(line: &str) -> bool {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return false;
    }
    let trimmed = line.trim();
    !trimmed.is_empty() && trimmed.chars().all(|ch| ch == '=')
}

/// setext 标题按 ATX 的同级映射渲染：`===` 与 `#` 一样落到 h2。
///
/// 理由与 [`heading_parts`] 一致——文章主标题由正文 H1 承担，微信正文只有一套
/// 小标题层级。作者不必猜「下划线写法算几级」。
fn render_setext_heading(text: &str, theme: &theme::Theme, refs: &LinkRefs) -> String {
    render_h2(text, theme, refs)
}

fn render_h2(text: &str, theme: &theme::Theme, refs: &LinkRefs) -> String {
    // 中文小标题原先同时挂了左侧粗条和下边框，两个方向抢视线；字距 1px 把
    // 三四个字的标题拉得很散。收成一条左侧色条 + 正常字距，靠上下留白分层。
    format!(
        "<h2 style=\"font-size: 21px; font-weight: 700; color: {}; margin: 2.4em 0 1em; padding: 2px 0 2px 13px; border-left: 4px solid {}; letter-spacing: 0.02em; line-height:1.45;\">{}</h2>\n\n",
        theme.heading_color,
        theme.heading_border,
        inline_md_with_refs(text, theme, refs)
    )
}

fn render_h3(text: &str, theme: &theme::Theme, refs: &LinkRefs) -> String {
    format!(
        "<h3 style=\"font-size: 17px; font-weight: 700; color: {}; margin: 1.9em 0 0.7em; padding: 7px 12px; border-left: 3px solid {}; background: {}; border-radius: 0 6px 6px 0; letter-spacing: 0.02em; line-height:1.5;\">{}</h3>\n\n",
        theme.heading_color,
        theme.accent,
        theme.accent_soft,
        inline_md_with_refs(text, theme, refs)
    )
}

fn render_h4(text: &str, theme: &theme::Theme, refs: &LinkRefs) -> String {
    format!(
        "<h4 style=\"font-size: 16px; font-weight: 700; color: {}; margin: 1.5em 0 0.7em; padding: 0 0 0 10px; border-left: 3px solid {}; letter-spacing: 0.02em; line-height:1.5;\">{}</h4>\n\n",
        theme.heading_color,
        theme.accent,
        inline_md_with_refs(text, theme, refs)
    )
}

/// 段落外壳。
fn paragraph_html(inner: &str, theme: &theme::Theme) -> String {
    // 首行缩进 + 1.25em 段距 = 双重分隔，读起来发散；两端对齐会让中英混排的
    // 行内空隙忽大忽小。统一成「不缩进 + 1.5em 段距 + 左对齐」，这是中文长文
    // 在手机上最稳的一组参数。
    format!(
        "<p style=\"margin: 0 0 1.5em; color: {}; font-size: 16px; line-height: 1.85; letter-spacing: 0.02em; text-align: left;\">{inner}</p>\n\n",
        theme.text_color,
    )
}

/// 把多行正文渲染成一个段落，行间用 `<br>`。
///
/// 为什么不合并成一行：中文写作里换行是有语义的——「现象 / 原因 / 解决」写成三行，
/// 压成一句读起来像糨糊。`<br>` 保住了作者的换行，而段落间距只出现一次，这正是
/// 用户写的样子。
///
/// 为什么先拼再渲染：行内语法可以跨行（`` `intake photos --draft `` 换行接
/// `` photos/day1` ``）。逐行渲染会让两半各看到奇数个反引号，把反引号原样印出来。
fn render_paragraph_lines(lines: &[&str], theme: &theme::Theme, refs: &LinkRefs) -> String {
    let joined = lines.join("\n");
    let rendered = inline_md_with_refs(&joined, theme, refs);
    paragraph_html(&rendered.replace('\n', "<br>"), theme)
}

fn render_image_figure(alt: &str, url: &str, theme: &theme::Theme, refs: &LinkRefs) -> String {
    let caption = if alt.trim().is_empty() {
        String::new()
    } else {
        format!(
            "<p style=\"margin:0;padding:9px 12px 0;color:{};font-size:12px;line-height:1.65;text-align:center;letter-spacing:0.04em;\">{}</p>",
            theme.text_muted,
            inline_md_with_refs(alt.trim(), theme, refs)
        )
    };

    format!(
        "<section style=\"margin: 1.8em 0 2em; padding: 10px; background: {}; border: 1px solid {}; border-radius: 8px; text-align:center;\">\n<img src=\"{}\" alt=\"{}\" style=\"max-width: 100%; display: block; margin: 0 auto; border-radius: 5px;\" />\n{caption}</section>\n\n",
        theme.block_bg,
        theme.border,
        crate::html::escape(crate::html::safe_destination(url)),
        crate::html::escape(alt)
    )
}

/// `>` 后可以带一个空格，也可以不带——CommonMark 两种都算引用，中文写作常写 `>引用`。
/// 只剥一层：`>> 内层` 剥完仍是 `> 内层`，交给递归渲染出真正的嵌套引用。
fn strip_quote_marker(line: &str) -> Option<&str> {
    let rest = line.strip_prefix('>')?;
    Some(rest.strip_prefix(' ').unwrap_or(rest))
}

/// 一段正文里是否含块级结构。纯文字走快路径（一次性行内渲染），
/// 保住现有的紧凑排版；一旦出现嵌套引用、代码块、标题、列表、表格或空行，
/// 就交给 [`render_markdown_segment`]，用同一套块解析器渲染。
///
/// 调用方是 `>` 引用块（`render_blockquote`）与 `:::` 块（`markdown::blocks`）：
/// 两处的正文都是「块里的 Markdown」，都不能当一串行内文本处理。
pub(super) fn body_needs_blocks(text: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return true;
        }
        if strip_quote_marker(line).is_some()
            || fence_open(trimmed).is_some()
            || is_table_row(trimmed)
            || is_delimiter_row(trimmed)
            || parse_list_item(line).is_some()
            || parse_checklist_item(line).is_some()
        {
            return true;
        }
        heading_parts(line).is_some()
    })
}

fn render_blockquote(text: &str, theme: &theme::Theme, refs: &LinkRefs) -> String {
    let body = if body_needs_blocks(text) {
        // 引用里的块级结构交给同一个块解析器：嵌套引用、代码块、标题、列表、表格
        // 都不该在引用里再实现一遍「半套 Markdown」。此前这里逐行 inline_md，
        // 于是 `> > 内层` 漏出 `>`、`> ```js` 把语言标记拼进代码正文、
        // `> ## 标题` 漏出井号。
        render_segment_with_refs(text, theme, refs)
    } else {
        // 纯文字引用：HTML 不认 \n，逐行 inline 后用 <br> 断开，
        // 否则三行信息会挤成一整段。
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| inline_md_with_refs(l.trim(), theme, refs))
            .collect::<Vec<_>>()
            .join("<br>")
    };
    // box-shadow 微信端不渲染，去掉；字距从 0.08em 收到 0.02em 跟正文一致。
    format!(
        "<section style=\"margin: 2em 0; padding: 18px 20px; background: {}; border: 1px solid {}; border-left: 4px solid {}; border-radius: 0 8px 8px 0; color: {}; font-size: 15px; line-height: 1.9; letter-spacing: 0.02em; overflow-wrap:break-word; word-break:break-word;\">{}</section>\n\n",
        theme.block_bg, theme.border, theme.accent, theme.text_color, body
    )
}

#[cfg(test)]
mod tests {
    use super::render_markdown_segment;
    use crate::theme;

    fn theme() -> theme::Theme {
        theme::Theme::default()
    }

    /// 只取可见文本。断言「标记没有漏进正文」时不能直接 `contains`——
    /// 标签自身的收尾 `>` 会紧贴正文，`">无空格引用"` 这种写法会误判。
    fn visible_text(html: &str) -> String {
        let mut out = String::new();
        let mut in_tag = false;
        for ch in html.chars() {
            if in_tag {
                // 只有「已经在标签里」时 `>` 才是收尾；正文里紧跟在标签后的
                // 字面 `>` 必须保留，否则「`>` 漏进正文」这类断言永远抓不住。
                if ch == '>' {
                    in_tag = false;
                }
            } else if ch == '<' {
                in_tag = true;
            } else {
                out.push(ch);
            }
        }
        out
    }

    #[test]
    fn renders_empty_without_panic() {
        let out = render_markdown_segment("", &theme());
        assert!(out.is_empty() || out.chars().all(|c| c.is_whitespace()));
    }

    #[test]
    fn renders_heading_text() {
        let out = render_markdown_segment("## 社区营造", &theme());
        assert!(out.contains("社区营造"));
    }

    /// setext 标题（`标题` 下面一行 `===`）此前认不出来，下划线掉进行内渲染器，
    /// 被 `==高亮==` 语法吃成两个空 `<mark>` 加半个 `==`：标题降级成正文，
    /// 读者看到一串标签残渣。断言必须同时锁「标题成立」和「没有一个 `=` 漏出来」。
    #[test]
    fn setext_equals_underline_becomes_a_heading_not_highlighter_debris() {
        let out = render_markdown_segment("社区营造\n====\n\n正文。", &theme());
        assert!(out.contains("<h2"), "应是标题：{out}");
        assert!(out.contains("社区营造"), "标题文字丢了：{out}");
        assert!(!out.contains("<mark"), "下划线被当成高亮：{out}");
        assert!(!visible_text(&out).contains('='), "等号漏进正文：{out}");
    }

    /// 下划线的缩进口径跟 CommonMark 一致：3 个空格仍是标题。
    #[test]
    fn setext_equals_underline_allows_three_leading_spaces() {
        let out = render_markdown_segment("社区营造\n   ===\n", &theme());
        assert!(out.contains("<h2"), "3 空格缩进仍是标题：{out}");
    }

    /// 4 个空格在 Markdown 里是代码块，不能顺手升级成标题——那会把缩进示例吃掉。
    #[test]
    fn setext_equals_underline_rejects_four_leading_spaces() {
        let out = render_markdown_segment("正文\n    ===\n", &theme());
        assert!(!out.contains("<h2"), "4 空格不是标题：{out}");
    }

    /// 下划线必须是「整行同一种符号」。`=-=` 这种残行按正文处理，不猜。
    #[test]
    fn setext_underline_needs_one_clean_run_of_equals() {
        let out = render_markdown_segment("正文\n=-=\n", &theme());
        assert!(!out.contains("<h2"), "混合符号不是标题：{out}");
    }

    /// 空行把上下两块切开了：下划线不能跨过空行去把上一段升级成标题。
    #[test]
    fn setext_lookahead_does_not_cross_a_blank_line() {
        let out = render_markdown_segment("正文\n\n===\n", &theme());
        assert!(!out.contains("<h2"), "空行之后不该拼成标题：{out}");
        assert!(visible_text(&out).contains("正文"));
    }

    /// `---` 在真实语料里更常是分隔线 / frontmatter 边界，这次刻意不收：
    /// 正文下面的 `---` 仍是「正文 + 分隔线」，不会被升级成小标题。
    #[test]
    fn dashes_after_text_stay_a_divider() {
        let out = render_markdown_segment("正文\n\n---\n", &theme());
        assert!(!out.contains("<h2"), "`---` 不该变成标题：{out}");
        assert!(
            !visible_text(&out).contains("---"),
            "分隔线不该印成正文：{out}"
        );
    }

    /// 波浪线围栏（CommonMark 的第二种代码围栏）此前整块漏成正文：`~~~bash`
    /// 自己印成一段，围栏里的代码被行内语法重新解析——`# 注释` 变标题、
    /// `**x**` 变粗体、`==x==` 变空高亮、`- 项` 变列表。断言要逐条锁住这些「变形」。
    #[test]
    fn tilde_fence_keeps_code_verbatim_instead_of_reparsing_it() {
        let md = "~~~bash\n# 注释\n**不该加粗**\n== 不该高亮 ==\n- 不该是列表\n~~~";
        let out = render_markdown_segment(md, &theme());
        assert!(!out.contains("<h2"), "代码里的井号成了标题：{out}");
        assert!(!out.contains("<strong"), "代码里的星号成了粗体：{out}");
        assert!(!out.contains("<mark"), "代码里的等号成了高亮：{out}");
        assert!(!out.contains('•'), "代码里的短横成了列表项：{out}");
        assert!(visible_text(&out).contains("# 注释"), "代码原文丢了：{out}");
        assert!(
            !visible_text(&out).contains("~~~"),
            "围栏标记漏进正文：{out}"
        );
    }

    /// 围栏里的链接定义是教学示例，不能当成本文的定义抽走——抽走那行，代码块就少一行，
    /// 而围栏外的同名引用还会突然指向「代码里的」地址。
    #[test]
    fn tilde_fence_keeps_link_definitions_inside_the_code() {
        let md = "~~~markdown\n[docs]: https://example.com/x\n~~~\n\n正文用 [文档][docs]。";
        let out = render_markdown_segment(md, &theme());
        assert!(
            visible_text(&out).contains("[docs]:"),
            "定义行被抽走了：{out}"
        );
        assert!(
            !out.contains("href=\"https://example.com/x\""),
            "围栏里的定义被当成真定义：{out}"
        );
    }

    /// 收尾围栏不能比开头短：`~~~` 开头时，里面的 `~~` 只是代码内容。
    #[test]
    fn fence_shorter_than_opening_does_not_close_the_block() {
        let out = render_markdown_segment("~~~\n~~\n尾行\n~~~", &theme());
        assert!(visible_text(&out).contains("尾行"), "尾行被吞了：{out}");
        assert!(visible_text(&out).contains("~~"), "短围栏不该收尾：{out}");
    }

    /// 两种围栏字符不能互相收尾：``` 开头只能 ``` 收，`~~~` 只是代码内容。
    #[test]
    fn fences_of_different_characters_do_not_close_each_other() {
        let out = render_markdown_segment("```\n~~~\n代码里的波浪线\n```", &theme());
        assert!(
            visible_text(&out).contains("代码里的波浪线"),
            "波浪线把代码块提前关了：{out}"
        );
    }

    #[test]
    fn renders_blockquote_text() {
        let out = render_markdown_segment("> 这是引用", &theme());
        assert!(out.contains("这是引用"));
    }

    #[test]
    fn renders_list_items() {
        let out = render_markdown_segment("- 第一项\n- 第二项", &theme());
        assert!(out.contains("第一项"));
        assert!(out.contains("第二项"));
    }

    #[test]
    fn heading_levels_five_and_six_render_as_headings() {
        // 老实现只认 1..=4 个 `#`，写 `#####` 的稿子会把井号原样印在正文里。
        let out = render_markdown_segment("##### 五级标题", &theme());
        assert!(!out.contains("##### 五级标题"), "五级标题漏出了井号: {out}");
        assert!(out.contains("五级标题"));
        assert!(out.contains("<h4"), "五六级标题应降级到 h4 样式: {out}");

        let out = render_markdown_segment("###### 六级标题", &theme());
        assert!(
            !out.contains("###### 六级标题"),
            "六级标题漏出了井号: {out}"
        );
        assert!(out.contains("<h4"), "{out}");
    }

    #[test]
    fn seven_hashes_is_not_a_heading() {
        // CommonMark 最多六级，七个井号是正文（本项目按普通段落处理）。
        let out = render_markdown_segment("####### 不是标题", &theme());
        assert!(out.contains("####### 不是标题"), "{out}");
        assert!(!out.contains("<h4"), "{out}");
    }

    #[test]
    fn renders_paragraph_text() {
        let out = render_markdown_segment("普通正文一段。", &theme());
        assert!(out.contains("普通正文一段。"));
    }

    #[test]
    fn soft_line_breaks_stay_inside_one_paragraph() {
        // 一段写三行是中文写作最正常的样子。此前每行各印一个 `<p>`，一段变三段、
        // 每段之间 1.5em 段距——用户写的是一段话，读者看到的是三个短句。
        let out = render_markdown_segment("第一行\n第二行\n第三行", &theme());
        assert_eq!(
            out.matches("margin: 0 0 1.5em").count(),
            1,
            "一段被拆成了多个 <p>：{out}"
        );
        assert_eq!(out.matches("<br>").count(), 2, "行间换行丢了：{out}");
    }

    #[test]
    fn blank_line_is_what_splits_paragraphs() {
        let out = render_markdown_segment("甲段\n\n乙段", &theme());
        assert_eq!(
            out.matches("margin: 0 0 1.5em").count(),
            2,
            "空行没有把两段分开：{out}"
        );
        assert!(!out.contains("<br>"), "空行被当成了软换行：{out}");
    }

    #[test]
    fn inline_syntax_spans_a_soft_line_break() {
        // 行内语法可以跨行。逐行渲染会让两半各看到奇数个反引号，把反引号原样印出来。
        let out = render_markdown_segment("代码 `a\nb` 结束", &theme());
        assert!(out.contains(">a<br>b<"), "行内代码跨行后没闭合：{out}");
        assert!(!out.contains('`'), "反引号漏进正文：{out}");

        let bold = render_markdown_segment("加粗 **甲\n乙** 结束", &theme());
        assert!(bold.contains("<strong"), "跨行加粗没生效：{bold}");
        assert!(!bold.contains("**"), "星号漏进正文：{bold}");
    }

    #[test]
    fn paragraph_is_flushed_before_every_block() {
        // 段落缓冲一旦漏 flush，内容顺序就会错——这种错读者能看见但很难归因。
        let out = render_markdown_segment(
            "段前\n\n## 标题\n\n- 项\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n\n段后\n",
            &theme(),
        );
        let at = |needle: &str| {
            out.find(needle)
                .unwrap_or_else(|| panic!("{needle} 丢了：{out}"))
        };
        assert!(
            at("段前") < at("标题") && at("标题") < at("项") && at("项") < at("段后"),
            "块级元素顺序错乱：{out}"
        );
    }

    #[test]
    fn list_continuation_line_stays_inside_the_item() {
        // 中文写列表最常见的写法：一项写长了就换行。此前续行掉出列表变成顶格段落，
        // 读者看到「列表只有一行，剩下是另一段话」——这一行必须留在项目符号下面。
        let out = render_markdown_segment("- 第一项\n  续行说明\n  再一行", &theme());
        assert!(
            out.contains("续行说明") && out.contains("再一行"),
            "续行文字丢了：{out}"
        );
        assert_eq!(
            out.matches("padding-left:1.2em").count(),
            1,
            "续行被当成了新的列表项：{out}"
        );
        assert!(
            !out.contains("margin: 0 0 1.5em"),
            "续行掉出列表成了独立段落：{out}"
        );
    }

    #[test]
    fn blank_line_before_an_indented_line_ends_the_item() {
        // 空行是用户明确的分段信号。跨过它继续拼就是把两段话粘成一段。
        let out = render_markdown_segment("- 第一项\n\n  另一段话", &theme());
        assert!(
            out.contains("margin: 0 0 1.5em"),
            "空行之后的缩进行被吞进列表项：{out}"
        );
        assert!(out.contains("另一段话"));
    }

    #[test]
    fn single_space_indent_is_not_a_continuation() {
        // `- ` 的内容从第 2 列开始，1 格缩进对齐不到它，不构成「属于这一项」。
        let out = render_markdown_segment("- 第一项\n 半格缩进", &theme());
        assert!(
            out.contains("margin: 0 0 1.5em"),
            "一格缩进被误认成续行：{out}"
        );
    }

    #[test]
    fn continuation_is_not_swallowed_when_it_starts_a_block() {
        // 缩进的块级语法宁可退化成正文，也不许被吞进列表项。
        for body in ["# 缩进标题", "> 缩进引用", "| a | b |", "---"] {
            let md = format!("- 第一项\n  {body}");
            let out = render_markdown_segment(&md, &theme());
            assert!(
                !out.contains(&format!("第一项 {}", body.trim())),
                "块级语法被拼进了列表项（{body}）：{out}"
            );
        }
    }

    #[test]
    fn ordered_list_continuation_matches_its_marker_width() {
        // `1. ` 的内容从第 3 列开始，3 空格缩进的续行同样属于这一项。
        let out = render_markdown_segment("1. 有序一\n   续行说明", &theme());
        assert!(
            out.contains("有序一 续行说明"),
            "有序列表的续行没拼进同一项：{out}"
        );
        assert_eq!(out.matches(">1.<").count(), 1, "续行成了新的有序项：{out}");
    }

    #[test]
    fn checklist_continuation_stays_inside_the_item() {
        // 勾选项的续行同 bullet 列表，不能掉成段落。
        let out = render_markdown_segment("- [ ] 待办\n  说明文字", &theme());
        assert!(
            out.contains("待办 说明文字"),
            "勾选项续行没拼进同一项：{out}"
        );
        assert!(
            !out.contains("margin: 0 0 1.5em"),
            "勾选项续行掉成了独立段落：{out}"
        );
    }

    #[test]
    fn nested_item_after_a_continuation_keeps_its_depth() {
        // 续行判定必须排在 parse_list_item 之后：`  - 子项` 要先进列表当独立项，
        // 不能被上一项吃掉。
        let out = render_markdown_segment("- 父项\n  续行\n  - 子项", &theme());
        assert!(
            out.contains("padding-left:2.5em;text-indent:-1.2em"),
            "子项被续行吞掉或压平：{out}"
        );
        assert!(out.contains("子项"));
    }

    #[test]
    fn nested_list_items_keep_their_depth() {
        // 缩进是列表唯一的层级信号。此前 parse 用的是 trim 过的行，
        // 缩进被吃掉，二级项渲染成和顶层一模一样的一行，读者分不出从属关系。
        let out = render_markdown_segment("- 顶层\n  - 二级\n- 顶层之二", &theme());
        assert!(
            out.contains("padding-left:1.2em;text-indent:-1.2em"),
            "顶层项缩进被改动：{out}"
        );
        assert!(
            out.contains("padding-left:2.5em;text-indent:-1.2em"),
            "二级项被压平成顶层，层级丢失：{out}"
        );
        assert_eq!(out.matches("二级").count(), 1, "文字不应被重复：{out}");
    }

    #[test]
    fn ordered_list_children_indent_by_marker_width() {
        // `1. ` 的子项按三个空格缩进是 GFM 常见写法，也要能进一级。
        let out = render_markdown_segment("1. 父项\n   1. 子项", &theme());
        assert!(
            out.contains("padding-left:2.5em;text-indent:-1.2em"),
            "三空格缩进的有序子项没进级：{out}"
        );
    }

    #[test]
    fn list_indent_is_capped_instead_of_escaping_the_card() {
        // 手机端正文窄，四级以上缩进会把文字挤出可视区；封顶到三级。
        let md = "- 一\n  - 二\n    - 三\n      - 四\n        - 五";
        let out = render_markdown_segment(md, &theme());
        assert_eq!(
            out.matches("padding-left:5.1em").count(),
            2,
            "四级、五级都该封在三级：{out}"
        );
        assert!(out.contains("五"));
    }

    #[test]
    fn four_space_indentation_is_one_level_not_two() {
        // 编辑器把 Tab 展开成 4 空格是最常见的写法，不能被算成两级。
        let out = render_markdown_segment("- 父项\n    - 子项", &theme());
        assert!(
            out.contains("padding-left:2.5em;text-indent:-1.2em"),
            "4 空格缩进被算成两级：{out}"
        );
        assert!(!out.contains("padding-left:3.8em"), "多进了一级：{out}");
    }

    #[test]
    fn blockquote_list_markers_do_not_leak_as_text() {
        // `> - 项` 此前被当成普通引用行原样输出，正文里出现字面的「- 」。
        let out = render_markdown_segment("> - 第一条\n> - 第二条", &theme());
        assert!(
            !out.contains("- 第一条"),
            "引用里的列表项漏出了字面 - 号：{out}"
        );
        assert!(out.contains('•'), "引用里的列表没有渲染成圆点：{out}");
        assert!(out.contains("第一条") && out.contains("第二条"));
    }

    #[test]
    fn blockquote_ordered_list_keeps_numbers() {
        let out = render_markdown_segment("> 1. 甲\n> 2. 乙", &theme());
        assert!(
            !out.contains("1. 甲"),
            "引用里的有序列表漏出了字面序号：{out}"
        );
        assert!(out.contains(">1.</span>") || out.contains(">1.</span>甲"));
        assert!(out.contains("甲") && out.contains("乙"));
    }

    #[test]
    fn blockquote_plain_text_is_untouched() {
        let out = render_markdown_segment("> 普通引用一句话", &theme());
        assert!(out.contains("普通引用一句话"));
        assert!(!out.contains('•'), "普通引用不该长出圆点：{out}");
    }

    #[test]
    fn blockquote_url_wraps_without_overflow() {
        // 长 URL / 长 token 在深色信息卡里必须能换行，不能在手机端横向溢出被裁切。
        let md = "> **仓库**：github.com/qiaopengjun5162/moonpub/blob/main/src/lib.rs";
        let out = render_markdown_segment(md, &theme());
        assert!(
            out.contains("overflow-wrap:break-word"),
            "blockquote 卡片缺少 overflow-wrap，长 URL 会在手机端溢出"
        );
        assert!(
            out.contains("word-break:break-word"),
            "blockquote 卡片缺少 word-break，长 URL 会在手机端溢出"
        );
        assert!(out.contains("github.com/qiaopengjun5162/moonpub"));
    }

    #[test]
    fn blockquote_nested_quote_does_not_leak_angle_bracket() {
        // `> > 内层` 此前只剥一层 `>`，内层那个原样印进正文。
        let out = render_markdown_segment("> 外层一句话。\n>\n> > 内层一句话。", &theme());
        assert!(
            !visible_text(&out).contains("> 内层一句话"),
            "嵌套引用漏出了字面 > 号：{out}"
        );
        assert!(out.contains("内层一句话"), "内层引用内容丢失：{out}");
        assert!(
            out.matches("border-left: 4px solid").count() >= 2,
            "嵌套引用没有渲成两层卡片：{out}"
        );
    }

    #[test]
    fn blockquote_code_fence_keeps_code_block() {
        // `> ```js` 此前被当普通行，语言标记 `js` 和代码拼进同一个行内代码块。
        let md = "> 引用里带代码：\n>\n> ```js\n> const a = 1;\n> ```";
        let out = render_markdown_segment(md, &theme());
        assert!(
            out.contains("font-family:Menlo"),
            "引用里的代码块降级成了正文：{out}"
        );
        assert!(!out.contains("jsconst"), "语言标记被拼进了代码正文：{out}");
        assert!(out.contains("const"), "代码内容丢失：{out}");
    }

    #[test]
    fn blockquote_heading_does_not_leak_hashes() {
        let out = render_markdown_segment("> ## 引用里的小标题\n>\n> 正文。", &theme());
        assert!(
            !out.contains("## 引用里的小标题"),
            "引用里的标题漏出了井号：{out}"
        );
        assert!(out.contains("引用里的小标题"));
        assert!(
            out.contains("<h2") || out.contains("<h3") || out.contains("<h4"),
            "引用里的标题没有渲染成标题：{out}"
        );
    }

    #[test]
    fn blockquote_marker_without_space_is_recognized() {
        // CommonMark 允许 `>` 后不带空格，中文写作常写 `>引用`。
        let out = render_markdown_segment(">无空格引用一句话", &theme());
        assert!(
            !visible_text(&out).contains(">无空格引用一句话"),
            "无空格引用漏出了字面 > 号：{out}"
        );
        assert!(out.contains("无空格引用一句话"), "引用内容丢失：{out}");
    }

    #[test]
    fn blockquote_blank_line_separates_paragraphs() {
        // 引用里的 `>` 空行是段落分隔，此前被整行丢掉，两段挤成一段。
        let out = render_markdown_segment("> 第一段。\n>\n> 第二段。", &theme());
        assert!(
            out.matches("<p ").count() == 2,
            "引用里的空行没有分开段落：{out}"
        );
    }

    #[test]
    fn reference_definition_does_not_leak_into_body() {
        // `[label]: url` 是链接定义，不是正文。老实现把它当普通段落输出，
        // 读者会在文章末尾看到「[homepage]: https://…」这一整行。
        let out = render_markdown_segment("[homepage]: https://example.com\n", &theme());
        assert!(
            !visible_text(&out).contains("[homepage]"),
            "定义行漏进了正文：{out}"
        );
        assert!(!out.contains("<a "), "定义行被自动链接成可见链接：{out}");
    }

    #[test]
    fn reference_link_resolves_to_anchor() {
        let out = render_markdown_segment(
            "见[贡献者公约][homepage]。\n\n[homepage]: https://example.com/coc",
            &theme(),
        );
        assert!(
            out.contains("href=\"https://example.com/coc\""),
            "引用式链接没有解析成链接：{out}"
        );
        assert!(
            !visible_text(&out).contains("[贡献者公约][homepage]"),
            "引用式链接标记漏进了正文：{out}"
        );
        assert!(visible_text(&out).contains("贡献者公约"), "{out}");
    }

    #[test]
    fn collapsed_reference_uses_its_own_text_as_label() {
        let out =
            render_markdown_segment("见[文档][]。\n\n[文档]: https://example.com/d", &theme());
        assert!(out.contains("href=\"https://example.com/d\""), "{out}");
        assert!(!visible_text(&out).contains("[]"), "{out}");
    }

    #[test]
    fn shortcut_reference_resolves_when_label_is_defined() {
        let out = render_markdown_segment("见[文档]。\n\n[文档]: https://example.com/d", &theme());
        assert!(out.contains("href=\"https://example.com/d\""), "{out}");
        assert!(!visible_text(&out).contains("[文档]"), "{out}");
    }

    #[test]
    fn undefined_reference_stays_literal_text() {
        // 没有定义时不能凭空吃掉方括号：`[文字][ref]` 要原样留着。
        let out = render_markdown_segment("见[文字][nope]。", &theme());
        assert!(visible_text(&out).contains("[文字][nope]"), "{out}");
        assert!(!out.contains("<a "), "{out}");
    }

    #[test]
    fn reference_labels_ignore_case_and_extra_spaces() {
        // CommonMark：标签比对大小写不敏感，内部空白折叠成一个空格。
        let out = render_markdown_segment(
            "见[公约][Contributor   Covenant]。\n\n[contributor covenant]: https://example.com/c",
            &theme(),
        );
        assert!(out.contains("href=\"https://example.com/c\""), "{out}");
    }

    #[test]
    fn reference_definition_inside_code_fence_is_left_alone() {
        // 代码块里的 `[doc]: url` 是教学示例，不能被当成定义吃掉。
        // 代码块有语法高亮（文本被拆进多个 span），所以按可见文本断言。
        let out = render_markdown_segment("```\n[doc]: https://example.com\n```", &theme());
        let text = visible_text(&out);
        assert!(text.contains("[doc]:"), "代码块内容被吃掉了：{out}");
        assert!(text.contains("https://example.com"), "{out}");
        assert!(!out.contains("<a "), "代码块里不该生成链接：{out}");
    }

    #[test]
    fn blockquote_can_use_outer_reference() {
        let out =
            render_markdown_segment("> 见[文档]。\n\n[文档]: https://example.com/d", &theme());
        assert!(
            out.contains("href=\"https://example.com/d\""),
            "引用块里用不到外层定义：{out}"
        );
    }

    #[test]
    fn reference_definition_accepts_zero_whitespace_and_titles() {
        // CommonMark 允许 `[a]:url`（冒号后不空空格），也允许定义后面跟标题。
        // 判定比规范严，合法写法就会安静地变成正文。
        let out = render_markdown_segment("见[文档][d]。\n\n[d]:https://example.com/z", &theme());
        assert!(
            out.contains("href=\"https://example.com/z\""),
            "零空白的定义没被识别：{out}"
        );

        let out = render_markdown_segment(
            "见[文档][d]。\n\n[d]: <https://example.com/s p> \"标题\"",
            &theme(),
        );
        assert!(
            out.contains("href=\"https://example.com/s&#32;p\"")
                || out.contains("href=\"https://example.com/s p\""),
            "尖括号目标被截坏：{out}"
        );
        assert!(
            !out.contains("href=\"&lt;"),
            "尖括号被当成地址的一部分：{out}"
        );
    }

    #[test]
    fn reference_image_resolves_when_label_is_defined() {
        let out = render_markdown_segment(
            "![封面][cover]\n\n[cover]: https://example.com/c.png",
            &theme(),
        );
        assert!(
            out.contains("src=\"https://example.com/c.png\""),
            "引用式图片没有解析：{out}"
        );
    }

    #[test]
    fn linked_badge_wraps_image_in_anchor() {
        // shields.io 徽章的标准写法：`[![alt][img]][link]`。图片套在链接里，
        // 链接文字本身又是一个引用式图片。老实现找链接文字时只认第一个 `]`，
        // 会把徽章拆成 `<a>![NPM version</a>]<a>npm-url</a>` 这种乱码。
        let md = "[![NPM version][img]][url]\n\n[img]: https://img.example.com/v.svg\n[url]: https://npmjs.org/package/x";
        let out = render_markdown_segment(md, &theme());
        assert!(
            out.contains("<a href=\"https://npmjs.org/package/x\""),
            "徽章的链接部分没解析成链接：{out}"
        );
        assert!(
            out.contains("src=\"https://img.example.com/v.svg\""),
            "徽章的图片部分没解析成图片：{out}"
        );
        assert!(
            !visible_text(&out).contains("]["),
            "徽章标记漏进了正文：{out}"
        );
    }

    #[test]
    fn inline_linked_image_keeps_nested_brackets() {
        // 行内形式的同一个问题：`[![a](x.png)](https://x)`。
        let out = render_markdown_segment("[![a](/x.png)](https://example.com)", &theme());
        assert!(
            out.contains("href=\"https://example.com\""),
            "外层链接目标丢失：{out}"
        );
        assert!(out.contains("src=\"/x.png\""), "内层图片丢失：{out}");
        assert!(!visible_text(&out).contains("]("), "{out}");
    }

    #[test]
    fn contributor_covenant_attribution_renders_a_real_link() {
        // 仓库 CODE_OF_CONDUCT.md 的真实两行。修复前正文露出
        // `[Contributor Covenant][homepage]`，文末还多一行 `[homepage]: https://…`。
        let md = "This Code of Conduct is adapted from the [Contributor Covenant][homepage],\n\n[homepage]: https://www.contributor-covenant.org";
        let out = render_markdown_segment(md, &theme());
        assert!(
            out.contains("href=\"https://www.contributor-covenant.org\""),
            "{out}"
        );
        let text = visible_text(&out);
        // 定义行一旦漏进正文就会多出一个段落。注意不能只断言可见文本里没有
        // `[homepage]`——定义被收进表之后，裸写的 `[homepage]` 会被简写引用
        // 解析成链接，方括号自己消失了，断言会假绿。段落数才是硬证据。
        assert!(
            out.matches("<p ").count() == 1,
            "定义行被当成正文多渲染了一个段落：{out}"
        );
        assert!(!text.contains("]["), "引用标记漏进正文：{out}");
    }
}
