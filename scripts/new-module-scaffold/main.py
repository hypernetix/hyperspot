#!/usr/bin/env python3
"""
New Module Scaffold Generator for Hyperspot

Generates a minimal, production-ready module following guidelines/NEW_MODULE.md.

Usage:
    python scripts/new-module-scaffold/main.py <module_name> [--force] [--validate]

Example:
    python scripts/new-module-scaffold/main.py my_new_module
"""

import argparse
import re
import sys
from pathlib import Path
from typing import Dict

try:
    from jinja2 import Environment, StrictUndefined
except ImportError as exc:  # pragma: no cover - informative early exit
    print("ERROR: Missing dependency 'jinja2'. Install it with `pip install jinja2` and rerun the scaffold.")
    raise


def snake_to_pascal(snake: str) -> str:
    """Convert snake_case to PascalCase."""
    return ''.join(word.capitalize() for word in snake.split('_'))


def snake_to_kebab(snake: str) -> str:
    """Convert snake_case to kebab-case."""
    return snake.replace('_', '-')


def validate_module_name(name: str) -> bool:
    """Validate that module name is valid snake_case."""
    return bool(re.match(r'^[a-z0-9_]+$', name))


def derive_names(module_name: str) -> Dict[str, str]:
    """Derive all naming conventions from snake_case module name."""
    return {
        'snake': module_name,
        'pascal': snake_to_pascal(module_name),
        'kebab': snake_to_kebab(module_name),
        'sdk_crate': f"{module_name}-sdk",
        'snake_upper': module_name.upper(),
    }


def get_script_dir() -> Path:
    """Get the directory where this script is located."""
    return Path(__file__).parent.resolve()


LITERAL_LBRACE_TOKEN = "__JINJA_LITERAL_LBRACE__"
LITERAL_RBRACE_TOKEN = "__JINJA_LITERAL_RBRACE__"
FORMAT_VAR_PATTERN = re.compile(r"\{([a-zA-Z0-9_]+)\}")


def create_template_env() -> Environment:
    """Create a shared Jinja environment for rendering templates."""
    return Environment(
        autoescape=False,
        keep_trailing_newline=True,
        lstrip_blocks=False,
        trim_blocks=False,
        undefined=StrictUndefined,
    )


JINJA_ENV = create_template_env()


def _format_to_jinja(template_content: str) -> str:
    """
    Convert legacy `str.format` placeholders to Jinja syntax.

    Existing templates rely on single-brace placeholders (e.g. {snake}) and double-brace
    escapes (e.g. {{ ... }}) to emit literal braces for Rust/TOML snippets. This helper
    preserves literal braces while rewriting substitution targets for the Jinja renderer.
    """

    def _replace_placeholder(match: re.Match[str]) -> str:
        key = match.group(1)
        return f"{{{{ {key} }}}}"

    converted = (
        template_content.replace("{{", LITERAL_LBRACE_TOKEN).replace("}}", LITERAL_RBRACE_TOKEN)
    )
    converted = FORMAT_VAR_PATTERN.sub(_replace_placeholder, converted)
    return converted.replace(LITERAL_LBRACE_TOKEN, "{").replace(LITERAL_RBRACE_TOKEN, "}")


def load_template(template_path: Path, names: Dict[str, str]) -> str:
    """Load a template file, convert to Jinja, and apply variable substitution."""
    if not template_path.exists():
        raise FileNotFoundError(f"Template not found: {template_path}")

    template_content = template_path.read_text()
    jinja_ready = _format_to_jinja(template_content)
    template = JINJA_ENV.from_string(jinja_ready)
    return template.render(**names)


def ensure_dir(path: Path):
    """Create directory if it doesn't exist."""
    path.mkdir(parents=True, exist_ok=True)


def write_file(path: Path, content: str, force: bool = False):
    """Write content to file, failing if file exists unless force=True."""
    if path.exists() and not force:
        print(f"ERROR: File already exists: {path}")
        print("Use --force to overwrite existing files.")
        sys.exit(1)
    
    ensure_dir(path.parent)
    path.write_text(content)
    print(f"✓ Created: {path}")


# ============================================================================
# Template File Mappings
# ============================================================================

SDK_TEMPLATES = {
    'Cargo.toml': 'sdk/Cargo.toml.template',
    'src/lib.rs': 'sdk/lib.rs.template',
    'src/api.rs': 'sdk/api.rs.template',
    'src/models.rs': 'sdk/models.rs.template',
    'src/errors.rs': 'sdk/errors.rs.template',
}

