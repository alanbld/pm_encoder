#!/usr/bin/env python3
"""
CFD Shadow Generation and Management CLI v2.0
Human-AI collaborative shadow generation with enhanced workflow support
Implements ADR-016 Human-AI Collaborative Workflow
"""

import argparse
import json
import yaml
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from datetime import datetime
import hashlib
import ast
import re
from dataclasses import dataclass, asdict

from cfd_protocol.cfd_protocol.hash_utils import CFDHashGenerator


@dataclass
class ShadowQualityScore:
    """Quality scoring for shadow files (ADR-016)"""
    completeness: float = 0.0        # All sections filled meaningfully
    accuracy: float = 0.0           # Checksum valid, content matches
    usefulness: float = 0.5         # AI consumption success rate (starts neutral)
    maintainability: float = 1.0    # Update frequency reasonable (starts good)
    size_efficiency: float = 1.0    # Shadow size vs subject size ratio
    author_ownership: float = 0.0   # Author validates and owns content
    source_clarity: float = 1.0     # Source self-explanatory vs shadow dependency
    
    @property
    def overall_score(self) -> float:
        return (
            self.completeness * 0.22 +
            self.accuracy * 0.13 + 
            self.usefulness * 0.32 +
            self.maintainability * 0.08 +
            self.size_efficiency * 0.1 +
            self.author_ownership * 0.05 +
            self.source_clarity * 0.1      # Source quality feedback
        )


class StaticAnalyzer:
    """Enhanced static analysis for better templates"""
    
    def __init__(self):
        self.hash_gen = CFDHashGenerator()
    
    def analyze_python_file(self, file_path: Path) -> Dict:
        """Analyze Python file structure for template generation"""
        content = file_path.read_text(encoding='utf-8', errors='ignore')
        
        analysis = {
            'content': content,
            'size': len(content),
            'lines': len(content.splitlines()),
            'checksum': self.hash_gen.hash7_base62(content),
            'imports': [],
            'classes': [],
            'functions': [],
            'docstring': None,
            'complexity_estimate': 1.0
        }
        
        try:
            tree = ast.parse(content)
            
            # Extract imports
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        analysis['imports'].append(alias.name)
                elif isinstance(node, ast.ImportFrom):
                    module = node.module or ''
                    for alias in node.names:
                        analysis['imports'].append(f"{module}.{alias.name}")
            
            # Extract classes and functions
            for node in tree.body:
                if isinstance(node, ast.ClassDef):
                    analysis['classes'].append({
                        'name': node.name,
                        'methods': [m.name for m in node.body if isinstance(m, ast.FunctionDef)],
                        'docstring': ast.get_docstring(node)
                    })
                elif isinstance(node, ast.FunctionDef):
                    analysis['functions'].append({
                        'name': node.name,
                        'args': [arg.arg for arg in node.args.args],
                        'docstring': ast.get_docstring(node)
                    })
            
            # Module docstring
            analysis['docstring'] = ast.get_docstring(tree)
            
            # Complexity estimate (rough)
            analysis['complexity_estimate'] = min(10.0, 1.0 + len(analysis['classes']) * 1.5 + len(analysis['functions']) * 0.8)
            
        except SyntaxError:
            # Fall back to regex parsing for non-Python or malformed files
            analysis = self._fallback_analysis(content, analysis)
        
        return analysis
    
    def analyze_source_clarity(self, file_path: Path, shadow_content: str) -> Dict:
        """Analyze if source is self-explanatory or overly dependent on shadow"""
        content = file_path.read_text(encoding='utf-8', errors='ignore')
        
        clarity_analysis = {
            'docstring_coverage': 0.0,
            'comment_density': 0.0,
            'self_explanatory_score': 0.0,
            'shadow_dependency_issues': [],
            'source_improvement_suggestions': []
        }
        
        try:
            tree = ast.parse(content)
            
            # Count functions/classes with docstrings
            total_definitions = 0
            documented_definitions = 0
            
            for node in ast.walk(tree):
                if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
                    total_definitions += 1
                    if ast.get_docstring(node):
                        documented_definitions += 1
            
            if total_definitions > 0:
                clarity_analysis['docstring_coverage'] = documented_definitions / total_definitions
                
        except SyntaxError:
            pass
        
        # Comment density analysis
        lines = content.split('\n')
        code_lines = [line for line in lines if line.strip() and not line.strip().startswith('#')]
        comment_lines = [line for line in lines if line.strip().startswith('#')]
        
        if len(code_lines) > 0:
            clarity_analysis['comment_density'] = len(comment_lines) / len(code_lines)
        
        # Detect shadow dependency issues
        self._detect_shadow_dependency_issues(content, shadow_content, clarity_analysis)
        
        # Calculate self-explanatory score
        clarity_analysis['self_explanatory_score'] = min(1.0, 
            (clarity_analysis['docstring_coverage'] * 0.6 + 
             clarity_analysis['comment_density'] * 0.4))
        
        return clarity_analysis
    
    def _detect_shadow_dependency_issues(self, source: str, shadow: str, analysis: Dict):
        """Detect when shadow provides essential info missing from source"""
        
        # Check if shadow explains purpose that's not clear from source
        if 'why_it_exists:' in shadow and 'TODO' not in shadow.split('why_it_exists:')[1].split('\n')[0]:
            # Shadow has meaningful purpose explanation
            if not any(keyword in source.lower() for keyword in ['purpose', 'why', 'because', 'reason']):
                analysis['shadow_dependency_issues'].append(
                    "Shadow explains purpose/rationale missing from source code"
                )
                analysis['source_improvement_suggestions'].append(
                    "Add module-level docstring explaining purpose and rationale"
                )
        
        # Check if shadow explains complex logic
        if 'design_decisions:' in shadow and 'TODO' not in shadow.split('design_decisions:')[1].split('\n')[0]:
            # Shadow explains design decisions
            complex_functions = self._find_complex_functions_without_docs(source)
            if complex_functions:
                analysis['shadow_dependency_issues'].append(
                    f"Shadow explains design decisions for undocumented complex code: {complex_functions[:2]}"
                )
                analysis['source_improvement_suggestions'].append(
                    f"Add docstrings/comments to complex functions: {', '.join(complex_functions[:3])}"
                )
        
        # Check if shadow explains gotchas/edge cases
        if 'gotchas:' in shadow and 'TODO' not in shadow.split('gotchas:')[1].split('\n')[0]:
            # Shadow explains gotchas/surprises
            if not any(keyword in source.lower() for keyword in ['warning', 'note', 'careful', 'edge', 'gotcha']):
                analysis['shadow_dependency_issues'].append(
                    "Shadow documents gotchas/edge cases not mentioned in source"
                )
                analysis['source_improvement_suggestions'].append(
                    "Add inline comments for gotchas and edge cases mentioned in shadow"
                )
    
    def _find_complex_functions_without_docs(self, source: str) -> List[str]:
        """Find complex functions that lack documentation"""
        try:
            tree = ast.parse(source)
            complex_undocumented = []
            
            for node in ast.walk(tree):
                if isinstance(node, ast.FunctionDef):
                    # Rough complexity: nested structures, long functions
                    complexity = 0
                    for child in ast.walk(node):
                        if isinstance(child, (ast.If, ast.For, ast.While, ast.Try)):
                            complexity += 1
                    
                    line_count = node.end_lineno - node.lineno if hasattr(node, 'end_lineno') else 0
                    
                    # Complex if: >3 control structures OR >20 lines
                    if (complexity > 3 or line_count > 20) and not ast.get_docstring(node):
                        complex_undocumented.append(node.name)
            
            return complex_undocumented
            
        except SyntaxError:
            return []
    
    def _fallback_analysis(self, content: str, base_analysis: Dict) -> Dict:
        """Fallback analysis using regex when AST parsing fails"""
        lines = content.split('\n')
        
        # Simple regex patterns
        base_analysis['imports'] = [line.strip() for line in lines 
                                   if re.match(r'^(import|from)\s+', line.strip())]
        base_analysis['classes'] = [line.strip() for line in lines 
                                   if re.match(r'^class\s+\w+', line.strip())]
        base_analysis['functions'] = [line.strip() for line in lines 
                                     if re.match(r'^def\s+\w+', line.strip())]
        
        return base_analysis


