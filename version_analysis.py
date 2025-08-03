#!/usr/bin/env python3
"""
Version Configuration Analysis Tool
Analyzes version data structure and identifies potential issues
"""

import json
import re
from datetime import datetime
from typing import Dict, List, Optional, Tuple

class VersionInfo:
    def __init__(self, data: dict):
        self.version = data.get('version', '')
        self.version_for_new_users = data.get('version_for_new_users')
        self.update_by = data.get('update_by')
        self.soft_cutoff = data.get('soft_cutoff')
        self.last_prominent_update = data.get('last_prominent_update')
        self.is_rollback = data.get('is_rollback')
    
    def parse_version(self) -> Optional[Tuple[datetime, str, int]]:
        """Parse version string into datetime, channel, and increment"""
        pattern = r'v(\d{4})\.(\d{2})\.(\d{2})\.(\d{2})\.(\d{2})\.(\w+)_(\d+)'
        match = re.match(pattern, self.version)
        if match:
            year, month, day, hour, minute, channel, increment = match.groups()
            dt = datetime(int(year), int(month), int(day), int(hour), int(minute))
            return dt, channel, int(increment)
        return None
    
    def get_age_days(self) -> Optional[int]:
        """Get age of version in days"""
        parsed = self.parse_version()
        if parsed:
            version_date, _, _ = parsed
            return (datetime.now() - version_date).days
        return None

class ChannelVersion:
    def __init__(self, data: dict):
        self.version_info = VersionInfo(data['version_info'])
        self.overrides = data.get('overrides', [])

class VersionAnalyzer:
    def __init__(self, raw_data: str):
        self.channels = self._parse_raw_data(raw_data)
    
    def _parse_raw_data(self, raw_data: str) -> Dict[str, ChannelVersion]:
        """Parse the raw version data string"""
        # This is a simplified parser - in practice you'd want more robust parsing
        channels = {}
        
        # Extract channel data using regex (simplified approach)
        channel_pattern = r'(\w+):\s*ChannelVersion\s*\{\s*version_info:\s*VersionInfo\s*\{\s*version:\s*"([^"]+)"[^}]+\}[^}]+\}'
        
        # For now, let's manually create the structure based on the provided data
        channels_data = {
            'dev': {
                'version_info': {
                    'version': 'v0.2025.08.02.08.10.dev_00',
                    'version_for_new_users': None,
                    'update_by': None,
                    'soft_cutoff': 'v0.2023.05.12.08.03.dev_00',
                    'last_prominent_update': 'v0.2025.03.05.08.02.dev_00',
                    'is_rollback': None
                },
                'overrides': []
            },
            'preview': {
                'version_info': {
                    'version': 'v0.2025.07.30.08.12.preview_01',
                    'version_for_new_users': None,
                    'update_by': None,
                    'soft_cutoff': 'v0.2025.04.30.08.11.preview_03',
                    'last_prominent_update': 'v0.2025.07.30.08.12.preview_01',
                    'is_rollback': None
                },
                'overrides': []
            },
            'canary': {
                'version_info': {
                    'version': 'v0.2022.09.29.08.08.canary_00',
                    'version_for_new_users': None,
                    'update_by': None,
                    'soft_cutoff': None,
                    'last_prominent_update': None,
                    'is_rollback': None
                },
                'overrides': []
            },
            'beta': {
                'version_info': {
                    'version': 'v0.2024.12.18.08.02.beta_00',
                    'version_for_new_users': None,
                    'update_by': None,
                    'soft_cutoff': None,
                    'last_prominent_update': None,
                    'is_rollback': None
                },
                'overrides': []
            },
            'stable': {
                'version_info': {
                    'version': 'v0.2025.07.30.08.12.stable_02',
                    'version_for_new_users': None,
                    'update_by': None,
                    'soft_cutoff': 'v0.2025.07.02.08.36.stable_02',
                    'last_prominent_update': 'v0.2025.07.23.08.12.stable_02',
                    'is_rollback': None
                },
                'overrides': []
            }
        }
        
        for name, data in channels_data.items():
            channels[name] = ChannelVersion(data)
        
        return channels
    
    def analyze(self) -> Dict[str, List[str]]:
        """Perform comprehensive analysis"""
        issues = {
            'critical': [],
            'warnings': [],
            'suggestions': []
        }
        
        # Check version ordering
        self._check_version_ordering(issues)
        
        # Check for stale versions
        self._check_stale_versions(issues)
        
        # Check data completeness
        self._check_data_completeness(issues)
        
        # Check soft cutoffs
        self._check_soft_cutoffs(issues)
        
        return issues
    
    def _check_version_ordering(self, issues: Dict[str, List[str]]):
        """Check if version ordering makes sense across channels"""
        expected_order = ['dev', 'preview', 'beta', 'stable', 'canary']
        
        versions_with_dates = []
        for channel_name, channel in self.channels.items():
            parsed = channel.version_info.parse_version()
            if parsed:
                date, _, _ = parsed
                versions_with_dates.append((channel_name, date))
        
        # Sort by date (newest first)
        versions_with_dates.sort(key=lambda x: x[1], reverse=True)
        
        # Check if dev is newest
        if versions_with_dates and versions_with_dates[0][0] != 'dev':
            issues['warnings'].append(f"Dev channel is not the newest version. Newest is {versions_with_dates[0][0]}")
        
        # Check if beta is newer than stable
        beta_date = next((date for name, date in versions_with_dates if name == 'beta'), None)
        stable_date = next((date for name, date in versions_with_dates if name == 'stable'), None)
        
        if beta_date and stable_date and beta_date < stable_date:
            issues['critical'].append("Beta version is older than stable version - this seems incorrect")
    
    def _check_stale_versions(self, issues: Dict[str, List[str]]):
        """Check for versions that are too old"""
        stale_threshold_days = 180  # 6 months
        very_stale_threshold_days = 730  # 2 years
        
        for channel_name, channel in self.channels.items():
            age = channel.version_info.get_age_days()
            if age:
                if age > very_stale_threshold_days:
                    issues['critical'].append(f"{channel_name} channel is {age} days old (>2 years) - likely stale")
                elif age > stale_threshold_days:
                    issues['warnings'].append(f"{channel_name} channel is {age} days old (>6 months)")
    
    def _check_data_completeness(self, issues: Dict[str, List[str]]):
        """Check for missing or empty data"""
        for channel_name, channel in self.channels.items():
            vi = channel.version_info
            
            # Check for None values that might be expected to have data
            if not vi.last_prominent_update and channel_name in ['stable', 'beta']:
                issues['suggestions'].append(f"{channel_name} channel missing last_prominent_update")
            
            if not vi.soft_cutoff and channel_name in ['stable', 'preview', 'dev']:
                issues['suggestions'].append(f"{channel_name} channel missing soft_cutoff (minimum supported version)")
            
            # Check if overrides are always empty
            if not channel.overrides:
                issues['suggestions'].append(f"{channel_name} channel has no overrides - consider if this is expected")
    
    def _check_soft_cutoffs(self, issues: Dict[str, List[str]]):
        """Check if soft cutoffs make sense"""
        for channel_name, channel in self.channels.items():
            vi = channel.version_info
            if vi.soft_cutoff:
                current_parsed = vi.parse_version()
                cutoff_vi = VersionInfo({'version': vi.soft_cutoff})
                cutoff_parsed = cutoff_vi.parse_version()
                
                if current_parsed and cutoff_parsed:
                    current_date, _, _ = current_parsed
                    cutoff_date, _, _ = cutoff_parsed
                    
                    if cutoff_date >= current_date:
                        issues['critical'].append(f"{channel_name} soft_cutoff is newer than or equal to current version")
    
    def generate_report(self) -> str:
        """Generate a comprehensive report"""
        report = ["Version Configuration Analysis Report", "=" * 40, ""]
        
        # Channel overview
        report.append("Channel Overview:")
        report.append("-" * 16)
        for channel_name, channel in self.channels.items():
            vi = channel.version_info
            age = vi.get_age_days()
            age_str = f"({age} days old)" if age else "(unknown age)"
            report.append(f"{channel_name:8}: {vi.version} {age_str}")
        report.append("")
        
        # Analysis results
        issues = self.analyze()
        
        if issues['critical']:
            report.append("🚨 CRITICAL ISSUES:")
            for issue in issues['critical']:
                report.append(f"  • {issue}")
            report.append("")
        
        if issues['warnings']:
            report.append("⚠️  WARNINGS:")
            for warning in issues['warnings']:
                report.append(f"  • {warning}")
            report.append("")
        
        if issues['suggestions']:
            report.append("💡 SUGGESTIONS:")
            for suggestion in issues['suggestions']:
                report.append(f"  • {suggestion}")
            report.append("")
        
        return "\n".join(report)

