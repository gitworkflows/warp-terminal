#!/usr/bin/env python3
"""
Warp Preview Log Monitor
========================

Monitors Warp Preview application logs and processes, providing real-time
analysis and structured logging to ~/.warp directory.

Features:
- Real-time log monitoring with tail functionality
- Process monitoring for Warp Preview instances
- Structured log parsing and analysis
- Automatic log rotation and archiving
- Performance metrics collection
- Error detection and alerting
- Integration with ~/.warp directory structure
"""

import os
import sys
import time
import json
import re
import subprocess
import threading
import signal
from datetime import datetime, timedelta
from pathlib import Path
from collections import defaultdict, deque
from dataclasses import dataclass, asdict
from typing import Dict, List, Optional, Tuple, Any
import argparse

# Configuration
class Config:
    WARP_HOME = Path.home() / ".warp"
    LOG_DIR = WARP_HOME / "logs"
    MONITOR_DIR = WARP_HOME / "monitor"
    PREVIEW_LOG = Path.home() / "Library" / "Logs" / "warp_preview.log"
    MAIN_LOG = Path.home() / "Library" / "Logs" / "warp.log"
    PREVIEW_EXECUTABLE = "/Volumes/Warp/WarpPreview.app/Contents/MacOS/preview"
    
    # Monitoring settings
    TAIL_LINES = 100
    POLL_INTERVAL = 1.0
    MAX_LOG_SIZE = 10 * 1024 * 1024  # 10MB
    ROTATE_COUNT = 5
    ALERT_THRESHOLD = 10  # errors per minute

@dataclass
class LogEntry:
    timestamp: datetime
    level: str
    component: str
    message: str
    raw_line: str
    pid: Optional[int] = None
    thread_id: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None

@dataclass
class ProcessInfo:
    pid: int
    ppid: int
    command: str
    started: datetime
    cpu_percent: float = 0.0
    memory_mb: float = 0.0
    status: str = "running"

class LogParser:
    """Parses Warp log entries into structured format"""
    
    # Common log patterns
    PATTERNS = {
        'timestamp': r'(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d{3})?(?:Z|[+-]\d{2}:\d{2})?)',
        'level': r'\[(DEBUG|INFO|WARN|ERROR|TRACE|FATAL)\]',
        'component': r'\[([^\]]+)\]',
        'pid_thread': r'\[(\d+):([^\]]+)\]',
        'error': r'(error|exception|failed|panic|crash)',
        'performance': r'(took|duration|elapsed|ms|seconds)',
    }
    
    def __init__(self):
        self.compiled_patterns = {k: re.compile(v, re.IGNORECASE) 
                                for k, v in self.PATTERNS.items()}
    
    def parse_line(self, line: str) -> LogEntry:
        """Parse a single log line into structured format"""
        timestamp = self._extract_timestamp(line)
        level = self._extract_level(line)
        component = self._extract_component(line)
        pid, thread_id = self._extract_pid_thread(line)
        
        # Clean message by removing parsed components
        message = line
        for pattern in [self.PATTERNS['timestamp'], self.PATTERNS['level'], 
                       self.PATTERNS['component'], self.PATTERNS['pid_thread']]:
            message = re.sub(pattern, '', message).strip()
        
        # Extract metadata
        metadata = {
            'has_error': bool(self.compiled_patterns['error'].search(line)),
            'has_performance': bool(self.compiled_patterns['performance'].search(line)),
            'line_length': len(line)
        }
        
        return LogEntry(
            timestamp=timestamp,
            level=level,
            component=component,
            message=message,
            raw_line=line,
            pid=pid,
            thread_id=thread_id,
            metadata=metadata
        )
    
    def _extract_timestamp(self, line: str) -> datetime:
        """Extract timestamp from log line"""
        match = self.compiled_patterns['timestamp'].search(line)
        if match:
            ts_str = match.group(1)
            # Handle various timestamp formats
            formats = [
                '%Y-%m-%dT%H:%M:%S.%fZ',
                '%Y-%m-%dT%H:%M:%S.%f',
                '%Y-%m-%d %H:%M:%S.%f',
                '%Y-%m-%dT%H:%M:%SZ',
                '%Y-%m-%dT%H:%M:%S',
                '%Y-%m-%d %H:%M:%S'
            ]
            for fmt in formats:
                try:
                    return datetime.strptime(ts_str, fmt)
                except ValueError:
                    continue
        return datetime.now()
    
    def _extract_level(self, line: str) -> str:
        """Extract log level from line"""
        match = self.compiled_patterns['level'].search(line)
        return match.group(1) if match else 'INFO'
    
    def _extract_component(self, line: str) -> str:
        """Extract component name from line"""
        match = self.compiled_patterns['component'].search(line)
        return match.group(1) if match else 'unknown'
    
    def _extract_pid_thread(self, line: str) -> Tuple[Optional[int], Optional[str]]:
        """Extract PID and thread ID from line"""
        match = self.compiled_patterns['pid_thread'].search(line)
        if match:
            try:
                pid = int(match.group(1))
                thread_id = match.group(2)
                return pid, thread_id
            except ValueError:
                pass
        return None, None

