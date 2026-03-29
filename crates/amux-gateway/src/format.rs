//! Gateway platform message formatting and chunking.

pub const SLACK_MAX_CHARS: usize = 4000;
pub const DISCORD_MAX_CHARS: usize = 2000;
pub const TELEGRAM_MAX_CHARS: usize = 4096;

pub fn markdown_to_slack_mrkdwn(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_code_block = false;
    let mut in_inline_code = false;

    while i < len {
        if i + 2 < len && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
            in_code_block = !in_code_block;
            result.push_str("```");
            i += 3;
            continue;
        }

        if chars[i] == '`' && !in_code_block {
            in_inline_code = !in_inline_code;
            result.push('`');
            i += 1;
            continue;
        }

        if in_code_block || in_inline_code {
            result.push(chars[i]);
            i += 1;
            continue;
        }

        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(close) = find_closing_marker(&chars, i + 2, &['*', '*']) {
                result.push('\x01');
                i += 2;
                while i < close {
                    result.push(chars[i]);
                    i += 1;
                }
                result.push('\x01');
                i += 2;
                continue;
            }
        }

        if i + 1 < len && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(close) = find_closing_marker(&chars, i + 2, &['~', '~']) {
                result.push('~');
                i += 2;
                while i < close {
                    result.push(chars[i]);
                    i += 1;
                }
                result.push('~');
                i += 2;
                continue;
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result.replace('\x01', "*")
}

fn find_closing_marker(chars: &[char], start: usize, marker: &[char; 2]) -> Option<usize> {
    let mut j = start;
    while j + 1 < chars.len() {
        if chars[j] == marker[0] && chars[j + 1] == marker[1] {
            return Some(j);
        }
        j += 1;
    }
    None
}

pub fn markdown_to_discord(input: &str) -> String {
    input.to_string()
}

pub fn markdown_to_telegram_v2(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        if is_telegram_special(ch) {
            result.push('\\');
        }
        result.push(ch);
    }
    result
}

fn is_telegram_special(ch: char) -> bool {
    matches!(
        ch,
        '_' | '*'
            | '['
            | ']'
            | '('
            | ')'
            | '~'
            | '`'
            | '>'
            | '#'
            | '+'
            | '-'
            | '='
            | '|'
            | '{'
            | '}'
            | '.'
            | '!'
    )
}

pub fn markdown_to_plain(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if i + 2 < len && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
            i += 3;
            continue;
        }
        if i + 1 < len
            && ((chars[i] == '*' && chars[i + 1] == '*')
                || (chars[i] == '_' && chars[i + 1] == '_'))
        {
            i += 2;
            continue;
        }
        if i + 1 < len && chars[i] == '~' && chars[i + 1] == '~' {
            i += 2;
            continue;
        }
        if matches!(chars[i], '*' | '_' | '`') {
            i += 1;
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

pub fn chunk_message(message: &str, max_chars: usize) -> Vec<String> {
    if message.len() <= max_chars {
        return vec![message.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = message;

    while !remaining.is_empty() {
        if remaining.len() <= max_chars {
            chunks.push(remaining.to_string());
            break;
        }

        let window = &remaining[..max_chars];

        if let Some(pos) = window.rfind('\n') {
            if pos > 0 {
                chunks.push(remaining[..pos].to_string());
                remaining = &remaining[pos + 1..];
                continue;
            }
        }

        if let Some(pos) = window.rfind(". ") {
            if pos > 0 {
                chunks.push(remaining[..=pos].to_string());
                remaining = &remaining[pos + 2..];
                continue;
            }
        }

        if let Some(pos) = window.rfind(char::is_whitespace) {
            if pos > 0 {
                chunks.push(remaining[..pos].to_string());
                remaining = &remaining[pos + 1..];
                continue;
            }
        }

        chunks.push(remaining[..max_chars].to_string());
        remaining = &remaining[max_chars..];
    }

    chunks
}
