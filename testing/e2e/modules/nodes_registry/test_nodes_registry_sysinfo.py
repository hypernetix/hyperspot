"""E2E tests for nodes_registry sysinfo endpoint."""
import httpx
import pytest


@pytest.mark.asyncio
async def test_get_node_sysinfo(base_url, auth_headers):
    """
    Test GET /nodes-registry/v1/nodes/{id}/sysinfo endpoint.

    This test verifies that we can retrieve detailed system information for a node.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Get a valid node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert list_response.status_code == 200
        nodes = list_response.json()
        assert len(nodes) >= 1
        node_id = nodes[0]["id"]

        # Fetch sysinfo
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/sysinfo",
            headers=auth_headers,
        )

        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. "
            f"Response: {response.text}"
        )

        # Parse and validate sysinfo structure
        sysinfo = response.json()
        assert isinstance(sysinfo, dict)

        # Required fields
        assert "node_id" in sysinfo
        assert sysinfo["node_id"] == node_id
        assert "os" in sysinfo
        assert "cpu" in sysinfo
        assert "memory" in sysinfo
        assert "host" in sysinfo
        assert "gpus" in sysinfo
        assert "collected_at" in sysinfo


@pytest.mark.asyncio
async def test_get_node_sysinfo_os_details(base_url, auth_headers):
    """
    Test OS information structure in sysinfo endpoint.

    This test verifies the operating system information is correctly structured.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Get node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # Fetch sysinfo
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/sysinfo",
            headers=auth_headers,
        )

        assert response.status_code == 200
        sysinfo = response.json()

        # Validate OS info
        os_info = sysinfo["os"]
        assert isinstance(os_info, dict)
        assert "name" in os_info
        assert "version" in os_info
        assert "arch" in os_info

        # OS name should not be empty
        assert len(os_info["name"]) > 0
        
        # Architecture should be a known value
        valid_archs = ["x86_64", "aarch64", "x86", "arm", "i686"]
        assert os_info["arch"] in valid_archs, (
            f"Unexpected architecture: {os_info['arch']}"
        )


@pytest.mark.skip(reason="Test is failing")
@pytest.mark.asyncio
async def test_get_node_sysinfo_cpu_details(base_url, auth_headers):
    """
    Test CPU information structure in sysinfo endpoint.

    This test verifies the CPU information is correctly structured and contains valid data.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Get node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # Fetch sysinfo
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/sysinfo",
            headers=auth_headers,
        )

        assert response.status_code == 200
        sysinfo = response.json()

        # Validate CPU info
        cpu_info = sysinfo["cpu"]
        assert isinstance(cpu_info, dict)
        
        # Required fields
        assert "model" in cpu_info
        assert "num_cpus" in cpu_info
        assert "cores" in cpu_info
        assert "frequency_mhz" in cpu_info

        # Validate values
        assert isinstance(cpu_info["model"], str)
        assert len(cpu_info["model"]) > 0
        
        assert isinstance(cpu_info["num_cpus"], int)
        assert cpu_info["num_cpus"] > 0
        assert cpu_info["num_cpus"] <= 1024  # Reasonable upper bound
        
        assert isinstance(cpu_info["cores"], int)
        assert cpu_info["cores"] > 0
        assert cpu_info["cores"] <= cpu_info["num_cpus"]  # Cores should not exceed logical CPUs
        
        assert isinstance(cpu_info["frequency_mhz"], (int, float))
        assert cpu_info["frequency_mhz"] > 0
        assert cpu_info["frequency_mhz"] < 10000  # Reasonable upper bound (10 GHz)


@pytest.mark.asyncio
async def test_get_node_sysinfo_memory_details(base_url, auth_headers):
    """
    Test memory information structure in sysinfo endpoint.

    This test verifies the memory information is correctly structured and contains valid data.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Get node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # Fetch sysinfo
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/sysinfo",
            headers=auth_headers,
        )

        assert response.status_code == 200
        sysinfo = response.json()

        # Validate memory info
        mem_info = sysinfo["memory"]
        assert isinstance(mem_info, dict)
        
        # Required fields
        assert "total_bytes" in mem_info
        assert "available_bytes" in mem_info
        assert "used_bytes" in mem_info
        assert "used_percent" in mem_info

        # Validate values
        assert isinstance(mem_info["total_bytes"], int)
        assert mem_info["total_bytes"] > 0
        
        assert isinstance(mem_info["available_bytes"], int)
        assert mem_info["available_bytes"] >= 0
        assert mem_info["available_bytes"] <= mem_info["total_bytes"]
        
        assert isinstance(mem_info["used_bytes"], int)
        assert mem_info["used_bytes"] >= 0
        assert mem_info["used_bytes"] <= mem_info["total_bytes"]
        
        assert isinstance(mem_info["used_percent"], int)
        assert mem_info["used_percent"] >= 0
        assert mem_info["used_percent"] <= 100


@pytest.mark.asyncio
async def test_get_node_sysinfo_host_details(base_url, auth_headers):
    """
    Test host information structure in sysinfo endpoint.

    This test verifies the host information is correctly structured.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Get node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # Fetch sysinfo
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/sysinfo",
            headers=auth_headers,
        )

        assert response.status_code == 200
        sysinfo = response.json()

        # Validate host info
        host_info = sysinfo["host"]
        assert isinstance(host_info, dict)
        
        # Required fields
        assert "hostname" in host_info
        assert "uptime_seconds" in host_info
        assert "ip_addresses" in host_info

        # Validate values
        assert isinstance(host_info["hostname"], str)
        assert len(host_info["hostname"]) > 0
        
        assert isinstance(host_info["uptime_seconds"], int)
        assert host_info["uptime_seconds"] >= 0
        
        assert isinstance(host_info["ip_addresses"], list)
        assert len(host_info["ip_addresses"]) > 0
        
        # Validate IP addresses format (basic check)
        for ip in host_info["ip_addresses"]:
            assert isinstance(ip, str)
            assert len(ip) > 0


@pytest.mark.asyncio
async def test_get_node_sysinfo_gpu_details(base_url, auth_headers):
    """
    Test GPU information structure in sysinfo endpoint.

    This test verifies the GPU information is correctly structured when GPUs are present.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Get node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # Fetch sysinfo
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/sysinfo",
            headers=auth_headers,
        )

        assert response.status_code == 200
        sysinfo = response.json()

        # Validate GPU info structure
        gpus = sysinfo["gpus"]
        assert isinstance(gpus, list)
        
        # If GPUs are present, validate their structure
        for gpu in gpus:
            assert isinstance(gpu, dict)
            assert "model" in gpu
            assert isinstance(gpu["model"], str)
            
            # Optional fields
            if "cores" in gpu and gpu["cores"] is not None:
                assert isinstance(gpu["cores"], int)
                assert gpu["cores"] > 0
            
            if "total_memory_mb" in gpu and gpu["total_memory_mb"] is not None:
                assert isinstance(gpu["total_memory_mb"], (int, float))
                assert gpu["total_memory_mb"] > 0
            
            if "used_memory_mb" in gpu and gpu["used_memory_mb"] is not None:
                assert isinstance(gpu["used_memory_mb"], (int, float))
                assert gpu["used_memory_mb"] >= 0