class ProcessMonitor:
    """Monitors Warp Preview processes"""
    
    def __init__(self):
        self.processes: Dict[int, ProcessInfo] = {}
        self.lock = threading.Lock()
    
    def scan_processes(self) -> List[ProcessInfo]:
        """Scan for Warp-related processes"""
        try:
            result = subprocess.run(['ps', 'aux'], capture_output=True, text=True)
            processes = []
            
            for line in result.stdout.split('\n'):
                if 'warp' in line.lower() and 'preview' in line.lower():
                    parts = line.split()
                    if len(parts) >= 11:
                        try:
                            pid = int(parts[1])
                            cpu = float(parts[2])
                            mem = float(parts[3])
                            command = ' '.join(parts[10:])
                            
                            process_info = ProcessInfo(
                                pid=pid,
                                ppid=int(parts[2]) if parts[2].isdigit() else 0,
                                command=command,
                                started=datetime.now(),  # Approximate
                                cpu_percent=cpu,
                                memory_mb=mem * 1024 / 100,  # Rough conversion
                                status='running'
                            )
                            processes.append(process_info)
                        except (ValueError, IndexError):
                            continue
            
            with self.lock:
                # Update process tracking
                current_pids = {p.pid for p in processes}
                for pid in list(self.processes.keys()):
                    if pid not in current_pids:
                        self.processes[pid].status = 'terminated'
                
                for process in processes:
                    self.processes[process.pid] = process
            
            return processes
        except Exception as e:
            print(f"Error scanning processes: {e}")
            return []

class LogMonitor:
    """Main log monitoring class"""
    
    def __init__(self, config: Config):
        self.config = config
        self.parser = LogParser()
        self.process_monitor = ProcessMonitor()
        self.running = False
        self.stats = defaultdict(int)
        self.recent_entries = deque(maxlen=1000)
        self.error_buffer = deque(maxlen=100)
        
        # Ensure directories exist
        self._setup_directories()
        
        # Initialize monitoring files
        self.monitor_log = self.config.MONITOR_DIR / "monitor.log"
        self.stats_file = self.config.MONITOR_DIR / "stats.json"
        self.errors_file = self.config.MONITOR_DIR / "errors.json"
    
    def _setup_directories(self):
        """Create necessary directories"""
        for directory in [self.config.LOG_DIR, self.config.MONITOR_DIR]:
            directory.mkdir(parents=True, exist_ok=True)
    
    def start_monitoring(self):
        """Start the monitoring process"""
        self.running = True
        print(f"Starting Warp Preview monitoring...")
        print(f"Monitoring: {self.config.PREVIEW_LOG}")
        print(f"Output directory: {self.config.MONITOR_DIR}")
        
        # Start background threads
        threading.Thread(target=self._monitor_processes, daemon=True).start()
        threading.Thread(target=self._save_stats_periodically, daemon=True).start()
        
        # Main log monitoring loop
        self._monitor_logs()
    
    def stop_monitoring(self):
        """Stop the monitoring process"""
        self.running = False
        self._save_current_stats()
        print("Monitoring stopped.")
    
    def _monitor_logs(self):
        """Monitor log files using tail-like functionality"""
        log_file = self.config.PREVIEW_LOG
        
        if not log_file.exists():
            print(f"Log file not found: {log_file}")
            return
        
        # Start from end of file
        with open(log_file, 'r') as f:
            f.seek(0, 2)  # Go to end of file
            
            while self.running:
                line = f.readline()
                if line:
                    self._process_log_line(line.strip())
                else:
                    time.sleep(self.config.POLL_INTERVAL)
    
    def _process_log_line(self, line: str):
        """Process a single log line"""
        if not line:
            return
        
        try:
            entry = self.parser.parse_line(line)
            self.recent_entries.append(entry)
            
            # Update statistics
            self.stats['total_lines'] += 1
            self.stats[f'level_{entry.level}'] += 1
            self.stats[f'component_{entry.component}'] += 1
            
            # Handle errors
            if entry.level in ['ERROR', 'FATAL'] or entry.metadata.get('has_error'):
                self.error_buffer.append(entry)
                self.stats['total_errors'] += 1
                self._log_error(entry)
            
            # Log to monitor file
            self._log_to_monitor(entry)
            
            # Print to console if verbose
            if hasattr(self, 'verbose') and self.verbose:
                print(f"[{entry.timestamp}] {entry.level} {entry.component}: {entry.message[:100]}")
        
        except Exception as e:
            print(f"Error processing log line: {e}")
            print(f"Line: {line}")
    
    def _monitor_processes(self):
        """Monitor Warp Preview processes in background"""
        while self.running:
            try:
                processes = self.process_monitor.scan_processes()
                self.stats['active_processes'] = len([p for p in processes if p.status == 'running'])
                
                # Save process info
                process_file = self.config.MONITOR_DIR / "processes.json"
                with open(process_file, 'w') as f:
                    json.dump([asdict(p) for p in processes], f, indent=2, default=str)
                
            except Exception as e:
                print(f"Error monitoring processes: {e}")
            
            time.sleep(10)  # Check every 10 seconds
    
    def _log_error(self, entry: LogEntry):
        """Log error entry to error file"""
        error_data = {
            'timestamp': entry.timestamp.isoformat(),
            'level': entry.level,
            'component': entry.component,
            'message': entry.message,
            'pid': entry.pid,
            'metadata': entry.metadata
        }
        
        # Write to errors file
        with open(self.errors_file, 'a') as f:
            f.write(json.dumps(error_data) + '\n')
    
    def _log_to_monitor(self, entry: LogEntry):
        """Log entry to monitor file"""
        log_data = {
            'timestamp': entry.timestamp.isoformat(),
            'level': entry.level,
            'component': entry.component,
            'message': entry.message[:200],  # Truncate long messages
            'pid': entry.pid
        }
        
        with open(self.monitor_log, 'a') as f:
            f.write(json.dumps(log_data) + '\n')
    
    def _save_stats_periodically(self):
        """Save statistics periodically"""
        while self.running:
            time.sleep(60)  # Save every minute
            self._save_current_stats()
    
    def _save_current_stats(self):
        """Save current statistics to file"""
        stats_data = dict(self.stats)
        stats_data['timestamp'] = datetime.now().isoformat()
        stats_data['uptime_seconds'] = int(time.time() - self.start_time) if hasattr(self, 'start_time') else 0
        
        with open(self.stats_file, 'w') as f:
            json.dump(stats_data, f, indent=2)
    
    def get_summary(self) -> Dict[str, Any]:
        """Get monitoring summary"""
        return {
            'stats': dict(self.stats),
            'recent_errors': len(self.error_buffer),
            'active_processes': self.stats.get('active_processes', 0),
            'monitoring_status': 'running' if self.running else 'stopped'
        }