MODULE_TEMPLATES = {
    'Cargo.toml': 'module/Cargo.toml.template',
    'src/lib.rs': 'module/lib.rs.template',
    'src/module.rs': 'module/module.rs.template',
    'src/config.rs': 'module/config.rs.template',
    'src/local_client.rs': 'module/local_client.rs.template',
    'src/domain/mod.rs': 'module/domain/mod.rs.template',
    'src/domain/error.rs': 'module/domain/error.rs.template',
    'src/domain/service.rs': 'module/domain/service.rs.template',
    'src/domain/ports.rs': 'module/domain/ports.rs.template',
    'src/domain/repo.rs': 'module/domain/repo.rs.template',
    'src/api/mod.rs': 'module/api/mod.rs.template',
    'src/api/rest/mod.rs': 'module/api/rest/mod.rs.template',
    'src/api/rest/dto.rs': 'module/api/rest/dto.rs.template',
    'src/api/rest/handlers.rs': 'module/api/rest/handlers.rs.template',
    'src/api/rest/routes.rs': 'module/api/rest/routes.rs.template',
    'src/api/rest/error.rs': 'module/api/rest/error.rs.template',
    'tests/smoke.rs': 'module/tests/smoke.rs.template',
}

MODULE_TEMPLATES_EXTRA_DB = {
    'src/infra/mod.rs': 'module/infra/mod.rs.template',
    'src/infra/storage/mod.rs': 'module/infra/storage/mod.rs.template',
    'src/infra/storage/odata_mapper.rs': 'module/infra/storage/odata_mapper.rs.template',
    'src/infra/storage/entities/mod.rs': 'module/infra/storage/entities/mod.rs.template',
    'src/infra/storage/entities/example_entity.rs': 'module/infra/storage/entities/example_entity.rs.template',
    'src/infra/storage/entities/mapper.rs': 'module/infra/storage/entities/mapper.rs.template',
    'src/infra/storage/migrations/mod.rs': 'module/infra/storage/migrations/mod.rs.template',
}


# ============================================================================
# Generator Functions
# ============================================================================

def generate_sdk_crate(base_path: Path, names: Dict[str, str], templates_dir: Path, force: bool):
    """Generate SDK crate structure."""
    # SDK crate goes inside the module directory
    module_root = base_path / names['snake']
    sdk_path = module_root / names['sdk_crate']
    
    print(f"\n📦 Generating SDK crate: {names['sdk_crate']}")
    
    for output_file, template_file in SDK_TEMPLATES.items():
        template_path = templates_dir / template_file
        content = load_template(template_path, names)
        output_path = sdk_path / output_file
        write_file(output_path, content, force)


def generate_module_crate(base_path: Path, names: Dict[str, str], templates_dir: Path, force: bool, with_db: bool):
    """Generate module crate structure."""
    # Module crate goes inside the module directory alongside SDK
    module_root = base_path / names['snake']
    module_path = module_root / names['snake']
    
    print(f"\n📦 Generating module crate: {names['snake']}")
    
    for output_file, template_file in MODULE_TEMPLATES.items():
        template_path = templates_dir / template_file
        content = load_template(template_path, names)
        output_path = module_path / output_file
        write_file(output_path, content, force)
    if with_db:
        for output_file, template_file in MODULE_TEMPLATES_EXTRA_DB.items():
            template_path = templates_dir / template_file
            content = load_template(template_path, names)
            output_path = module_path / output_file
            write_file(output_path, content, force)


def print_wiring_instructions(names: Dict[str, str]):
    """Print manual wiring instructions for server integration."""
    print("\n" + "="*70)
    print("📋 MANUAL WIRING INSTRUCTIONS")
    print("="*70)
    
    print("\n1️⃣  Add to root Cargo.toml [workspace].members:")
    print(f'   "modules/{names["snake"]}/{names["sdk_crate"]}",')
    print(f'   "modules/{names["snake"]}/{names["snake"]}",')
    
    print("\n2️⃣  Add to apps/hyperspot-server/Cargo.toml dependencies:")
    print(f'   {names["snake"]} = {{ path = "../../modules/{names["snake"]}/{names["snake"]}" }}')
    
    print("\n3️⃣  Add to apps/hyperspot-server/src/registered_modules.rs:")
    print(f'   use {names["snake"]} as _;')
    
    print("\n" + "="*70)


