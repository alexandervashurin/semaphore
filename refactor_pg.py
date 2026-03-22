#!/usr/bin/env python3
"""
Refactor Rust SQL files to remove SQLite/MySQL match arms,
keeping only PostgreSQL code paths.
"""

import re
import os
import sys

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


def find_match_get_dialect_blocks(text):
    """Find all 'match self.get_dialect() { ... }' blocks and return list of (start, end) positions."""
    pattern = re.compile(r'match\s+self\.get_dialect\(\)\s*\{')
    results = []
    for m in pattern.finditer(text):
        # Find the opening brace of match
        open_brace = text.index('{', m.start())
        close_brace = extract_block_from_pos(text, open_brace)
        if close_brace == -1:
            continue
        results.append((m.start(), close_brace + 1))
    return results


def extract_postgres_arm_body(match_block_text):
    """
    From the full text of 'match self.get_dialect() { ... }',
    extract just the body of the PostgreSQL arm.
    """
    # Find PostgreSQL arm
    pg_pattern = re.compile(r'SqlDialect::PostgreSQL\s*=>\s*\{')
    m = pg_pattern.search(match_block_text)
    if not m:
        return None

    # Find opening brace of this arm
    open_brace_pos = match_block_text.index('{', m.start())
    close_brace_pos = extract_block_from_pos(match_block_text, open_brace_pos)
    if close_brace_pos == -1:
        return None

    # Extract body (without the outer braces)
    body = match_block_text[open_brace_pos + 1:close_brace_pos]

    # Dedent by one level (4 spaces)
    lines = body.split('\n')
    dedented = []
    for line in lines:
        if line.startswith('                '):
            dedented.append(line[4:])
        elif line.startswith('    '):
            dedented.append(line[4:])
        else:
            dedented.append(line)

    return '\n'.join(dedented)


def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        text = f.read()

    original = text

    # Find all match blocks from end to beginning to not mess up positions
    blocks = find_match_get_dialect_blocks(text)
    if not blocks:
        # No match blocks, but still need to clean up imports
        pass
    else:
        # Process from last to first to preserve positions
        for start, end in reversed(blocks):
            match_block_text = text[start:end]
            pg_body = extract_postgres_arm_body(match_block_text)
            if pg_body is None:
                print(f"  WARNING: No PostgreSQL arm found in {filepath} at pos {start}")
                continue
            text = text[:start] + pg_body.strip() + text[end:]

    # Clean up imports - remove SqlDialect, SqlitePool, MySqlPool references
    # Remove 'use crate::db::sql::types::SqlDialect;' or 'use crate::db::sql::SqlDialect;'
    text = re.sub(r'\nuse crate::db::sql::types::SqlDialect;\n', '\n', text)
    text = re.sub(r'\nuse crate::db::sql::SqlDialect;\n', '\n', text)
    text = re.sub(r',\s*SqlDialect\b', '', text)
    text = re.sub(r'\bSqlDialect\b[^;,\n]*', '', text)

    # Remove SqlitePool, MySqlPool from use statements
    text = re.sub(r'use sqlx::\{SqlitePool, PgPool, MySqlPool, Row\};', 'use sqlx::{PgPool, Row};', text)
    text = re.sub(r'use sqlx::\{SqlitePool, PgPool, MySqlPool\};', 'use sqlx::PgPool;', text)
    text = re.sub(r'use sqlx::\{SqlitePool, MySqlPool, PgPool\};', 'use sqlx::PgPool;', text)
    text = re.sub(r'use sqlx::\{SqlitePool, PgPool\};', 'use sqlx::PgPool;', text)
    text = re.sub(r'use sqlx::\{PgPool, MySqlPool\};', 'use sqlx::PgPool;', text)
    text = re.sub(r'use sqlx::\{PgPool, Row\};', 'use sqlx::{PgPool, Row};', text)  # keep this

    # Remove old pool accessor calls that would now fail
    # Replace .ok_or_else(|| Error::Other("PostgreSQL pool not found"...))? with simpler ?
    # We'll handle this via the get_postgres_pool() method in mod.rs returning Result<>

    if text != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(text)
        return True
    return False


def process_manager_file(filepath):
    """Process a manager file - extract PostgreSQL arms from match blocks."""
    with open(filepath, 'r', encoding='utf-8') as f:
        text = f.read()

    original = text

    # Find all match blocks
    blocks = find_match_get_dialect_blocks(text)

    if blocks:
        # Process from last to first
        for start, end in reversed(blocks):
            match_block_text = text[start:end]
            pg_body = extract_postgres_arm_body(match_block_text)
            if pg_body is None:
                print(f"  WARNING: No PostgreSQL arm found in {filepath} at pos {start}")
                continue
            text = text[:start] + pg_body.strip() + text[end:]

    # Now fix the imports
    # Remove SqlDialect import
    text = re.sub(r'\nuse crate::db::sql::types::SqlDialect;\n', '\n', text)
    text = re.sub(r'\nuse crate::db::sql::SqlDialect;\n', '\n', text)
    text = re.sub(r'\nuse crate::db::sql::types::\{SqlDb, SqlDialect\};\n',
                  '\nuse crate::db::sql::types::SqlDb;\n', text)

    # Fix sqlx imports to remove Sqlite/MySQL pools
    text = re.sub(r'use sqlx::\{SqlitePool,\s*PgPool,\s*MySqlPool,\s*Row\};', 'use sqlx::{PgPool, Row};', text)
    text = re.sub(r'use sqlx::\{SqlitePool,\s*MySqlPool,\s*PgPool,\s*Row\};', 'use sqlx::{PgPool, Row};', text)
    text = re.sub(r'use sqlx::\{SqlitePool,\s*PgPool,\s*Row\};', 'use sqlx::{PgPool, Row};', text)
    text = re.sub(r'use sqlx::\{PgPool,\s*MySqlPool,\s*Row\};', 'use sqlx::{PgPool, Row};', text)
    text = re.sub(r'use sqlx::\{SqlitePool,\s*PgPool,\s*MySqlPool\};', 'use sqlx::PgPool;', text)
    text = re.sub(r'use sqlx::\{SqlitePool,\s*PgPool\};', 'use sqlx::PgPool;', text)
    text = re.sub(r'use sqlx::\{PgPool,\s*MySqlPool\};', 'use sqlx::PgPool;', text)
    text = re.sub(r'use sqlx::\{SqlitePool,\s*Row\};', 'use sqlx::Row;', text)
    text = re.sub(r'use sqlx::SqlitePool;', '', text)
    text = re.sub(r'use sqlx::MySqlPool;', '', text)

    if text != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(text)
        return True
    return False


def main():
    managers_dir = os.path.join(BASE, "managers")

    print("=== Processing manager files ===")
    for fname in sorted(os.listdir(managers_dir)):
        if not fname.endswith('.rs'):
            continue
        fpath = os.path.join(managers_dir, fname)
        changed = process_manager_file(fpath)
        if changed:
            print(f"  Modified: {fname}")
        else:
            print(f"  Unchanged: {fname}")


if __name__ == '__main__':
    main()
