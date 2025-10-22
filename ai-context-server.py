#!/usr/bin/env python3
"""
AI Context Server for Data Designer
Provides real-time codebase context to AI assistants via HTTP API
"""

import os
import json
import time
import hashlib
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
from flask import Flask, jsonify, request, send_file
from flask_cors import CORS
import sqlite3
import threading
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class FileInfo:
    path: str
    size: int
    modified: float
    content_hash: str
    file_type: str
    lines: int

@dataclass
class CodebaseSnapshot:
    timestamp: float
    total_files: int
    total_lines: int
    file_types: Dict[str, int]
    files: List[FileInfo]

class AIContextServer:
    def __init__(self, project_root: str, port: int = 3737):
        self.project_root = Path(project_root).resolve()
        self.port = port
        self.app = Flask(__name__)
        CORS(self.app)

        # Database for caching and history
        self.db_path = self.project_root / ".ai-context-cache.db"
        self.init_database()

        # File watcher state
        self.last_scan = 0
        self.current_snapshot: Optional[CodebaseSnapshot] = None
        self.scan_interval = 30  # seconds

        # Setup routes
        self.setup_routes()

        # Start background file watcher
        self.start_file_watcher()

    def init_database(self):
        """Initialize SQLite database for caching"""
        conn = sqlite3.connect(self.db_path)
        conn.execute('''
            CREATE TABLE IF NOT EXISTS file_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp REAL,
                file_path TEXT,
                content_hash TEXT,
                file_size INTEGER,
                modified_time REAL,
                lines INTEGER
            )
        ''')
        conn.execute('''
            CREATE TABLE IF NOT EXISTS codebase_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp REAL,
                total_files INTEGER,
                total_lines INTEGER,
                file_types TEXT,
                git_commit TEXT
            )
        ''')
        conn.commit()
        conn.close()

    def start_file_watcher(self):
        """Start background thread to monitor file changes"""
        def watch_files():
            while True:
                try:
                    current_time = time.time()
                    if current_time - self.last_scan > self.scan_interval:
                        self.scan_codebase()
                        self.last_scan = current_time
                    time.sleep(5)  # Check every 5 seconds
                except Exception as e:
                    logger.error(f"File watcher error: {e}")
                    time.sleep(30)  # Wait longer on error

        watcher_thread = threading.Thread(target=watch_files, daemon=True)
        watcher_thread.start()
        logger.info("File watcher started")

    def scan_codebase(self) -> CodebaseSnapshot:
        """Scan the codebase and create a snapshot"""
        logger.info("Scanning codebase...")
        start_time = time.time()

        files = []
        file_types = {}
        total_lines = 0

        # Define file extensions to scan
        code_extensions = {
            '.rs': 'rust',
            '.py': 'python',
            '.js': 'javascript',
            '.ts': 'typescript',
            '.proto': 'protobuf',
            '.sql': 'sql',
            '.toml': 'toml',
            '.yaml': 'yaml',
            '.yml': 'yaml',
            '.json': 'json',
            '.md': 'markdown',
            '.sh': 'shell',
            '.txt': 'text',
            '.lisp': 'lisp',
            '.cbu': 'cbu-dsl'
        }

        # Directories to ignore
        ignore_dirs = {
            'target', '.git', 'node_modules', '.next', 'dist',
            'build', '__pycache__', '.vscode', '.idea', 'export'
        }

        for file_path in self.project_root.rglob('*'):
            # Skip if in ignored directory
            if any(part in ignore_dirs for part in file_path.parts):
                continue

            if file_path.is_file():
                ext = file_path.suffix.lower()
                if ext in code_extensions:
                    try:
                        stat = file_path.stat()

                        # Read file content for hash and line count
                        try:
                            with open(file_path, 'r', encoding='utf-8') as f:
                                content = f.read()
                                content_hash = hashlib.md5(content.encode()).hexdigest()
                                lines = len(content.splitlines())
                                total_lines += lines
                        except UnicodeDecodeError:
                            # Binary file, just get hash of path
                            content_hash = hashlib.md5(str(file_path).encode()).hexdigest()
                            lines = 0

                        file_type = code_extensions[ext]
                        file_types[file_type] = file_types.get(file_type, 0) + 1

                        file_info = FileInfo(
                            path=str(file_path.relative_to(self.project_root)),
                            size=stat.st_size,
                            modified=stat.st_mtime,
                            content_hash=content_hash,
                            file_type=file_type,
                            lines=lines
                        )
                        files.append(file_info)

                    except Exception as e:
                        logger.warning(f"Error processing {file_path}: {e}")
                        continue

        snapshot = CodebaseSnapshot(
            timestamp=time.time(),
            total_files=len(files),
            total_lines=total_lines,
            file_types=file_types,
            files=files
        )

        self.current_snapshot = snapshot
        self.save_snapshot(snapshot)

        scan_time = time.time() - start_time
        logger.info(f"Codebase scan completed in {scan_time:.2f}s: {len(files)} files, {total_lines} lines")

        return snapshot

    def save_snapshot(self, snapshot: CodebaseSnapshot):
        """Save snapshot to database"""
        try:
            # Get git commit if available
            git_commit = self.get_git_commit()

            conn = sqlite3.connect(self.db_path)

            # Save codebase snapshot
            conn.execute('''
                INSERT INTO codebase_snapshots (timestamp, total_files, total_lines, file_types, git_commit)
                VALUES (?, ?, ?, ?, ?)
            ''', (
                snapshot.timestamp,
                snapshot.total_files,
                snapshot.total_lines,
                json.dumps(snapshot.file_types),
                git_commit
            ))

            # Save individual file snapshots
            for file_info in snapshot.files:
                conn.execute('''
                    INSERT INTO file_snapshots (timestamp, file_path, content_hash, file_size, modified_time, lines)
                    VALUES (?, ?, ?, ?, ?, ?)
                ''', (
                    snapshot.timestamp,
                    file_info.path,
                    file_info.content_hash,
                    file_info.size,
                    file_info.modified,
                    file_info.lines
                ))

            conn.commit()
            conn.close()

        except Exception as e:
            logger.error(f"Error saving snapshot: {e}")

    def get_git_commit(self) -> Optional[str]:
        """Get current git commit hash"""
        try:
            result = subprocess.run(
                ['git', 'rev-parse', 'HEAD'],
                cwd=self.project_root,
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                return result.stdout.strip()
        except Exception:
            pass
        return None

    def setup_routes(self):
        """Setup Flask routes"""

        @self.app.route('/health')
        def health():
            return jsonify({
                'status': 'healthy',
                'timestamp': time.time(),
                'project_root': str(self.project_root),
                'last_scan': self.last_scan
            })

        @self.app.route('/api/codebase/current')
        def get_current_snapshot():
            """Get current codebase snapshot"""
            if not self.current_snapshot:
                self.scan_codebase()

            return jsonify({
                'snapshot': asdict(self.current_snapshot),
                'git_commit': self.get_git_commit(),
                'scan_age': time.time() - self.current_snapshot.timestamp if self.current_snapshot else 0
            })

        @self.app.route('/api/codebase/files')
        def get_files():
            """Get list of files with optional filtering"""
            if not self.current_snapshot:
                self.scan_codebase()

            file_type = request.args.get('type')
            search = request.args.get('search', '').lower()

            files = self.current_snapshot.files

            if file_type:
                files = [f for f in files if f.file_type == file_type]

            if search:
                files = [f for f in files if search in f.path.lower()]

            return jsonify({
                'files': [asdict(f) for f in files],
                'total_count': len(self.current_snapshot.files),
                'filtered_count': len(files)
            })

        @self.app.route('/api/codebase/file/<path:file_path>')
        def get_file_content(file_path):
            """Get content of specific file"""
            full_path = self.project_root / file_path

            if not full_path.exists() or not full_path.is_file():
                return jsonify({'error': 'File not found'}), 404

            try:
                with open(full_path, 'r', encoding='utf-8') as f:
                    content = f.read()

                stat = full_path.stat()

                return jsonify({
                    'path': file_path,
                    'content': content,
                    'size': stat.st_size,
                    'modified': stat.st_mtime,
                    'lines': len(content.splitlines())
                })

            except UnicodeDecodeError:
                return jsonify({'error': 'Binary file cannot be displayed as text'}), 400
            except Exception as e:
                return jsonify({'error': str(e)}), 500

        @self.app.route('/api/codebase/search')
        def search_content():
            """Search file contents"""
            query = request.args.get('q', '').strip()
            file_type = request.args.get('type')
            case_sensitive = request.args.get('case_sensitive', 'false').lower() == 'true'

            if not query:
                return jsonify({'error': 'Query parameter required'}), 400

            if not self.current_snapshot:
                self.scan_codebase()

            results = []
            search_query = query if case_sensitive else query.lower()

            for file_info in self.current_snapshot.files:
                if file_type and file_info.file_type != file_type:
                    continue

                try:
                    file_path = self.project_root / file_info.path
                    with open(file_path, 'r', encoding='utf-8') as f:
                        content = f.read()
                        search_content = content if case_sensitive else content.lower()

                        if search_query in search_content:
                            # Find matching lines
                            lines = content.splitlines()
                            matching_lines = []

                            for i, line in enumerate(lines):
                                search_line = line if case_sensitive else line.lower()
                                if search_query in search_line:
                                    matching_lines.append({
                                        'line_number': i + 1,
                                        'content': line.strip(),
                                        'context_before': lines[max(0, i-2):i] if i > 1 else [],
                                        'context_after': lines[i+1:min(len(lines), i+3)] if i < len(lines)-1 else []
                                    })

                            if matching_lines:
                                results.append({
                                    'file': file_info.path,
                                    'file_type': file_info.file_type,
                                    'matches': matching_lines[:10]  # Limit matches per file
                                })

                except (UnicodeDecodeError, FileNotFoundError):
                    continue

            return jsonify({
                'query': query,
                'results': results,
                'total_files_searched': len(self.current_snapshot.files),
                'files_with_matches': len(results)
            })

        @self.app.route('/api/codebase/exports')
        def get_exports():
            """Get available AI exports"""
            export_dir = self.project_root / 'export'

            if not export_dir.exists():
                return jsonify({'exports': []})

            exports = []
            for export_file in export_dir.glob('*.txt'):
                stat = export_file.stat()
                exports.append({
                    'name': export_file.stem,
                    'filename': export_file.name,
                    'size': stat.st_size,
                    'modified': stat.st_mtime,
                    'url': f'/api/codebase/exports/{export_file.name}'
                })

            return jsonify({'exports': exports})

        @self.app.route('/api/codebase/exports/<filename>')
        def download_export(filename):
            """Download specific export file"""
            export_path = self.project_root / 'export' / filename

            if not export_path.exists():
                return jsonify({'error': 'Export file not found'}), 404

            return send_file(export_path, as_attachment=True)

        @self.app.route('/api/codebase/history')
        def get_history():
            """Get codebase change history"""
            days = request.args.get('days', 7, type=int)
            cutoff_time = time.time() - (days * 24 * 60 * 60)

            conn = sqlite3.connect(self.db_path)
            cursor = conn.execute('''
                SELECT timestamp, total_files, total_lines, file_types, git_commit
                FROM codebase_snapshots
                WHERE timestamp > ?
                ORDER BY timestamp DESC
                LIMIT 100
            ''', (cutoff_time,))

            history = []
            for row in cursor.fetchall():
                history.append({
                    'timestamp': row[0],
                    'total_files': row[1],
                    'total_lines': row[2],
                    'file_types': json.loads(row[3]) if row[3] else {},
                    'git_commit': row[4]
                })

            conn.close()

            return jsonify({
                'history': history,
                'days': days
            })

        @self.app.route('/api/codebase/stats')
        def get_stats():
            """Get codebase statistics"""
            if not self.current_snapshot:
                self.scan_codebase()

            # Calculate additional stats
            stats = {
                'current_snapshot': asdict(self.current_snapshot),
                'git_info': {
                    'commit': self.get_git_commit(),
                    'branch': self.get_git_branch(),
                    'status': self.get_git_status()
                },
                'file_size_distribution': self.calculate_size_distribution(),
                'recent_changes': self.get_recent_file_changes()
            }

            return jsonify(stats)

        @self.app.route('/api/ai/context')
        def get_ai_context():
            """Get AI-specific context information"""
            context = {
                'project_name': 'Data Designer',
                'description': 'Web-First Financial DSL Platform with AI Integration',
                'architecture': 'Rust WASM + gRPC Microservices',
                'key_technologies': ['Rust', 'WASM', 'egui', 'gRPC', 'PostgreSQL', 'LSP'],
                'ai_features': [
                    'S-expression DSL with syntax highlighting',
                    'Language Server Protocol implementation',
                    'Multi-provider AI integration',
                    'Real-time code completion',
                    'Semantic search and RAG'
                ],
                'current_state': asdict(self.current_snapshot) if self.current_snapshot else None,
                'quick_start': './runwasm.sh --with-lsp',
                'documentation': [
                    'CLAUDE.md - Complete project overview',
                    'ai-context.md - AI assistant quick reference',
                    'LSP_USAGE.md - Language Server documentation',
                    'docs/ai-integration/ - Comprehensive AI guides'
                ],
                'development_workflow': [
                    'Review exports for context',
                    'Follow Rust best practices',
                    'Test with ./runwasm.sh',
                    'Update documentation',
                    'Regenerate AI exports'
                ]
            }

            return jsonify(context)

    def get_git_branch(self) -> Optional[str]:
        """Get current git branch"""
        try:
            result = subprocess.run(
                ['git', 'branch', '--show-current'],
                cwd=self.project_root,
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                return result.stdout.strip()
        except Exception:
            pass
        return None

    def get_git_status(self) -> Dict[str, Any]:
        """Get git status information"""
        try:
            result = subprocess.run(
                ['git', 'status', '--porcelain'],
                cwd=self.project_root,
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                lines = result.stdout.strip().split('\n') if result.stdout.strip() else []
                return {
                    'clean': len(lines) == 0,
                    'modified_files': len([l for l in lines if l.startswith(' M')]),
                    'added_files': len([l for l in lines if l.startswith('A ')]),
                    'untracked_files': len([l for l in lines if l.startswith('??')]),
                    'total_changes': len(lines)
                }
        except Exception:
            pass
        return {'clean': None, 'error': 'Could not get git status'}

    def calculate_size_distribution(self) -> Dict[str, int]:
        """Calculate file size distribution"""
        if not self.current_snapshot:
            return {}

        distribution = {
            'tiny': 0,      # < 1KB
            'small': 0,     # 1KB - 10KB
            'medium': 0,    # 10KB - 100KB
            'large': 0,     # 100KB - 1MB
            'huge': 0       # > 1MB
        }

        for file_info in self.current_snapshot.files:
            size = file_info.size
            if size < 1024:
                distribution['tiny'] += 1
            elif size < 10240:
                distribution['small'] += 1
            elif size < 102400:
                distribution['medium'] += 1
            elif size < 1048576:
                distribution['large'] += 1
            else:
                distribution['huge'] += 1

        return distribution

    def get_recent_file_changes(self, hours: int = 24) -> List[Dict[str, Any]]:
        """Get recently modified files"""
        if not self.current_snapshot:
            return []

        cutoff_time = time.time() - (hours * 60 * 60)
        recent_files = [
            asdict(f) for f in self.current_snapshot.files
            if f.modified > cutoff_time
        ]

        # Sort by modification time (newest first)
        recent_files.sort(key=lambda f: f['modified'], reverse=True)

        return recent_files[:20]  # Limit to 20 most recent

    def run(self):
        """Run the AI context server"""
        logger.info(f"Starting AI Context Server for {self.project_root}")
        logger.info(f"Server will be available at http://localhost:{self.port}")

        # Initial scan
        self.scan_codebase()

        # Start Flask app
        self.app.run(host='0.0.0.0', port=self.port, debug=False)

def main():
    import argparse

    parser = argparse.ArgumentParser(description='AI Context Server for Data Designer')
    parser.add_argument('--port', type=int, default=3737, help='Server port (default: 3737)')
    parser.add_argument('--project-root', default='.', help='Project root directory (default: current directory)')
    parser.add_argument('--scan-interval', type=int, default=30, help='File scan interval in seconds (default: 30)')

    args = parser.parse_args()

    server = AIContextServer(args.project_root, args.port)
    server.scan_interval = args.scan_interval

    try:
        server.run()
    except KeyboardInterrupt:
        logger.info("Server stopped by user")
    except Exception as e:
        logger.error(f"Server error: {e}")

if __name__ == '__main__':
    main()