class ShadowTemplate:
    """Enhanced template system for human-authored shadows (ADR-016)"""
    
    SCHEMA_VERSION = "cfd_shadow_v1.0"
    
    def __init__(self):
        self.analyzer = StaticAnalyzer()
    
    def generate_template(self, file_path: Path, author: str = "Unknown", 
                         template_type: str = "python") -> str:
        """Generate intelligent template based on file analysis"""
        
        if template_type == "python" and file_path.suffix == ".py":
            return self._generate_python_template(file_path, author)
        elif template_type == "markdown" and file_path.suffix == ".md":
            return self._generate_markdown_template(file_path, author)
        else:
            return self._generate_generic_template(file_path, author)
    
    def _generate_python_template(self, file_path: Path, author: str) -> str:
        """Generate Python-specific template with intelligent hints"""
        analysis = self.analyzer.analyze_python_file(file_path)
        
        # Generate intelligent suggestions based on analysis
        purpose_hints = self._generate_purpose_hints(analysis)
        api_hints = self._generate_api_hints(analysis)
        dependency_hints = analysis['imports'][:5]  # Top 5 imports
        semantic_hints = self._generate_semantic_hints(file_path, analysis)
        
        return f"""# CFD Shadow File v1.0 - Human-Authored by {author}
# Generated: {datetime.now().isoformat()}Z

author_essence:
  purpose: "{purpose_hints}"
  why_it_exists: "TODO: What problem does this solve? Why was it created?"
  core_responsibility: "TODO: What is this file's main job in the system?"
  
public_contracts:
  primary_api:
    # TODO: List the main functions/classes that other code uses
    # Detected: {api_hints}
    
  guarantees:
    # TODO: What can callers rely on? What invariants does this maintain?
    
dependencies:
  strong: {json.dumps(dependency_hints[:3], indent=4) if dependency_hints else '[]'}
  weak: []  # TODO: Add optional/weak dependencies

semantic_triggers:
  primary: {json.dumps(semantic_hints[:3], indent=4)}  # TODO: Review and expand
  secondary: []  # TODO: Add related concepts
  use_cases: []  # TODO: When would someone need this file?

technical_metrics:
  complexity_score: {analysis['complexity_estimate']:.1f}  # Estimated 1-10
  token_cost: {len(analysis['content'].split())}  # Approximate token count
  
performance_characteristics:
  typical_usage: "TODO: How is this typically used?"
  bottlenecks: "TODO: What might slow this down?"
  memory_footprint: "TODO: Memory usage characteristics"

author_insights:
  design_decisions:
    # TODO: Why did you implement it this way?
    
  gotchas:
    # TODO: What surprises or edge cases should others know about?
    
  future_evolution:
    # TODO: How might this file change? What would you improve?

distillation_confidence: 0.0  # TODO: Rate your confidence (0.0-1.0)
last_shadow_update: "{datetime.now().isoformat()}Z"
source_checksum: "{analysis['checksum']}"
schema_version: "{self.SCHEMA_VERSION}"

# ANALYSIS RESULTS (delete after completing):
# File size: {analysis['lines']} lines, {analysis['size']} chars
# Classes found: {len(analysis['classes'])} - {[c['name'] for c in analysis['classes']][:3]}
# Functions found: {len(analysis['functions'])} - {[f['name'] for f in analysis['functions']][:3]}
# Top imports: {analysis['imports'][:5]}
# Complexity estimate: {analysis['complexity_estimate']:.1f}/10.0
"""
    
    def _generate_purpose_hints(self, analysis: Dict) -> str:
        """Generate intelligent purpose hints based on file analysis"""
        # Simple heuristics based on file structure
        if analysis['classes'] and analysis['functions']:
            return "TODO: Class-based module with utility functions"
        elif analysis['classes']:
            if len(analysis['classes']) == 1:
                return f"TODO: {analysis['classes'][0]['name']} class implementation"
            else:
                return "TODO: Multiple related classes"
        elif analysis['functions']:
            if any('main' in f['name'] for f in analysis['functions']):
                return "TODO: Main execution module or CLI tool"
            else:
                return "TODO: Utility functions and helpers"
        else:
            return "TODO: Configuration, constants, or data definitions"
    
    def _generate_api_hints(self, analysis: Dict) -> List[str]:
        """Generate API hints from class and function analysis"""
        api_hints = []
        
        for cls in analysis['classes'][:3]:  # Top 3 classes
            api_hints.append(f"{cls['name']} class")
            for method in cls['methods'][:2]:  # Top 2 methods
                if not method.startswith('_'):  # Skip private methods
                    api_hints.append(f"{cls['name']}.{method}()")
        
        for func in analysis['functions'][:3]:  # Top 3 functions
            if not func['name'].startswith('_'):  # Skip private functions
                api_hints.append(f"{func['name']}()")
        
        return api_hints
    
    def _generate_semantic_hints(self, file_path: Path, analysis: Dict) -> List[str]:
        """Generate semantic trigger hints from filename and content"""
        hints = []
        
        # From filename
        filename_parts = file_path.stem.lower().split('_')
        hints.extend(filename_parts)
        
        # From class/function names
        for cls in analysis['classes']:
            name_parts = re.findall(r'[A-Z][a-z]*', cls['name'])
            hints.extend([part.lower() for part in name_parts])
        
        for func in analysis['functions']:
            func_parts = func['name'].lower().split('_')
            hints.extend(func_parts)
        
        # From imports (domain hints)
        for imp in analysis['imports']:
            if any(domain in imp.lower() for domain in ['auth', 'db', 'api', 'web', 'test', 'util']):
                hints.append(imp.split('.')[-1].lower())
        
        return list(set(hints))[:10]  # Unique, max 10
    
    def _generate_markdown_template(self, file_path: Path, author: str) -> str:
        """Generate template for markdown files (ADRs, docs)"""
        content = file_path.read_text(encoding='utf-8', errors='ignore')
        checksum = self.analyzer.hash_gen.hash7_base62(content)
        
        # Detect if it's an ADR
        is_adr = 'ADR-' in file_path.name or 'decision' in file_path.parent.name.lower()
        
        return f"""# CFD Shadow File v1.0 - Human-Authored by {author}
# Generated: {datetime.now().isoformat()}Z

author_essence:
  purpose: "{'TODO: Architecture decision documentation' if is_adr else 'TODO: Documentation purpose'}"
  why_it_exists: "TODO: What information does this document provide?"
  core_responsibility: "TODO: What decisions or knowledge does this capture?"

document_characteristics:
  document_type: "{'Architecture Decision Record (ADR)' if is_adr else 'Documentation'}"
  audience: "TODO: Who is the primary audience?"
  scope: "TODO: What area or system does this cover?"

semantic_triggers:
  primary: []  # TODO: Key concepts and terms
  secondary: []  # TODO: Related topics
  use_cases: []  # TODO: When would someone reference this?

distillation_confidence: 0.0  # TODO: Rate your confidence (0.0-1.0)
last_shadow_update: "{datetime.now().isoformat()}Z"
source_checksum: "{checksum}"
schema_version: "{self.SCHEMA_VERSION}"
"""
    
    def _generate_generic_template(self, file_path: Path, author: str) -> str:
        """Generate generic template for unknown file types"""
        content = file_path.read_text(encoding='utf-8', errors='ignore')
        checksum = self.analyzer.hash_gen.hash7_base62(content)
        
        return f"""# CFD Shadow File v1.0 - Human-Authored by {author}
# Generated: {datetime.now().isoformat()}Z

author_essence:
  purpose: "TODO: What is this file's purpose?"
  why_it_exists: "TODO: Why was this file created?"
  core_responsibility: "TODO: What role does this play?"

file_characteristics:
  file_type: "{file_path.suffix}"
  size_bytes: {len(content)}

semantic_triggers:
  primary: []  # TODO: Key concepts
  use_cases: []  # TODO: When is this needed?

distillation_confidence: 0.0  # TODO: Rate confidence (0.0-1.0)
last_shadow_update: "{datetime.now().isoformat()}Z"
source_checksum: "{checksum}"
schema_version: "{self.SCHEMA_VERSION}"
"""


