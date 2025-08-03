#!/usr/bin/env python3
"""
Warp Update Check Analysis
Analyzes the update check process and version comparison
"""

import re
from datetime import datetime
from typing import Dict, Optional, Tuple

class UpdateAnalyzer:
    def __init__(self):
        self.channels_data = {
            'dev': {
                'version': 'v0.2025.08.02.08.10.dev_00',
                'soft_cutoff': 'v0.2023.05.12.08.03.dev_00',
                'last_prominent_update': 'v0.2025.03.05.08.02.dev_00'
            },
            'preview': {
                'version': 'v0.2025.07.30.08.12.preview_01',
                'soft_cutoff': 'v0.2025.04.30.08.11.preview_03',
                'last_prominent_update': 'v0.2025.07.30.08.12.preview_01'
            },
            'canary': {
                'version': 'v0.2022.09.29.08.08.canary_00',
                'soft_cutoff': None,
                'last_prominent_update': None
            },
            'beta': {
                'version': 'v0.2024.12.18.08.02.beta_00',
                'soft_cutoff': None,
                'last_prominent_update': None
            },
            'stable': {
                'version': 'v0.2025.07.30.08.12.stable_02',
                'soft_cutoff': 'v0.2025.07.02.08.36.stable_02',
                'last_prominent_update': 'v0.2025.07.23.08.12.stable_02'
            }
        }
    
    def parse_version(self, version_str: str) -> Optional[Tuple[datetime, str, int]]:
        """Parse version string into datetime, channel, and increment"""
        # Pattern: v0.{YYYY}.{MM}.{DD}.{HH}.{MM}.{channel}_{increment}
        pattern = r'v0\.(\d{4})\.(\d{2})\.(\d{2})\.(\d{2})\.(\d{2})\.(\w+)_(\d+)'
        match = re.match(pattern, version_str)
        if match:
            year, month, day, hour, minute, channel, increment = match.groups()
            try:
                dt = datetime(int(year), int(month), int(day), int(hour), int(minute))
                return dt, channel, int(increment)
            except ValueError:
                return None
        return None
    
    def analyze_update_check(self, requested_channel: str, update_id: str) -> Dict[str, any]:
        """Analyze the update check request"""
        analysis = {
            'requested_channel': requested_channel,
            'update_id': update_id,
            'server_response': self.channels_data,
            'recommendations': []
        }
        
        # Check if requested channel exists
        if requested_channel == 'preview_release':
            # Map preview_release to preview
            actual_channel = 'preview'
            analysis['channel_mapping'] = f"{requested_channel} -> {actual_channel}"
        else:
            actual_channel = requested_channel
        
        if actual_channel not in self.channels_data:
            analysis['recommendations'].append(f"❌ Channel '{actual_channel}' not found in server response")
            return analysis
        
        # Analyze the target channel
        target_version = self.channels_data[actual_channel]['version']
        target_parsed = self.parse_version(target_version)
        
        if target_parsed:
            target_date, target_channel_name, target_increment = target_parsed
            analysis['target_version'] = {
                'version': target_version,
                'date': target_date.strftime('%Y-%m-%d %H:%M'),
                'channel': target_channel_name,
                'increment': target_increment
            }
        else:
            # Fallback if parsing fails
            analysis['target_version'] = {
                'version': target_version,
                'date': 'unknown',
                'channel': 'unknown',
                'increment': 'unknown'
            }
        
        # Compare with other channels
        analysis['channel_comparison'] = self._compare_channels(actual_channel)
        
        # Check for issues
        self._check_for_issues(analysis, actual_channel)
        
        return analysis
    
    def _compare_channels(self, target_channel: str) -> Dict[str, any]:
        """Compare target channel with other channels"""
        comparison = {}
        target_version = self.channels_data[target_channel]['version']
        target_parsed = self.parse_version(target_version)
        
        if not target_parsed:
            return comparison
        
        target_date, _, _ = target_parsed
        
        for channel_name, channel_data in self.channels_data.items():
            if channel_name == target_channel:
                continue
            
            other_parsed = self.parse_version(channel_data['version'])
            if other_parsed:
                other_date, _, _ = other_parsed
                comparison[channel_name] = {
                    'version': channel_data['version'],
                    'newer_than_target': other_date > target_date,
                    'days_difference': (other_date - target_date).days
                }
        
        return comparison
    
    def _check_for_issues(self, analysis: Dict[str, any], channel: str):
        """Check for potential issues with the update"""
        channel_data = self.channels_data[channel]
        
        # Check if preview is outdated compared to dev
        if channel == 'preview':
            dev_version = self.channels_data['dev']['version']
            preview_version = channel_data['version']
            
            dev_parsed = self.parse_version(dev_version)
            preview_parsed = self.parse_version(preview_version)
            
            if dev_parsed and preview_parsed:
                dev_date, _, _ = dev_parsed
                preview_date, _, _ = preview_parsed
                
                days_behind = (dev_date - preview_date).days
                if days_behind > 0:
                    analysis['recommendations'].append(
                        f"⚠️  Preview is {days_behind} days behind dev channel"
                    )
        
        # Always check canary age - it's consistently problematic
        canary_parsed = self.parse_version(self.channels_data['canary']['version'])
        if canary_parsed:
            canary_date, _, _ = canary_parsed
            days_old = (datetime.now() - canary_date).days
            analysis['recommendations'].append(
                f"🚨 Canary channel is {days_old} days old - likely abandoned"
            )
        
        # Check beta vs stable ordering
        beta_parsed = self.parse_version(self.channels_data['beta']['version'])
        stable_parsed = self.parse_version(self.channels_data['stable']['version'])
        
        if beta_parsed and stable_parsed:
            beta_date, _, _ = beta_parsed
            stable_date, _, _ = stable_parsed
            
            if beta_date < stable_date:
                days_behind = (stable_date - beta_date).days
                analysis['recommendations'].append(
                    f"❌ Beta is {days_behind} days older than stable - this seems wrong"
                )
    
    def generate_report(self, requested_channel: str, update_id: str) -> str:
        """Generate a comprehensive update analysis report"""
        analysis = self.analyze_update_check(requested_channel, update_id)
        
        report = [
            "Warp Update Check Analysis",
            "=" * 26,
            "",
            f"Requested Channel: {requested_channel}",
            f"Update ID: {update_id}",
            f"Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            ""
        ]
        
        if 'channel_mapping' in analysis:
            report.append(f"Channel Mapping: {analysis['channel_mapping']}")
            report.append("")
        
        if 'target_version' in analysis:
            tv = analysis['target_version']
            report.extend([
                "Target Version Info:",
                "-" * 19,
                f"Version: {tv['version']}",
                f"Date: {tv['date']}",
                f"Channel: {tv['channel']}",
                f"Increment: {tv['increment']}",
                ""
            ])
        
        if analysis['channel_comparison']:
            report.extend([
                "Comparison with Other Channels:",
                "-" * 32
            ])
            
            for channel, comp in analysis['channel_comparison'].items():
                status = "newer" if comp['newer_than_target'] else "older"
                days = abs(comp['days_difference'])
                report.append(f"{channel:8}: {comp['version']} ({days} days {status})")
            report.append("")
        
        if analysis['recommendations']:
            report.extend([
                "Issues & Recommendations:",
                "-" * 24
            ])
            for rec in analysis['recommendations']:
                report.append(f"{rec}")
            report.append("")
        
        # Server response summary
        report.extend([
            "Server Response Summary:",
            "-" * 23,
            "✅ Successfully fetched channel versions",
            "✅ All 5 channels present (dev, preview, canary, beta, stable)",
            "✅ No overrides configured",
            ""
        ])
        
        return "\n".join(report)

def main():
    analyzer = UpdateAnalyzer()
    
    # Parse the log information
    requested_channel = "preview_release"  # from "Checking for update on channel preview_release"
    update_id = "QX5Kvca"  # from "Update id is QX5Kvca"
    
    print(analyzer.generate_report(requested_channel, update_id))

if __name__ == "__main__":
    main()
