import requests
import json
import sys
import time

def test_sql_injection_protection():
    print("SQL Injection test ediliyor...")

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
            print(f"  ✓ SQL injection engellendi: {query[:30]}...")
        else:
            print(f"  ✗ SQL injection engellenemedi: {query[:30]}...")
            return False

    return True

def test_path_traversal_protection():
    print("Path traversal testi...")

    malicious_slugs = [
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32",
        "test/../admin",
        "test/../../secret"
    ]

    for slug in malicious_slugs:
        response = requests.get(f"http://localhost:8099/food/{slug}")
        if response.status_code in [400, 404]:
            print(f"  ✓ Traversal path engellendi: {slug}")
        else:
            print(f"  ✗ Traversal path engellenemedi: {slug}")
            return False

    return True

def test_security_headers():
    print("Güvenlik emniyetleri testi...")

    response = requests.get("http://localhost:8099/health")
    headers = response.headers

    required_headers = {
        "x-frame-options": "DENY",
        "x-content-type-options": "nosniff",
        "x-xss-protection": "1; mode=block",
        "content-security-policy": lambda v: "default-src" in v,
        "referrer-policy": "no-referrer",
    }

    for header, expected in required_headers.items():
        if header in headers:
            if callable(expected):
                if expected(headers[header]):
                    print(f"  ✓ {header}: {headers[header][:50]}...")
                else:
                    print(f"  ✗ {header} header yanlış değer")
                    return False
            else:
                if headers[header] == expected:
                    print(f"  ✓ {header}: {headers[header]}")
                else:
                    print(f"  ✗ {header} yanlış değer {headers[header]}")
                    return False
        else:
            print(f"  ✗ Eksik header: {header}")
            return False

    return True

def check_cors_header(response):
    return "access-control-allow-origin" in response.headers

def test_cors_headers():
    print("CORS header testi...")

    # Try OPTIONS request first
    response = requests.options(
        "http://localhost:8099/health",
        headers={"Origin": "http://example.com"}
    )

    if check_cors_header(response):
        print(f"  ✓ CORS ayarlı: {response.headers['access-control-allow-origin']}")
        return True

    # CORS might only be on GET requests
    response = requests.get(
        "http://localhost:8099/health",
        headers={"Origin": "http://example.com"}
    )

    if check_cors_header(response):
        print(f"  ✓ CORS ayarlı: {response.headers['access-control-allow-origin']}")
        return True

    print("  ✗ CORS headerları bulunamadı")
    return False

def test_input_validation():
    print("Girdi kontrolü testi...")

    response = requests.get(
        "http://localhost:8099/foods/search",
        params={"q": "   "}
    )
    if response.status_code == 400:
        print("  ✓ Boş sorgu engellendi")
    else:
        print("  ✗ Boş sorgu engellenemedi")
        return False

    response = requests.get(
        "http://localhost:8099/foods/search",
        params={"q": "test", "mode": "invalid"}
    )
    if response.status_code == 400:
        print("  ✓ Geçersiz mod engellendi")
    else:
        print("  ✗ Geçersiz mod engellenemedi")
        return False

    response = requests.get(
        "http://localhost:8099/foods/search",
        params={"q": "test", "limit": 1000}
    )
    if response.status_code == 400:
        print("  ✓ Limit üstü arama engellendi")
    else:
        print("  ✗ Limit üstü arama engellenemedi")
        return False

    return True

def test_valid_requests():
    print("Testing valid requests still work...")

    response = requests.get("http://localhost:8099/health")
    if response.status_code == 200:
        print("  ✓ Health endpointi çalışıyor")
    else:
        print("  ✗ Health endpointi çalışmıyor")
        return False

    response = requests.get(
        "http://localhost:8099/foods/search",
        params={"q": "test", "mode": "description", "limit": 5}
    )
    if response.status_code in [200, 404]:
        print("  ✓ Arama çalışıyor")
    else:
        print(f"  ✗ Arama çalışmıyor {response.status_code}")
        return False

    return True

def main():
    print("=" * 60)
    print("BesinVeri API Güvenlik Entegrasyonu Testleri")
    print("=" * 60)
    print()
    print("Not: API sunucusunun 8099 portu üzerinden çalıştığına emin olun, test edilen: localhost:8099")
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
    connection_error = False

    for test in tests:
        time.sleep(1)
        try:
            if test():
                passed += 1
                print(f"✓ {test.__name__} geçti\n")
            else:
                failed += 1
                print(f"✗ {test.__name__} başarısız\n")
        except requests.exceptions.ConnectionError:
            if not connection_error:
                print(f"✗ API sunucusuna bağlanılamadı, çalıştığına emin olun: localhost:8099?")
                connection_error = True
            failed += 1
            print(f"✗ {test.__name__} bağlantı hatasından dolayı atlandı\n")
        except Exception as e:
            failed += 1
            print(f"✗ {test.__name__} şu hatayla başarısız oldu: {e}\n")

    print("=" * 60)
    print(f"Sonuçlar: {passed} geçti, {failed} başarısız")
    if connection_error:
        print("Uyarı: bazı testler bağlantı hatalarından dolayı atlandı")
    print("=" * 60)

    if failed > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()