class ShadowSpecsGenerator:
    """Generate specifications and test requirements from shadow files"""
    
    def __init__(self):
        self.hash_gen = CFDHashGenerator()
    
    def generate_api_specs(self, shadow_path: Path) -> str:
        """Generate API specification from shadow file"""
        if not shadow_path.exists():
            return "# Error: Shadow file not found"
            
        content = shadow_path.read_text()
        source_file = self._get_source_file(shadow_path)
        
        # Parse shadow content
        shadow_data = self._parse_shadow_yaml(content)
        
        spec = f"""# API Specification
# Generated from: {shadow_path.name}
# Source: {source_file.name}
# Generated: {datetime.now().isoformat()}Z

## Purpose
{shadow_data.get('author_essence', {}).get('purpose', 'TODO: Add purpose from shadow')}

## Core Responsibility  
{shadow_data.get('author_essence', {}).get('core_responsibility', 'TODO: Add responsibility from shadow')}

## Public API Contracts
"""
        
        # Extract API contracts
        api_contracts = shadow_data.get('public_contracts', {}).get('primary_api', [])
        if api_contracts:
            spec += "### Primary API\n"
            for contract in api_contracts:
                if isinstance(contract, str):
                    spec += f"- `{contract}`\n"
                    
        # Extract guarantees
        guarantees = shadow_data.get('public_contracts', {}).get('guarantees', [])
        if guarantees:
            spec += "\n### Guarantees\n"
            for guarantee in guarantees:
                if isinstance(guarantee, str):
                    spec += f"- {guarantee}\n"
                    
        # Extract dependencies
        dependencies = shadow_data.get('dependencies', {})
        if dependencies:
            spec += "\n### Dependencies\n"
            strong_deps = dependencies.get('strong', [])
            if strong_deps:
                spec += "**Required:**\n"
                for dep in strong_deps[:5]:  # Limit to avoid bloat
                    spec += f"- {dep}\n"
                    
        # Extract performance characteristics
        perf = shadow_data.get('performance_characteristics', {})
        if perf:
            spec += "\n### Performance Characteristics\n"
            for key, value in perf.items():
                if value and 'TODO' not in str(value):
                    spec += f"- **{key.replace('_', ' ').title()}**: {value}\n"
        
        return spec
    
    def generate_test_requirements(self, shadow_path: Path) -> str:
        """Generate test requirements from shadow file"""
        if not shadow_path.exists():
            return "# Error: Shadow file not found"
            
        content = shadow_path.read_text()
        source_file = self._get_source_file(shadow_path)
        
        # Parse shadow content  
        shadow_data = self._parse_shadow_yaml(content)
        
        tests = f"""# Test Requirements
# Generated from: {shadow_path.name}
# Source: {source_file.name}
# Generated: {datetime.now().isoformat()}Z

## Test Categories

### Functional Tests
"""
        
        # Extract use cases for functional tests
        use_cases = shadow_data.get('semantic_triggers', {}).get('use_cases', [])
        if use_cases:
            tests += "#### Use Case Tests\n"
            for use_case in use_cases:
                if isinstance(use_case, str) and 'TODO' not in use_case:
                    tests += f"- **Test**: {use_case}\n"
                    tests += f"  - Verify: Expected behavior for '{use_case}'\n"
                    tests += f"  - Assert: Correct output/state changes\n\n"
        
        # Extract API contracts for interface tests
        api_contracts = shadow_data.get('public_contracts', {}).get('primary_api', [])
        if api_contracts:
            tests += "#### API Contract Tests\n"
            for contract in api_contracts:
                if isinstance(contract, str):
                    # Parse function signature
                    func_name = contract.split('(')[0].split('.')[-1]
                    tests += f"- **Test**: `{contract}`\n"
                    tests += f"  - Verify: Function {func_name} accepts expected parameters\n"
                    tests += f"  - Assert: Returns expected type/value\n"
                    tests += f"  - Edge cases: Invalid inputs, boundary conditions\n\n"
        
        # Extract gotchas for edge case tests
        gotchas = shadow_data.get('author_insights', {}).get('gotchas', [])
        if gotchas:
            tests += "### Edge Case Tests\n"
            for gotcha in gotchas:
                if isinstance(gotcha, str) and 'TODO' not in gotcha:
                    tests += f"- **Test**: {gotcha}\n"
                    tests += f"  - Verify: System handles this edge case gracefully\n"
                    tests += f"  - Assert: No crashes, appropriate error handling\n\n"
        
        # Extract performance requirements
        complexity = shadow_data.get('technical_metrics', {}).get('complexity_score', 0)
        if complexity and complexity > 7.0:  # High complexity needs performance tests
            tests += "### Performance Tests\n"
            tests += f"- **Test**: Performance under load (complexity: {complexity}/10)\n"
            tests += f"  - Verify: Acceptable response times\n"
            tests += f"  - Assert: No performance degradation\n\n"
            
        # Extract guarantees for contract tests
        guarantees = shadow_data.get('public_contracts', {}).get('guarantees', [])
        if guarantees:
            tests += "### Contract Tests\n"
            for guarantee in guarantees:
                if isinstance(guarantee, str) and 'TODO' not in guarantee:
                    tests += f"- **Test**: {guarantee}\n"
                    tests += f"  - Verify: Contract guarantee is maintained\n"
                    tests += f"  - Assert: Behavior matches specification\n\n"
        
        return tests
    
    def _parse_shadow_yaml(self, content: str) -> Dict:
        """Parse shadow content into structured data"""
        try:
            import yaml
            # Try proper YAML parsing first
            return yaml.safe_load(content)
        except:
            pass
            
        # Fallback to simple parsing for shadow files
        data = {}
        current_section = None
        current_key = None
        
        lines = content.split('\n')
        i = 0
        
        while i < len(lines):
            line = lines[i].rstrip()
            original_line = line
            line = line.strip()
            
            if not line or line.startswith('#'):
                i += 1
                continue
                
            # Top-level sections (no indentation)
            if line.endswith(':') and not original_line.startswith(' '):
                current_section = line[:-1]
                data[current_section] = {}
                current_key = None
                i += 1
                continue
                
            # Second-level keys (2 spaces)
            if original_line.startswith('  ') and not original_line.startswith('    ') and ':' in line:
                if current_section:
                    key, value = line.split(':', 1)
                    key = key.strip()
                    value = value.strip().strip('"')
                    current_key = key
                    
                    if not value or value in ['[]', '""']:
                        # Empty value, might have list items following
                        data[current_section][key] = []
                    else:
                        data[current_section][key] = value
                i += 1
                continue
                
            # Third-level items (4+ spaces) - list items or sub-values
            if original_line.startswith('    ') and current_section:
                if line.startswith('- '):
                    # List item
                    item = line[2:].strip().strip('"')
                    if current_key:
                        if current_key not in data[current_section]:
                            data[current_section][current_key] = []
                        elif not isinstance(data[current_section][current_key], list):
                            data[current_section][current_key] = [data[current_section][current_key]]
                        data[current_section][current_key].append(item)
                elif ':' in line:
                    # Sub-key
                    key, value = line.split(':', 1) 
                    key = key.strip()
                    value = value.strip().strip('"')
                    if current_key not in data[current_section]:
                        data[current_section][current_key] = {}
                    if isinstance(data[current_section][current_key], dict):
                        data[current_section][current_key][key] = value
                        
            i += 1
        
        return data
    
    def _get_source_file(self, shadow_path: Path) -> Path:
        """Get source file path from shadow file path"""
        name = shadow_path.name[1:]  # Remove leading dot
        if name.endswith('.cfd'):
            name = name[:-4]  # Remove .cfd extension
        return shadow_path.parent / name


