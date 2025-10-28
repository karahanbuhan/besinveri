#!/usr/bin/env python3
"""
Security integration test script for BesinVeri API
Tests various security vulnerabilities that have been fixed
"""

import requests
import json
import sys

def test_sql_injection_protection():
    """Test that SQL injection attempts are blocked"""
    print("Testing SQL injection protection...")
    
    malicious_queries = [
        "test'; DROP TABLE foods; --",
        "test\" OR 1=1--",
        "'; SELECT * FROM foods; --",
        "1' UNION SELECT * FROM foods--"
    ]
    
    for query in malicious_queries:
        response = requests.get(
            "http://localhost:8099/foods/search",
            params={"q": query}
        )
        if response.status_code == 400:
            print(f"  ✓ Blocked SQL injection: {query[:30]}...")
        else:
            print(f"  ✗ Failed to block SQL injection: {query[:30]}...")
            return False
    
    return True

def test_path_traversal_protection():
    """Test that path traversal attempts are blocked"""
    print("Testing path traversal protection...")
    
    malicious_slugs = [
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32",
        "test/../admin",
        "test/../../secret"
    ]
    
    for slug in malicious_slugs:
        response = requests.get(f"http://localhost:8099/food/{slug}")
        if response.status_code in [400, 404]:
            print(f"  ✓ Blocked path traversal: {slug}")
        else:
            print(f"  ✗ Failed to block path traversal: {slug}")
            return False
    
    return True

def test_security_headers():
    """Test that security headers are present"""
    print("Testing security headers...")
    
    response = requests.get("http://localhost:8099/health")
    headers = response.headers
    
    required_headers = {
        "x-frame-options": "DENY",
        "x-content-type-options": "nosniff",
        "x-xss-protection": "1; mode=block",
        "content-security-policy": lambda v: "default-src" in v,
        "referrer-policy": "strict-origin-when-cross-origin",
    }
    
    for header, expected in required_headers.items():
        if header in headers:
            if callable(expected):
                if expected(headers[header]):
                    print(f"  ✓ {header}: {headers[header][:50]}...")
                else:
                    print(f"  ✗ {header} has incorrect value")
                    return False
            else:
                if headers[header] == expected:
                    print(f"  ✓ {header}: {headers[header]}")
                else:
                    print(f"  ✗ {header} has incorrect value: {headers[header]}")
                    return False
        else:
            print(f"  ✗ Missing required header: {header}")
            return False
    
    return True

def test_cors_headers():
    """Test that CORS headers are configured"""
    print("Testing CORS headers...")
    
    response = requests.options(
        "http://localhost:8099/health",
        headers={"Origin": "http://example.com"}
    )
    
    if "access-control-allow-origin" in response.headers:
        print(f"  ✓ CORS configured: {response.headers['access-control-allow-origin']}")
        return True
    else:
        # CORS might only be on GET requests
        response = requests.get(
            "http://localhost:8099/health",
            headers={"Origin": "http://example.com"}
        )
        if "access-control-allow-origin" in response.headers:
            print(f"  ✓ CORS configured: {response.headers['access-control-allow-origin']}")
            return True
        else:
            print("  ✗ CORS headers not found")
            return False

def test_input_validation():
    """Test various input validation scenarios"""
    print("Testing input validation...")
    
    # Test empty query
    response = requests.get(
        "http://localhost:8099/foods/search",
        params={"q": "   "}
    )
    if response.status_code == 400:
        print("  ✓ Rejected empty query")
    else:
        print("  ✗ Failed to reject empty query")
        return False
    
    # Test invalid mode
    response = requests.get(
        "http://localhost:8099/foods/search",
        params={"q": "test", "mode": "invalid"}
    )
    if response.status_code == 400:
        print("  ✓ Rejected invalid mode")
    else:
        print("  ✗ Failed to reject invalid mode")
        return False
    
    # Test excessive limit
    response = requests.get(
        "http://localhost:8099/foods/search",
        params={"q": "test", "limit": 1000}
    )
    if response.status_code == 400:
        print("  ✓ Rejected excessive limit")
    else:
        print("  ✗ Failed to reject excessive limit")
        return False
    
    return True

def test_valid_requests():
    """Test that valid requests still work"""
    print("Testing valid requests still work...")
    
    # Test health endpoint
    response = requests.get("http://localhost:8099/health")
    if response.status_code == 200:
        print("  ✓ Health endpoint works")
    else:
        print("  ✗ Health endpoint failed")
        return False
    
    # Test valid search
    response = requests.get(
        "http://localhost:8099/foods/search",
        params={"q": "test", "mode": "description", "limit": 5}
    )
    if response.status_code in [200, 404]:  # 404 is OK if no results
        print("  ✓ Valid search works")
    else:
        print(f"  ✗ Valid search failed: {response.status_code}")
        return False
    
    return True

def main():
    """Run all security tests"""
    print("=" * 60)
    print("BesinVeri API Security Integration Tests")
    print("=" * 60)
    print()
    print("Note: Make sure the API server is running on localhost:8099")
    print()
    
    tests = [
        test_valid_requests,
        test_security_headers,
        test_cors_headers,
        test_sql_injection_protection,
        test_path_traversal_protection,
        test_input_validation,
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            if test():
                passed += 1
                print(f"✓ {test.__name__} passed\n")
            else:
                failed += 1
                print(f"✗ {test.__name__} failed\n")
        except requests.exceptions.ConnectionError:
            print(f"✗ Cannot connect to API server. Is it running on localhost:8099?")
            sys.exit(1)
        except Exception as e:
            failed += 1
            print(f"✗ {test.__name__} failed with exception: {e}\n")
    
    print("=" * 60)
    print(f"Results: {passed} passed, {failed} failed")
    print("=" * 60)
    
    if failed > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
