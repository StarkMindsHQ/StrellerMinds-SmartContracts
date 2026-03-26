#!/usr/bin/env python3
"""
Documentation extraction script for StrellerMinds smart contracts.

This script extracts documentation from Rust source files and generates
structured documentation for the StrellerMinds platform.
"""

import os
import re
import json
import argparse
from pathlib import Path
from typing import Dict, List, Any, Optional

class DocumentationExtractor:
    def __init__(self, contract_path: str, output_path: str):
        self.contract_path = Path(contract_path)
        self.output_path = Path(output_path)
        self.output_path.mkdir(parents=True, exist_ok=True)
        
        # Documentation patterns
        self.doc_comment_pattern = re.compile(r'///\s*(.+)')
        self.module_doc_pattern = re.compile(r'#!\[doc\s*=\s*"([^"]+)"\]')
        self.function_pattern = re.compile(r'pub\s+fn\s+(\w+)\s*\(')
        self.struct_pattern = re.compile(r'pub\s+struct\s+(\w+)')
        self.enum_pattern = re.compile(r'pub\s+enum\s+(\w+)')
        self.impl_pattern = re.compile(r'impl\s+(\w+)')
        
    def extract_from_file(self, file_path: Path) -> Dict[str, Any]:
        """Extract documentation from a single Rust file."""
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Extract module-level documentation
        module_docs = self.extract_module_docs(content)
        
        # Extract functions and their documentation
        functions = self.extract_functions(content)
        
        # Extract structs and their documentation
        structs = self.extract_structs(content)
        
        # Extract enums and their documentation
        enums = self.extract_enums(content)
        
        # Extract impl blocks
        impls = self.extract_impls(content)
        
        return {
            'file': str(file_path.relative_to(self.contract_path)),
            'module_docs': module_docs,
            'functions': functions,
            'structs': structs,
            'enums': enums,
            'impls': impls
        }
    
    def extract_module_docs(self, content: str) -> List[str]:
        """Extract module-level documentation."""
        docs = []
        
        # Look for #![doc = "..."] attributes
        for match in self.module_doc_pattern.finditer(content):
            docs.append(match.group(1))
        
        # Look for leading /// comments before the first item
        lines = content.split('\n')
        doc_lines = []
        in_doc_block = False
        
        for line in lines:
            line = line.strip()
            if line.startswith('///') and not in_doc_block:
                in_doc_block = True
                doc_content = line[3:].strip()
                if doc_content:
                    doc_lines.append(doc_content)
            elif line.startswith('///') and in_doc_block:
                doc_content = line[3:].strip()
                if doc_content:
                    doc_lines.append(doc_content)
            elif in_doc_block and not line.startswith('///'):
                break
        
        if doc_lines:
            docs.append('\n'.join(doc_lines))
        
        return docs
    
    def extract_functions(self, content: str) -> List[Dict[str, Any]]:
        """Extract function documentation."""
        functions = []
        lines = content.split('\n')
        
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            
            # Check for function definition
            func_match = self.function_pattern.search(line)
            if func_match:
                func_name = func_match.group(1)
                
                # Look backwards for documentation
                doc_lines = []
                j = i - 1
                while j >= 0 and lines[j].strip().startswith('///'):
                    doc_line = lines[j].strip()[3:].strip()
                    if doc_line:
                        doc_lines.insert(0, doc_line)
                    j -= 1
                
                # Parse function signature
                signature = self.extract_function_signature(lines[i:])
                
                functions.append({
                    'name': func_name,
                    'documentation': '\n'.join(doc_lines),
                    'signature': signature,
                    'line_number': i + 1
                })
            
            i += 1
        
        return functions
    
    def extract_structs(self, content: str) -> List[Dict[str, Any]]:
        """Extract struct documentation."""
        structs = []
        lines = content.split('\n')
        
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            
            # Check for struct definition
            struct_match = self.struct_pattern.search(line)
            if struct_match:
                struct_name = struct_match.group(1)
                
                # Look backwards for documentation
                doc_lines = []
                j = i - 1
                while j >= 0 and lines[j].strip().startswith('///'):
                    doc_line = lines[j].strip()[3:].strip()
                    if doc_line:
                        doc_lines.insert(0, doc_line)
                    j -= 1
                
                # Extract struct fields
                fields = self.extract_struct_fields(lines, i)
                
                structs.append({
                    'name': struct_name,
                    'documentation': '\n'.join(doc_lines),
                    'fields': fields,
                    'line_number': i + 1
                })
            
            i += 1
        
        return structs
    
    def extract_enums(self, content: str) -> List[Dict[str, Any]]:
        """Extract enum documentation."""
        enums = []
        lines = content.split('\n')
        
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            
            # Check for enum definition
            enum_match = self.enum_pattern.search(line)
            if enum_match:
                enum_name = enum_match.group(1)
                
                # Look backwards for documentation
                doc_lines = []
                j = i - 1
                while j >= 0 and lines[j].strip().startswith('///'):
                    doc_line = lines[j].strip()[3:].strip()
                    if doc_line:
                        doc_lines.insert(0, doc_line)
                    j -= 1
                
                # Extract enum variants
                variants = self.extract_enum_variants(lines, i)
                
                enums.append({
                    'name': enum_name,
                    'documentation': '\n'.join(doc_lines),
                    'variants': variants,
                    'line_number': i + 1
                })
            
            i += 1
        
        return enums
    
    def extract_impls(self, content: str) -> List[Dict[str, Any]]:
        """Extract impl block documentation."""
        impls = []
        lines = content.split('\n')
        
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            
            # Check for impl definition
            impl_match = self.impl_pattern.search(line)
            if impl_match:
                impl_name = impl_match.group(1)
                
                # Extract functions in impl block
                impl_functions = []
                j = i + 1
                brace_count = 0
                in_impl = False
                
                while j < len(lines):
                    line = lines[j].strip()
                    
                    if line.startswith('{'):
                        brace_count += 1
                        in_impl = True
                    elif line.startswith('}'):
                        brace_count -= 1
                        if brace_count == 0 and in_impl:
                            break
                    elif in_impl and line.startswith('pub fn'):
                        func_match = self.function_pattern.search(line)
                        if func_match:
                            func_name = func_match.group(1)
                            
                            # Look for documentation
                            doc_lines = []
                            k = j - 1
                            while k >= 0 and lines[k].strip().startswith('///'):
                                doc_line = lines[k].strip()[3:].strip()
                                if doc_line:
                                    doc_lines.insert(0, doc_line)
                                k -= 1
                            
                            impl_functions.append({
                                'name': func_name,
                                'documentation': '\n'.join(doc_lines)
                            })
                    
                    j += 1
                
                impls.append({
                    'name': impl_name,
                    'functions': impl_functions,
                    'line_number': i + 1
                })
            
            i += 1
        
        return impls
    
    def extract_function_signature(self, lines: List[str], start_idx: int) -> str:
        """Extract complete function signature."""
        signature_lines = []
        i = start_idx
        
        while i < len(lines):
            line = lines[i].strip()
            signature_lines.append(line)
            
            if line.endswith('{') or line.endswith(';'):
                break
            
            i += 1
        
        return ' '.join(signature_lines)
    
    def extract_struct_fields(self, lines: List[str], start_idx: int) -> List[Dict[str, str]]:
        """Extract struct fields with documentation."""
        fields = []
        i = start_idx + 1
        brace_count = 0
        
        while i < len(lines):
            line = lines[i].strip()
            
            if line.startswith('{'):
                brace_count += 1
            elif line.startswith('}'):
                brace_count -= 1
                if brace_count == 0:
                    break
            elif '///' in line:
                # Field documentation
                doc_line = line.split('///')[1].strip()
                if doc_line:
                    # Look for the field on the next line
                    next_line = lines[i + 1].strip() if i + 1 < len(lines) else ''
                    field_match = re.search(r'pub\s+(\w+):', next_line)
                    if field_match:
                        field_name = field_match.group(1)
                        fields.append({
                            'name': field_name,
                            'documentation': doc_line
                        })
            elif re.search(r'pub\s+(\w+):', line):
                # Field without documentation
                field_match = re.search(r'pub\s+(\w+):', line)
                if field_match:
                    field_name = field_match.group(1)
                    fields.append({
                        'name': field_name,
                        'documentation': ''
                    })
            
            i += 1
        
        return fields
    
    def extract_enum_variants(self, lines: List[str], start_idx: int) -> List[Dict[str, str]]:
        """Extract enum variants with documentation."""
        variants = []
        i = start_idx + 1
        brace_count = 0
        
        while i < len(lines):
            line = lines[i].strip()
            
            if line.startswith('{'):
                brace_count += 1
            elif line.startswith('}'):
                brace_count -= 1
                if brace_count == 0:
                    break
            elif '///' in line:
                # Variant documentation
                doc_line = line.split('///')[1].strip()
                if doc_line:
                    # Look for the variant on the next line
                    next_line = lines[i + 1].strip() if i + 1 < len(lines) else ''
                    variant_match = re.search(r'(\w+)(?:\([^)]*\))?', next_line)
                    if variant_match:
                        variant_name = variant_match.group(1)
                        variants.append({
                            'name': variant_name,
                            'documentation': doc_line
                        })
            elif re.search(r'(\w+)(?:\([^)]*\))?,', line):
                # Variant without documentation
                variant_match = re.search(r'(\w+)(?:\([^)]*\))?', line)
                if variant_match:
                    variant_name = variant_match.group(1)
                    variants.append({
                        'name': variant_name,
                        'documentation': ''
                    })
            
            i += 1
        
        return variants
    
    def generate_markdown_docs(self, extracted_data: Dict[str, Any]) -> str:
        """Generate markdown documentation from extracted data."""
        markdown = []
        
        # Add file header
        file_name = extracted_data['file']
        markdown.append(f"# {file_name}\n")
        
        # Add module documentation
        if extracted_data['module_docs']:
            markdown.append("## Module Documentation\n")
            for doc in extracted_data['module_docs']:
                markdown.append(doc)
                markdown.append("")
        
        # Add structs
        if extracted_data['structs']:
            markdown.append("## Structs\n")
            for struct in extracted_data['structs']:
                markdown.append(f"### {struct['name']}\n")
                if struct['documentation']:
                    markdown.append(struct['documentation'])
                    markdown.append("")
                
                if struct['fields']:
                    markdown.append("**Fields:**\n")
                    for field in struct['fields']:
                        markdown.append(f"- **{field['name']}**: {field['documentation']}")
                    markdown.append("")
        
        # Add enums
        if extracted_data['enums']:
            markdown.append("## Enums\n")
            for enum in extracted_data['enums']:
                markdown.append(f"### {enum['name']}\n")
                if enum['documentation']:
                    markdown.append(enum['documentation'])
                    markdown.append("")
                
                if enum['variants']:
                    markdown.append("**Variants:**\n")
                    for variant in enum['variants']:
                        markdown.append(f"- **{variant['name']}**: {variant['documentation']}")
                    markdown.append("")
        
        # Add impl blocks
        if extracted_data['impls']:
            markdown.append("## Implementations\n")
            for impl in extracted_data['impls']:
                markdown.append(f"### impl {impl['name']}\n")
                
                if impl['functions']:
                    for func in impl['functions']:
                        markdown.append(f"#### {func['name']}\n")
                        if func['documentation']:
                            markdown.append(func['documentation'])
                            markdown.append("")
                        markdown.append(f"```rust\n{func['signature']}\n```\n")
        
        # Add standalone functions
        if extracted_data['functions']:
            markdown.append("## Functions\n")
            for func in extracted_data['functions']:
                markdown.append(f"### {func['name']}\n")
                if func['documentation']:
                    markdown.append(func['documentation'])
                    markdown.append("")
                markdown.append(f"```rust\n{func['signature']}\n```\n")
        
        return '\n'.join(markdown)
    
    def extract_from_contract(self) -> Dict[str, Any]:
        """Extract documentation from all files in the contract."""
        contract_data = {
            'contract_name': self.contract_path.name,
            'files': {},
            'summary': {
                'total_files': 0,
                'total_functions': 0,
                'total_structs': 0,
                'total_enums': 0,
                'total_impls': 0,
                'documented_items': 0
            }
        }
        
        # Process all Rust files
        for rust_file in self.contract_path.rglob('*.rs'):
            if rust_file.is_file():
                relative_path = rust_file.relative_to(self.contract_path)
                extracted = self.extract_from_file(rust_file)
                contract_data['files'][str(relative_path)] = extracted
                
                # Update summary
                contract_data['summary']['total_files'] += 1
                contract_data['summary']['total_functions'] += len(extracted['functions'])
                contract_data['summary']['total_structs'] += len(extracted['structs'])
                contract_data['summary']['total_enums'] += len(extracted['enums'])
                contract_data['summary']['total_impls'] += len(extracted['impls'])
                
                # Count documented items
                documented = 0
                for func in extracted['functions']:
                    if func['documentation']:
                        documented += 1
                for struct in extracted['structs']:
                    if struct['documentation']:
                        documented += 1
                for enum in extracted['enums']:
                    if enum['documentation']:
                        documented += 1
                
                contract_data['summary']['documented_items'] += documented
        
        return contract_data
    
    def save_documentation(self, contract_data: Dict[str, Any]):
        """Save extracted documentation to files."""
        # Save raw data as JSON
        json_path = self.output_path / 'extracted_data.json'
        with open(json_path, 'w', encoding='utf-8') as f:
            json.dump(contract_data, f, indent=2)
        
        # Generate markdown for each file
        for file_path, extracted in contract_data['files'].items():
            markdown = self.generate_markdown_docs(extracted)
            
            # Create safe filename
            safe_filename = file_path.replace('/', '_').replace('.rs', '.md')
            markdown_path = self.output_path / safe_filename
            
            with open(markdown_path, 'w', encoding='utf-8') as f:
                f.write(markdown)
        
        # Generate index file
        self.generate_index_file(contract_data)
        
        # Generate summary report
        self.generate_summary_report(contract_data)
    
    def generate_index_file(self, contract_data: Dict[str, Any]):
        """Generate an index file for the contract documentation."""
        index_content = []
        
        contract_name = contract_data['contract_name']
        index_content.append(f"# {contract_name} Documentation\n")
        index_content.append(f"Generated on: {self.get_current_timestamp()}\n")
        
        # Add summary
        summary = contract_data['summary']
        index_content.append("## Summary\n")
        index_content.append(f"- **Total Files**: {summary['total_files']}")
        index_content.append(f"- **Total Functions**: {summary['total_functions']}")
        index_content.append(f"- **Total Structs**: {summary['total_structs']}")
        index_content.append(f"- **Total Enums**: {summary['total_enums']}")
        index_content.append(f"- **Total Implementations**: {summary['total_impls']}")
        index_content.append(f"- **Documented Items**: {summary['documented_items']}")
        index_content.append("")
        
        # Add file list
        index_content.append("## Files\n")
        for file_path in sorted(contract_data['files'].keys()):
            safe_filename = file_path.replace('/', '_').replace('.rs', '.md')
            index_content.append(f"- [{file_path}]({safe_filename})")
        
        index_content.append("")
        
        # Save index file
        index_path = self.output_path / 'index.md'
        with open(index_path, 'w', encoding='utf-8') as f:
            f.write('\n'.join(index_content))
    
    def generate_summary_report(self, contract_data: Dict[str, Any]):
        """Generate a summary report of the documentation extraction."""
        report = {
            'contract': contract_data['contract_name'],
            'extraction_timestamp': self.get_current_timestamp(),
            'summary': contract_data['summary'],
            'files': {}
        }
        
        for file_path, extracted in contract_data['files'].items():
            file_report = {
                'functions': len(extracted['functions']),
                'structs': len(extracted['structs']),
                'enums': len(extracted['enums']),
                'impls': len(extracted['impls']),
                'has_module_docs': len(extracted['module_docs']) > 0,
                'documented_functions': sum(1 for f in extracted['functions'] if f['documentation']),
                'documented_structs': sum(1 for s in extracted['structs'] if s['documentation']),
                'documented_enums': sum(1 for e in extracted['enums'] if e['documentation'])
            }
            report['files'][file_path] = file_report
        
        # Save summary report
        summary_path = self.output_path / 'summary_report.json'
        with open(summary_path, 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=2)
    
    def get_current_timestamp(self) -> str:
        """Get current timestamp in ISO format."""
        from datetime import datetime
        return datetime.now().isoformat()

def main():
    parser = argparse.ArgumentParser(description='Extract documentation from StrellerMinds smart contracts')
    parser.add_argument('contract_path', help='Path to the contract directory')
    parser.add_argument('output_path', help='Path to the output directory')
    parser.add_argument('--verbose', '-v', action='store_true', help='Enable verbose output')
    
    args = parser.parse_args()
    
    if args.verbose:
        print(f"Extracting documentation from: {args.contract_path}")
        print(f"Output directory: {args.output_path}")
    
    # Create extractor and run extraction
    extractor = DocumentationExtractor(args.contract_path, args.output_path)
    contract_data = extractor.extract_from_contract()
    
    # Save documentation
    extractor.save_documentation(contract_data)
    
    if args.verbose:
        summary = contract_data['summary']
        print(f"\nExtraction complete!")
        print(f"Files processed: {summary['total_files']}")
        print(f"Functions found: {summary['total_functions']}")
        print(f"Structs found: {summary['total_structs']}")
        print(f"Enums found: {summary['total_enums']}")
        print(f"Documented items: {summary['documented_items']}")

if __name__ == '__main__':
    main()