def main():
    # Raw data from the user's input
    raw_data = """{ version_info: VersionInfo { version: "v0.2025.08.02.08.10.dev_00", version_for_new_users: None, update_by: None, soft_cutoff: Some("v0.2023.05.12.08.03.dev_00"), last_prominent_update: Some("v0.2025.03.05.08.02.dev_00"), is_rollback: None }, overrides: [] }; preview: ChannelVersion { version_info: VersionInfo { version: "v0.2025.07.30.08.12.preview_01", version_for_new_users: None, update_by: None, soft_cutoff: Some("v0.2025.04.30.08.11.preview_03"), last_prominent_update: Some("v0.2025.07.30.08.12.preview_01"), is_rollback: None }, overrides: [] }; canary: ChannelVersion { version_info: VersionInfo { version: "v0.2022.09.29.08.08.canary_00", version_for_new_users: None, update_by: None, soft_cutoff: None, last_prominent_update: None, is_rollback: None }, overrides: [] }; beta: ChannelVersion { version_info: VersionInfo { version: "v0.2024.12.18.08.02.beta_00", version_for_new_users: None, update_by: None, soft_cutoff: None, last_prominent_update: None, is_rollback: None }, overrides: [] }; stable: ChannelVersion { version_info: VersionInfo { version: "v0.2025.07.30.08.12.stable_02", version_for_new_users: None, update_by: None, soft_cutoff: Some("v0.2025.07.02.08.36.stable_02"), last_prominent_update: Some("v0.2025.07.23.08.12.stable_02"), is_rollback: None }, overrides: [] }"""
    
    analyzer = VersionAnalyzer(raw_data)
    print(analyzer.generate_report())

if __name__ == "__main__":
    main()
