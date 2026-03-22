#!/usr/bin/env python3
"""
Second-pass refactoring:
1. Replace 'self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?'
   with 'self.get_postgres_pool()?'
2. Remove any remaining SqlDialect references
3. Fix any other leftover artifacts
"""

import re
import os

BASE = r"C:\semaphore_ui_rust\rust_semaphore\rust\src\db\sql"

# Pattern to replace
OLD_POOL_CALL = r'self\.get_postgres_pool\(\)\.ok_or_else\(\|\| Error::Other\("PostgreSQL pool not found"\.to_string\(\)\)\)\?'
NEW_POOL_CALL = 'self.get_postgres_pool()?'

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        text = f.read()

    original = text

    # Replace the verbose ok_or_else pattern with the simpler ?
    text = re.sub(OLD_POOL_CALL, NEW_POOL_CALL, text)

    # Remove any remaining SqlDialect:: references that might exist
    # These would be unreachable match arms or imports
    text = re.sub(r'\s*SqlDialect::SQLite\s*=>\s*\{[^{}]*\}\s*', ' ', text)
    text = re.sub(r'\s*SqlDialect::MySQL\s*=>\s*\{[^{}]*\}\s*', ' ', text)

    # Remove standalone SqlDialect imports that might remain
    text = re.sub(r'\nuse crate::db::sql::types::SqlDialect;\n', '\n', text)
    text = re.sub(r'\nuse crate::db::sql::SqlDialect;\n', '\n', text)
    # Remove from combined imports
    text = re.sub(r',\s*SqlDialect\b', '', text)
    text = re.sub(r'\bSqlDialect\b,\s*', '', text)

    # Remove get_sqlite_pool, get_mysql_pool references
    text = re.sub(r'self\.get_sqlite_pool\(\)', 'self.get_postgres_pool()', text)
    text = re.sub(r'self\.get_mysql_pool\(\)', 'self.get_postgres_pool()', text)
    text = re.sub(r'self\.get_dialect\(\)', '/* dialect removed */', text)

    if text != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(text)
        return True
    return False


def process_all_rs_files(directory):
    changed = []
    for root, dirs, files in os.walk(directory):
        # Skip the sqlite/ and mysql/ subdirectories from processing (we'll handle those separately)
        dirs[:] = [d for d in dirs if d not in ('sqlite', 'mysql')]
        for fname in files:
            if fname.endswith('.rs'):
                fpath = os.path.join(root, fname)
                if process_file(fpath):
                    changed.append(fpath)
    return changed


if __name__ == '__main__':
    changed = process_all_rs_files(BASE)
    print(f"Modified {len(changed)} files:")
    for f in changed:
        print(f"  {os.path.basename(f)}")