def print_next_steps(names: Dict[str, str]):
    """Print next steps for the user."""
    print("\n🎉 Module scaffold generated successfully!")
    print("\n📝 Next steps:")
    print("   1. Apply the manual wiring instructions above")
    print("   2. Run: cargo check --workspace")
    print("   3. Run: cargo fmt --all")
    print("   4. Run: cargo test --workspace")
    print(f"   5. Start server and test: GET /{names['kebab']}/v1/health")
    print(f"   6. Check OpenAPI docs: http://127.0.0.1:8087/docs")
    print("\n💡 To add database, SSE, or plugin support, edit the generated files")
    print("   following guidelines/NEW_MODULE.md")
    print()


def main():
    parser = argparse.ArgumentParser(
        description="Generate a minimal Hyperspot module scaffold",
        epilog="Example: python scripts/new-module-scaffold/main.py my_module"
    )
    parser.add_argument(
        "module_name",
        help="Module name in snake_case (e.g., users_info, types_registry)"
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite existing files"
    )
    parser.add_argument(
        "--validate",
        action="store_true",
        help="Run cargo check after generation (requires Rust toolchain)"
    )
    parser.add_argument(
        "--with-db",
        action="store_true",
        help="Scaffold module with database-ready placeholders (available as `with_db` in templates)"
    )
    
    args = parser.parse_args()
    
    # Validate module name
    if not validate_module_name(args.module_name):
        print(f"ERROR: Invalid module name: {args.module_name}")
        print("Module name must be snake_case: [a-z0-9_]+")
        sys.exit(1)
    
    # Derive naming conventions
    names = derive_names(args.module_name)
    names["with_db"] = args.with_db
    
    print("🚀 Hyperspot Module Scaffold Generator")
    print("="*70)
    print(f"Module name (snake_case): {names['snake']}")
    print(f"Type name (PascalCase):   {names['pascal']}")
    print(f"REST path (kebab-case):   /{names['kebab']}/v1")
    print(f"SDK crate:                {names['sdk_crate']}")
    print(f"Database scaffolding:     {'enabled' if args.with_db else 'disabled'} (--with-db)")
    print("="*70)
    
    # Get script directory and templates directory
    script_dir = get_script_dir()
    templates_dir = script_dir / "templates"
    
    if not templates_dir.exists():
        print(f"\nERROR: Templates directory not found at {templates_dir}")
        print("Templates are required for scaffold generation.")
        sys.exit(1)
    
    # Find workspace root (where Cargo.toml with [workspace] is)
    current_dir = Path.cwd()
    workspace_root = current_dir
    
    # Try to find workspace root by looking for Cargo.toml with [workspace]
    while workspace_root != workspace_root.parent:
        cargo_toml = workspace_root / "Cargo.toml"
        if cargo_toml.exists():
            content = cargo_toml.read_text()
            if "[workspace]" in content:
                break
        workspace_root = workspace_root.parent
    
    modules_path = workspace_root / "modules"
    
    if not modules_path.exists():
        print(f"\nERROR: modules/ directory not found at {modules_path}")
        print("Are you running this from the workspace root?")
        sys.exit(1)
    
    # Generate SDK crate
    try:
        generate_sdk_crate(modules_path, names, templates_dir, args.force)
    except Exception as e:
        print(f"\n❌ ERROR generating SDK crate: {e}")
        sys.exit(1)
    
    # Generate module crate
    try:
        generate_module_crate(modules_path, names, templates_dir, args.force, args.with_db)
    except Exception as e:
        print(f"\n❌ ERROR generating module crate: {e}")
        sys.exit(1)
    
    # Print wiring instructions
    print_wiring_instructions(names)
    
    # Validate if requested
    if args.validate:
        print("\n🔍 Running validation...")
        import subprocess
        
        try:
            print("   Running: cargo check --workspace")
            result = subprocess.run(
                ["cargo", "check", "--workspace"],
                cwd=workspace_root,
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                print("   ✓ cargo check passed")
            else:
                print("   ✗ cargo check failed:")
                print(result.stderr)
                
            print("   Running: cargo fmt --all")
            subprocess.run(
                ["cargo", "fmt", "--all"],
                cwd=workspace_root,
                check=False
            )
            print("   ✓ cargo fmt completed")
            
        except FileNotFoundError:
            print("   ⚠ cargo not found - skipping validation")
            print("   Install Rust toolchain to enable validation")
    
    # Print next steps
    print_next_steps(names)


if __name__ == "__main__":
    main()
