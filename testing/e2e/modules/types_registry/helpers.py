"""Helper functions for types-registry e2e tests."""
import uuid


def unique_gts_id(vendor: str, package: str, namespace: str, name: str) -> str:
    """
    Generate a unique GTS ID to avoid conflicts between test runs.
    
    Format: gts.{vendor}.{package}.{namespace}.{name}{uuid}.v1~
    """
    short_uuid = uuid.uuid4().hex[:8]
    return f"gts.{vendor}.{package}.{namespace}.{name}{short_uuid}.v1~"


def unique_type_id(name: str) -> str:
    """Generate a unique type GTS ID."""
    short_uuid = uuid.uuid4().hex[:8]
    return f"gts.e2e.test.models.{name}{short_uuid}.v1~"


def unique_instance_id(type_id: str, name: str) -> str:
    """Generate a unique instance GTS ID based on a type ID."""
    short_uuid = uuid.uuid4().hex[:8]
    return f"{type_id}e2e.test.instances.{name}{short_uuid}.v1"
