#!/usr/bin/env python3
"""
Warp Terminal Code Analysis & Implementation Tool

This script provides comprehensive code analysis and project command capabilities
for the Warp terminal project, including:
- Code quality analysis
- Architecture review
- Performance metrics
- Dependency analysis
- Security scanning
- Test coverage analysis
- Documentation generation
- Build optimization
"""

import os
import sys
import json
import subprocess
import argparse
import datetime
from pathlib import Path
from typing import Dict, List, Optional, Any
import re

class WarpCodeAnalyzer:
    """Main code analysis class for Warp terminal project"""
    
    def __init__(self, project_path: str = "/Users/KhulnaSoft/.warp"):
        self.project_path = Path(project_path)
        self.cargo_toml = self.project_path / "Cargo.toml"
        self.src_path = self.project_path / "src"
        self.analysis_results = {}
        
    def run_analysis(self, analysis_types: List[str] = None) -> Dict[str, Any]:
        """Run comprehensive code analysis"""
        if analysis_types is None:
            analysis_types = ["all"]
            
        print("🚀 Starting Warp Terminal Code Analysis...")
        
        results = {
            "timestamp": datetime.datetime.now().isoformat(),
            "project_path": str(self.project_path),
            "analysis_types": analysis_types
        }
        
        if "all" in analysis_types or "structure" in analysis_types:
            results["project_structure"] = self.analyze_project_structure()
            
        if "all" in analysis_types or "dependencies" in analysis_types:
            results["dependencies"] = self.analyze_dependencies()
            
        if "all" in analysis_types or "code_quality" in analysis_types:
            results["code_quality"] = self.analyze_code_quality()
            
        if "all" in analysis_types or "security" in analysis_types:
            results["security"] = self.analyze_security()
            
        if "all" in analysis_types or "performance" in analysis_types:
            results["performance"] = self.analyze_performance()
            
        if "all" in analysis_types or "tests" in analysis_types:
            results["test_coverage"] = self.analyze_test_coverage()
            
        if "all" in analysis_types or "architecture" in analysis_types:
            results["architecture"] = self.analyze_architecture()
            
        self.analysis_results = results
        return results
    
    def analyze_project_structure(self) -> Dict[str, Any]:
        """Analyze project structure and organization"""
        print("📁 Analyzing project structure...")
        
        structure = {
            "total_files": 0,
            "rust_files": 0,
            "test_files": 0,
            "config_files": 0,
            "modules": {},
            "file_sizes": {},
            "complexity_indicators": {}
        }
        
        # Count different file types
        for root, dirs, files in os.walk(self.src_path):
            for file in files:
                filepath = Path(root) / file
                structure["total_files"] += 1
                
                if file.endswith('.rs'):
                    structure["rust_files"] += 1
                    if 'test' in file or 'tests' in str(filepath):
                        structure["test_files"] += 1
                        
                    # Analyze module structure
                    rel_path = filepath.relative_to(self.src_path)
                    module_path = str(rel_path.parent) if rel_path.parent != Path('.') else 'root'
                    
                    if module_path not in structure["modules"]:
                        structure["modules"][module_path] = []
                    structure["modules"][module_path].append(file)
                    
                    # File size analysis
                    try:
                        size = filepath.stat().st_size
                        structure["file_sizes"][str(rel_path)] = size
                    except Exception:
                        pass
                        
                elif file.endswith(('.toml', '.yaml', '.yml', '.json')):
                    structure["config_files"] += 1
        
        # Calculate complexity indicators
        structure["complexity_indicators"] = {
            "modules_count": len(structure["modules"]),
            "avg_files_per_module": structure["rust_files"] / max(len(structure["modules"]), 1),
            "largest_files": sorted(structure["file_sizes"].items(), 
                                  key=lambda x: x[1], reverse=True)[:10]
        }
        
        return structure
    
    def analyze_dependencies(self) -> Dict[str, Any]:
        """Analyze project dependencies"""
        print("📦 Analyzing dependencies...")
        
        deps_info = {
            "direct_dependencies": {},
            "dev_dependencies": {},
            "workspace_dependencies": {},
            "features": {},
            "dependency_tree": {},
            "outdated_check": {},
            "security_advisories": {}
        }
        
        # Parse Cargo.toml
        try:
            with open(self.cargo_toml, 'r') as f:
                cargo_content = f.read()
                
            # Extract dependencies sections (basic parsing)
            deps_section = re.search(r'\[dependencies\](.*?)(?=\[|\Z)', cargo_content, re.DOTALL)
            if deps_section:
                deps_info["direct_dependencies"] = self._parse_deps_section(deps_section.group(1))
                
            dev_deps_section = re.search(r'\[dev-dependencies\](.*?)(?=\[|\Z)', cargo_content, re.DOTALL)
            if dev_deps_section:
                deps_info["dev_dependencies"] = self._parse_deps_section(dev_deps_section.group(1))
                
            features_section = re.search(r'\[features\](.*?)(?=\[|\Z)', cargo_content, re.DOTALL)
            if features_section:
                deps_info["features"] = self._parse_features_section(features_section.group(1))
                
        except Exception as e:
            deps_info["error"] = f"Failed to parse Cargo.toml: {e}"
        
        # Run cargo commands for more detailed info
        try:
            # Get dependency tree
            result = subprocess.run(['cargo', 'tree', '--format', '{p} {f}'], 
                                  cwd=self.project_path, capture_output=True, text=True)
            if result.returncode == 0:
                deps_info["dependency_tree"] = result.stdout.strip().split('\n')
                
            # Check for outdated dependencies
            result = subprocess.run(['cargo', 'outdated', '--root-deps-only'], 
                                  cwd=self.project_path, capture_output=True, text=True)
            if result.returncode == 0:
                deps_info["outdated_check"] = result.stdout.strip()
                
        except Exception as e:
            deps_info["cargo_commands_error"] = str(e)
            
        return deps_info
    
    def analyze_code_quality(self) -> Dict[str, Any]:
        """Analyze code quality using various Rust tools"""
        print("🔍 Analyzing code quality...")
        
        quality_info = {
            "clippy_results": {},
            "fmt_check": {},
            "code_metrics": {},
            "complexity_analysis": {},
            "documentation_coverage": {}
        }
        
        # Run clippy
        try:
            result = subprocess.run(['cargo', 'clippy', '--all-targets', '--', '-D', 'warnings'], 
                                  cwd=self.project_path, capture_output=True, text=True)
            quality_info["clippy_results"] = {
                "exit_code": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "warnings_count": result.stderr.count("warning:"),
                "errors_count": result.stderr.count("error:")
            }
        except Exception as e:
            quality_info["clippy_results"]["error"] = str(e)
        
        # Check formatting
        try:
            result = subprocess.run(['cargo', 'fmt', '--check'], 
                                  cwd=self.project_path, capture_output=True, text=True)
            quality_info["fmt_check"] = {
                "formatted": result.returncode == 0,
                "output": result.stdout + result.stderr
            }
        except Exception as e:
            quality_info["fmt_check"]["error"] = str(e)
        
        # Code metrics
        quality_info["code_metrics"] = self._calculate_code_metrics()
        
        return quality_info
    
    def analyze_security(self) -> Dict[str, Any]:
        """Run security analysis"""
        print("🔒 Analyzing security...")
        
        security_info = {
            "cargo_audit": {},
            "dependency_vulnerabilities": {},
            "unsafe_code_analysis": {},
            "security_best_practices": {}
        }
        
        # Run cargo audit
        try:
            result = subprocess.run(['cargo', 'audit'], 
                                  cwd=self.project_path, capture_output=True, text=True)
            security_info["cargo_audit"] = {
                "exit_code": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr
            }
        except Exception as e:
            security_info["cargo_audit"]["error"] = str(e)
        
        # Analyze unsafe code usage
        security_info["unsafe_code_analysis"] = self._analyze_unsafe_code()
        
        return security_info
    
    def analyze_performance(self) -> Dict[str, Any]:
        """Analyze performance characteristics"""
        print("⚡ Analyzing performance...")
        
        perf_info = {
            "compilation_time": {},
            "binary_size": {},
            "memory_usage_patterns": {},
            "async_usage": {},
            "optimization_suggestions": []
        }
        
        # Measure compilation time
        try:
            import time
            start_time = time.time()
            result = subprocess.run(['cargo', 'check'], 
                                  cwd=self.project_path, capture_output=True, text=True)
            compile_time = time.time() - start_time
            
            perf_info["compilation_time"] = {
                "check_time_seconds": compile_time,
                "success": result.returncode == 0
            }
        except Exception as e:
            perf_info["compilation_time"]["error"] = str(e)
        
        # Analyze binary size
        try:
            target_dir = self.project_path / "target" / "debug"
            if target_dir.exists():
                for binary in target_dir.glob("warp*"):
                    if binary.is_file() and binary.stat().st_mode & 0o111:  # executable
                        perf_info["binary_size"][binary.name] = binary.stat().st_size
        except Exception as e:
            perf_info["binary_size"]["error"] = str(e)
        
        # Analyze async usage patterns
        perf_info["async_usage"] = self._analyze_async_patterns()
        
        return perf_info
    
    def analyze_test_coverage(self) -> Dict[str, Any]:
        """Analyze test coverage"""
        print("🧪 Analyzing test coverage...")
        
        test_info = {
            "test_execution": {},
            "coverage_report": {},
            "test_organization": {},
            "benchmark_results": {}
        }
        
        # Run tests
        try:
            result = subprocess.run(['cargo', 'test', '--verbose'], 
                                  cwd=self.project_path, capture_output=True, text=True)
            test_info["test_execution"] = {
                "exit_code": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "tests_run": result.stdout.count("test result:"),
                "passed": "failures: 0" in result.stdout
            }
        except Exception as e:
            test_info["test_execution"]["error"] = str(e)
        
        # Analyze test organization
        test_info["test_organization"] = self._analyze_test_organization()
        
        return test_info
    
    def analyze_architecture(self) -> Dict[str, Any]:
        """Analyze software architecture"""
        print("🏗️ Analyzing architecture...")
        
        arch_info = {
            "module_dependencies": {},
            "design_patterns": {},
            "async_architecture": {},
            "error_handling_patterns": {},
            "architectural_suggestions": []
        }
        
        # Analyze module dependencies
        arch_info["module_dependencies"] = self._analyze_module_dependencies()
        
        # Identify design patterns
        arch_info["design_patterns"] = self._identify_design_patterns()
        
        # Analyze error handling
        arch_info["error_handling_patterns"] = self._analyze_error_handling()
        
        return arch_info
    
    def _parse_deps_section(self, section: str) -> Dict[str, str]:
        """Parse dependencies from Cargo.toml section"""
        deps = {}
        for line in section.strip().split('\n'):
            line = line.strip()
            if line and not line.startswith('#'):
                if '=' in line:
                    parts = line.split('=', 1)
                    if len(parts) == 2:
                        key = parts[0].strip()
                        value = parts[1].strip().strip('"')
                        deps[key] = value
        return deps
    
    def _parse_features_section(self, section: str) -> Dict[str, List[str]]:
        """Parse features from Cargo.toml"""
        features = {}
        for line in section.strip().split('\n'):
            line = line.strip()
            if line and not line.startswith('#') and '=' in line:
                parts = line.split('=', 1)
                if len(parts) == 2:
                    key = parts[0].strip()
                    value = parts[1].strip()
                    # Parse array-like values
                    if value.startswith('[') and value.endswith(']'):
                        items = [item.strip().strip('"') for item in value[1:-1].split(',') if item.strip()]
                        features[key] = items
        return features
    
    def _calculate_code_metrics(self) -> Dict[str, Any]:
        """Calculate various code metrics"""
        metrics = {
            "total_lines": 0,
            "code_lines": 0,
            "comment_lines": 0,
            "blank_lines": 0,
            "functions_count": 0,
            "structs_count": 0,
            "enums_count": 0,
            "traits_count": 0
        }
        
        for rust_file in self.src_path.rglob("*.rs"):
            try:
                with open(rust_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    lines = content.split('\n')
                    
                    for line in lines:
                        metrics["total_lines"] += 1
                        stripped = line.strip()
                        
                        if not stripped:
                            metrics["blank_lines"] += 1
                        elif stripped.startswith('//') or stripped.startswith('/*'):
                            metrics["comment_lines"] += 1
                        else:
                            metrics["code_lines"] += 1
                    
                    # Count language constructs
                    metrics["functions_count"] += len(re.findall(r'\bfn\s+\w+', content))
                    metrics["structs_count"] += len(re.findall(r'\bstruct\s+\w+', content))
                    metrics["enums_count"] += len(re.findall(r'\benum\s+\w+', content))
                    metrics["traits_count"] += len(re.findall(r'\btrait\s+\w+', content))
                    
            except Exception:
                continue
                
        return metrics
    
    def _analyze_unsafe_code(self) -> Dict[str, Any]:
        """Analyze usage of unsafe code"""
        unsafe_analysis = {
            "unsafe_blocks": [],
            "unsafe_functions": [],
            "total_unsafe_lines": 0,
            "files_with_unsafe": []
        }
        
        for rust_file in self.src_path.rglob("*.rs"):
            try:
                with open(rust_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    
                if 'unsafe' in content:
                    rel_path = rust_file.relative_to(self.project_path)
                    unsafe_analysis["files_with_unsafe"].append(str(rel_path))
                    
                    # Count unsafe blocks and functions
                    unsafe_blocks = len(re.findall(r'unsafe\s*{', content))
                    unsafe_functions = len(re.findall(r'unsafe\s+fn', content))
                    
                    unsafe_analysis["unsafe_blocks"].extend([{
                        "file": str(rel_path),
                        "count": unsafe_blocks
                    }])
                    
                    unsafe_analysis["unsafe_functions"].extend([{
                        "file": str(rel_path),
                        "count": unsafe_functions
                    }])
                    
                    unsafe_analysis["total_unsafe_lines"] += content.count('unsafe')
                    
            except Exception:
                continue
                
        return unsafe_analysis
    
    def _analyze_async_patterns(self) -> Dict[str, Any]:
        """Analyze async/await usage patterns"""
        async_analysis = {
            "async_functions": 0,
            "await_calls": 0,
            "tokio_usage": [],
            "async_traits": 0,
            "files_with_async": []
        }
        
        for rust_file in self.src_path.rglob("*.rs"):
            try:
                with open(rust_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    
                has_async = False
                
                # Count async patterns
                async_fns = len(re.findall(r'\basync\s+fn', content))
                if async_fns > 0:
                    async_analysis["async_functions"] += async_fns
                    has_async = True
                
                await_calls = len(re.findall(r'\.await', content))
                if await_calls > 0:
                    async_analysis["await_calls"] += await_calls
                    has_async = True
                
                async_traits = len(re.findall(r'#\[async_trait\]', content))
                if async_traits > 0:
                    async_analysis["async_traits"] += async_traits
                    has_async = True
                
                if 'tokio::' in content or 'use tokio' in content:
                    rel_path = rust_file.relative_to(self.project_path)
                    async_analysis["tokio_usage"].append(str(rel_path))
                    has_async = True
                
                if has_async:
                    rel_path = rust_file.relative_to(self.project_path)
                    async_analysis["files_with_async"].append(str(rel_path))
                    
            except Exception:
                continue
                
        return async_analysis
    
    def _analyze_test_organization(self) -> Dict[str, Any]:
        """Analyze how tests are organized"""
        test_org = {
            "unit_tests": 0,
            "integration_tests": 0,
            "test_modules": [],
            "test_files": [],
            "benchmark_tests": 0
        }
        
        # Check tests directory
        tests_dir = self.project_path / "tests"
        if tests_dir.exists():
            for test_file in tests_dir.rglob("*.rs"):
                rel_path = test_file.relative_to(self.project_path)
                test_org["test_files"].append(str(rel_path))
                test_org["integration_tests"] += 1
        
        # Check for inline tests
        for rust_file in self.src_path.rglob("*.rs"):
            try:
                with open(rust_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    
                # Count test functions
                unit_tests = len(re.findall(r'#\[test\]', content))
                test_org["unit_tests"] += unit_tests
                
                # Count benchmark tests
                bench_tests = len(re.findall(r'#\[bench\]', content))
                test_org["benchmark_tests"] += bench_tests
                
                # Check for test modules
                if '#[cfg(test)]' in content:
                    rel_path = rust_file.relative_to(self.project_path)
                    test_org["test_modules"].append(str(rel_path))
                    
            except Exception:
                continue
                
        return test_org
    
    def _analyze_module_dependencies(self) -> Dict[str, Any]:
        """Analyze dependencies between modules"""
        deps = {
            "internal_imports": {},
            "external_imports": {},
            "circular_dependencies": [],
            "dependency_graph": {}
        }
        
        for rust_file in self.src_path.rglob("*.rs"):
            try:
                with open(rust_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    
                rel_path = rust_file.relative_to(self.src_path)
                module_name = str(rel_path).replace('.rs', '').replace('/', '::')
                
                # Find use statements
                use_statements = re.findall(r'use\s+([^;]+);', content)
                
                internal_uses = []
                external_uses = []
                
                for use_stmt in use_statements:
                    use_stmt = use_stmt.strip()
                    if use_stmt.startswith('crate::') or use_stmt.startswith('super::') or use_stmt.startswith('self::'):
                        internal_uses.append(use_stmt)
                    elif not use_stmt.startswith('std::'):
                        external_uses.append(use_stmt)
                
                if internal_uses:
                    deps["internal_imports"][module_name] = internal_uses
                if external_uses:
                    deps["external_imports"][module_name] = external_uses
                    
            except Exception:
                continue
                
        return deps
    
    def _identify_design_patterns(self) -> Dict[str, Any]:
        """Identify common design patterns in the code"""
        patterns = {
            "builder_pattern": [],
            "factory_pattern": [],
            "observer_pattern": [],
            "strategy_pattern": [],
            "singleton_pattern": [],
            "command_pattern": []
        }
        
        for rust_file in self.src_path.rglob("*.rs"):
            try:
                with open(rust_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    
                rel_path = rust_file.relative_to(self.project_path)
                
                # Look for builder pattern
                if re.search(r'impl.*Builder', content) or '.build()' in content:
                    patterns["builder_pattern"].append(str(rel_path))
                
                # Look for factory pattern
                if re.search(r'fn\s+create_|fn\s+new_|fn\s+make_', content):
                    patterns["factory_pattern"].append(str(rel_path))
                
                # Look for observer pattern
                if 'Observer' in content or 'subscribe' in content or 'notify' in content:
                    patterns["observer_pattern"].append(str(rel_path))
                
                # Look for strategy pattern
                if re.search(r'trait.*Strategy', content) or 'dyn ' in content:
                    patterns["strategy_pattern"].append(str(rel_path))
                
                # Look for singleton pattern
                if 'static' in content and 'Lazy' in content:
                    patterns["singleton_pattern"].append(str(rel_path))
                
                # Look for command pattern
                if 'Command' in content or 'execute' in content:
                    patterns["command_pattern"].append(str(rel_path))
                    
            except Exception:
                continue
                
        return patterns
    
    def _analyze_error_handling(self) -> Dict[str, Any]:
        """Analyze error handling patterns"""
        error_analysis = {
            "result_usage": 0,
            "option_usage": 0,
            "custom_errors": [],
            "panic_usage": [],
            "unwrap_usage": 0,
            "expect_usage": 0,
            "error_handling_quality": "good"  # will be calculated
        }
        
        for rust_file in self.src_path.rglob("*.rs"):
            try:
                with open(rust_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    
                rel_path = rust_file.relative_to(self.project_path)
                
                # Count error handling patterns
                error_analysis["result_usage"] += content.count('Result<')
                error_analysis["option_usage"] += content.count('Option<')
                error_analysis["unwrap_usage"] += content.count('.unwrap()')
                error_analysis["expect_usage"] += content.count('.expect(')
                
                # Look for custom error types
                if re.search(r'enum.*Error|struct.*Error', content):
                    error_analysis["custom_errors"].append(str(rel_path))
                
                # Look for panic usage
                if 'panic!' in content:
                    error_analysis["panic_usage"].append(str(rel_path))
                    
            except Exception:
                continue
        
        # Calculate error handling quality
        total_error_handling = (error_analysis["result_usage"] + 
                              error_analysis["option_usage"])
        bad_practices = (error_analysis["unwrap_usage"] + 
                        len(error_analysis["panic_usage"]))
        
        if total_error_handling > 0:
            quality_ratio = bad_practices / total_error_handling
            if quality_ratio < 0.1:
                error_analysis["error_handling_quality"] = "excellent"
            elif quality_ratio < 0.3:
                error_analysis["error_handling_quality"] = "good"
            elif quality_ratio < 0.5:
                error_analysis["error_handling_quality"] = "fair"
            else:
                error_analysis["error_handling_quality"] = "needs_improvement"
                
        return error_analysis
    
    def generate_report(self, output_format: str = "json", output_file: Optional[str] = None) -> str:
        """Generate analysis report"""
        if not self.analysis_results:
            raise ValueError("No analysis results available. Run analysis first.")
        
        if output_format == "json":
            report_content = json.dumps(self.analysis_results, indent=2, default=str)
        elif output_format == "markdown":
            report_content = self._generate_markdown_report()
        else:
            raise ValueError(f"Unsupported output format: {output_format}")
        
        if output_file:
            output_path = Path(output_file)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            with open(output_path, 'w') as f:
                f.write(report_content)
            print(f"📄 Report saved to: {output_path}")
        
        return report_content
    
    def _generate_markdown_report(self) -> str:
        """Generate markdown report from analysis results"""
        report = f"""# Warp Terminal Code Analysis Report

Generated: {self.analysis_results.get('timestamp', 'Unknown')}
Project Path: {self.analysis_results.get('project_path', 'Unknown')}

## Executive Summary

"""
        
        # Project Structure Summary
        if 'project_structure' in self.analysis_results:
            structure = self.analysis_results['project_structure']
            report += f"""### Project Structure
- **Total Files**: {structure.get('total_files', 0)}
- **Rust Files**: {structure.get('rust_files', 0)}
- **Test Files**: {structure.get('test_files', 0)}
- **Config Files**: {structure.get('config_files', 0)}
- **Modules**: {structure.get('complexity_indicators', {}).get('modules_count', 0)}

"""
        
        # Code Quality Summary
        if 'code_quality' in self.analysis_results:
            quality = self.analysis_results['code_quality']
            clippy = quality.get('clippy_results', {})
            report += f"""### Code Quality
- **Clippy Warnings**: {clippy.get('warnings_count', 0)}
- **Clippy Errors**: {clippy.get('errors_count', 0)}
- **Code Formatted**: {'✅' if quality.get('fmt_check', {}).get('formatted', False) else '❌'}

"""
        
        # Dependencies Summary
        if 'dependencies' in self.analysis_results:
            deps = self.analysis_results['dependencies']
            direct_deps = len(deps.get('direct_dependencies', {}))
            dev_deps = len(deps.get('dev_dependencies', {}))
            report += f"""### Dependencies
- **Direct Dependencies**: {direct_deps}
- **Dev Dependencies**: {dev_deps}
- **Features**: {len(deps.get('features', {}))}

"""
        
        # Security Summary
        if 'security' in self.analysis_results:
            security = self.analysis_results['security']
            unsafe_analysis = security.get('unsafe_code_analysis', {})
            report += f"""### Security
- **Files with Unsafe Code**: {len(unsafe_analysis.get('files_with_unsafe', []))}
- **Total Unsafe Lines**: {unsafe_analysis.get('total_unsafe_lines', 0)}
- **Cargo Audit Status**: {'✅ Passed' if security.get('cargo_audit', {}).get('exit_code', 1) == 0 else '❌ Issues Found'}

"""
        
        # Performance Summary
        if 'performance' in self.analysis_results:
            perf = self.analysis_results['performance']
            compile_time = perf.get('compilation_time', {}).get('check_time_seconds', 0)
            report += f"""### Performance
- **Compilation Time (check)**: {compile_time:.2f} seconds
- **Async Functions**: {perf.get('async_usage', {}).get('async_functions', 0)}
- **Await Calls**: {perf.get('async_usage', {}).get('await_calls', 0)}

"""
        
        # Test Coverage Summary
        if 'test_coverage' in self.analysis_results:
            tests = self.analysis_results['test_coverage']
            test_org = tests.get('test_organization', {})
            report += f"""### Test Coverage
- **Unit Tests**: {test_org.get('unit_tests', 0)}
- **Integration Tests**: {test_org.get('integration_tests', 0)}
- **Test Modules**: {len(test_org.get('test_modules', []))}

"""
        
        # Architecture Summary
        if 'architecture' in self.analysis_results:
            arch = self.analysis_results['architecture']
            error_handling = arch.get('error_handling_patterns', {})
            patterns = arch.get('design_patterns', {})
            report += f"""### Architecture
- **Error Handling Quality**: {error_handling.get('error_handling_quality', 'unknown').title()}
- **Result Usage**: {error_handling.get('result_usage', 0)}
- **Custom Error Types**: {len(error_handling.get('custom_errors', []))}
- **Design Patterns Found**: {sum(1 for pattern_files in patterns.values() if pattern_files)}

"""
        
        report += """
## Detailed Analysis

For detailed analysis results, please refer to the JSON output or run specific analysis commands.

### Recommendations

Based on the analysis, here are some recommendations:

1. **Code Quality**: Ensure all clippy warnings are addressed
2. **Testing**: Increase test coverage with more unit and integration tests  
3. **Security**: Review any unsafe code usage and ensure it's necessary
4. **Performance**: Monitor compilation times and binary sizes
5. **Architecture**: Consider implementing more design patterns where appropriate

---
*Report generated by Warp Terminal Code Analysis Tool*
"""
        
        return report
    
    def run_project_commands(self, commands: List[str]) -> Dict[str, Any]:
        """Run various project-related commands"""
        print("🔧 Running project commands...")
        
        results = {}
        
        for command in commands:
            print(f"  Running: {command}")
            try:
                if command == "build":
                    result = subprocess.run(['cargo', 'build'], 
                                          cwd=self.project_path, capture_output=True, text=True)
                elif command == "test":
                    result = subprocess.run(['cargo', 'test'], 
                                          cwd=self.project_path, capture_output=True, text=True)
                elif command == "clean":
                    result = subprocess.run(['cargo', 'clean'], 
                                          cwd=self.project_path, capture_output=True, text=True)
                elif command == "fmt":
                    result = subprocess.run(['cargo', 'fmt'], 
                                          cwd=self.project_path, capture_output=True, text=True)
                elif command == "clippy":
                    result = subprocess.run(['cargo', 'clippy', '--fix', '--allow-dirty'], 
                                          cwd=self.project_path, capture_output=True, text=True)
                elif command == "update":
                    result = subprocess.run(['cargo', 'update'], 
                                          cwd=self.project_path, capture_output=True, text=True)
                elif command == "doc":
                    result = subprocess.run(['cargo', 'doc', '--no-deps'], 
                                          cwd=self.project_path, capture_output=True, text=True)
                else:
                    # Try to run as a raw cargo command
                    cmd_parts = command.split()
                    if cmd_parts[0] == "cargo":
                        result = subprocess.run(cmd_parts, 
                                              cwd=self.project_path, capture_output=True, text=True)
                    else:
                        result = subprocess.run(['cargo'] + cmd_parts, 
                                              cwd=self.project_path, capture_output=True, text=True)
                
                results[command] = {
                    "exit_code": result.returncode,
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                    "success": result.returncode == 0
                }
                
                status = "✅" if result.returncode == 0 else "❌"
                print(f"    {status} {command} completed")
                
            except Exception as e:
                results[command] = {
                    "error": str(e),
                    "success": False
                }
                print(f"    ❌ {command} failed: {e}")
        
        return results


def main():
    """Main CLI entry point"""
    parser = argparse.ArgumentParser(description="Warp Terminal Code Analysis & Implementation Tool")
    
    parser.add_argument("--project-path", default="/Users/KhulnaSoft/.warp",
                       help="Path to the Warp project")
    parser.add_argument("--analysis", nargs="+", 
                       choices=["all", "structure", "dependencies", "code_quality", 
                               "security", "performance", "tests", "architecture"],
                       default=["all"], help="Types of analysis to run")
    parser.add_argument("--output-format", choices=["json", "markdown"], default="json",
                       help="Output format for the report")
    parser.add_argument("--output-file", help="Output file for the report")
    parser.add_argument("--commands", nargs="+",
                       help="Project commands to run (build, test, clean, fmt, clippy, update, doc)")
    parser.add_argument("--interactive", action="store_true",
                       help="Run in interactive mode")
    
    args = parser.parse_args()
    
    # Create analyzer
    analyzer = WarpCodeAnalyzer(args.project_path)
    
    if args.interactive:
        print("🚀 Welcome to Warp Terminal Code Analysis Tool (Interactive Mode)")
        print("Available commands:")
        print("  1. analyze - Run code analysis")
        print("  2. build   - Build the project")
        print("  3. test    - Run tests")
        print("  4. report  - Generate report")
        print("  5. quit    - Exit")
        
        while True:
            command = input("\nEnter command: ").strip().lower()
            
            if command == "quit" or command == "q":
                break
            elif command == "analyze" or command == "1":
                results = analyzer.run_analysis()
                print("✅ Analysis completed")
            elif command == "build" or command == "2":
                results = analyzer.run_project_commands(["build"])
                print("✅ Build completed")
            elif command == "test" or command == "3":
                results = analyzer.run_project_commands(["test"])
                print("✅ Tests completed")
            elif command == "report" or command == "4":
                if hasattr(analyzer, 'analysis_results') and analyzer.analysis_results:
                    report = analyzer.generate_report("markdown")
                    print("\n" + "="*50)
                    print(report)
                    print("="*50)
                else:
                    print("❌ No analysis results available. Run 'analyze' first.")
            else:
                print("❌ Unknown command")
    else:
        # Non-interactive mode
        if args.commands:
            print("🔧 Running project commands...")
            command_results = analyzer.run_project_commands(args.commands)
            
            for cmd, result in command_results.items():
                print(f"\n{'='*20} {cmd.upper()} {'='*20}")
                if result.get("success"):
                    print("✅ SUCCESS")
                    if result.get("stdout"):
                        print("STDOUT:", result["stdout"])
                else:
                    print("❌ FAILED")
                    if result.get("stderr"):
                        print("STDERR:", result["stderr"])
                    if result.get("error"):
                        print("ERROR:", result["error"])
        
        # Run analysis
        print("\n🔍 Running code analysis...")
        results = analyzer.run_analysis(args.analysis)
        
        # Generate report
        report = analyzer.generate_report(args.output_format, args.output_file)
        
        if not args.output_file:
            if args.output_format == "markdown":
                print("\n" + "="*50)
                print(report)
                print("="*50)
            else:
                print("\n📊 Analysis Results:")
                print(report)
        
        print("\n✅ Analysis completed successfully!")


if __name__ == "__main__":
    main()