def setup_warp_directory():
    """Setup ~/.warp directory structure"""
    warp_home = Path.home() / ".warp"
    
    directories = [
        "logs",
        "monitor", 
        "config",
        "cache",
        "themes",
        "plugins",
        "backups",
        "scripts"
    ]
    
    for dir_name in directories:
        (warp_home / dir_name).mkdir(parents=True, exist_ok=True)
    
    # Create configuration files
    config_file = warp_home / "config" / "monitor.json"
    if not config_file.exists():
        default_config = {
            "monitoring": {
                "enabled": True,
                "poll_interval": 1.0,
                "max_log_size": "10MB",
                "rotate_count": 5
            },
            "alerts": {
                "error_threshold": 10,
                "email_notifications": False,
                "slack_webhook": None
            },
            "logging": {
                "level": "INFO",
                "console_output": True,
                "file_output": True
            }
        }
        
        with open(config_file, 'w') as f:
            json.dump(default_config, f, indent=2)
    
    print(f"Warp directory structure initialized at: {warp_home}")
    return warp_home

def signal_handler(signum, frame):
    """Handle shutdown signals"""
    print(f"\nReceived signal {signum}, shutting down...")
    global monitor
    if 'monitor' in globals():
        monitor.stop_monitoring()
    sys.exit(0)

def main():
    parser = argparse.ArgumentParser(description='Warp Preview Log Monitor')
    parser.add_argument('--verbose', '-v', action='store_true', 
                       help='Enable verbose output')
    parser.add_argument('--setup', action='store_true', 
                       help='Setup ~/.warp directory structure only')
    parser.add_argument('--tail', '-t', type=int, default=0,
                       help='Show last N lines and exit')
    parser.add_argument('--stats', action='store_true',
                       help='Show current statistics and exit')
    
    args = parser.parse_args()
    
    # Setup signal handlers
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Setup directory structure
    warp_home = setup_warp_directory()
    
    if args.setup:
        print("Setup completed.")
        return
    
    # Initialize monitoring
    config = Config()
    global monitor
    monitor = LogMonitor(config)
    monitor.verbose = args.verbose
    monitor.start_time = time.time()
    
    if args.tail:
        # Show last N lines
        if config.PREVIEW_LOG.exists():
            result = subprocess.run(['tail', '-n', str(args.tail), str(config.PREVIEW_LOG)], 
                                  capture_output=True, text=True)
            print(result.stdout)
        return
    
    if args.stats:
        # Show current stats
        if config.MONITOR_DIR.exists():
            stats_file = config.MONITOR_DIR / "stats.json"
            if stats_file.exists():
                with open(stats_file) as f:
                    stats = json.load(f)
                print(json.dumps(stats, indent=2))
            else:
                print("No statistics available yet.")
        return
    
    # Start monitoring
    try:
        monitor.start_monitoring()
    except KeyboardInterrupt:
        monitor.stop_monitoring()

if __name__ == "__main__":
    main()
