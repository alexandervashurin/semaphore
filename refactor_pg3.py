#!/usr/bin/env python3
"""
Fix mangled match blocks in legacy sql/ files.
The script broke things by:
1. Replacing 'self.get_dialect()' with '/* dialect removed */'
   making 'match /* dialect removed */ { ... }' invalid
2. Leaving 'crate::db::sql::types:: _' which is invalid syntax

Strategy:
- For 'match /* dialect removed */ { ... }' blocks:
  Find the first valid arm body (SQLite or PostgreSQL) and use it directly
  (converting SQLite ? placeholders to PostgreSQL $N if needed)
- Remove 'crate::db::sql::types::SqlDialect::SQLite =>' arms
"""

import re
import os

BASE = r"C:\semaphore_ui_rust\rust_semaphore\rust\src\db\sql"


def extract_block_from_pos(text, open_brace_pos):
    """Given position of '{', find matching '}'."""
    depth = 0
    i = open_brace_pos
    while i < len(text):
        if text[i] == '{':
            depth += 1
        elif text[i] == '}':
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def find_arm_body(block_text, dialect_pattern):
    """Find arm body for a given dialect pattern."""
    m = re.search(dialect_pattern + r'\s*=>\s*\{', block_text)
    if not m:
        return None
    open_brace = block_text.index('{', m.start())
    close_brace = extract_block_from_pos(block_text, open_brace)
    if close_brace == -1:
        return None
    return block_text[open_brace + 1:close_brace]


def fix_broken_match_blocks(text):
    """
    Fix 'match /* dialect removed */ { ... }' by extracting the first available arm body.
    Priority: PostgreSQL > SQLite (since we're removing SQLite).
    """
    pattern = re.compile(r'match\s*/\*\s*dialect removed\s*\*/\s*\{')
    results = []
    for m in pattern.finditer(text):
        open_brace = text.index('{', m.start())
        close_brace = extract_block_from_pos(text, open_brace)
        if close_brace == -1:
            continue
        results.append((m.start(), close_brace + 1))

    # Process from last to first
    for start, end in reversed(results):
        block = text[start:end]

        # Try PostgreSQL first
        body = find_arm_body(block, r'(?:super::)+\s*SqlDialect::PostgreSQL|SqlDialect::PostgreSQL|crate::db::sql::types::SqlDialect::PostgreSQL')
        if body is None:
            # Fall back to SQLite (convert ? to $N)
            body = find_arm_body(block, r'crate::db::sql::types::SqlDialect::SQLite|SqlDialect::SQLite')

        if body is None:
            # Try wildcard arm
            m_wild = re.search(r'_\s*=>\s*\{', block)
            if m_wild:
                ob = block.index('{', m_wild.start())
                cb = extract_block_from_pos(block, ob)
                if cb != -1:
                    body = block[ob + 1:cb]

        if body is not None:
            text = text[:start] + body.strip() + text[end:]

    return text


def fix_invalid_dialect_refs(text):
    """Fix 'crate::db::sql::types:: _' and similar invalid references."""
    # 'crate::db::sql::types:: _' - broken wildcard from script
    # These appear as 'crate::db::sql::types:: _ => expr' and need to be simplified
    text = re.sub(r'crate::db::sql::types::\s*_\s*=>\s*[^\n]+\n', '\n', text)
    text = re.sub(r'SqlDialect::\s*_\s*=>\s*[^\n]+\n', '\n', text)
    return text


def fix_super_dialect_refs(text):
    """Fix 'super:: super::SqlDialect::PostgreSQL' patterns."""
    # These come from 'super::' + SqlDialect concatenation
    text = re.sub(r'(?:super::\s*)+SqlDialect::PostgreSQL\s*=>', 'SqlDialect::PostgreSQL =>', text)
    # Also fix 'super:: };' artifacts
    text = re.sub(r'super::\s*\};', '}', text)
    text = re.sub(r'super::\s*\}', '}', text)
    return text


def remove_dialect_imports(text):
    """Remove SqlDialect imports and get_dialect references that are no longer valid."""
    text = re.sub(r'\nuse crate::db::sql::types::SqlDialect;\n', '\n', text)
    text = re.sub(r'\nuse crate::db::sql::SqlDialect;\n', '\n', text)
    # Remove get_dialect() that returns /* dialect removed */
    text = re.sub(r'/\*\s*dialect removed\s*\*/', 'unreachable!()', text)
    return text


def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        text = f.read()
    original = text

    text = fix_broken_match_blocks(text)
    text = fix_invalid_dialect_refs(text)
    text = fix_super_dialect_refs(text)
    text = remove_dialect_imports(text)

    if text != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(text)
        return True
    return False


def main():
    # Process legacy files in sql/ (not managers/)
    sql_dir = BASE
    changed = []
    for fname in os.listdir(sql_dir):
        if fname.endswith('.rs'):
            fpath = os.path.join(sql_dir, fname)
            if process_file(fpath):
                changed.append(fname)

    # Also process managers/
    managers_dir = os.path.join(BASE, 'managers')
    for fname in os.listdir(managers_dir):
        if fname.endswith('.rs'):
            fpath = os.path.join(managers_dir, fname)
            if process_file(fpath):
                changed.append('managers/' + fname)

    print(f"Fixed {len(changed)} files:")
    for f in sorted(changed):
        print(f"  {f}")


if __name__ == '__main__':
    main()