class EnhancedShadowWorkflow:
    """Enhanced workflow implementing ADR-016 collaborative process"""
    
    def __init__(self):
        self.hash_gen = CFDHashGenerator()
        self.template = ShadowTemplate()
    
    def create_shadow(self, file_path: Path, author: str = "Unknown", 
                     template_type: str = "auto") -> Path:
        """Create shadow with intelligent template"""
        shadow_path = file_path.with_name(f".{file_path.name}.cfd")
        
        if shadow_path.exists():
            print(f"⚠️  Shadow already exists: {shadow_path}")
            return shadow_path
        
        # Auto-detect template type
        if template_type == "auto":
            if file_path.suffix == ".py":
                template_type = "python"
            elif file_path.suffix == ".md":
                template_type = "markdown"
            else:
                template_type = "generic"
        
        template_content = self.template.generate_template(file_path, author, template_type)
        shadow_path.write_text(template_content)
        
        print(f"✨ Created {template_type} shadow: {shadow_path}")
        print(f"📝 Please complete TODO sections for full shadow")
        return shadow_path
    
    def validate_shadow(self, shadow_path: Path, detailed: bool = False) -> Dict:
        """Enhanced validation with quality scoring"""
        if not shadow_path.exists():
            return {"valid": False, "error": "Shadow file not found", "score": ShadowQualityScore()}
        
        content = shadow_path.read_text()
        score = ShadowQualityScore()
        
        # Check completeness
        todo_count = content.count("TODO:")
        total_sections = content.count("TODO:") + len([line for line in content.split('\n') 
                                                      if ':' in line and not line.strip().startswith('#')])
        
        if total_sections > 0:
            score.completeness = max(0.0, 1.0 - (todo_count / total_sections))
        
        # Check accuracy (checksum validation) and size efficiency
        source_file = self._get_source_file(shadow_path)
        size_ratio = 0.0
        
        if source_file.exists():
            actual_hash = self.hash_gen.hash7_base62(source_file.read_text())
            shadow_hash = self._extract_checksum(content)
            score.accuracy = 1.0 if actual_hash == shadow_hash else 0.0
            
            # Check size efficiency (Paradigm: shadow shouldn't be bigger than subject)
            source_size = len(source_file.read_text())
            shadow_size = len(content)
            size_ratio = shadow_size / source_size if source_size > 0 else 0.0
            
            if size_ratio <= 0.5:  # Shadow ≤ 50% of source
                score.size_efficiency = 1.0
            elif size_ratio <= 0.8:  # Shadow ≤ 80% of source  
                score.size_efficiency = 0.8
            elif size_ratio <= 1.0:  # Shadow ≤ 100% of source
                score.size_efficiency = 0.5
            else:  # Shadow > source (VIOLATION)
                score.size_efficiency = 0.0
                
        # Check author ownership (distillation_confidence > 0.8 indicates author validation)
        confidence_line = next((line for line in content.split('\n') if 'distillation_confidence:' in line), None)
        if confidence_line:
            try:
                # Extract confidence value (handle comments after #)
                confidence_part = confidence_line.split('#')[0].strip()
                confidence = float(confidence_part.split(':')[1].strip())
                score.author_ownership = confidence if confidence >= 0.8 else 0.0
            except (ValueError, IndexError):
                score.author_ownership = 0.0
                
        # Check source clarity (Paradigm 4: Shadow shouldn't be more meaningful than source)
        clarity_analysis = None
        if source_file.exists():
            clarity_analysis = self.template.analyzer.analyze_source_clarity(source_file, content)
            
            # Score based on how self-explanatory the source is
            score.source_clarity = clarity_analysis['self_explanatory_score']
            
            # Penalty if shadow explains what source should explain
            if len(clarity_analysis['shadow_dependency_issues']) > 0:
                # Reduce score based on dependency issues
                penalty = min(0.5, len(clarity_analysis['shadow_dependency_issues']) * 0.15)
                score.source_clarity = max(0.0, score.source_clarity - penalty)
        
        validation_result = {
            "valid": (score.completeness > 0.8 and 
                     score.accuracy == 1.0 and 
                     score.size_efficiency >= 0.5 and  # Must not exceed source size
                     score.author_ownership >= 0.8 and  # Author must validate
                     score.source_clarity >= 0.6),     # Source should be reasonably self-explanatory
            "score": score,
            "todo_count": todo_count,
            "checksum_valid": score.accuracy == 1.0,
            "size_efficient": score.size_efficiency >= 0.5,
            "author_validated": score.author_ownership >= 0.8,
            "source_clear": score.source_clarity >= 0.6,
            "completeness_percent": score.completeness * 100,
            "overall_score": score.overall_score,
            "size_ratio": size_ratio,
            "source_clarity_score": score.source_clarity
        }
        
        if detailed:
            detailed_info = {
                "incomplete_sections": self._find_incomplete_sections(content),
                "suggestions": self._generate_improvement_suggestions(content, score)
            }
            
            # Add source clarity analysis if available
            if clarity_analysis is not None:
                detailed_info.update({
                    "source_issues": clarity_analysis['shadow_dependency_issues'],
                    "source_improvements": clarity_analysis['source_improvement_suggestions'],
                    "docstring_coverage": clarity_analysis['docstring_coverage'],
                    "comment_density": clarity_analysis['comment_density']
                })
                
            validation_result.update(detailed_info)
        
        return validation_result
    
    def _extract_checksum(self, content: str) -> Optional[str]:
        """Extract checksum from shadow content"""
        for line in content.split('\n'):
            if 'source_checksum:' in line:
                # Extract quoted string
                parts = line.split('"')
                if len(parts) >= 2:
                    return parts[1]
        return None
    
    def _find_incomplete_sections(self, content: str) -> List[str]:
        """Find sections that still contain TODO markers"""
        incomplete = []
        for line in content.split('\n'):
            if 'TODO:' in line:
                incomplete.append(line.strip())
        return incomplete[:10]  # Max 10 examples
    
    def _generate_improvement_suggestions(self, content: str, score: ShadowQualityScore) -> List[str]:
        """Generate suggestions for shadow improvement"""
        suggestions = []
        
        if score.completeness < 0.5:
            suggestions.append("Complete more TODO sections for better AI understanding")
        if score.accuracy == 0.0:
            suggestions.append("Update source_checksum - the original file has changed")
        if score.size_efficiency < 0.5:
            suggestions.append("⚠️  PARADIGM VIOLATION: Shadow larger than source - distill more aggressively")
        if score.author_ownership < 0.8:
            suggestions.append("Author must validate: Set distillation_confidence ≥ 0.8 after review")
        if score.source_clarity < 0.6:
            suggestions.append("🔄 PARADIGM 4: Shadow more meaningful than source - improve source documentation")
        if 'distillation_confidence: 0.0' in content:
            suggestions.append("Set your confidence level (0.0-1.0) based on shadow quality")
        
        return suggestions
    
    def update_shadow_checksum(self, shadow_path: Path) -> bool:
        """Update shadow checksum when source file changes"""
        source_file = self._get_source_file(shadow_path)
        if not source_file.exists():
            return False
        
        new_hash = self.hash_gen.hash7_base62(source_file.read_text())
        content = shadow_path.read_text()
        
        # Replace checksum and update timestamp
        lines = content.split('\n')
        updated = False
        
        for i, line in enumerate(lines):
            if 'source_checksum:' in line:
                lines[i] = f'source_checksum: "{new_hash}"'
                updated = True
            elif 'last_shadow_update:' in line:
                lines[i] = f'last_shadow_update: "{datetime.now().isoformat()}Z"'
        
        if updated:
            shadow_path.write_text('\n'.join(lines))
        return updated
    
    def _get_source_file(self, shadow_path: Path) -> Path:
        """Get source file path from shadow file path"""
        # .filename.ext.cfd -> filename.ext
        name = shadow_path.name[1:]  # Remove leading dot
        if name.endswith('.cfd'):
            name = name[:-4]  # Remove .cfd extension
        return shadow_path.parent / name


# Keep original workflow for backward compatibility
class ShadowWorkflow(EnhancedShadowWorkflow):
    """Backward compatibility wrapper"""
    pass


def main():
    parser = argparse.ArgumentParser(description="CFD Shadow Management CLI")
    subparsers = parser.add_subparsers(dest="command", required=True)
    
    # Create shadow
    create_parser = subparsers.add_parser("create", help="Create shadow file template")
    create_parser.add_argument("file", help="Source file to create shadow for")
    create_parser.add_argument("--author", default="Unknown", help="Author name")
    
    # Validate shadow
    validate_parser = subparsers.add_parser("validate", help="Validate shadow completeness")
    validate_parser.add_argument("shadow", help="Shadow file to validate")
    
    # Update checksum
    update_parser = subparsers.add_parser("update", help="Update shadow checksum")
    update_parser.add_argument("shadow", help="Shadow file to update")
    
    # Analyze source improvements
    analyze_parser = subparsers.add_parser("analyze", help="Analyze source code improvements from shadow")
    analyze_parser.add_argument("shadow", help="Shadow file to analyze")
    analyze_parser.add_argument("--detailed", action="store_true", help="Show detailed analysis")
    
    # Generate specs and tests from shadow
    generate_parser = subparsers.add_parser("generate", help="Generate specs/tests from shadow")
    generate_parser.add_argument("shadow", help="Shadow file to process")
    generate_parser.add_argument("--specs", action="store_true", help="Generate API specifications")
    generate_parser.add_argument("--tests", action="store_true", help="Generate test requirements")
    generate_parser.add_argument("--output", help="Output file path")
    
    # Batch operations
    batch_parser = subparsers.add_parser("batch", help="Batch operations on directory")
    batch_parser.add_argument("directory", help="Directory to process")
    batch_parser.add_argument("--create", action="store_true", help="Create missing shadows")
    batch_parser.add_argument("--validate", action="store_true", help="Validate all shadows")
    batch_parser.add_argument("--update", action="store_true", help="Update outdated checksums")
    batch_parser.add_argument("--author", default="Unknown", help="Author for new shadows")
    
    args = parser.parse_args()
    workflow = EnhancedShadowWorkflow()
    
    if args.command == "create":
        file_path = Path(args.file)
        if not file_path.exists():
            print(f"❌ File not found: {file_path}")
            return 1
        
        shadow_path = workflow.create_shadow(file_path, args.author)
        return 0
    
    elif args.command == "validate":
        shadow_path = Path(args.shadow)
        result = workflow.validate_shadow(shadow_path)
        
        if result["valid"]:
            print(f"✅ Shadow is complete and valid: {shadow_path}")
            print(f"   📊 Quality: {result['overall_score']:.2f} | Size: {result['size_ratio']:.1%} | Clarity: {result['source_clarity_score']:.2f}")
        else:
            print(f"⚠️  Shadow needs work: {shadow_path}")
            print(f"   📊 Quality: {result['overall_score']:.2f} | Size: {result['size_ratio']:.1%} | Clarity: {result['source_clarity_score']:.2f}")
            if result["todo_count"] > 0:
                print(f"   📝 {result['todo_count']} TODO sections remaining")
            if not result["checksum_valid"]:
                print(f"   🔄 Checksum outdated - source file changed")
            if not result["size_efficient"]:
                print(f"   🚨 PARADIGM VIOLATION: Shadow larger than source file!")
            if not result["author_validated"]:
                print(f"   👤 Author validation required (confidence ≥ 0.8)")
            if not result["source_clear"]:
                print(f"   🔄 PARADIGM 4: Source needs better documentation (shadow too meaningful)")
        
        return 0 if result["valid"] else 1
    
    elif args.command == "update":
        shadow_path = Path(args.shadow)
        if workflow.update_shadow_checksum(shadow_path):
            print(f"✅ Updated checksum: {shadow_path}")
        else:
            print(f"❌ Failed to update checksum: {shadow_path}")
        return 0
    
    elif args.command == "analyze":
        shadow_path = Path(args.shadow)
        result = workflow.validate_shadow(shadow_path, detailed=True)
        
        print(f"🔍 Source Code Analysis: {shadow_path}")
        print(f"   📊 Source Clarity Score: {result['source_clarity_score']:.2f}")
        
        if "source_issues" in result and result["source_issues"]:
            print(f"\n📋 Shadow Dependency Issues:")
            for issue in result["source_issues"]:
                print(f"   • {issue}")
                
        if "source_improvements" in result and result["source_improvements"]:
            print(f"\n💡 Recommended Source Improvements:")
            for improvement in result["source_improvements"]:
                print(f"   → {improvement}")
                
        if args.detailed and "docstring_coverage" in result:
            print(f"\n📈 Source Code Metrics:")
            print(f"   • Docstring Coverage: {result['docstring_coverage']:.1%}")
            print(f"   • Comment Density: {result['comment_density']:.1%}")
            
        return 0
    
    elif args.command == "generate":
        shadow_path = Path(args.shadow)
        generator = ShadowSpecsGenerator()
        
        if args.specs:
            print("📋 Generating API Specifications...")
            spec_content = generator.generate_api_specs(shadow_path)
            
            if args.output:
                output_path = Path(args.output)
                output_path.write_text(spec_content)
                print(f"✅ API specs saved to: {output_path}")
            else:
                print(spec_content)
                
        if args.tests:
            print("🧪 Generating Test Requirements...")
            test_content = generator.generate_test_requirements(shadow_path)
            
            if args.output:
                # If both specs and tests, append to same file
                output_path = Path(args.output)
                if args.specs and output_path.exists():
                    existing = output_path.read_text()
                    test_content = existing + "\n\n---\n\n" + test_content
                output_path.write_text(test_content)
                print(f"✅ Test requirements saved to: {output_path}")
            else:
                print(test_content)
                
        if not args.specs and not args.tests:
            print("❌ Please specify --specs and/or --tests")
            return 1
            
        return 0
    
    elif args.command == "batch":
        directory = Path(args.directory)
        if not directory.is_dir():
            print(f"❌ Directory not found: {directory}")
            return 1
        
        # Find Python files
        py_files = list(directory.rglob("*.py"))
        shadow_files = list(directory.rglob(".*.py.cfd"))
        
        print(f"📁 Found {len(py_files)} Python files, {len(shadow_files)} shadows")
        
        if args.create:
            for py_file in py_files:
                shadow_path = py_file.with_name(f".{py_file.name}.cfd")
                if not shadow_path.exists():
                    workflow.create_shadow(py_file, args.author)
        
        if args.validate:
            valid_count = 0
            for shadow_file in shadow_files:
                result = workflow.validate_shadow(shadow_file)
                if result["valid"]:
                    valid_count += 1
                else:
                    print(f"⚠️  {shadow_file.name}: {result['todo_count']} TODOs")
            
            print(f"📊 {valid_count}/{len(shadow_files)} shadows are complete")
        
        if args.update:
            updated_count = 0
            for shadow_file in shadow_files:
                if workflow.update_shadow_checksum(shadow_file):
                    updated_count += 1
            
            print(f"🔄 Updated {updated_count} shadow checksums")
        
        return 0


if __name__ == "__main__":
    exit(main())